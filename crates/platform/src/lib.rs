//! The adapters: everything the program asks of the operating system, the file system
//! and the network, behind the ports the application and the presenter declare. Each
//! backend (windows.rs, macos.rs, linux.rs) provides the same set of functions and a
//! `SpriteWindow`; `adapters` turns them into the port implementations, and `files`
//! keeps what survives between runs.
//!
//! All coordinates are "world pixels": y grows downward from the top of the primary
//! display, and a monitor's `scale` says how big one cell is drawn there.
// Style lints only: the FFI declarations keep the OS's own type names, and the adapters
// are built by the composition root, not by `Default`. Compiler warnings are not silenced.
#![allow(
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    clippy::new_without_default
)]
mod adapters;
mod files;
pub use adapters::{
    capturable_sprite_factory, sprite_factory, CurlReleaseFeed, NativeDesktop, NativeWorkstation,
    SystemClock, TempFileJournal,
};
pub use files::{FileBrainStore, FilePreferences};
pub use idlewalker_application::{BrightnessInfo, Modifier, MouseButtons, Screen};

pub(crate) use idlewalker_domain::rt;
pub(crate) mod prelude {
    pub use idlewalker_rendering::prelude::*;
}
use crate::prelude::*;
use std::cell::RefCell;

#[cfg(windows)]
#[path = "windows.rs"]
mod backend;
#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod backend;
#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod backend;

pub use backend::*;

/// The monitor layout, read once per frame by `refresh`; every query after that is
/// a lookup in a tiny array, however many walkers are asking.
#[derive(Default)]
struct WorldInner {
    screens: Vec<Screen>,
    bounds: Arr<Rectangle>,
    scales: Arr<i32>,
    edges: Arr<i32>,
    left: i32,
    right: i32,
}

/// `IWorld` backed by the real monitors.
#[derive(Clone, Copy)]
pub struct MonitorWorld(&'static RefCell<WorldInner>);

impl MonitorWorld {
    pub fn new() -> Self {
        let inner = WorldInner {
            bounds: Arr::from_vec(vec![]),
            scales: Arr::from_vec(vec![]),
            edges: Arr::from_vec(vec![]),
            ..Default::default()
        };
        let w = MonitorWorld(Box::leak(Box::new(RefCell::new(inner))));
        w.refresh();
        w
    }

    /// Re-reads the layout: plug/unplug and resolution changes apply at once.
    pub fn refresh(&self) {
        let screens = screens();
        let mut s = self.0.borrow_mut();
        if s.screens == screens {
            return;
        }
        s.bounds = Arr::from_vec(screens.iter().map(|m| m.bounds).collect());
        s.scales = Arr::from_vec(screens.iter().map(|m| m.cell()).collect());
        s.edges = Arr::from_vec(
            screens
                .iter()
                .flat_map(|m| [m.bounds.left(), m.bounds.right()])
                .collect(),
        );
        s.left = screens.iter().map(|m| m.bounds.left()).min().unwrap_or(0);
        s.right = screens.iter().map(|m| m.bounds.right()).max().unwrap_or(0);
        s.screens = screens;
    }

    pub fn screens(&self) -> Vec<Screen> {
        self.0.borrow().screens.clone()
    }
    pub fn monitor_bounds(&self) -> Arr<Rectangle> {
        self.0.borrow().bounds.clone()
    }
    pub fn monitor_scales(&self) -> Arr<i32> {
        self.0.borrow().scales.clone()
    }

    /// The screen holding a point, or the nearest one.
    pub fn screen_at(&self, p: Point) -> Screen {
        let s = self.0.borrow();
        let dist = |m: &Screen| {
            let b = m.bounds;
            let dx = if p.x < b.left() {
                b.left() - p.x
            } else if p.x >= b.right() {
                p.x - b.right() + 1
            } else {
                0
            };
            let dy = if p.y < b.top() {
                b.top() - p.y
            } else if p.y >= b.bottom() {
                p.y - b.bottom() + 1
            } else {
                0
            };
            dx.max(dy)
        };
        s.screens
            .iter()
            .min_by_key(|m| dist(m))
            .copied()
            .unwrap_or_default()
    }

    // The monitor whose horizontal span contains x. When monitors are stacked
    // vertically the lowest one wins; in a gap, the nearest one.
    fn index_under(s: &WorldInner, x: i32) -> usize {
        let (mut best, mut best_distance) = (0, i32::MAX);
        for (i, m) in s.screens.iter().enumerate() {
            let b = m.bounds;
            let d = if x < b.left() {
                b.left() - x
            } else if x >= b.right() {
                x - b.right() + 1
            } else {
                0
            };
            if d < best_distance
                || (d == best_distance && b.bottom() > s.screens[best].bounds.bottom())
            {
                best = i;
                best_distance = d;
            }
        }
        best
    }
}

impl rt::IWorldApi for MonitorWorld {
    fn left(&self) -> i32 {
        self.0.borrow().left
    }
    fn right(&self) -> i32 {
        self.0.borrow().right
    }
    fn monitor_edges(&self) -> Arr<i32> {
        self.0.borrow().edges.clone()
    }
    fn monitor_bounds_at(&self, p: Point) -> Rectangle {
        self.screen_at(p).bounds
    }
    fn ground_at(&self, x: f64) -> f64 {
        let s = self.0.borrow();
        s.screens
            .get(Self::index_under(&s, x.floor() as i32))
            .map_or(0.0, |m| m.work.bottom() as f64)
    }
    fn scale_at(&self, x: f64) -> i32 {
        let s = self.0.borrow();
        s.screens
            .get(Self::index_under(&s, x.floor() as i32))
            .map_or(Settings::BASE_PIXEL_SIZE, |m| m.cell())
    }
}

/// Seconds to add to UTC for local time; used for the night rhythm of the cast.
#[cfg(windows)]
pub fn utc_offset_seconds() -> i64 {
    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct St {
        y: u16,
        mo: u16,
        dw: u16,
        d: u16,
        h: u16,
        mi: u16,
        s: u16,
        ms: u16,
    }
    extern "system" {
        fn GetLocalTime(st: *mut St);
        fn GetSystemTime(st: *mut St);
    }
    let (mut l, mut u) = (St::default(), St::default());
    unsafe {
        GetLocalTime(&mut l);
        GetSystemTime(&mut u);
    }
    let mins = |t: St| t.h as i64 * 60 + t.mi as i64;
    let mut delta = mins(l) - mins(u);
    if l.d != u.d {
        delta += if (l.y, l.mo, l.d) > (u.y, u.mo, u.d) {
            1440
        } else {
            -1440
        };
    }
    delta * 60
}

#[cfg(unix)]
pub fn utc_offset_seconds() -> i64 {
    unsafe {
        let t = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        tm.tm_gmtoff as i64
    }
}

/// Runs a helper program without waiting for it; false if it could not start.
pub(crate) fn spawn(program: &str, args: &[&str]) -> bool {
    let mut c = std::process::Command::new(program);
    c.args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    hide_console(&mut c);
    c.spawn().is_ok()
}

/// The address a redirecting link ends at, found with curl: following a redirect needs
/// neither a rate-limited API nor a JSON parser.
pub(crate) fn final_address(url: &str) -> Option<String> {
    let args = [
        "--fail",
        "--silent",
        "--head",
        "--location",
        "--max-time",
        "20",
        "--write-out",
        "\\n%{url_effective}\\n",
        url,
    ];
    let out = output_of("curl", &args)?;
    let last = out.lines().rev().find(|l| !l.trim().is_empty());
    Some(last.unwrap_or("").to_string())
}

/// A program on the PATH, as a shell would find it.
/// The PATH a login shell has. A program started from the Dock or a desktop menu gets a
/// shorter one, without Homebrew, Nix or the person's own folders.
fn login_path() -> Option<std::ffi::OsString> {
    use std::io::Read;
    static PATH: std::sync::OnceLock<Option<std::ffi::OsString>> = std::sync::OnceLock::new();
    PATH.get_or_init(|| {
        if cfg!(windows) {
            return None;
        }
        let shell = std::env::var_os("SHELL").unwrap_or_else(|| "/bin/sh".into());
        let mut child = std::process::Command::new(shell)
            .args(["-lc", "printf %s \"$PATH\""])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .ok()?;
        let mut out = child.stdout.take()?;
        let reader = std::thread::spawn(move || {
            let mut text = String::new();
            let _ = out.read_to_string(&mut text);
            text
        });
        // A profile that waits for something must not hold the window up.
        for _ in 0..30 {
            match child.try_wait() {
                Ok(Some(_)) => {
                    return reader
                        .join()
                        .ok()
                        .filter(|t| !t.is_empty())
                        .map(std::ffi::OsString::from);
                }
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                Err(_) => break,
            }
        }
        let _ = child.kill();
        let _ = child.wait();
        None
    })
    .clone()
}

pub(crate) fn find_program(name: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    let exts: Vec<String> = if cfg!(windows) {
        let has_ext = std::path::Path::new(name).extension().is_some();
        let listed = std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.CMD;.BAT".into())
            .split(';')
            .map(|e| e.to_lowercase())
            .collect::<Vec<_>>();
        let mut exts = if has_ext {
            vec![String::new()]
        } else {
            Vec::new()
        };
        exts.extend(listed);
        exts
    } else {
        vec![String::new()]
    };
    let login = login_path();
    let extra = login.iter().flat_map(std::env::split_paths);
    for dir in std::env::split_paths(&path).chain(extra) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        for ext in &exts {
            let candidate = dir.join(format!("{name}{ext}"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Runs a helper program and returns what it printed, if it succeeded.
pub(crate) fn output_of(program: &str, args: &[&str]) -> Option<String> {
    let mut c = std::process::Command::new(program);
    c.args(args)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    hide_console(&mut c);
    let o = c.output().ok()?;
    if o.status.success() {
        Some(String::from_utf8_lossy(&o.stdout).into_owned())
    } else {
        None
    }
}
