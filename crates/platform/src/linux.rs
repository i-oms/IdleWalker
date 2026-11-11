//! Linux: X11 through a pure-Rust connection, so there is no C library to install.
//!
//! Sprites are override-redirect ARGB windows with an empty input shape. That needs a
//! compositing window manager, which every mainstream desktop has; without one no
//! sprites are shown rather than black boxes. Under Wayland the program runs through
//! XWayland: walkers and the brightness panel work, but Wayland does not let any
//! program read the global cursor, so the position only updates while the pointer is
//! over an X11 window. See "Platform differences" in the README.
use super::{BrightnessInfo, MouseButtons, Screen};
use crate::prelude::*;
use crate::rt::{Point, Rectangle};
use std::ffi::CString;
use std::path::PathBuf;
use std::sync::OnceLock;
use x11rb::connection::Connection;
use x11rb::protocol::randr::ConnectionExt as _;
use x11rb::protocol::shape::{self, ConnectionExt as _};
use x11rb::protocol::xproto::*;
use x11rb::protocol::xtest::ConnectionExt as _;
use x11rb::rust_connection::RustConnection;

struct X {
    conn: RustConnection,
    root: Window,
    visual: Option<(Visualid, Colormap)>,
    a_workarea: Atom,
    a_active: Atom,
    a_state: Atom,
    a_fullscreen: Atom,
    a_wm_cm: Atom,
    dpi_scale: f64,
}
unsafe impl Sync for X {}

static XCONN: OnceLock<Option<X>> = OnceLock::new();

fn x() -> Option<&'static X> {
    XCONN.get_or_init(connect).as_ref()
}

fn atom(c: &RustConnection, name: &str) -> Atom {
    c.intern_atom(false, name.as_bytes())
        .ok()
        .and_then(|r| r.reply().ok())
        .map_or(0, |r| r.atom)
}

fn connect() -> Option<X> {
    let (conn, screen_num) = RustConnection::connect(None).ok()?;
    let screen = conn.setup().roots[screen_num].clone();
    // The first 32-bit true-colour visual is the compositor's ARGB one.
    let visual_id = screen
        .allowed_depths
        .iter()
        .filter(|d| d.depth == 32)
        .flat_map(|d| d.visuals.iter())
        .find(|v| v.class == VisualClass::TRUE_COLOR)
        .map(|v| v.visual_id);
    let visual = visual_id.and_then(|v| {
        let cm = conn.generate_id().ok()?;
        conn.create_colormap(ColormapAlloc::NONE, cm, screen.root, v)
            .ok()?;
        Some((v, cm))
    });
    let a_wm_cm = atom(&conn, &format!("_NET_WM_CM_S{}", screen_num));
    let dpi_scale = xft_dpi(&conn, screen.root).unwrap_or(96.0) / 96.0;
    Some(X {
        root: screen.root,
        visual,
        a_workarea: atom(&conn, "_NET_WORKAREA"),
        a_active: atom(&conn, "_NET_ACTIVE_WINDOW"),
        a_state: atom(&conn, "_NET_WM_STATE"),
        a_fullscreen: atom(&conn, "_NET_WM_STATE_FULLSCREEN"),
        a_wm_cm,
        dpi_scale,
        conn,
    })
}

/// Xft.dpi from the resource database: how desktops publish display scaling to X11.
fn xft_dpi(c: &RustConnection, root: Window) -> Option<f64> {
    let r = c
        .get_property(
            false,
            root,
            AtomEnum::RESOURCE_MANAGER,
            AtomEnum::STRING,
            0,
            65536,
        )
        .ok()?
        .reply()
        .ok()?;
    let text = String::from_utf8_lossy(&r.value);
    text.lines()
        .find_map(|l| l.strip_prefix("Xft.dpi:"))
        .and_then(|v| v.trim().parse().ok())
}

pub(crate) fn hide_console(_c: &mut std::process::Command) {}

// ---- The process --------------------------------------------------------------------

pub fn init_process() {}

/// False when another copy is already running; the lock is held until the process ends.
pub fn single_instance() -> bool {
    let Some(dir) = config_dir() else { return true };
    let _ = std::fs::create_dir_all(&dir);
    let Ok(path) = CString::new(dir.join("instance.lock").to_string_lossy().as_bytes()) else {
        return true;
    };
    unsafe {
        let fd = libc::open(path.as_ptr(), libc::O_CREAT | libc::O_RDWR, 0o600);
        fd < 0 || libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) == 0
    }
}

pub fn lower_thread_priority() {
    unsafe {
        let tid = libc::syscall(libc::SYS_gettid) as libc::id_t;
        libc::setpriority(libc::PRIO_PROCESS, tid, 5);
    }
}

pub fn config_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("idlewalker"))
}

pub fn legacy_setting(_name: &str) -> Option<i32> {
    None
}

// ---- Monitors and the cursor -----------------------------------------------------------

pub fn screens() -> Vec<Screen> {
    let Some(x) = x() else { return vec![] };
    let scale = x.dpi_scale;
    let monitors = x
        .conn
        .randr_get_monitors(x.root, true)
        .ok()
        .and_then(|r| r.reply().ok());
    let mut out: Vec<Screen> = monitors
        .map(|r| {
            r.monitors
                .iter()
                .map(|m| {
                    let b = Rectangle::new(m.x as i32, m.y as i32, m.width as i32, m.height as i32);
                    Screen {
                        bounds: b,
                        work: b,
                        dpi_scale: scale,
                        primary: m.primary,
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    if out.is_empty() {
        let s = &x.conn.setup().roots[0];
        let b = Rectangle::new(0, 0, s.width_in_pixels as i32, s.height_in_pixels as i32);
        out.push(Screen {
            bounds: b,
            work: b,
            dpi_scale: scale,
            primary: true,
        });
    }
    // _NET_WORKAREA is one rectangle over the whole desktop: panels at the very bottom
    // of the lowest monitor shorten the working area of the monitors they touch.
    if let Some(area) = x
        .conn
        .get_property(false, x.root, x.a_workarea, AtomEnum::CARDINAL, 0, 4)
        .ok()
        .and_then(|r| r.reply().ok())
        .and_then(|r| r.value32().map(|v| v.collect::<Vec<u32>>()))
    {
        if area.len() == 4 {
            let bottom = (area[1] + area[3]) as i32;
            for s in &mut out {
                if s.bounds.bottom() > bottom && s.bounds.top() < bottom {
                    s.work = Rectangle::from_ltrb(
                        s.bounds.left(),
                        s.bounds.top(),
                        s.bounds.right(),
                        bottom,
                    );
                }
            }
        }
    }
    out.sort_by_key(|s| !s.primary);
    out
}

pub fn cursor_pos() -> Option<Point> {
    let x = x()?;
    let r = x.conn.query_pointer(x.root).ok()?.reply().ok()?;
    Some(Point::new(r.root_x as i32, r.root_y as i32))
}

/// The pointer's button and modifier bits; 0 when the server will not say.
fn pointer_mask() -> u16 {
    x().and_then(|x| x.conn.query_pointer(x.root).ok()?.reply().ok())
        .map_or(0, |r| u16::from(r.mask))
}

pub fn mouse_button_down() -> bool {
    pointer_mask()
        & (u16::from(KeyButMask::BUTTON1)
            | u16::from(KeyButMask::BUTTON2)
            | u16::from(KeyButMask::BUTTON3))
        != 0
}

/// Held right now, after the person's button mapping. The server keeps no record of
/// presses, so a click shorter than the caller's poll is not seen.
pub fn mouse_buttons() -> MouseButtons {
    let mask = pointer_mask();
    MouseButtons {
        left: mask & u16::from(KeyButMask::BUTTON1) != 0,
        right: mask & u16::from(KeyButMask::BUTTON3) != 0,
    }
}

pub fn ctrl_down() -> bool {
    pointer_mask() & u16::from(KeyButMask::CONTROL) != 0
}

/// More keys are held than the one Ctrl accounts for.
pub fn other_key_down() -> bool {
    let Some(x) = x() else { return false };
    let held: u32 = x
        .conn
        .query_keymap()
        .ok()
        .and_then(|r| r.reply().ok())
        .map_or(0, |r| r.keys.iter().map(|b| b.count_ones()).sum());
    held > ctrl_down() as u32
}

// ---- Pacing and power ---------------------------------------------------------------------

/// No portable blocking wait for the compositor: false makes the pump pace by the clock.
pub fn wait_vblank() -> bool {
    false
}

/// The pump has nothing to animate; nothing to release here.
pub fn pacing_idle() {}

pub fn on_battery() -> bool {
    let Ok(dir) = std::fs::read_dir("/sys/class/power_supply") else {
        return false;
    };
    let mut has_mains = false;
    for e in dir.flatten() {
        let p = e.path();
        if std::fs::read_to_string(p.join("type")).is_ok_and(|t| t.trim() == "Mains") {
            has_mains = true;
            if std::fs::read_to_string(p.join("online")).is_ok_and(|v| v.trim() == "1") {
                return false;
            }
        }
    }
    has_mains
}

// ---- Keeping the machine awake ---------------------------------------------------------------

/// A `systemd-inhibit` child holds off idle and sleep, for as long as it lives.
pub fn keep_awake(on: bool) {
    use std::sync::Mutex;
    static CHILD: Mutex<Option<std::process::Child>> = Mutex::new(None);
    let mut child = CHILD.lock().unwrap();
    if let Some(mut c) = child.take() {
        let _ = c.kill();
        let _ = c.wait();
    }
    if on {
        *child = std::process::Command::new("systemd-inhibit")
            .args([
                "--what=idle:sleep",
                "--who=IdleWalker",
                "--why=Keep system awake",
                "--mode=block",
                "sleep",
                "infinity",
            ])
            .spawn()
            .ok();
    }
}

pub fn system_idle_seconds() -> f64 {
    use x11rb::protocol::screensaver::ConnectionExt as _;
    let Some(x) = x() else { return 0.0 };
    x.conn
        .screensaver_query_info(x.root)
        .ok()
        .and_then(|r| r.reply().ok())
        .map_or(0.0, |r| r.ms_since_user_input as f64 / 1000.0)
}

/// Resets the X screen-saver timer; the inhibitor covers the rest.
pub fn nudge_activity() {
    if let Some(x) = x() {
        let _ = x.conn.force_screen_saver(ScreenSaver::RESET);
        let _ = x.conn.flush();
    }
}

// ---- The foreground window -------------------------------------------------------------------

fn active_window(x: &X) -> Option<Window> {
    x.conn
        .get_property(false, x.root, x.a_active, AtomEnum::WINDOW, 0, 1)
        .ok()?
        .reply()
        .ok()?
        .value32()?
        .next()
        .filter(|w| *w != 0)
}

/// True when the window in front has asked for the whole screen. A maximised window does not.
pub fn is_fullscreen() -> bool {
    let Some(x) = x() else { return false };
    let Some(w) = active_window(x) else {
        return false;
    };
    x.conn
        .get_property(false, w, x.a_state, AtomEnum::ATOM, 0, 32)
        .ok()
        .and_then(|r| r.reply().ok())
        .and_then(|r| r.value32().map(|mut v| v.any(|a| a == x.a_fullscreen)))
        .unwrap_or(false)
}

pub fn foreground_description() -> String {
    let Some(x) = x() else { return "(no X)".into() };
    let Some(w) = active_window(x) else {
        return "(none)".into();
    };
    x.conn
        .get_property(false, w, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 64)
        .ok()
        .and_then(|r| r.reply().ok())
        .map_or("(unknown)".into(), |r| {
            String::from_utf8_lossy(&r.value)
                .replace('\0', " ")
                .trim()
                .to_string()
        })
}

// ---- Hot corners -------------------------------------------------------------------------------

pub const CORNER_ACTION_NAMES: [&str; 10] = [
    "Nothing",
    "Overview",
    "Show desktop",
    "App menu",
    "Notifications",
    "Quick settings",
    "Lock screen",
    "Screen saver",
    "Monitor off",
    "Call the Minions",
];

const KEY_LCTRL: u8 = 37;
const KEY_LALT: u8 = 64;
const KEY_LSUPER: u8 = 133;
const KEY_TAB: u8 = 23;
const KEY_D: u8 = 40;
const KEY_V: u8 = 55;
const KEY_S: u8 = 39;
const KEY_F1: u8 = 67;
const KEY_F10: u8 = 76;

/// Presses keys through the XTEST extension, in order, and releases them in reverse.
fn press(keys: &[u8]) {
    let Some(x) = x() else { return };
    for &k in keys {
        let _ = x
            .conn
            .xtest_fake_input(KEY_PRESS_EVENT, k, 0, x.root, 0, 0, 0);
    }
    for &k in keys.iter().rev() {
        let _ = x
            .conn
            .xtest_fake_input(KEY_RELEASE_EVENT, k, 0, x.root, 0, 0, 0);
    }
    let _ = x.conn.flush();
}

fn desktop() -> String {
    std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_uppercase()
}

/// Carries out a corner's action with the desktop's own keyboard shortcuts.
pub fn perform_corner(action: CornerAction) {
    let d = desktop();
    let gnome = d.contains("GNOME") || d.contains("UNITY") || d.contains("CINNAMON");
    let kde = d.contains("KDE");
    match action {
        CornerAction::TaskView => {
            if kde {
                press(&[KEY_LSUPER, KEY_TAB])
            } else {
                press(&[KEY_LSUPER])
            }
        }
        CornerAction::ShowDesktop => {
            if gnome {
                press(&[KEY_LCTRL, KEY_LSUPER, KEY_D])
            } else if kde {
                press(&[KEY_LSUPER, KEY_D])
            } else {
                press(&[KEY_LCTRL, KEY_LALT, KEY_D])
            }
        }
        CornerAction::StartMenu => {
            if gnome {
                press(&[KEY_LALT, KEY_F1])
            } else {
                press(&[KEY_LSUPER])
            }
        }
        CornerAction::Notifications => {
            if gnome {
                press(&[KEY_LSUPER, KEY_V])
            } else {
                press(&[KEY_LSUPER, KEY_F10])
            }
        }
        CornerAction::QuickSettings => {
            if gnome {
                press(&[KEY_LSUPER, KEY_S])
            }
        }
        CornerAction::LockScreen => {
            if !super::spawn("loginctl", &["lock-session"]) {
                super::spawn("xdg-screensaver", &["lock"]);
            }
        }
        CornerAction::ScreenSaver => {
            super::spawn("xdg-screensaver", &["activate"]);
        }
        CornerAction::MonitorOff => {
            super::spawn("xset", &["dpms", "force", "off"]);
        }
        _ => {}
    }
}

// ---- Starting at sign-in ---------------------------------------------------------------------------

pub const STARTUP_LABEL: &str = "Start at login";

fn autostart_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("autostart/idlewalker.desktop"))
}

pub fn startup_is_enabled() -> bool {
    autostart_path().is_some_and(|p| p.exists())
}

pub fn startup_set(enabled: bool) -> Result<(), String> {
    let path = autostart_path().ok_or("no home folder")?;
    if !enabled {
        return match std::fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
            _ => Ok(()),
        };
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let text = format!("[Desktop Entry]\nType=Application\nName=IdleWalker\nExec=\"{}\"\nX-GNOME-Autostart-enabled=true\n", exe.display());
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, text).map_err(|e| e.to_string())
}

/// Opens the folder holding a file.
pub fn reveal_file(path: &std::path::Path) {
    if let Some(d) = path.parent() {
        let d = d.display().to_string();
        super::spawn("xdg-open", &[&d]);
    }
}

/// Opens a web address in the default browser.
pub fn open_url(url: &str) {
    super::spawn("xdg-open", &[url]);
}

// ---- Brightness: sysfs for a built-in panel, ddcutil for external monitors -------------------------

fn backlight() -> Option<PathBuf> {
    std::fs::read_dir("/sys/class/backlight")
        .ok()?
        .flatten()
        .map(|e| e.path())
        .next()
}

fn read_num(p: PathBuf) -> Option<f64> {
    std::fs::read_to_string(p).ok()?.trim().parse().ok()
}

pub fn brightness_enumerate() -> Vec<BrightnessInfo> {
    let mut out = Vec::new();
    let mut number = 1;
    if let Some(b) = backlight() {
        if let (Some(cur), Some(max)) = (
            read_num(b.join("brightness")),
            read_num(b.join("max_brightness")),
        ) {
            if max > 0.0 {
                out.push(BrightnessInfo {
                    name: format!("Display {} - Built-in", number),
                    primary: true,
                    supported: true,
                    percent: (cur * 100.0 / max).round() as i32,
                    key: format!("sys:{}", b.display()),
                });
                number += 1;
            }
        }
    }
    // ddcutil is optional: without it external monitors are listed greyed.
    let mut external = 0;
    if let Some(list) = super::output_of("ddcutil", &["detect", "--brief"]) {
        let ids: Vec<String> = list
            .lines()
            .filter_map(|l| {
                l.trim()
                    .strip_prefix("Display ")
                    .map(|n| n.trim().to_string())
            })
            .collect();
        for id in ids {
            let reading =
                super::output_of("ddcutil", &["--display", &id, "getvcp", "10", "--brief"]);
            let percent = reading.and_then(|r| {
                let f: Vec<&str> = r.split_whitespace().collect();
                let (cur, max) = (
                    f.get(3)?.parse::<f64>().ok()?,
                    f.get(4)?.parse::<f64>().ok()?,
                );
                (max > 0.0).then(|| (cur * 100.0 / max).round() as i32)
            });
            if let Some(percent) = percent {
                out.push(BrightnessInfo {
                    name: format!("Display {}", number),
                    primary: out.is_empty(),
                    supported: true,
                    percent,
                    key: format!("ddc:{}", id),
                });
                number += 1;
                external += 1;
            }
        }
    }
    let expected = screens().len();
    while out.len() < expected {
        out.push(BrightnessInfo {
            name: format!("Display {}", number),
            primary: out.is_empty(),
            supported: false,
            percent: 0,
            key: String::new(),
        });
        number += 1;
    }
    let _ = external;
    out
}

pub fn brightness_write(key: &str, percent: i32) {
    if let Some(path) = key.strip_prefix("sys:") {
        let p = PathBuf::from(path);
        if let Some(max) = read_num(p.join("max_brightness")) {
            let raw = (max * percent as f64 / 100.0).round() as i64;
            // Usually needs a udev rule; brightnessctl goes through logind without one.
            if std::fs::write(p.join("brightness"), raw.to_string()).is_err() {
                super::spawn("brightnessctl", &["set", &format!("{}%", percent)]);
            }
        }
    } else if let Some(id) = key.strip_prefix("ddc:") {
        let _ = super::output_of(
            "ddcutil",
            &["--display", id, "setvcp", "10", &percent.to_string()],
        );
    }
}

// ---- The sprite window ---------------------------------------------------------------------------------

fn compositing(x: &X) -> bool {
    x.a_wm_cm != 0
        && x.conn
            .get_selection_owner(x.a_wm_cm)
            .ok()
            .and_then(|r| r.reply().ok())
            .is_some_and(|r| r.owner != 0)
}

/// A borderless, always-on-top, click-through window that shows a `Surface` with
/// per-pixel transparency. Never takes focus.
pub struct SpriteWindow {
    win: Option<Window>,
    gc: Gcontext,
    visible: bool,
    last: (i32, i32, u32, u32),
    bytes: Vec<u8>,
}

impl SpriteWindow {
    /// Nothing hides an X11 window from a capture, so this is the same window.
    pub fn capturable() -> Self {
        Self::new()
    }

    pub fn new() -> Self {
        let mut s = SpriteWindow {
            win: None,
            gc: 0,
            visible: false,
            last: (0, 0, 0, 0),
            bytes: Vec::new(),
        };
        let Some(x) = x() else { return s };
        let Some((visual, cm)) = x.visual else {
            return s;
        };
        if !compositing(x) {
            return s;
        }
        let Ok(win) = x.conn.generate_id() else {
            return s;
        };
        let aux = CreateWindowAux::new()
            .override_redirect(1)
            .border_pixel(0)
            .background_pixel(0)
            .colormap(cm);
        if x.conn
            .create_window(
                32,
                win,
                x.root,
                0,
                0,
                1,
                1,
                0,
                WindowClass::INPUT_OUTPUT,
                visual,
                &aux,
            )
            .is_err()
        {
            return s;
        }
        // An empty input shape: every click falls through to what is underneath.
        let _ = x.conn.shape_rectangles(
            shape::SO::SET,
            shape::SK::INPUT,
            ClipOrdering::UNSORTED,
            win,
            0,
            0,
            &[],
        );
        if let Ok(gc) = x.conn.generate_id() {
            if x.conn.create_gc(gc, win, &CreateGCAux::new()).is_ok() {
                s.gc = gc;
            }
        }
        let _ = x.conn.flush();
        s.win = Some(win);
        s
    }

    /// Shows the surface with its top-left corner at (x, y). Only the part that has
    /// content is uploaded and shown; the window shrinks to fit it.
    pub fn present(&mut self, surface: Surface, px: i32, py: i32) {
        let (Some(win), Some(x)) = (self.win, x()) else {
            return;
        };
        let mut b = surface.bounds();
        if b.is_empty() {
            b = Rectangle::new(0, 0, 1, 1);
        }
        let sw = surface.width();
        self.bytes.clear();
        self.bytes.reserve((b.width * b.height * 4) as usize);
        let pixels = surface.pixels.as_slice();
        for row in b.top()..b.bottom() {
            let start = (row * sw + b.left()) as usize;
            for v in &pixels[start..start + b.width as usize] {
                self.bytes.extend_from_slice(&v.to_le_bytes());
            }
        }
        let geometry = (px + b.left(), py + b.top(), b.width as u32, b.height as u32);
        if geometry != self.last {
            let aux = ConfigureWindowAux::new()
                .x(geometry.0)
                .y(geometry.1)
                .width(geometry.2)
                .height(geometry.3);
            let _ = x.conn.configure_window(win, &aux);
            self.last = geometry;
        }
        if !self.visible {
            let _ = x.conn.map_window(win);
            self.visible = true;
        }
        let _ = x.conn.put_image(
            ImageFormat::Z_PIXMAP,
            win,
            self.gc,
            b.width as u16,
            b.height as u16,
            0,
            0,
            0,
            32,
            &self.bytes,
        );
        let _ = x.conn.flush();
    }

    pub fn stay_on_top(&self) {
        if let (Some(win), Some(x), true) = (self.win, x(), self.visible) {
            let _ = x
                .conn
                .configure_window(win, &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE));
            let _ = x.conn.flush();
        }
    }

    pub fn hide(&mut self) {
        if let (Some(win), Some(x), true) = (self.win, x(), self.visible) {
            let _ = x.conn.unmap_window(win);
            let _ = x.conn.flush();
            self.visible = false;
        }
    }
}

impl Drop for SpriteWindow {
    fn drop(&mut self) {
        if let (Some(win), Some(x)) = (self.win, x()) {
            let _ = x.conn.destroy_window(win);
            let _ = x.conn.flush();
        }
    }
}
