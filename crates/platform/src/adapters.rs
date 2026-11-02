//! The OS backends behind the ports. Each implementation only forwards to the backend
//! compiled for this OS, so the ports stay the one place a capability is described.
use crate::prelude::*;
use crate::{backend, BrightnessInfo, MonitorWorld, Screen, SpriteWindow};
use idlewalker_application::{Desktop, Displays, Journal, ReleaseFeed, Updates};
use std::path::Path;
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
        let url = Updates::latest_release_link();
        let args = [
            "--fail",
            "--silent",
            "--head",
            "--location",
            "--max-time",
            "20",
            "--write-out",
            "\\n%{url_effective}\\n",
            url.as_str(),
        ];
        let out = crate::output_of("curl", &args)?;
        let last = out.lines().rev().find(|l| !l.trim().is_empty());
        Some(last.unwrap_or("").to_string())
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
