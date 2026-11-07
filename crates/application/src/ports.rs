//! The ports: everything the use cases need from the world outside, declared here and
//! implemented by the platform crate. Nothing in this crate knows which OS it runs on,
//! where files live or how the network is reached.
use idlewalker_domain::prelude::*;
use std::path::PathBuf;

// ---- Data crossing the ports ------------------------------------------------------------

/// One monitor, in world pixels: y grows downward from the top of the primary display.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Screen {
    pub bounds: Rectangle,
    /// The part not taken by the taskbar, dock or panels.
    pub work: Rectangle,
    /// Display scaling: 1 at 100%.
    pub dpi_scale: f64,
    pub primary: bool,
}

impl Screen {
    /// Sprite cell size for this monitor, following its display scaling.
    pub fn cell(&self) -> i32 {
        ((Settings::BASE_PIXEL_SIZE as f64 * self.dpi_scale).round_ties_even() as i32).max(2)
    }
}

/// A display whose backlight can (perhaps) be set.
#[derive(Clone, Debug, Default)]
pub struct BrightnessInfo {
    pub name: String,
    pub primary: bool,
    pub supported: bool,
    pub percent: i32,
    /// What the adapter needs to find it again.
    pub key: String,
}

// ---- The desktop -------------------------------------------------------------------------

/// The OS services the program uses. Every method is cheap unless it says otherwise,
/// and may be called from any thread.
pub trait Desktop: Send + Sync {
    /// Where the cursor is, in world pixels; `None` when the OS will not say.
    fn cursor_pos(&self) -> Option<Point>;
    fn mouse_button_down(&self) -> bool;
    /// Seconds since the last input the OS counted, from anyone.
    fn system_idle_seconds(&self) -> f64;
    /// A harmless input that resets the OS idle clock.
    fn nudge_activity(&self);
    /// Holds off sleep and the screen saver while on.
    fn keep_awake(&self, on: bool);
    /// Whether the foreground app covers its whole monitor.
    fn is_fullscreen(&self) -> bool;
    /// What is in front, for the log.
    fn foreground_description(&self) -> String;
    fn on_battery(&self) -> bool;

    /// Blocks until the next screen refresh; false where there is no such wait.
    fn wait_vblank(&self) -> bool;
    /// Called by the frame pump while idle, between polls.
    fn pacing_idle(&self);
    /// World pixels per point: differs from a window's own scale only on macOS.
    fn world_per_point(&self) -> f64;

    /// The menu's name for each `CornerAction`, in this OS's words.
    fn corner_action_names(&self) -> [&'static str; 10];
    fn perform_corner(&self, action: CornerAction);

    /// The menu's name for "start at sign-in", in this OS's words.
    fn startup_label(&self) -> &'static str;
    fn startup_is_enabled(&self) -> bool;
    fn startup_set(&self, enabled: bool) -> Result<(), String>;

    fn open_url(&self, url: &str);
    /// Shows a file in the OS file manager.
    fn reveal_file(&self, path: &std::path::Path);

    /// Slow: reads every display, DDC/CI included. Off the UI thread.
    fn brightness_enumerate(&self) -> Vec<BrightnessInfo>;
    /// Slow: off the UI thread.
    fn brightness_write(&self, key: &str, percent: i32);
}

/// The monitor layout. Read once per frame by `refresh`; every query after that is a
/// lookup. Lives on the UI thread.
pub trait Displays {
    fn refresh(&self);
    fn screens(&self) -> Vec<Screen>;
    fn monitor_bounds(&self) -> Arr<Rectangle>;
    fn monitor_scales(&self) -> Arr<i32>;
    fn screen_at(&self, p: Point) -> Screen;
    /// The same layout as the cast sees it.
    fn world(&self) -> IWorld;
}

// ---- Storage -----------------------------------------------------------------------------

/// Named whole numbers that survive between runs.
pub trait PreferenceStore: Send + Sync {
    /// `None` when the value was never saved.
    fn load(&self, name: &str) -> Option<i32>;
    /// Not worth interrupting anyone over: on failure the value just won't stick.
    fn save(&self, name: &str, value: i32);
}

/// The one text file holding what the walkers learned.
pub trait BrainStore: Send + Sync {
    fn read(&self) -> Option<String>;
    /// Atomic: a crash never leaves a half-written file behind.
    fn write(&self, text: &str);
    fn delete(&self);
    /// Size in bytes; -1 when nothing is saved yet.
    fn size(&self) -> i64;
    /// Where it lives, to show it in the file manager.
    fn location(&self) -> Option<PathBuf>;
}

// ---- The workstation ---------------------------------------------------------------------

/// Which operating system the workstation runs, for the use cases that must do things
/// differently on each: they ask the port instead of the compiler, so every plan can be
/// tested on any OS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Os {
    Windows,
    MacOs,
    Linux,
}

/// What a program did when run to completion.
#[derive(Clone, Debug, Default)]
pub struct RunResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// The person's shells, programs and files, as the terminal-prompt use case needs them.
/// Every method may be called from any thread; `run` and `final_address` are slow.
pub trait Workstation: Send + Sync {
    fn os(&self) -> Os;
    fn home_dir(&self) -> Option<PathBuf>;
    fn temp_dir(&self) -> PathBuf;
    /// This program's own folder, for what it must remember between runs.
    fn app_dir(&self) -> PathBuf;
    fn env(&self, name: &str) -> Option<String>;
    /// A program on the PATH, as the shells see it.
    fn find_program(&self, name: &str) -> Option<PathBuf>;
    /// Runs a program and waits for it. Slow: off the UI thread.
    fn run(&self, program: &str, args: &[&str]) -> RunResult;
    /// `None` when the file is missing or not text.
    fn read_text(&self, path: &std::path::Path) -> Option<String>;
    /// Creates the folders on the way.
    fn write_text(&self, path: &std::path::Path, text: &str) -> Result<(), String>;
    fn exists(&self, path: &std::path::Path) -> bool;
    fn make_dir(&self, path: &std::path::Path) -> Result<(), String>;
    fn remove_file(&self, path: &std::path::Path);
    /// A folder and everything in it.
    fn remove_dir(&self, path: &std::path::Path) -> Result<(), String>;
    /// The files under a folder, `depth` levels deep (1 is the folder itself).
    fn list_files(&self, dir: &std::path::Path, depth: u32) -> Vec<PathBuf>;
    /// The address a redirecting link ends at. Slow, and needs the network.
    fn final_address(&self, url: &str) -> Option<String>;
}

// ---- The network -------------------------------------------------------------------------

/// Where releases are published.
pub trait ReleaseFeed: Send + Sync {
    /// The address the "latest release" link ends up at. Slow, and needs the network.
    fn latest_release_address(&self) -> Option<String>;
    /// The download this OS needs for a version, as the release workflow names it.
    fn download_file_name(&self, version: &str) -> String;
}

// ---- Diagnostics -------------------------------------------------------------------------

/// A human-paced log, so an odd report can be settled by reading it.
pub trait Journal: Send + Sync {
    fn note(&self, note: &str);
}
