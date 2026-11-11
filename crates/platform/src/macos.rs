//! macOS: borderless NSWindows that ignore the mouse, with the frame in a CALayer.
//!
//! World pixels here are points times the largest backing scale of any display, so a
//! Retina screen gets full-resolution sprites. AppKit's y axis points up from the
//! primary display's bottom edge; everything is converted at this boundary.
#![allow(non_upper_case_globals)]
use super::{BrightnessInfo, Modifier, MouseButtons, Screen};
use crate::prelude::*;
use objc2::runtime::AnyObject;
use objc2::{class, msg_send};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use std::ffi::{c_void, CString};
use std::path::PathBuf;

type Id = *mut AnyObject;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGColorSpaceCreateDeviceRGB() -> *mut c_void;
    fn CGDataProviderCreateWithData(
        info: *mut c_void,
        data: *const c_void,
        size: usize,
        release: Option<unsafe extern "C" fn(*mut c_void, *const c_void, usize)>,
    ) -> *mut c_void;
    fn CGDataProviderRelease(p: *mut c_void);
    fn CGImageCreate(
        w: usize,
        h: usize,
        bpc: usize,
        bpp: usize,
        row: usize,
        space: *mut c_void,
        info: u32,
        provider: *mut c_void,
        decode: *const f64,
        interpolate: bool,
        intent: i32,
    ) -> *mut c_void;
    fn CGImageRelease(i: *mut c_void);
    fn CGEventSourceSecondsSinceLastEventType(state: i32, kind: u32) -> f64;
    fn CGEventSourceCounterForEventType(state: i32, kind: u32) -> u32;
    fn CGEventCreateKeyboardEvent(src: *mut c_void, key: u16, down: bool) -> *mut c_void;
    fn CGEventSetFlags(e: *mut c_void, flags: u64);
    fn CGEventPost(tap: u32, e: *mut c_void);
}
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(o: *const c_void);
    fn CFStringGetCString(
        s: *const c_void,
        out: *mut std::ffi::c_char,
        size: isize,
        enc: u32,
    ) -> u8;
}
#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOPSCopyPowerSourcesInfo() -> *const c_void;
    fn IOPSGetProvidingPowerSourceType(snapshot: *const c_void) -> *const c_void;
}
#[link(name = "CoreVideo", kind = "framework")]
extern "C" {
    fn CVDisplayLinkCreateWithActiveCGDisplays(link: *mut *mut c_void) -> i32;
    fn CVDisplayLinkSetOutputCallback(
        link: *mut c_void,
        callback: unsafe extern "C" fn(
            *mut c_void,
            *const c_void,
            *const c_void,
            u64,
            *mut u64,
            *mut c_void,
        ) -> i32,
        context: *mut c_void,
    ) -> i32;
    fn CVDisplayLinkStart(link: *mut c_void) -> i32;
    fn CVDisplayLinkStop(link: *mut c_void) -> i32;
}
#[link(name = "AppKit", kind = "framework")]
extern "C" {}
#[link(name = "QuartzCore", kind = "framework")]
extern "C" {}

const kCGBitmapByteOrder32Little: u32 = 2 << 12;
const kCGImageAlphaPremultipliedFirst: u32 = 2;
const kCGEventFlagMaskControl: u64 = 0x40000;
const kCGEventFlagMaskCommand: u64 = 0x100000;
const kCGEventFlagMaskSecondaryFn: u64 = 0x800000;

pub(crate) fn hide_console(_c: &mut std::process::Command) {}

// ---- The process -------------------------------------------------------------------

pub fn init_process() {}

/// False when another copy is already running; the lock is held until the process ends.
pub fn single_instance() -> bool {
    flock_instance()
}

fn flock_instance() -> bool {
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

/// Below the UI thread, but not so low that Apple Silicon parks the painter on its
/// efficiency cores: at Retina size a frame is four times the pixels, and a utility-class
/// thread there falls behind the display.
pub fn lower_thread_priority() {
    extern "C" {
        fn pthread_set_qos_class_self_np(class: u32, rel: i32) -> i32;
    }
    unsafe { pthread_set_qos_class_self_np(0x19, 0) }; // QOS_CLASS_USER_INITIATED
}

pub fn config_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(|h| PathBuf::from(h).join("Library/Application Support/IdleWalker"))
}

pub fn legacy_setting(_name: &str) -> Option<i32> {
    None
}

// ---- Monitors and the cursor ----------------------------------------------------------

#[derive(Clone, Copy)]
struct Mac {
    /// World pixels per point.
    factor: f64,
    /// Height of the primary display, in points: where the y axis flips.
    primary_h: f64,
}

type ScreenList = Vec<(NSRect, NSRect, f64)>;

/// How long a reading of the display layout is reused. Asking AppKit is a round trip
/// to the window server, and it was asked several times a frame; a plugged-in monitor
/// or a resolution change still shows within this.
const SCREENS_TTL: std::time::Duration = std::time::Duration::from_millis(500);

thread_local! {
    static SCREENS: std::cell::RefCell<Option<(std::time::Instant, Mac, std::rc::Rc<ScreenList>)>> =
        const { std::cell::RefCell::new(None) };
}

fn read_screens() -> (Mac, ScreenList) {
    objc2::rc::autoreleasepool(|_| unsafe {
        let list: Id = msg_send![class!(NSScreen), screens];
        let n: usize = msg_send![list, count];
        let mut out = Vec::with_capacity(n);
        let mut factor: f64 = 1.0;
        for i in 0..n {
            let s: Id = msg_send![list, objectAtIndex: i];
            let frame: NSRect = msg_send![s, frame];
            let visible: NSRect = msg_send![s, visibleFrame];
            let scale: f64 = msg_send![s, backingScaleFactor];
            factor = factor.max(scale);
            out.push((frame, visible, scale));
        }
        let primary_h = out.first().map_or(0.0, |f| f.0.size.height);
        (Mac { factor, primary_h }, out)
    })
}

fn mac_screens() -> (Mac, std::rc::Rc<ScreenList>) {
    SCREENS.with(|cell| {
        let mut cache = cell.borrow_mut();
        if let Some((at, m, list)) = cache.as_ref() {
            if at.elapsed() < SCREENS_TTL {
                return (*m, list.clone());
            }
        }
        let (m, list) = read_screens();
        let list = std::rc::Rc::new(list);
        *cache = Some((std::time::Instant::now(), m, list.clone()));
        (m, list)
    })
}

/// World pixels per point. Unlike a window's own scale, this is the same on every display.
pub fn world_per_point() -> f64 {
    mac_screens().0.factor
}

fn to_world(m: &Mac, r: NSRect) -> Rectangle {
    let top = m.primary_h - (r.origin.y + r.size.height);
    Rectangle::new(
        (r.origin.x * m.factor).round() as i32,
        (top * m.factor).round() as i32,
        (r.size.width * m.factor).round() as i32,
        (r.size.height * m.factor).round() as i32,
    )
}

pub fn screens() -> Vec<Screen> {
    let (m, list) = mac_screens();
    list.iter()
        .enumerate()
        .map(|(i, (frame, visible, _))| Screen {
            bounds: to_world(&m, *frame),
            work: to_world(&m, *visible),
            // A cell is the same physical size as at 100% on a Windows display of this density.
            dpi_scale: m.factor,
            primary: i == 0,
        })
        .collect()
}

pub fn cursor_pos() -> Option<Point> {
    let (m, _) = mac_screens();
    let p: NSPoint = unsafe { msg_send![class!(NSEvent), mouseLocation] };
    Some(Point::new(
        (p.x * m.factor).round() as i32,
        ((m.primary_h - p.y) * m.factor).round() as i32,
    ))
}

pub fn mouse_button_down() -> bool {
    let b: usize = unsafe { msg_send![class!(NSEvent), pressedMouseButtons] };
    b != 0
}

/// Held, or clicked since the last call: the session counts every button press, so a
/// click that came and went between two polls still shows as a count that moved.
pub fn mouse_buttons() -> MouseButtons {
    static COUNTS: std::sync::Mutex<Option<(u32, u32)>> = std::sync::Mutex::new(None);
    let held: usize = unsafe { msg_send![class!(NSEvent), pressedMouseButtons] };
    // kCGEventLeftMouseDown and kCGEventRightMouseDown, in the combined session state.
    let now = unsafe {
        (
            CGEventSourceCounterForEventType(0, 1),
            CGEventSourceCounterForEventType(0, 3),
        )
    };
    let mut counts = COUNTS.lock().unwrap_or_else(|e| e.into_inner());
    let before = counts.replace(now).unwrap_or(now);
    MouseButtons {
        left: held & 1 != 0 || now.0 != before.0,
        right: held & 2 != 0 || now.1 != before.1,
    }
}

/// The modifier keys held right now, as NSEvent's flags. Needs no permission.
fn modifier_flags() -> usize {
    unsafe { msg_send![class!(NSEvent), modifierFlags] }
}

/// NSEvent's flag for a modifier.
fn modifier_flag(key: Modifier) -> usize {
    match key {
        Modifier::Shift => 1 << 17,
        Modifier::Ctrl => 1 << 18,
        Modifier::Alt => 1 << 19, // Option
    }
}

pub fn modifier_down(key: Modifier) -> bool {
    modifier_flags() & modifier_flag(key) != 0
}

/// Another modifier held (Command too), or any key pressed since the last call: the session
/// counts key presses, so one shows as a count that moved. A reading older than a second
/// is not compared, since the keys typed before anyone was asking are not news.
pub fn other_key_down(key: Modifier) -> bool {
    static SEEN: std::sync::Mutex<Option<(std::time::Instant, u32)>> = std::sync::Mutex::new(None);
    let all = (1 << 17) | (1 << 18) | (1 << 19) | (1 << 20);
    let modifiers = modifier_flags() & all & !modifier_flag(key) != 0;
    let count = unsafe { CGEventSourceCounterForEventType(0, 10) }; // kCGEventKeyDown
    let mut seen = SEEN.lock().unwrap_or_else(|e| e.into_inner());
    let before = seen.replace((std::time::Instant::now(), count));
    let typed = before.is_some_and(|(at, n)| at.elapsed().as_secs_f64() < 1.0 && n != count);
    modifiers || typed
}

// ---- Pacing and power --------------------------------------------------------------------

/// The display link: a CoreVideo callback at every refresh of the active display,
/// which the frame pump waits on. Started on the first wait and stopped while idle, so a
/// cast that is away costs no wake-ups.
struct Vsync {
    link: usize,
    running: bool,
}

/// The number of refreshes seen so far.
static VSYNC_FRAMES: (std::sync::Mutex<u64>, std::sync::Condvar) =
    (std::sync::Mutex::new(0), std::sync::Condvar::new());
static VSYNC: std::sync::Mutex<Option<Vsync>> = std::sync::Mutex::new(None);

unsafe extern "C" fn on_refresh(
    _link: *mut c_void,
    _now: *const c_void,
    _out: *const c_void,
    _flags: u64,
    _flags_out: *mut u64,
    _context: *mut c_void,
) -> i32 {
    let (count, signal) = &VSYNC_FRAMES;
    *count.lock().unwrap_or_else(|e| e.into_inner()) += 1;
    signal.notify_all();
    0
}

/// Blocks until the display's next refresh; false if there is no display link (or the
/// display is asleep), and the pump then paces by the clock.
pub fn wait_vblank() -> bool {
    {
        let mut v = VSYNC.lock().unwrap_or_else(|e| e.into_inner());
        if v.is_none() {
            let mut link: *mut c_void = std::ptr::null_mut();
            let ok = unsafe {
                CVDisplayLinkCreateWithActiveCGDisplays(&mut link) == 0
                    && !link.is_null()
                    && CVDisplayLinkSetOutputCallback(link, on_refresh, std::ptr::null_mut()) == 0
            };
            // A failed link is remembered as null, so it is not retried every frame.
            *v = Some(Vsync {
                link: if ok { link as usize } else { 0 },
                running: false,
            });
        }
        let Some(s) = v.as_mut() else { return false };
        if s.link == 0 {
            return false;
        }
        if !s.running {
            s.running = unsafe { CVDisplayLinkStart(s.link as *mut c_void) } == 0;
            if !s.running {
                return false;
            }
        }
    }
    let (count, signal) = &VSYNC_FRAMES;
    let guard = count.lock().unwrap_or_else(|e| e.into_inner());
    let seen = *guard;
    let (guard, timeout) = signal
        .wait_timeout_while(guard, std::time::Duration::from_millis(50), |c| *c == seen)
        .unwrap_or_else(|e| e.into_inner());
    drop(guard);
    !timeout.timed_out()
}

/// The pump has nothing to animate: let the display link rest.
pub fn pacing_idle() {
    let mut v = VSYNC.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(s) = v.as_mut() {
        if s.running {
            unsafe { CVDisplayLinkStop(s.link as *mut c_void) };
            s.running = false;
        }
    }
}

/// Whether the Mac runs on its battery, from IOKit's power sources: no process launch,
/// so it is cheap enough to ask on the UI thread. Cached briefly all the same.
pub fn on_battery() -> bool {
    use std::sync::Mutex;
    use std::time::Instant;
    static CACHE: Mutex<Option<(Instant, bool)>> = Mutex::new(None);
    let mut c = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((at, v)) = *c {
        if at.elapsed().as_secs() < 5 {
            return v;
        }
    }
    let v = unsafe {
        let snapshot = IOPSCopyPowerSourcesInfo();
        if snapshot.is_null() {
            false
        } else {
            // Borrowed from the snapshot: read before it is released.
            let kind = IOPSGetProvidingPowerSourceType(snapshot);
            let mut text = [0 as std::ffi::c_char; 32];
            let on = !kind.is_null()
                && CFStringGetCString(kind, text.as_mut_ptr(), 32, 0x0800_0100) != 0
                && std::ffi::CStr::from_ptr(text.as_ptr()).to_bytes() == b"Battery Power";
            CFRelease(snapshot);
            on
        }
    };
    *c = Some((Instant::now(), v));
    v
}

// ---- Keeping the machine awake -----------------------------------------------------------------

/// `caffeinate` tied to our pid, so it cannot outlive us.
pub fn keep_awake(on: bool) {
    use std::sync::Mutex;
    static CHILD: Mutex<Option<std::process::Child>> = Mutex::new(None);
    let mut child = CHILD.lock().unwrap();
    if let Some(mut c) = child.take() {
        let _ = c.kill();
        let _ = c.wait();
    }
    if on {
        *child = std::process::Command::new("caffeinate")
            .args(["-d", "-i", "-w", &std::process::id().to_string()])
            .spawn()
            .ok();
    }
}

pub fn system_idle_seconds() -> f64 {
    unsafe { CGEventSourceSecondsSinceLastEventType(0, u32::MAX) }
}

/// `caffeinate -d` already holds off the screen saver and display sleep.
pub fn nudge_activity() {}

// ---- The foreground window ------------------------------------------------------------------

/// True while the active app has taken the screen: the system reports the full-screen
/// presentation option then, which a merely maximised window does not set.
pub fn is_fullscreen() -> bool {
    unsafe {
        let app: Id = msg_send![class!(NSApplication), sharedApplication];
        let options: usize = msg_send![app, currentSystemPresentationOptions];
        options & (1 << 10) != 0
    }
}

pub fn foreground_description() -> String {
    unsafe {
        let ws: Id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let app: Id = msg_send![ws, frontmostApplication];
        if app.is_null() {
            return "(none)".into();
        }
        let name: Id = msg_send![app, localizedName];
        if name.is_null() {
            return "(unnamed)".into();
        }
        let utf8: *const std::os::raw::c_char = msg_send![name, UTF8String];
        std::ffi::CStr::from_ptr(utf8)
            .to_string_lossy()
            .into_owned()
    }
}

// ---- Hot corners ---------------------------------------------------------------------------------

pub const CORNER_ACTION_NAMES: [&str; 10] = [
    "Nothing",
    "Mission Control",
    "Show desktop",
    "Launchpad",
    "Notification Center",
    "Control Center",
    "Lock screen",
    "Screen saver",
    "Display sleep",
    "Call the Minions",
];

/// A synthetic key press. macOS asks once for the Accessibility permission; without it
/// the press does nothing and the other actions still work.
fn key(code: u16, flags: u64) {
    unsafe {
        for down in [true, false] {
            let e = CGEventCreateKeyboardEvent(std::ptr::null_mut(), code, down);
            if e.is_null() {
                return;
            }
            CGEventSetFlags(e, flags);
            CGEventPost(0, e);
            CFRelease(e);
        }
    }
}

fn click_menu_extra(name: &str) {
    let script = format!(
        "tell application \"System Events\" to tell process \"ControlCenter\" to click (first menu bar item of menu bar 1 whose description contains \"{}\")",
        name
    );
    super::spawn("osascript", &["-e", &script]);
}

/// Carries out a corner's action the way the keyboard shortcut would.
pub fn perform_corner(action: CornerAction) {
    match action {
        CornerAction::TaskView => {
            super::spawn("open", &["-a", "Mission Control"]);
        }
        CornerAction::ShowDesktop => key(103, kCGEventFlagMaskSecondaryFn), // F11
        CornerAction::StartMenu => {
            super::spawn("open", &["-a", "Launchpad"]);
        }
        CornerAction::Notifications => click_menu_extra("Clock"),
        CornerAction::QuickSettings => click_menu_extra("Control Centre"),
        CornerAction::LockScreen => key(12, kCGEventFlagMaskControl | kCGEventFlagMaskCommand), // Ctrl+Cmd+Q
        CornerAction::ScreenSaver => {
            super::spawn("open", &["-a", "ScreenSaverEngine"]);
        }
        CornerAction::MonitorOff => {
            super::spawn("pmset", &["displaysleepnow"]);
        }
        _ => {}
    }
}

// ---- Starting at sign-in ----------------------------------------------------------------------------

pub const STARTUP_LABEL: &str = "Start at login";

fn agent_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(|h| PathBuf::from(h).join("Library/LaunchAgents/com.idlewalker.app.plist"))
}

pub fn startup_is_enabled() -> bool {
    agent_path().is_some_and(|p| p.exists())
}

pub fn startup_set(enabled: bool) -> Result<(), String> {
    let path = agent_path().ok_or("no home folder")?;
    if !enabled {
        return match std::fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
            _ => Ok(()),
        };
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe = exe
        .display()
        .to_string()
        .replace('&', "&amp;")
        .replace('<', "&lt;");
    let plist = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\"><dict>\n<key>Label</key><string>com.idlewalker.app</string>\n<key>ProgramArguments</key><array><string>{}</string></array>\n<key>RunAtLoad</key><true/>\n</dict></plist>\n",
        exe
    );
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, plist).map_err(|e| e.to_string())
}

/// Shows a file in Finder, selected.
pub fn reveal_file(path: &std::path::Path) {
    let p = path.display().to_string();
    super::spawn("open", &["-R", &p]);
}

/// Opens a web address in the default browser.
pub fn open_url(url: &str) {
    super::spawn("open", &[url]);
}

// ---- Brightness ------------------------------------------------------------------------------------

type GetB = unsafe extern "C" fn(u32, *mut f32) -> i32;
type SetB = unsafe extern "C" fn(u32, f32) -> i32;

/// The built-in panel through DisplayServices, which has no public equivalent. It is a
/// private framework, looked up at run time: if a macOS release drops it the slider
/// greys out. External monitors go through DDC/CI, in `ddc` below.
fn display_services() -> Option<(GetB, SetB)> {
    unsafe {
        let lib = libc::dlopen(
            c"/System/Library/PrivateFrameworks/DisplayServices.framework/DisplayServices".as_ptr(),
            libc::RTLD_LAZY,
        );
        if lib.is_null() {
            return None;
        }
        let g = libc::dlsym(lib, c"DisplayServicesGetBrightness".as_ptr());
        let s = libc::dlsym(lib, c"DisplayServicesSetBrightness".as_ptr());
        if g.is_null() || s.is_null() {
            return None;
        }
        Some((
            std::mem::transmute::<*mut c_void, GetB>(g),
            std::mem::transmute::<*mut c_void, SetB>(s),
        ))
    }
}

extern "C" {
    fn CGDisplayIsBuiltin(d: u32) -> u32;
    fn CGMainDisplayID() -> u32;
    #[allow(clippy::too_many_arguments)]
    fn CGSetDisplayTransferByFormula(
        d: u32,
        red_min: f32,
        red_max: f32,
        red_gamma: f32,
        green_min: f32,
        green_max: f32,
        green_gamma: f32,
        blue_min: f32,
        blue_max: f32,
        blue_gamma: f32,
    ) -> i32;
}

/// The level each software-dimmed monitor was last set to. macOS undoes the dimming when
/// the program ends, so every display starts a run at 100%.
static DIMMED: std::sync::Mutex<Vec<(u32, i32)>> = std::sync::Mutex::new(Vec::new());

fn dimmed_percent(id: u32) -> i32 {
    let dimmed = DIMMED.lock().unwrap_or_else(|e| e.into_inner());
    dimmed.iter().find(|d| d.0 == id).map_or(100, |d| d.1)
}

/// Dims a monitor that cannot be reached over DDC/CI by scaling its colour output: the
/// backlight stays as it is. Never fully black, or the slider could not be seen to undo it.
fn dim_write(id: u32, percent: i32) {
    let percent = percent.clamp(0, 100);
    let top = 0.1 + 0.9 * percent as f32 / 100.0;
    let status =
        unsafe { CGSetDisplayTransferByFormula(id, 0.0, top, 1.0, 0.0, top, 1.0, 0.0, top, 1.0) };
    if status == 0 {
        let mut dimmed = DIMMED.lock().unwrap_or_else(|e| e.into_inner());
        dimmed.retain(|d| d.0 != id);
        dimmed.push((id, percent));
    }
}

/// Display ids in `NSScreen` order, the order the on-screen "Display N" cards use, so a
/// slider and its card always agree.
fn display_ids() -> Vec<u32> {
    objc2::rc::autoreleasepool(|_| unsafe {
        let key: Id = msg_send![class!(NSString), stringWithUTF8String: c"NSScreenNumber".as_ptr()];
        let list: Id = msg_send![class!(NSScreen), screens];
        let n: usize = msg_send![list, count];
        (0..n)
            .filter_map(|i| {
                let s: Id = msg_send![list, objectAtIndex: i];
                let description: Id = msg_send![s, deviceDescription];
                let number: Id = msg_send![description, objectForKey: key];
                if number.is_null() {
                    return None;
                }
                let id: u32 = msg_send![number, unsignedIntValue];
                Some(id)
            })
            .collect()
    })
}

pub fn brightness_enumerate() -> Vec<BrightnessInfo> {
    let services = display_services();
    let ids = display_ids();
    let main = unsafe { CGMainDisplayID() };
    let external: Vec<u32> = ids
        .iter()
        .copied()
        .filter(|&id| unsafe { CGDisplayIsBuiltin(id) } == 0)
        .collect();
    let levels = ddc::refresh(&external);
    ids.iter()
        .enumerate()
        .map(|(i, &id)| {
            let label = format!("Display {}", i + 1);
            let primary = id == main;
            if let Some(&(_, percent)) = levels.iter().find(|(d, _)| *d == id) {
                return BrightnessInfo {
                    name: label,
                    primary,
                    supported: true,
                    percent,
                    key: format!("ddc:{}", id),
                };
            }
            let builtin = unsafe { CGDisplayIsBuiltin(id) } != 0;
            let mut value = 0f32;
            if let (true, Some((get, _))) = (builtin, services) {
                if unsafe { get(id, &mut value) } == 0 {
                    return BrightnessInfo {
                        name: format!("{} - Built-in", label),
                        primary,
                        supported: true,
                        percent: (value * 100.0).round() as i32,
                        key: id.to_string(),
                    };
                }
            }
            if !builtin {
                return BrightnessInfo {
                    name: format!("{} - Software dimming", label),
                    primary,
                    supported: true,
                    percent: dimmed_percent(id),
                    key: format!("dim:{}", id),
                };
            }
            BrightnessInfo {
                name: label,
                primary,
                supported: false,
                percent: 0,
                key: String::new(),
            }
        })
        .collect()
}

pub fn brightness_write(key: &str, percent: i32) {
    if let Some(id) = key.strip_prefix("ddc:").and_then(|k| k.parse::<u32>().ok()) {
        ddc::write(id, percent);
    } else if let Some(id) = key.strip_prefix("dim:").and_then(|k| k.parse::<u32>().ok()) {
        dim_write(id, percent);
    } else if let (Ok(id), Some((_, set))) = (key.parse::<u32>(), display_services()) {
        unsafe { set(id, percent as f32 / 100.0) };
    }
}

/// Intel Macs reach monitors by another route, which is not implemented: their external
/// monitors get software dimming.
#[cfg(not(target_arch = "aarch64"))]
mod ddc {
    pub fn refresh(_displays: &[u32]) -> Vec<(u32, i32)> {
        Vec::new()
    }
    pub fn write(_display: u32, _percent: i32) {}
}

/// External monitors on Apple Silicon: DDC/CI through IOAVService, private IOKit calls
/// looked up at run time. The same bytes and timings as MonitorControl and m1ddc.
#[cfg(target_arch = "aarch64")]
mod ddc {
    use std::ffi::{c_char, c_void, CStr};
    use std::sync::Mutex;
    use std::time::Duration;

    type Create = unsafe extern "C" fn(*const c_void, u32) -> *mut c_void;
    type ReadI2c = unsafe extern "C" fn(*mut c_void, u32, u32, *mut c_void, u32) -> i32;
    type WriteI2c = unsafe extern "C" fn(*mut c_void, u32, u32, *const c_void, u32) -> i32;

    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IORegistryGetRootEntry(main_port: u32) -> u32;
        fn IORegistryEntryCreateIterator(
            entry: u32,
            plane: *const c_char,
            options: u32,
            iterator: *mut u32,
        ) -> i32;
        fn IOIteratorNext(iterator: u32) -> u32;
        fn IOObjectRelease(object: u32) -> i32;
        fn IORegistryEntryGetName(entry: u32, name: *mut c_char) -> i32;
        fn IORegistryEntryCreateCFProperty(
            entry: u32,
            key: *const c_void,
            allocator: *const c_void,
            options: u32,
        ) -> *const c_void;
    }
    extern "C" {
        fn CFRelease(o: *const c_void);
        fn CFGetTypeID(o: *const c_void) -> usize;
        fn CFStringGetTypeID() -> usize;
        fn CFDictionaryGetTypeID() -> usize;
        fn CFNumberGetTypeID() -> usize;
        fn CFStringCreateWithCString(
            allocator: *const c_void,
            text: *const c_char,
            encoding: u32,
        ) -> *const c_void;
        fn CFStringGetCString(s: *const c_void, out: *mut c_char, size: isize, encoding: u32)
            -> u8;
        fn CFDictionaryGetValue(d: *const c_void, key: *const c_void) -> *const c_void;
        fn CFNumberGetValue(n: *const c_void, kind: isize, out: *mut c_void) -> u8;
        fn CGDisplayVendorNumber(d: u32) -> u32;
        fn CGDisplayModelNumber(d: u32) -> u32;
        fn CGDisplaySerialNumber(d: u32) -> u32;
    }

    const UTF8: u32 = 0x0800_0100;
    const RECURSIVELY: u32 = 1;
    const SINT64: isize = 4;
    const CHIP: u32 = 0x37;
    const DATA: u32 = 0x51;
    const BRIGHTNESS: u8 = 0x10;

    #[derive(Clone, Copy)]
    struct Api {
        read: ReadI2c,
        write: WriteI2c,
    }

    /// What a framebuffer says about the monitor on it.
    #[derive(Clone, Copy, Default)]
    struct Identity {
        vendor: u32,
        product: u32,
        serial: u32,
    }

    struct Candidate {
        service: *mut c_void,
        who: Identity,
    }

    /// The monitors found by the last `refresh`: display id, service, the monitor's own
    /// maximum, and whether it will not report its level. The lock also keeps reads and
    /// writes off the I2C bus at the same time.
    static HELD: Mutex<Vec<(u32, usize, u16, bool)>> = Mutex::new(Vec::new());

    fn symbols() -> Option<(Create, Api)> {
        unsafe {
            let lib = libc::dlopen(
                c"/System/Library/Frameworks/IOKit.framework/IOKit".as_ptr(),
                libc::RTLD_LAZY,
            );
            if lib.is_null() {
                return None;
            }
            let c = libc::dlsym(lib, c"IOAVServiceCreateWithService".as_ptr());
            let r = libc::dlsym(lib, c"IOAVServiceReadI2C".as_ptr());
            let w = libc::dlsym(lib, c"IOAVServiceWriteI2C".as_ptr());
            if c.is_null() || r.is_null() || w.is_null() {
                return None;
            }
            Some((
                std::mem::transmute::<*mut c_void, Create>(c),
                Api {
                    read: std::mem::transmute::<*mut c_void, ReadI2c>(r),
                    write: std::mem::transmute::<*mut c_void, WriteI2c>(w),
                },
            ))
        }
    }

    unsafe fn cf_string(text: &CStr) -> *const c_void {
        CFStringCreateWithCString(std::ptr::null(), text.as_ptr(), UTF8)
    }

    /// An owned registry property of the given CF type, or null.
    unsafe fn property(entry: u32, name: &CStr, type_id: usize) -> *const c_void {
        let key = cf_string(name);
        if key.is_null() {
            return std::ptr::null();
        }
        let value = IORegistryEntryCreateCFProperty(entry, key, std::ptr::null(), 0);
        CFRelease(key);
        if !value.is_null() && CFGetTypeID(value) != type_id {
            CFRelease(value);
            return std::ptr::null();
        }
        value
    }

    /// A borrowed dictionary value of the given CF type, or null.
    unsafe fn entry_of(dict: *const c_void, name: &CStr, type_id: usize) -> *const c_void {
        let key = cf_string(name);
        if key.is_null() {
            return std::ptr::null();
        }
        let value = CFDictionaryGetValue(dict, key);
        CFRelease(key);
        if value.is_null() || CFGetTypeID(value) != type_id {
            return std::ptr::null();
        }
        value
    }

    unsafe fn number_of(dict: *const c_void, name: &CStr) -> u32 {
        let n = entry_of(dict, name, CFNumberGetTypeID());
        let mut out = 0i64;
        if n.is_null() || CFNumberGetValue(n, SINT64, &mut out as *mut i64 as *mut c_void) == 0 {
            return 0;
        }
        out as u32
    }

    /// The EDID's three-letter maker code as the number CoreGraphics reports.
    fn vendor_number(code: &[u8]) -> u32 {
        code.iter()
            .take(3)
            .fold(0, |v, &c| (v << 5) | ((c as u32).wrapping_sub(64) & 31))
    }

    unsafe fn identity(framebuffer: u32) -> Identity {
        let mut who = Identity::default();
        let attrs = property(framebuffer, c"DisplayAttributes", CFDictionaryGetTypeID());
        if attrs.is_null() {
            return who;
        }
        let product = entry_of(attrs, c"ProductAttributes", CFDictionaryGetTypeID());
        if !product.is_null() {
            who.product = number_of(product, c"ProductID");
            who.serial = number_of(product, c"SerialNumber");
            let maker = entry_of(product, c"ManufacturerID", CFStringGetTypeID());
            let mut text = [0 as c_char; 8];
            if !maker.is_null() && CFStringGetCString(maker, text.as_mut_ptr(), 8, UTF8) != 0 {
                who.vendor = vendor_number(CStr::from_ptr(text.as_ptr()).to_bytes());
            }
        }
        CFRelease(attrs);
        who
    }

    unsafe fn is_external(proxy: u32) -> bool {
        let location = property(proxy, c"Location", CFStringGetTypeID());
        if location.is_null() {
            return false;
        }
        let mut text = [0 as c_char; 32];
        let ok = CFStringGetCString(location, text.as_mut_ptr(), 32, UTF8) != 0
            && CStr::from_ptr(text.as_ptr()).to_bytes() == b"External";
        CFRelease(location);
        ok
    }

    /// Walks the registry, where each framebuffer comes just before the proxy of its port.
    unsafe fn candidates(create: Create) -> Vec<Candidate> {
        let mut out = Vec::new();
        let mut iterator = 0u32;
        let root = IORegistryGetRootEntry(0); // kIOMainPortDefault
        let status =
            IORegistryEntryCreateIterator(root, c"IOService".as_ptr(), RECURSIVELY, &mut iterator);
        IOObjectRelease(root);
        if status != 0 {
            return out;
        }
        let mut who = Identity::default();
        loop {
            let entry = IOIteratorNext(iterator);
            if entry == 0 {
                break;
            }
            let mut name = [0 as c_char; 128];
            if IORegistryEntryGetName(entry, name.as_mut_ptr()) == 0 {
                match CStr::from_ptr(name.as_ptr()).to_bytes() {
                    b"AppleCLCD2" | b"IOMobileFramebufferShim" => who = identity(entry),
                    b"DCPAVServiceProxy" if is_external(entry) => {
                        let service = create(std::ptr::null(), entry);
                        if !service.is_null() {
                            out.push(Candidate { service, who });
                        }
                        who = Identity::default();
                    }
                    _ => {}
                }
            }
            IOObjectRelease(entry);
        }
        IOObjectRelease(iterator);
        out
    }

    fn checksum(seed: u8, bytes: &[u8]) -> u8 {
        bytes.iter().fold(seed, |sum, b| sum ^ b)
    }

    fn pause(ms: u64) {
        std::thread::sleep(Duration::from_millis(ms));
    }

    /// What a monitor says when asked for its brightness.
    enum Answer {
        /// Its current and maximum levels.
        Level(u16, u16),
        /// A DDC/CI null message: it is listening but will not report a level. Such a
        /// monitor can still take new levels.
        Listening,
        Silent,
    }

    unsafe fn read(api: Api, service: *mut c_void) -> Answer {
        // Monitors differ in what they accept. MonitorControl leaves 0x51 out of the checksum
        // and reads from offset 0; m1ddc includes it, reads from offset 0x51 and takes 12 bytes.
        let spec = ((CHIP << 1) ^ DATA) as u8;
        let legacy = (CHIP << 1) as u8;
        let mut listening = false;
        for (seed, offset, size) in [(legacy, 0, 11), (spec, 0, 11), (spec, DATA, 12)] {
            let mut ask = [0x82, 0x01, BRIGHTNESS, 0];
            ask[3] = checksum(seed, &ask[..3]);
            for _ in 0..2 {
                for _ in 0..2 {
                    pause(10);
                    (api.write)(service, CHIP, DATA, ask.as_ptr() as *const c_void, 4);
                }
                pause(50);
                let mut r = [0u8; 12];
                let status = (api.read)(service, CHIP, offset, r.as_mut_ptr() as *mut c_void, size);
                if status == 0 && r[1] == 0x80 && checksum(0x50, &r[..2]) == r[2] {
                    listening = true;
                }
                if status == 0
                    && checksum(0x50, &r[..10]) == r[10]
                    && r[3] == 0
                    && r[4] == BRIGHTNESS
                {
                    let max = u16::from_be_bytes([r[6], r[7]]);
                    let current = u16::from_be_bytes([r[8], r[9]]);
                    if max > 0 {
                        return Answer::Level(current.min(max), max);
                    }
                }
                pause(20);
            }
        }
        if listening {
            Answer::Listening
        } else {
            Answer::Silent
        }
    }

    fn levels_path() -> Option<std::path::PathBuf> {
        super::config_dir().map(|d| d.join("ddc-levels.txt"))
    }

    /// Names a monitor across runs and reconnections, unlike its display id.
    unsafe fn monitor_name(display: u32) -> String {
        format!(
            "{}-{}-{}",
            CGDisplayVendorNumber(display),
            CGDisplayModelNumber(display),
            CGDisplaySerialNumber(display)
        )
    }

    /// The level last set on a monitor that will not report its own.
    unsafe fn saved_level(display: u32) -> Option<i32> {
        let name = monitor_name(display);
        let text = std::fs::read_to_string(levels_path()?).ok()?;
        text.lines().find_map(|line| {
            let (who, level) = line.split_once(' ')?;
            (who == name).then(|| level.parse().ok())?
        })
    }

    unsafe fn save_level(display: u32, percent: i32) {
        let Some(path) = levels_path() else { return };
        let name = monitor_name(display);
        let old = std::fs::read_to_string(&path).unwrap_or_default();
        let mut text: String = old
            .lines()
            .filter(|line| line.split(' ').next() != Some(name.as_str()))
            .map(|line| format!("{line}\n"))
            .collect();
        text.push_str(&format!("{name} {percent}\n"));
        let _ = std::fs::write(path, text);
    }

    unsafe fn send(api: Api, service: *mut c_void, value: u16) {
        let [hi, lo] = value.to_be_bytes();
        let mut packet = [0x84, 0x03, BRIGHTNESS, hi, lo, 0];
        packet[5] = checksum(((CHIP << 1) ^ DATA) as u8, &packet[..5]);
        for _ in 0..2 {
            pause(10);
            (api.write)(service, CHIP, DATA, packet.as_ptr() as *const c_void, 6);
        }
    }

    /// How well a framebuffer's monitor fits a display: 0 is no evidence either way.
    unsafe fn score(display: u32, who: Identity) -> u32 {
        let mut s = 0;
        if who.vendor != 0 && who.vendor == CGDisplayVendorNumber(display) {
            s += 2;
        }
        if who.product != 0 && who.product == CGDisplayModelNumber(display) {
            s += 2;
        }
        if who.serial != 0 && who.serial == CGDisplaySerialNumber(display) {
            s += 1;
        }
        s
    }

    /// Pairs external displays with services and reads each one, returning (display, percent).
    /// A service counts only once its monitor answers. Sure matches are placed first, then
    /// whatever is left, since some monitors report no usable identity.
    pub fn refresh(displays: &[u32]) -> Vec<(u32, i32)> {
        let mut held = HELD.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            for &(_, service, _, _) in held.iter() {
                CFRelease(service as *const c_void);
            }
        }
        held.clear();
        let mut found = Vec::new();
        let Some((create, api)) = symbols() else {
            return found;
        };
        unsafe {
            let mut free = candidates(create);
            let mut tried: Vec<(u32, *mut c_void)> = Vec::new();
            for least in [4, 1, 0] {
                for &display in displays {
                    if found.iter().any(|&(d, _)| d == display) {
                        continue;
                    }
                    let mut order: Vec<usize> = (0..free.len())
                        .filter(|&i| score(display, free[i].who) >= least)
                        .collect();
                    order.sort_by_key(|&i| std::cmp::Reverse(score(display, free[i].who)));
                    for i in order {
                        if tried.contains(&(display, free[i].service)) {
                            continue;
                        }
                        tried.push((display, free[i].service));
                        match read(api, free[i].service) {
                            Answer::Level(current, max) => {
                                let c = free.remove(i);
                                held.push((display, c.service as usize, max, false));
                                let percent = (current as f64 * 100.0 / max as f64).round() as i32;
                                found.push((display, percent));
                                break;
                            }
                            Answer::Listening => {
                                // The maximum is unknown; 100 is what monitors nearly always use.
                                let c = free.remove(i);
                                held.push((display, c.service as usize, 100, true));
                                let percent = saved_level(display).unwrap_or(50);
                                found.push((display, percent));
                                break;
                            }
                            Answer::Silent => {}
                        }
                    }
                }
            }
            for c in free {
                CFRelease(c.service as *const c_void);
            }
        }
        found
    }

    pub fn write(display: u32, percent: i32) {
        let held = HELD.lock().unwrap_or_else(|e| e.into_inner());
        let Some(&(_, service, max, write_only)) = held.iter().find(|h| h.0 == display) else {
            return;
        };
        let Some((_, api)) = symbols() else { return };
        let percent = percent.clamp(0, 100);
        let value = (percent as f64 * max as f64 / 100.0).round() as u16;
        unsafe {
            send(api, service as *mut c_void, value);
            if write_only {
                save_level(display, percent);
            }
        }
    }
}

// ---- The sprite window -----------------------------------------------------------------------------

/// Frame buffers Core Animation has let go of, kept for the next frames.
static FRAME_POOL: std::sync::Mutex<Vec<Vec<u8>>> = std::sync::Mutex::new(Vec::new());
const FRAME_POOL_CAP: usize = 32;

fn frame_buffer(len: usize) -> Vec<u8> {
    let mut v = FRAME_POOL
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .pop()
        .unwrap_or_default();
    v.clear();
    v.reserve(len);
    v
}

/// Called by CoreGraphics, on whatever thread, once the image is no longer shown.
unsafe extern "C" fn free_frame(info: *mut c_void, _data: *const c_void, _size: usize) {
    let buffer = Box::from_raw(info as *mut Vec<u8>);
    let mut pool = FRAME_POOL.lock().unwrap_or_else(|e| e.into_inner());
    if pool.len() < FRAME_POOL_CAP {
        pool.push(*buffer);
    }
}

fn device_rgb() -> *mut c_void {
    static SPACE: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *SPACE.get_or_init(|| unsafe { CGColorSpaceCreateDeviceRGB() } as usize) as *mut c_void
}

/// A borderless, always-on-top, click-through window that shows a `Surface` with
/// per-pixel transparency. Never takes focus; left out of screen sharing where macOS
/// still honours that.
///
/// The window is as big as the whole canvas and sits on whole points; the drawn part
/// is a sublayer placed inside it. So a new frame never resizes the window (which
/// AppKit and Core Animation apply at different moments, stretching the picture for a
/// frame), and its position, size and pixels change together in one transaction.
pub struct SpriteWindow {
    window: Id,
    sprite: Id,
    visible: bool,
    /// The window's frame as last set, in points: (left, bottom, width, height).
    frame: (f64, f64, f64, f64),
}

impl SpriteWindow {
    pub fn new() -> Self {
        Self::create(true)
    }

    /// One that screenshots and screen sharing include.
    pub fn capturable() -> Self {
        Self::create(false)
    }

    fn create(private: bool) -> Self {
        unsafe {
            let rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1.0, 1.0));
            let w: Id = msg_send![class!(NSWindow), alloc];
            // NSWindowStyleMaskBorderless, NSBackingStoreBuffered, no deferring.
            let w: Id = msg_send![w, initWithContentRect: rect, styleMask: 0usize, backing: 2usize, defer: false];
            let _: () = msg_send![w, setReleasedWhenClosed: false];
            let _: () = msg_send![w, setOpaque: false];
            let clear: Id = msg_send![class!(NSColor), clearColor];
            let _: () = msg_send![w, setBackgroundColor: clear];
            let _: () = msg_send![w, setHasShadow: false];
            let _: () = msg_send![w, setIgnoresMouseEvents: true];
            let _: () = msg_send![w, setLevel: 25isize]; // NSStatusWindowLevel
                                                         // All spaces, stays put in Mission Control, allowed beside full-screen apps.
            let _: () =
                msg_send![w, setCollectionBehavior: (1usize << 0) | (1 << 4) | (1 << 6) | (1 << 8)];
            if private {
                let _: () = msg_send![w, setSharingType: 0usize]; // NSWindowSharingNone
            }
            let view: Id = msg_send![w, contentView];
            let _: () = msg_send![view, setWantsLayer: true];
            let root: Id = msg_send![view, layer];
            // Owned by its superlayer, which lives as long as the window.
            let sprite: Id = msg_send![class!(CALayer), layer];
            let _: () = msg_send![root, addSublayer: sprite];
            SpriteWindow {
                window: w,
                sprite,
                visible: false,
                frame: (0.0, 0.0, 1.0, 1.0),
            }
        }
    }

    /// Shows the surface with its top-left corner at (x, y). Only the part that has
    /// content is uploaded; it is placed where it belongs inside the canvas-sized window.
    pub fn present(&mut self, surface: Surface, x: i32, y: i32) {
        let mut b = surface.bounds();
        if b.is_empty() {
            b = Rectangle::new(0, 0, 1, 1);
        }
        let sw = surface.width();
        let len = (b.width * b.height * 4) as usize;
        let mut bytes = frame_buffer(len);
        let px = surface.pixels.as_slice();
        for row in b.top()..b.bottom() {
            let start = (row * sw + b.left()) as usize;
            for v in &px[start..start + b.width as usize] {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
        }
        let (m, _) = mac_screens();
        let f = m.factor;

        // The window on whole points (so AppKit never has to round it), one point
        // larger than the canvas to take up the remainder, which the sprite carries.
        let left_pt = x as f64 / f;
        let top_pt = m.primary_h - y as f64 / f; // AppKit y: up from the primary's bottom
        let win_left = left_pt.floor();
        let win_top = top_pt.ceil();
        let win_w = (surface.width() as f64 / f).ceil() + 1.0;
        let win_h = (surface.height() as f64 / f).ceil() + 1.0;
        let frame = (win_left, win_top - win_h, win_w, win_h);

        // The sprite inside the window, in the layer's own y-up coordinates.
        let sprite_w = b.width as f64 / f;
        let sprite_h = b.height as f64 / f;
        let sprite_left = (left_pt - win_left) + b.left() as f64 / f;
        let sprite_top = (win_top - top_pt) + b.top() as f64 / f;
        let sprite = NSRect::new(
            NSPoint::new(sprite_left, win_h - sprite_top - sprite_h),
            NSSize::new(sprite_w, sprite_h),
        );

        unsafe {
            let info = Box::into_raw(Box::new(bytes));
            let provider = CGDataProviderCreateWithData(
                info as *mut c_void,
                (*info).as_ptr() as *const c_void,
                len,
                Some(free_frame),
            );
            let image = CGImageCreate(
                b.width as usize,
                b.height as usize,
                8,
                32,
                b.width as usize * 4,
                device_rgb(),
                kCGBitmapByteOrder32Little | kCGImageAlphaPremultipliedFirst,
                provider,
                std::ptr::null(),
                false,
                0,
            );
            CGDataProviderRelease(provider);

            // Move the window first (only when it really moves), then commit the
            // sprite straight after, so both land on the same refresh.
            if frame != self.frame {
                let rect = NSRect::new(
                    NSPoint::new(frame.0, frame.1),
                    NSSize::new(frame.2, frame.3),
                );
                if (frame.2, frame.3) == (self.frame.2, self.frame.3) {
                    let _: () = msg_send![self.window, setFrameOrigin: rect.origin];
                } else {
                    let _: () = msg_send![self.window, setFrame: rect, display: false];
                }
                self.frame = frame;
            }

            // Animations off: contents and frame would otherwise cross-fade and slide.
            let _: () = msg_send![class!(CATransaction), begin];
            let _: () = msg_send![class!(CATransaction), setDisableActions: true];
            let _: () = msg_send![self.sprite, setFrame: sprite];
            let _: () = msg_send![self.sprite, setContents: image as Id];
            let _: () = msg_send![self.sprite, setContentsScale: f];
            let _: () = msg_send![class!(CATransaction), commit];
            CGImageRelease(image);

            if !self.visible {
                let _: () = msg_send![self.window, orderFrontRegardless];
                self.visible = true;
            }
        }
    }

    /// Nothing to do here: the status window level already keeps every sprite above
    /// ordinary windows, and re-ordering is a window-server round trip per window
    /// (it was costing a hitch once a second).
    pub fn stay_on_top(&self) {}

    pub fn hide(&mut self) {
        if self.visible {
            let _: () =
                unsafe { msg_send![self.window, orderOut: std::ptr::null_mut::<AnyObject>()] };
            self.visible = false;
        }
    }
}

impl Drop for SpriteWindow {
    fn drop(&mut self) {
        unsafe {
            let _: () = msg_send![self.window, close];
            let _: () = msg_send![self.window, release];
        }
    }
}
