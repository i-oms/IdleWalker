//! The OS backends behind the ports. Each implementation only forwards to the backend
//! compiled for this OS, so the ports stay the one place a capability is described.
use crate::prelude::*;
use crate::{backend, BrightnessInfo, Modifier, MonitorWorld, MouseButtons, Screen, SpriteWindow};
use idlewalker_application::{
    Desktop, Displays, Journal, Os, ReleaseFeed, RunResult, Updates, Workstation,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;

// ---- The desktop -------------------------------------------------------------------------

/// This machine's desktop.
#[derive(Clone, Copy, Default)]
pub struct NativeDesktop;

impl Desktop for NativeDesktop {
    fn cursor_pos(&self) -> Option<Point> {
        backend::cursor_pos()
    }
    fn mouse_button_down(&self) -> bool {
        backend::mouse_button_down()
    }
    fn mouse_buttons(&self) -> MouseButtons {
        backend::mouse_buttons()
    }
    fn modifier_down(&self, key: Modifier) -> bool {
        backend::modifier_down(key)
    }
    fn other_key_down(&self, key: Modifier) -> bool {
        backend::other_key_down(key)
    }
    fn modifier_name(&self, key: Modifier) -> &'static str {
        match key {
            Modifier::Ctrl if cfg!(target_os = "macos") => "Control",
            Modifier::Ctrl => "Ctrl",
            Modifier::Shift => "Shift",
            Modifier::Alt if cfg!(target_os = "macos") => "Option",
            Modifier::Alt => "Alt",
        }
    }
    fn system_idle_seconds(&self) -> f64 {
        backend::system_idle_seconds()
    }
    fn nudge_activity(&self) {
        backend::nudge_activity()
    }
    fn keep_awake(&self, on: bool) {
        backend::keep_awake(on)
    }
    fn is_fullscreen(&self) -> bool {
        backend::is_fullscreen()
    }
    fn foreground_description(&self) -> String {
        backend::foreground_description()
    }
    fn on_battery(&self) -> bool {
        backend::on_battery()
    }
    fn wait_vblank(&self) -> bool {
        backend::wait_vblank()
    }
    fn pacing_idle(&self) {
        backend::pacing_idle()
    }
    #[cfg(target_os = "macos")]
    fn world_per_point(&self) -> f64 {
        backend::world_per_point()
    }
    #[cfg(not(target_os = "macos"))]
    fn world_per_point(&self) -> f64 {
        1.0
    }
    fn corner_action_names(&self) -> [&'static str; 10] {
        backend::CORNER_ACTION_NAMES
    }
    fn perform_corner(&self, action: CornerAction) {
        backend::perform_corner(action)
    }
    fn startup_label(&self) -> &'static str {
        backend::STARTUP_LABEL
    }
    fn startup_is_enabled(&self) -> bool {
        backend::startup_is_enabled()
    }
    fn startup_set(&self, enabled: bool) -> Result<(), String> {
        backend::startup_set(enabled)
    }
    fn open_url(&self, url: &str) {
        backend::open_url(url)
    }
    fn reveal_file(&self, path: &Path) {
        backend::reveal_file(path)
    }
    fn brightness_enumerate(&self) -> Vec<BrightnessInfo> {
        backend::brightness_enumerate()
    }
    fn brightness_write(&self, key: &str, percent: i32) {
        backend::brightness_write(key, percent)
    }
}

// ---- The monitors ------------------------------------------------------------------------

impl Displays for MonitorWorld {
    fn refresh(&self) {
        MonitorWorld::refresh(self)
    }
    fn screens(&self) -> Vec<Screen> {
        MonitorWorld::screens(self)
    }
    fn monitor_bounds(&self) -> Arr<Rectangle> {
        MonitorWorld::monitor_bounds(self)
    }
    fn monitor_scales(&self) -> Arr<i32> {
        MonitorWorld::monitor_scales(self)
    }
    fn screen_at(&self, p: Point) -> Screen {
        MonitorWorld::screen_at(self, p)
    }
    fn world(&self) -> IWorld {
        IWorld::of(*self)
    }
}

// ---- Sprite windows ----------------------------------------------------------------------

impl SpriteSink for SpriteWindow {
    fn present(&mut self, surface: Surface, x: i32, y: i32) {
        SpriteWindow::present(self, surface, x, y)
    }
    fn stay_on_top(&self) {
        SpriteWindow::stay_on_top(self)
    }
    fn hide(&mut self) {
        SpriteWindow::hide(self)
    }
}

/// Makes this OS's sprite windows.
pub fn sprite_factory() -> SpriteFactory {
    Arc::new(|| Box::new(SpriteWindow::new()) as Box<dyn SpriteSink>)
}

/// Makes sprite windows that screenshots, recordings and screen sharing include: the
/// click highlight is there to be seen by whoever is watching.
pub fn capturable_sprite_factory() -> SpriteFactory {
    Arc::new(|| Box::new(SpriteWindow::capturable()) as Box<dyn SpriteSink>)
}

// ---- The local clock ---------------------------------------------------------------------

/// Local time as the OS keeps it.
pub struct SystemClock;

impl rt::LocalClock for SystemClock {
    fn utc_offset_seconds(&self) -> i64 {
        crate::utc_offset_seconds()
    }
}

// ---- Releases ----------------------------------------------------------------------------

/// GitHub's releases, reached with curl: following the "latest release" redirect needs
/// neither the rate-limited API nor a JSON parser.
pub struct CurlReleaseFeed;

impl ReleaseFeed for CurlReleaseFeed {
    fn latest_release_address(&self) -> Option<String> {
        crate::final_address(&Updates::latest_release_link())
    }

    fn download_file_name(&self, version: &str) -> String {
        if cfg!(windows) {
            format!("idlewalker-{version}-windows-x64.zip")
        } else if cfg!(target_os = "macos") {
            format!("idlewalker-{version}-macos.zip")
        } else {
            format!("idlewalker-{version}-linux-x64.zip")
        }
    }
}

// ---- The workstation ---------------------------------------------------------------------

/// This machine's shells, programs and files.
#[derive(Clone, Copy, Default)]
pub struct NativeWorkstation;

impl Workstation for NativeWorkstation {
    fn os(&self) -> Os {
        if cfg!(windows) {
            Os::Windows
        } else if cfg!(target_os = "macos") {
            Os::MacOs
        } else {
            Os::Linux
        }
    }
    fn home_dir(&self) -> Option<PathBuf> {
        let name = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
        std::env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    }
    fn temp_dir(&self) -> PathBuf {
        std::env::temp_dir()
    }
    fn app_dir(&self) -> PathBuf {
        crate::config_dir().unwrap_or_else(|| std::env::temp_dir().join("IdleWalker"))
    }
    fn env(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
    fn find_program(&self, name: &str) -> Option<PathBuf> {
        crate::find_program(name)
    }
    fn run(&self, program: &str, args: &[&str]) -> RunResult {
        let mut c = std::process::Command::new(program);
        c.args(args).stdin(std::process::Stdio::null());
        backend::hide_console(&mut c);
        match c.output() {
            Ok(o) => RunResult {
                success: o.status.success(),
                stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
            },
            Err(e) => RunResult {
                success: false,
                stdout: String::new(),
                stderr: format!("{program}: {e}"),
            },
        }
    }
    fn read_text(&self, path: &Path) -> Option<String> {
        String::from_utf8(std::fs::read(path).ok()?).ok()
    }
    fn write_text(&self, path: &Path, text: &str) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, text).map_err(|e| e.to_string())
    }
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
    fn make_dir(&self, path: &Path) -> Result<(), String> {
        std::fs::create_dir_all(path).map_err(|e| e.to_string())
    }
    fn remove_file(&self, path: &Path) {
        let _ = std::fs::remove_file(path);
    }
    fn remove_dir(&self, path: &Path) -> Result<(), String> {
        std::fs::remove_dir_all(path).map_err(|e| e.to_string())
    }
    fn list_files(&self, dir: &Path, depth: u32) -> Vec<PathBuf> {
        let mut out = Vec::new();
        list_files_into(dir, depth, &mut out);
        out
    }
    fn final_address(&self, url: &str) -> Option<String> {
        crate::final_address(url)
    }
}

fn list_files_into(dir: &Path, depth: u32, out: &mut Vec<PathBuf>) {
    if depth == 0 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            list_files_into(&p, depth - 1, out);
        } else {
            out.push(p);
        }
    }
}

// ---- The journal -------------------------------------------------------------------------

/// One line per note in the temp folder: corner fires, full-screen transitions and the
/// like, so an open-close-open report can be settled by counting. Notes are human-paced.
pub struct TempFileJournal;

impl Journal for TempFileJournal {
    fn note(&self, note: &str) {
        use std::io::Write;
        let path = std::env::temp_dir().join("IdleWalker-hotcorners.log");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let d = DateTime::now();
            let _ = writeln!(f, "{:02}:{:02} {}", d.hour(), d.minute(), note);
        }
    }
}
