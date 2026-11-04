//! Windows: layered click-through windows, the Run key, DWM pacing, DDC/CI and WMI.
//! The same Win32 calls as the C# original's Native.cs, declared by hand so there is
//! no dependency beyond what ships with Windows.
#![allow(non_snake_case, non_camel_case_types)]
use super::{BrightnessInfo, Screen};
use crate::prelude::*;
use std::collections::HashMap;
use std::ffi::c_void;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

type HWND = isize;
type HANDLE = isize;

const WS_POPUP: u32 = 0x80000000;
const WS_EX_TOPMOST: u32 = 0x8;
const WS_EX_TRANSPARENT: u32 = 0x20;
const WS_EX_TOOLWINDOW: u32 = 0x80;
const WS_EX_LAYERED: u32 = 0x80000;
const WS_EX_NOACTIVATE: u32 = 0x8000000;
const SW_HIDE: i32 = 0;
const SW_SHOWNOACTIVATE: i32 = 4;
const SWP_NOSIZE: u32 = 1;
const SWP_NOMOVE: u32 = 2;
const SWP_NOACTIVATE: u32 = 0x10;
const HWND_TOPMOST: HWND = -1;
const ULW_ALPHA: u32 = 2;
const WM_SYSCOMMAND: u32 = 0x112;
const SC_SCREENSAVE: usize = 0xF140;
const SC_MONITORPOWER: usize = 0xF170;
const SC_TASKLIST: usize = 0xF130;
const HWND_BROADCAST: HWND = 0xFFFF;
const KEYUP: u32 = 2;
const VK_TAB: u8 = 0x09;
const VK_CONTROL: u8 = 0x11;
const VK_ESCAPE: u8 = 0x1B;
const VK_A: u8 = 0x41;
const VK_D: u8 = 0x44;
const VK_N: u8 = 0x4E;
const VK_LWIN: u8 = 0x5B;
const VK_F15: u8 = 0x7E; // a key no application binds: safe to tap just to count as activity
const GWL_STYLE: i32 = -16;
const GWL_EXSTYLE: i32 = -20;
const WS_CAPTION: i32 = 0x00C00000;
const DWMWA_CLOAKED: u32 = 14;
const WDA_MONITOR: u32 = 1;
const WDA_EXCLUDEFROMCAPTURE: u32 = 0x11;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct POINT {
    x: i32,
    y: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RECT {
    l: i32,
    t: i32,
    r: i32,
    b: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct SIZE {
    cx: i32,
    cy: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct BLEND {
    op: u8,
    flags: u8,
    alpha: u8,
    format: u8,
}
#[repr(C)]
#[derive(Default)]
struct BITMAPINFOHEADER {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bits: u16,
    compression: u32,
    size_image: u32,
    x_ppm: i32,
    y_ppm: i32,
    used: u32,
    important: u32,
}
#[repr(C)]
struct MONITORINFO {
    size: u32,
    monitor: RECT,
    work: RECT,
    flags: u32,
}
#[repr(C)]
#[derive(Default)]
struct LASTINPUTINFO {
    size: u32,
    time: u32,
}
#[repr(C)]
#[derive(Default)]
struct POWER {
    ac: u8,
    flag: u8,
    percent: u8,
    saver: u8,
    life: u32,
    full: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct PHYSICAL_MONITOR {
    handle: HANDLE,
    description: [u16; 128],
}

#[link(name = "user32")]
extern "system" {
    fn CreateWindowExW(
        ex: u32,
        class: *const u16,
        name: *const u16,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        parent: HWND,
        menu: isize,
        inst: isize,
        param: *const c_void,
    ) -> HWND;
    fn DestroyWindow(h: HWND) -> i32;
    fn RegisterClassW(c: *const WNDCLASSW) -> u16;
    fn DefWindowProcW(h: HWND, m: u32, w: usize, l: isize) -> isize;
    fn UpdateLayeredWindow(
        h: HWND,
        dst: isize,
        pos: *const POINT,
        size: *const SIZE,
        src: isize,
        srcpos: *const POINT,
        key: u32,
        blend: *const BLEND,
        flags: u32,
    ) -> i32;
    fn ShowWindow(h: HWND, cmd: i32) -> i32;
    fn SetWindowPos(h: HWND, after: HWND, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> i32;
    fn SetWindowDisplayAffinity(h: HWND, a: u32) -> i32;
    fn GetCursorPos(p: *mut POINT) -> i32;
    fn GetAsyncKeyState(vk: i32) -> i16;
    fn EnumDisplayMonitors(
        dc: isize,
        clip: *const RECT,
        cb: unsafe extern "system" fn(isize, isize, *mut RECT, isize) -> i32,
        data: isize,
    ) -> i32;
    fn GetMonitorInfoW(m: isize, info: *mut MONITORINFO) -> i32;
    fn SetProcessDpiAwarenessContext(v: isize) -> i32;
    fn GetForegroundWindow() -> HWND;
    fn IsWindowVisible(h: HWND) -> i32;
    fn IsZoomed(h: HWND) -> i32;
    fn GetWindowRect(h: HWND, r: *mut RECT) -> i32;
    fn GetWindowLongW(h: HWND, i: i32) -> i32;
    fn GetClassNameW(h: HWND, s: *mut u16, n: i32) -> i32;
    fn GetWindowThreadProcessId(h: HWND, pid: *mut u32) -> u32;
    fn MonitorFromWindow(h: HWND, flags: u32) -> isize;
    fn FindWindowW(class: *const u16, name: *const u16) -> HWND;
    fn PostMessageW(h: HWND, m: u32, w: usize, l: isize) -> i32;
    fn SendMessageW(h: HWND, m: u32, w: usize, l: isize) -> isize;
    fn keybd_event(vk: u8, scan: u8, flags: u32, extra: usize);
    fn mouse_event(flags: u32, dx: i32, dy: i32, data: u32, extra: usize);
    fn LockWorkStation() -> i32;
    fn GetLastInputInfo(i: *mut LASTINPUTINFO) -> i32;
}
#[repr(C)]
struct WNDCLASSW {
    style: u32,
    proc_: unsafe extern "system" fn(HWND, u32, usize, isize) -> isize,
    cls: i32,
    wnd: i32,
    inst: isize,
    icon: isize,
    cursor: isize,
    bg: isize,
    menu: *const u16,
    name: *const u16,
}
#[link(name = "gdi32")]
extern "system" {
    fn CreateCompatibleDC(dc: isize) -> isize;
    fn CreateDIBSection(
        dc: isize,
        h: *const BITMAPINFOHEADER,
        usage: u32,
        bits: *mut *mut c_void,
        sec: isize,
        off: u32,
    ) -> isize;
    fn SelectObject(dc: isize, o: isize) -> isize;
    fn DeleteDC(dc: isize) -> i32;
    fn DeleteObject(o: isize) -> i32;
}
#[link(name = "dwmapi")]
extern "system" {
    fn DwmFlush() -> i32;
    fn DwmGetWindowAttribute(h: HWND, attr: u32, v: *mut i32, size: u32) -> i32;
}
#[link(name = "shcore")]
extern "system" {
    fn GetDpiForMonitor(m: isize, kind: i32, x: *mut u32, y: *mut u32) -> i32;
}
#[link(name = "kernel32")]
extern "system" {
    fn CreateMutexW(attr: *const c_void, own: i32, name: *const u16) -> HANDLE;
    fn GetLastError() -> u32;
    fn SetThreadExecutionState(s: u32) -> u32;
    fn GetSystemPowerStatus(p: *mut POWER) -> i32;
    fn GetCurrentThread() -> HANDLE;
    fn SetThreadPriority(t: HANDLE, p: i32) -> i32;
    fn GetTickCount() -> u32;
    fn OpenProcess(access: u32, inherit: i32, pid: u32) -> HANDLE;
    fn GetCurrentProcess() -> HANDLE;
    fn CloseHandle(h: HANDLE) -> i32;
}
#[link(name = "advapi32")]
extern "system" {
    fn RegOpenKeyExW(k: isize, sub: *const u16, opt: u32, access: u32, out: *mut isize) -> i32;
    fn RegCreateKeyExW(
        k: isize,
        sub: *const u16,
        res: u32,
        class: *const u16,
        opt: u32,
        access: u32,
        sec: *const c_void,
        out: *mut isize,
        disp: *mut u32,
    ) -> i32;
    fn RegQueryValueExW(
        k: isize,
        name: *const u16,
        res: *const u32,
        ty: *mut u32,
        data: *mut u8,
        size: *mut u32,
    ) -> i32;
    fn RegSetValueExW(
        k: isize,
        name: *const u16,
        res: u32,
        ty: u32,
        data: *const u8,
        size: u32,
    ) -> i32;
    fn RegDeleteValueW(k: isize, name: *const u16) -> i32;
    fn RegCloseKey(k: isize) -> i32;
    fn OpenProcessToken(p: HANDLE, access: u32, t: *mut HANDLE) -> i32;
    fn GetTokenInformation(
        t: HANDLE,
        class: i32,
        info: *mut c_void,
        len: u32,
        ret: *mut u32,
    ) -> i32;
    fn GetSidSubAuthorityCount(sid: *mut c_void) -> *mut u8;
    fn GetSidSubAuthority(sid: *mut c_void, i: u32) -> *mut u32;
}
#[link(name = "dxva2")]
extern "system" {
    fn GetNumberOfPhysicalMonitorsFromHMONITOR(m: isize, n: *mut u32) -> i32;
    fn GetPhysicalMonitorsFromHMONITOR(m: isize, n: u32, out: *mut PHYSICAL_MONITOR) -> i32;
    fn GetMonitorBrightness(m: HANDLE, lo: *mut u32, cur: *mut u32, hi: *mut u32) -> i32;
    fn SetMonitorBrightness(m: HANDLE, v: u32) -> i32;
    fn DestroyPhysicalMonitor(m: HANDLE) -> i32;
}

fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

const HKCU: isize = 0x80000001u32 as i32 as isize;
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

pub(crate) fn hide_console(c: &mut std::process::Command) {
    c.creation_flags(0x08000000); // CREATE_NO_WINDOW
}

// ---- The process ----------------------------------------------------------------

/// Every coordinate is a physical pixel on its own monitor. Before any window exists.
pub fn init_process() {
    unsafe { SetProcessDpiAwarenessContext(-4) }; // PER_MONITOR_AWARE_V2
}

/// False when another copy is already running.
pub fn single_instance() -> bool {
    if std::env::var_os("IDLEWALKER_TEST").is_some() {
        return true; // a test may run beside the real thing
    }
    unsafe {
        let name = w("IdleWalker.SingleInstance");
        let h = CreateMutexW(std::ptr::null(), 1, name.as_ptr());
        h != 0 && GetLastError() != 183 // ERROR_ALREADY_EXISTS; held until the process ends
    }
}

pub fn lower_thread_priority() {
    unsafe { SetThreadPriority(GetCurrentThread(), -1) }; // THREAD_PRIORITY_BELOW_NORMAL
}

pub fn config_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join("IdleWalker"))
}

/// A choice the C# version left in the registry, so an upgrade keeps it.
pub fn legacy_setting(name: &str) -> Option<i32> {
    if std::env::var_os("IDLEWALKER_TEST").is_some() {
        return None;
    }
    unsafe {
        let mut key = 0;
        if RegOpenKeyExW(
            HKCU,
            w(r"Software\IdleWalker").as_ptr(),
            0,
            0x20019,
            &mut key,
        ) != 0
        {
            return None;
        }
        let (mut ty, mut v, mut size) = (0u32, 0i32, 4u32);
        let r = RegQueryValueExW(
            key,
            w(name).as_ptr(),
            std::ptr::null(),
            &mut ty,
            &mut v as *mut i32 as *mut u8,
            &mut size,
        );
        RegCloseKey(key);
        (r == 0 && ty == 4).then_some(v)
    }
}

// ---- Monitors and the cursor ---------------------------------------------------------

pub fn screens() -> Vec<Screen> {
    unsafe extern "system" fn each(m: isize, _dc: isize, _r: *mut RECT, data: isize) -> i32 {
        let out = &mut *(data as *mut Vec<isize>);
        out.push(m);
        1
    }
    let mut handles: Vec<isize> = Vec::new();
    unsafe { EnumDisplayMonitors(0, std::ptr::null(), each, &mut handles as *mut _ as isize) };
    let rect = |r: RECT| Rectangle::from_ltrb(r.l, r.t, r.r, r.b);
    let mut out: Vec<Screen> = handles
        .iter()
        .filter_map(|&m| unsafe {
            let mut info = MONITORINFO {
                size: 40,
                monitor: RECT::default(),
                work: RECT::default(),
                flags: 0,
            };
            if GetMonitorInfoW(m, &mut info) == 0 {
                return None;
            }
            let (mut dx, mut dy) = (96u32, 96u32);
            if GetDpiForMonitor(m, 0, &mut dx, &mut dy) != 0 {
                dx = 96;
            }
            Some(Screen {
                bounds: rect(info.monitor),
                work: rect(info.work),
                dpi_scale: dx as f64 / 96.0,
                primary: info.flags & 1 != 0,
            })
        })
        .collect();
    // The primary first, as Screen.AllScreens has it; so "Display 1" is the same display.
    out.sort_by_key(|s| !s.primary);
    out
}

/// None on the secure desktop (UAC, lock screen), which counts as not moving.
pub fn cursor_pos() -> Option<Point> {
    let mut p = POINT::default();
    (unsafe { GetCursorPos(&mut p) } != 0).then(|| Point::new(p.x, p.y))
}

pub fn mouse_button_down() -> bool {
    unsafe {
        [1, 2, 4]
            .iter()
            .any(|&vk| GetAsyncKeyState(vk) as u16 & 0x8000 != 0)
    }
}

// ---- Pacing and power -----------------------------------------------------------------

/// Blocks until the compositor has presented a frame; false if it cannot.
pub fn wait_vblank() -> bool {
    unsafe { DwmFlush() == 0 }
}

/// The pump has nothing to animate; nothing to release here.
pub fn pacing_idle() {}

pub fn on_battery() -> bool {
    let mut p = POWER::default();
    unsafe { GetSystemPowerStatus(&mut p) != 0 && p.ac == 0 }
}

// ---- Keeping the machine awake -----------------------------------------------------------

/// Stops idle sleep, and with `on` the screen saver and monitor-off, until released.
/// The request lives on the calling thread, so it must stay alive: the UI thread.
pub fn keep_awake(on: bool) {
    unsafe { SetThreadExecutionState(if on { 0x80000003 } else { 0x80000000 }) };
}

/// Seconds since the last input of any kind, as Windows counts it for the screen saver.
pub fn system_idle_seconds() -> f64 {
    let mut i = LASTINPUTINFO { size: 8, time: 0 };
    unsafe {
        if GetLastInputInfo(&mut i) == 0 {
            return 0.0;
        }
        GetTickCount().wrapping_sub(i.time) as f64 / 1000.0
    }
}

/// Resets the last-input timer with an F15 tap and a zero-distance mouse move, so the
/// screen saver, auto-lock and "Away" presence never fire while the user is away.
pub fn nudge_activity() {
    unsafe {
        keybd_event(VK_F15, 0, 0, 0);
        keybd_event(VK_F15, 0, KEYUP, 0);
        mouse_event(1, 0, 0, 0, 0);
    }
}

// ---- The foreground window -------------------------------------------------------------

// The desktop, the taskbar and the shell's own full-screen surfaces: borderless and
// monitor-sized, but not apps. Task view in particular must not count, or the corner
// that opened it could never close it again.
const SHELL_CLASSES: [&str; 11] = [
    "Progman",
    "WorkerW",
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
    "XamlExplorerHostIslandWindow",
    "Windows.UI.Core.CoreWindow",
    "MultitaskingViewFrame",
    "ForegroundStaging",
    "TopLevelWindowForOverflowXamlIsland",
    "ControlCenterWindow",
    "NotifyIconOverflowWindow",
];

fn class_of(h: HWND) -> String {
    let mut buf = [0u16; 64];
    let n = unsafe { GetClassNameW(h, buf.as_mut_ptr(), 64) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

/// True when the window in front covers its whole monitor, taskbar included, and has
/// no title bar: a game, a film, a slide show. A merely maximised window does not
/// count, nor does one the user cannot see.
pub fn is_fullscreen() -> bool {
    unsafe {
        let h = GetForegroundWindow();
        if h == 0 || IsWindowVisible(h) == 0 || IsZoomed(h) != 0 {
            return false;
        }
        let mut cloaked = 0;
        if DwmGetWindowAttribute(h, DWMWA_CLOAKED, &mut cloaked, 4) == 0 && cloaked != 0 {
            return false;
        }
        let mut r = RECT::default();
        if GetWindowRect(h, &mut r) == 0 {
            return false;
        }
        let mut info = MONITORINFO {
            size: 40,
            monitor: RECT::default(),
            work: RECT::default(),
            flags: 0,
        };
        if GetMonitorInfoW(MonitorFromWindow(h, 2), &mut info) == 0 {
            return false;
        }
        let m = info.monitor;
        if !(r.l <= m.l && r.t <= m.t && r.r >= m.r && r.b >= m.b) {
            return false;
        }
        if GetWindowLongW(h, GWL_STYLE) & WS_CAPTION == WS_CAPTION {
            return false;
        }
        if GetWindowLongW(h, GWL_EXSTYLE) as u32 & WS_EX_TRANSPARENT != 0 {
            return false;
        }
        !SHELL_CLASSES.contains(&class_of(h).as_str())
    }
}

fn integrity_of(process: HANDLE) -> i32 {
    unsafe {
        let mut token = 0;
        if OpenProcessToken(process, 8, &mut token) == 0 {
            return i32::MAX;
        }
        let mut len = 0u32;
        GetTokenInformation(token, 25, std::ptr::null_mut(), 0, &mut len);
        let mut result = i32::MAX;
        if len > 0 {
            let mut buf = vec![0usize; len as usize / 8 + 1];
            if GetTokenInformation(token, 25, buf.as_mut_ptr() as *mut c_void, len, &mut len) != 0 {
                let sid = buf[0] as *mut c_void; // TOKEN_MANDATORY_LABEL.Label.Sid
                let n = *GetSidSubAuthorityCount(sid) as u32;
                result = *GetSidSubAuthority(sid, n - 1) as i32;
            }
        }
        CloseHandle(token);
        result
    }
}

/// True when the app in front runs as administrator: Windows silently drops synthetic
/// keystrokes aimed at it (UIPI), so the shell must be asked directly instead.
fn foreground_is_elevated() -> bool {
    unsafe {
        let h = GetForegroundWindow();
        if h == 0 {
            return false;
        }
        let mut pid = 0;
        GetWindowThreadProcessId(h, &mut pid);
        if pid == 0 {
            return false;
        }
        let p = OpenProcess(0x1000, 0, pid); // PROCESS_QUERY_LIMITED_INFORMATION
        if p == 0 {
            return true; // not even a limited query: protected, or above us
        }
        let above = integrity_of(p) > integrity_of(GetCurrentProcess());
        CloseHandle(p);
        above
    }
}

/// The foreground window, for the hot-corner log.
pub fn foreground_description() -> String {
    let h = unsafe { GetForegroundWindow() };
    if h == 0 {
        "(none)".into()
    } else {
        format!(
            "[{}]{}",
            class_of(h),
            if foreground_is_elevated() {
                " elevated"
            } else {
                ""
            }
        )
    }
}

// ---- Hot corners ------------------------------------------------------------------------

pub const CORNER_ACTION_NAMES: [&str; 10] = [
    "Nothing",
    "Task view",
    "Show desktop",
    "Start menu",
    "Notification centre",
    "Quick settings",
    "Lock screen",
    "Screen saver",
    "Monitor off",
    "Call the Minions",
];

fn chord(modifier: u8, key: u8) {
    unsafe {
        keybd_event(modifier, 0, 0, 0);
        keybd_event(key, 0, 0, 0);
        keybd_event(key, 0, KEYUP, 0);
        keybd_event(modifier, 0, KEYUP, 0);
    }
}

fn shell_route(action: CornerAction) -> bool {
    match action {
        CornerAction::TaskView => super::spawn(
            "explorer.exe",
            &["shell:::{3080F90E-D7AD-11D9-BD98-0000947B0257}"],
        ),
        CornerAction::ShowDesktop => super::spawn(
            "explorer.exe",
            &["shell:::{3080F90D-D7AD-11D9-BD98-0000947B0257}"],
        ),
        CornerAction::StartMenu => unsafe {
            let t = FindWindowW(w("Shell_TrayWnd").as_ptr(), std::ptr::null());
            t != 0 && PostMessageW(t, WM_SYSCOMMAND, SC_TASKLIST, 0) != 0
        },
        CornerAction::Notifications => super::spawn("explorer.exe", &["ms-actioncenter:"]),
        CornerAction::QuickSettings => super::spawn(
            "explorer.exe",
            &["ms-actioncenter:controlcenter/&showFooter=true"],
        ),
        _ => false,
    }
}

/// Carries out a corner's action the way the keyboard shortcut would.
pub fn perform_corner(action: CornerAction) {
    if foreground_is_elevated() && shell_route(action) {
        return;
    }
    match action {
        CornerAction::TaskView => chord(VK_LWIN, VK_TAB),
        CornerAction::ShowDesktop => chord(VK_LWIN, VK_D),
        CornerAction::StartMenu => chord(VK_CONTROL, VK_ESCAPE),
        CornerAction::Notifications => chord(VK_LWIN, VK_N),
        CornerAction::QuickSettings => chord(VK_LWIN, VK_A),
        CornerAction::LockScreen => unsafe {
            LockWorkStation();
        },
        CornerAction::ScreenSaver => unsafe {
            // Any window of ours will do: DefWindowProc starts the configured saver.
            let h = plain_window(0);
            SendMessageW(h, WM_SYSCOMMAND, SC_SCREENSAVE, 0);
            DestroyWindow(h);
        },
        CornerAction::MonitorOff => unsafe {
            PostMessageW(HWND_BROADCAST, WM_SYSCOMMAND, SC_MONITORPOWER, 2);
        },
        _ => {}
    }
}

// ---- Starting at sign-in -------------------------------------------------------------------

pub const STARTUP_LABEL: &str = "Start with Windows";

pub fn startup_is_enabled() -> bool {
    unsafe {
        let mut key = 0;
        if RegOpenKeyExW(HKCU, w(RUN_KEY).as_ptr(), 0, 0x20019, &mut key) != 0 {
            return false;
        }
        let mut size = 0u32;
        let r = RegQueryValueExW(
            key,
            w("IdleWalker").as_ptr(),
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut size,
        );
        RegCloseKey(key);
        r == 0
    }
}

pub fn startup_set(enabled: bool) -> Result<(), String> {
    unsafe {
        let mut key = 0;
        let r = RegCreateKeyExW(
            HKCU,
            w(RUN_KEY).as_ptr(),
            0,
            std::ptr::null(),
            0,
            0x20006,
            std::ptr::null(),
            &mut key,
            std::ptr::null_mut(),
        );
        if r != 0 {
            return Err(format!("the Run key could not be opened ({})", r));
        }
        let name = w("IdleWalker");
        let r = if enabled {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let value = w(&format!("\"{}\"", exe.display()));
            RegSetValueExW(
                key,
                name.as_ptr(),
                0,
                1,
                value.as_ptr() as *const u8,
                (value.len() * 2) as u32,
            )
        } else {
            let d = RegDeleteValueW(key, name.as_ptr());
            if d == 2 {
                0
            } else {
                d
            }
        };
        RegCloseKey(key);
        if r == 0 {
            Ok(())
        } else {
            Err(format!("error {}", r))
        }
    }
}

/// Shows a file in Explorer, selected.
pub fn reveal_file(path: &std::path::Path) {
    let arg = format!("/select,{}", path.display());
    super::spawn("explorer.exe", &[&arg]);
}

/// Opens a web address in the default browser.
pub fn open_url(url: &str) {
    super::spawn("rundll32.exe", &["url.dll,FileProtocolHandler", url]);
}

// ---- Brightness: DDC/CI for external monitors, WMI for the built-in panel -------------------

fn hmonitors() -> Vec<isize> {
    unsafe extern "system" fn each(m: isize, _dc: isize, _r: *mut RECT, data: isize) -> i32 {
        (*(data as *mut Vec<isize>)).push(m);
        1
    }
    let mut handles: Vec<isize> = Vec::new();
    unsafe { EnumDisplayMonitors(0, std::ptr::null(), each, &mut handles as *mut _ as isize) };
    let primary = |m: &isize| unsafe {
        let mut info = MONITORINFO {
            size: 40,
            monitor: RECT::default(),
            work: RECT::default(),
            flags: 0,
        };
        GetMonitorInfoW(*m, &mut info);
        info.flags & 1 == 0
    };
    handles.sort_by_key(primary);
    handles
}

fn physical_monitors(m: isize) -> Vec<PHYSICAL_MONITOR> {
    unsafe {
        let mut n = 0;
        if GetNumberOfPhysicalMonitorsFromHMONITOR(m, &mut n) == 0 || n == 0 {
            return vec![];
        }
        let mut v = vec![
            PHYSICAL_MONITOR {
                handle: 0,
                description: [0; 128]
            };
            n as usize
        ];
        if GetPhysicalMonitorsFromHMONITOR(m, n, v.as_mut_ptr()) == 0 {
            return vec![];
        }
        v
    }
}

/// Each DDC/CI monitor's raw brightness range, keyed by (HMONITOR index, physical index),
/// learned when the panel enumerates. A write then needs no slow DDC read first.
fn ddc_ranges() -> &'static Mutex<HashMap<(usize, usize), (u32, u32)>> {
    static R: OnceLock<Mutex<HashMap<(usize, usize), (u32, u32)>>> = OnceLock::new();
    R.get_or_init(Default::default)
}

fn powershell(script: &str) -> Option<String> {
    super::output_of(
        "powershell.exe",
        &["-NoProfile", "-NonInteractive", "-Command", script],
    )
}

// ---- WMI over COM, for the built-in panel ---------------------------------------------------
// Starting powershell.exe costs most of a second per call, which is the lag people feel
// between moving the slider and the screen changing. Talking to WMI directly from one
// long-lived thread that keeps its connection costs a few tens of milliseconds.

#[repr(C)]
struct GUID(u32, u16, u16, [u8; 8]);
const CLSID_WBEM_LOCATOR: GUID = GUID(
    0x4590f811,
    0x1d3a,
    0x11d0,
    [0x89, 0x1f, 0x00, 0xaa, 0x00, 0x4b, 0x2e, 0x24],
);
const IID_IWBEM_LOCATOR: GUID = GUID(
    0xdc12a687,
    0x737f,
    0x11cf,
    [0x88, 0x4d, 0x00, 0xaa, 0x00, 0x4b, 0x2e, 0x24],
);

#[repr(C)]
struct VARIANT {
    vt: u16,
    _reserved: [u16; 3],
    data: [usize; 2],
}
const VT_I4: u16 = 3;
const VT_BSTR: u16 = 8;
const VT_UI1: u16 = 17;

#[link(name = "ole32")]
extern "system" {
    fn CoInitializeEx(reserved: *mut c_void, coinit: u32) -> i32;
    fn CoCreateInstance(
        clsid: *const GUID,
        outer: *mut c_void,
        ctx: u32,
        iid: *const GUID,
        out: *mut *mut c_void,
    ) -> i32;
    fn CoSetProxyBlanket(
        proxy: *mut c_void,
        authn: u32,
        authz: u32,
        principal: *const u16,
        level: u32,
        impersonation: u32,
        identity: *mut c_void,
        caps: u32,
    ) -> i32;
}
#[link(name = "oleaut32")]
extern "system" {
    fn SysAllocString(s: *const u16) -> *mut u16;
    fn SysFreeString(s: *mut u16);
    fn VariantClear(v: *mut VARIANT) -> i32;
}

struct Bstr(*mut u16);
impl Bstr {
    fn new(s: &str) -> Bstr {
        Bstr(unsafe { SysAllocString(w(s).as_ptr()) })
    }
}
impl Drop for Bstr {
    fn drop(&mut self) {
        unsafe { SysFreeString(self.0) }
    }
}

/// A COM interface pointer, released when dropped.
struct Com(*mut c_void);
impl Drop for Com {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { slot::<unsafe extern "system" fn(*mut c_void) -> u32>(self.0, 2)(self.0) };
        }
    }
}

/// Entry `i` of a COM object's vtable, as a function of type `F`.
unsafe fn slot<F: Copy>(obj: *mut c_void, i: usize) -> F {
    let vtable = *(obj as *const *const usize);
    std::mem::transmute_copy(&*vtable.add(i))
}

type Out = *mut *mut c_void;
const NUL: *mut c_void = std::ptr::null_mut();

struct Wmi {
    /// IWbemServices on ROOT\WMI.
    svc: Com,
    /// The active panel's WmiMonitorBrightnessMethods path and a ready in-params object,
    /// or None on a machine without a WMI-driven panel.
    setter: Option<(Bstr, Com)>,
}

impl Wmi {
    /// None only when COM or the namespace is unavailable (then PowerShell is tried).
    fn connect() -> Option<Wmi> {
        unsafe {
            CoInitializeEx(NUL, 0); // multithreaded; a repeat call is harmless
            let mut loc = NUL;
            if CoCreateInstance(&CLSID_WBEM_LOCATOR, NUL, 1, &IID_IWBEM_LOCATOR, &mut loc) < 0 {
                return None;
            }
            let loc = Com(loc);
            type Connect = unsafe extern "system" fn(
                *mut c_void,
                *mut u16,
                *mut u16,
                *mut u16,
                *mut u16,
                i32,
                *mut u16,
                *mut c_void,
                Out,
            ) -> i32;
            let ns = Bstr::new(r"ROOT\WMI");
            let n = std::ptr::null_mut();
            let mut svc = NUL;
            if slot::<Connect>(loc.0, 3)(loc.0, ns.0, n, n, n, 0, n, NUL, &mut svc) < 0 {
                return None;
            }
            let svc = Com(svc);
            // WINNT authentication, call-level, impersonate.
            CoSetProxyBlanket(svc.0, 10, 0, std::ptr::null(), 3, 3, NUL, 0);
            let mut me = Wmi { svc, setter: None };
            me.setter = me.prepare_setter();
            Some(me)
        }
    }

    /// The first object a WQL query returns.
    fn first(&self, wql: &str) -> Option<Com> {
        unsafe {
            type ExecQuery = unsafe extern "system" fn(
                *mut c_void,
                *mut u16,
                *mut u16,
                i32,
                *mut c_void,
                Out,
            ) -> i32;
            type Next = unsafe extern "system" fn(*mut c_void, i32, u32, Out, *mut u32) -> i32;
            let (lang, q) = (Bstr::new("WQL"), Bstr::new(wql));
            let mut e = NUL;
            // Forward-only, return immediately.
            if slot::<ExecQuery>(self.svc.0, 20)(self.svc.0, lang.0, q.0, 0x30, NUL, &mut e) < 0 {
                return None;
            }
            let e = Com(e);
            let (mut obj, mut got) = (NUL, 0u32);
            slot::<Next>(e.0, 4)(e.0, -1, 1, &mut obj, &mut got);
            (got == 1 && !obj.is_null()).then_some(Com(obj))
        }
    }

    fn get(obj: &Com, name: &str) -> Option<VARIANT> {
        type Get = unsafe extern "system" fn(
            *mut c_void,
            *const u16,
            i32,
            *mut VARIANT,
            *mut i32,
            *mut i32,
        ) -> i32;
        let mut v = VARIANT {
            vt: 0,
            _reserved: [0; 3],
            data: [0; 2],
        };
        let n = std::ptr::null_mut();
        let ok = unsafe { slot::<Get>(obj.0, 4)(obj.0, w(name).as_ptr(), 0, &mut v, n, n) } >= 0;
        ok.then_some(v)
    }

    fn put(obj: &Com, name: &str, vt: u16, value: usize) -> bool {
        type Put =
            unsafe extern "system" fn(*mut c_void, *const u16, i32, *mut VARIANT, i32) -> i32;
        let mut v = VARIANT {
            vt,
            _reserved: [0; 3],
            data: [value, 0],
        };
        unsafe { slot::<Put>(obj.0, 5)(obj.0, w(name).as_ptr(), 0, &mut v, 0) >= 0 }
    }

    fn prepare_setter(&self) -> Option<(Bstr, Com)> {
        unsafe {
            let inst = self.first("SELECT * FROM WmiMonitorBrightnessMethods WHERE Active=TRUE")?;
            let mut p = Self::get(&inst, "__PATH")?;
            if p.vt != VT_BSTR || p.data[0] == 0 {
                VariantClear(&mut p);
                return None;
            }
            let path = Bstr(p.data[0] as *mut u16); // ownership moves to the Bstr
            type GetObject =
                unsafe extern "system" fn(*mut c_void, *mut u16, i32, *mut c_void, Out, Out) -> i32;
            type GetMethod =
                unsafe extern "system" fn(*mut c_void, *const u16, i32, Out, Out) -> i32;
            type SpawnInstance = unsafe extern "system" fn(*mut c_void, i32, Out) -> i32;
            let name = Bstr::new("WmiMonitorBrightnessMethods");
            let mut class = NUL;
            if slot::<GetObject>(self.svc.0, 6)(
                self.svc.0,
                name.0,
                0,
                NUL,
                &mut class,
                std::ptr::null_mut(),
            ) < 0
            {
                return None;
            }
            let class = Com(class);
            let (mut in_def, mut out_def) = (NUL, NUL);
            let method = w("WmiSetBrightness");
            if slot::<GetMethod>(class.0, 19)(
                class.0,
                method.as_ptr(),
                0,
                &mut in_def,
                &mut out_def,
            ) < 0
            {
                return None;
            }
            let (in_def, _out_def) = (Com(in_def), Com(out_def));
            if in_def.0.is_null() {
                return None;
            }
            let mut input = NUL;
            if slot::<SpawnInstance>(in_def.0, 15)(in_def.0, 0, &mut input) < 0 {
                return None;
            }
            let input = Com(input);
            // Timeout is a uint32, which WMI carries as VT_I4.
            Self::put(&input, "Timeout", VT_I4, 1).then_some((path, input))
        }
    }

    fn brightness(&self) -> Option<i32> {
        let o =
            self.first("SELECT CurrentBrightness FROM WmiMonitorBrightness WHERE Active=TRUE")?;
        let mut v = Self::get(&o, "CurrentBrightness")?;
        let r = match v.vt {
            VT_UI1 => Some((v.data[0] & 0xFF) as i32),
            VT_I4 => Some(v.data[0] as i32),
            _ => None,
        };
        unsafe { VariantClear(&mut v) };
        r
    }

    fn set_brightness(&self, percent: i32) -> bool {
        let Some((path, input)) = &self.setter else {
            return false;
        };
        if !Self::put(input, "Brightness", VT_UI1, percent.clamp(0, 100) as usize) {
            return false;
        }
        type ExecMethod = unsafe extern "system" fn(
            *mut c_void,
            *mut u16,
            *mut u16,
            i32,
            *mut c_void,
            *mut c_void,
            Out,
            Out,
        ) -> i32;
        let method = Bstr::new("WmiSetBrightness");
        let mut out = NUL;
        let hr = unsafe {
            slot::<ExecMethod>(self.svc.0, 24)(
                self.svc.0,
                path.0,
                method.0,
                0,
                NUL,
                input.0,
                &mut out,
                std::ptr::null_mut(),
            )
        };
        drop(Com(out));
        hr >= 0
    }
}

/// Runs `f` on the one thread that owns the WMI connection, waiting for its answer.
/// None when that thread could not be reached.
fn on_wmi_thread<R: Send + 'static>(
    f: impl FnOnce(&mut Option<Wmi>) -> R + Send + 'static,
) -> Option<R> {
    type Job = Box<dyn FnOnce(&mut Option<Wmi>) + Send>;
    static JOBS: OnceLock<Mutex<std::sync::mpsc::Sender<Job>>> = OnceLock::new();
    let jobs = JOBS.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel::<Job>();
        std::thread::spawn(move || {
            let mut wmi: Option<Wmi> = None;
            for job in rx {
                if wmi.is_none() {
                    wmi = Wmi::connect();
                }
                job(&mut wmi);
            }
        });
        Mutex::new(tx)
    });
    let (tx, rx) = std::sync::mpsc::channel();
    jobs.lock()
        .ok()?
        .send(Box::new(move |w| {
            let _ = tx.send(f(w));
        }))
        .ok()?;
    rx.recv().ok()
}

fn wmi_get() -> i32 {
    // Some(None): WMI answered and there is no WMI panel. None: COM failed, ask PowerShell.
    match on_wmi_thread(|w| w.as_mut().map(|w| {
        // Re-learn the panel each time the brightness panel opens: displays come and go.
        w.setter = w.prepare_setter();
        w.brightness()
    })) {
        Some(Some(found)) => found.unwrap_or(-1),
        _ => powershell("(Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightness -Filter 'Active=TRUE' | Select-Object -First 1).CurrentBrightness")
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(-1),
    }
}

fn wmi_set(percent: i32) {
    let done = on_wmi_thread(move |w| {
        let Some(wmi) = w else { return false };
        if wmi.set_brightness(percent) {
            return true;
        }
        // A stale connection or panel path: reconnect once and retry.
        *w = Wmi::connect();
        w.as_ref().is_some_and(|wmi| wmi.set_brightness(percent))
    });
    if done != Some(true) {
        powershell(&format!(
            "Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods -Filter 'Active=TRUE' | Invoke-CimMethod -MethodName WmiSetBrightness -Arguments @{{Timeout=[uint32]1;Brightness=[byte]{}}}",
            percent
        ));
    }
}

/// Every monitor, primary first, each with its current brightness. Slow (DDC/CI reads
/// can take a while per monitor), so call it off the UI thread.
pub fn brightness_enumerate() -> Vec<BrightnessInfo> {
    let screens = screens();
    let mut wmi = wmi_get();
    let mut ranges = HashMap::new();
    let found = hmonitors()
        .iter()
        .enumerate()
        .map(|(i, &m)| {
            let label = format!("Display {}", i + 1);
            let primary = screens.get(i).is_some_and(|s| s.primary);
            let mut chosen = None;
            // Mirrored outputs share one HMONITOR: the first that answers drives the slider.
            for (j, p) in physical_monitors(m).into_iter().enumerate() {
                let (mut lo, mut cur, mut hi) = (0, 0, 0);
                if chosen.is_none()
                    && unsafe { GetMonitorBrightness(p.handle, &mut lo, &mut cur, &mut hi) } != 0
                    && hi > lo
                {
                    ranges.insert((i, j), (lo, hi));
                    let end = p.description.iter().position(|&c| c == 0).unwrap_or(128);
                    let d = String::from_utf16_lossy(&p.description[..end]);
                    let d = d.trim();
                    let name = if !d.is_empty() && d != "Generic PnP Monitor" {
                        format!("{} - {}", label, d)
                    } else {
                        label.clone()
                    };
                    chosen = Some(BrightnessInfo {
                        name,
                        primary,
                        supported: true,
                        percent: ((cur - lo) as f64 * 100.0 / (hi - lo) as f64).round_ties_even()
                            as i32,
                        key: format!("ddc:{}", i),
                    });
                }
                unsafe { DestroyPhysicalMonitor(p.handle) };
            }
            chosen.unwrap_or_else(|| {
                // A laptop has one built-in panel; the first monitor without DDC/CI is taken to be it.
                if wmi >= 0 {
                    let percent = wmi;
                    wmi = -1;
                    BrightnessInfo {
                        name: format!("{} - Built-in", label),
                        primary,
                        supported: true,
                        percent,
                        key: "wmi".into(),
                    }
                } else {
                    BrightnessInfo {
                        name: label.clone(),
                        primary,
                        supported: false,
                        percent: 0,
                        key: String::new(),
                    }
                }
            })
        })
        .collect();
    if let Ok(mut r) = ddc_ranges().lock() {
        *r = ranges;
    }
    found
}

/// Writes a brightness; can take a tenth of a second, so off the UI thread.
pub fn brightness_write(key: &str, percent: i32) {
    if key == "wmi" {
        wmi_set(percent);
    } else if let Some(i) = key
        .strip_prefix("ddc:")
        .and_then(|s| s.parse::<usize>().ok())
    {
        if let Some(&m) = hmonitors().get(i) {
            for (j, p) in physical_monitors(m).into_iter().enumerate() {
                unsafe {
                    // The range learned at enumeration; only a monitor not seen then is read.
                    let known = ddc_ranges()
                        .lock()
                        .ok()
                        .and_then(|r| r.get(&(i, j)).copied());
                    let (mut lo, mut cur, mut hi) = (0, 0, 0);
                    if let Some(r) = known {
                        (lo, hi) = r;
                    }
                    if (known.is_some()
                        || GetMonitorBrightness(p.handle, &mut lo, &mut cur, &mut hi) != 0)
                        && hi > lo
                    {
                        SetMonitorBrightness(
                            p.handle,
                            (lo as f64 + (hi - lo) as f64 * percent as f64 / 100.0 + 0.5) as u32,
                        );
                    }
                    DestroyPhysicalMonitor(p.handle);
                }
            }
        }
    }
}

// ---- The sprite window ---------------------------------------------------------------------

unsafe extern "system" fn sprite_proc(h: HWND, m: u32, wp: usize, lp: isize) -> isize {
    DefWindowProcW(h, m, wp, lp)
}

fn plain_window(ex: u32) -> HWND {
    use std::sync::Once;
    static CLASS: Once = Once::new();
    let name = w("IdleWalkerSprite");
    CLASS.call_once(|| unsafe {
        let c = WNDCLASSW {
            style: 0,
            proc_: sprite_proc,
            cls: 0,
            wnd: 0,
            inst: 0,
            icon: 0,
            cursor: 0,
            bg: 0,
            menu: std::ptr::null(),
            name: name.as_ptr(),
        };
        RegisterClassW(&c);
    });
    unsafe {
        CreateWindowExW(
            ex,
            name.as_ptr(),
            w("IdleWalker").as_ptr(),
            WS_POPUP,
            0,
            0,
            1,
            1,
            0,
            0,
            0,
            std::ptr::null(),
        )
    }
}

/// A borderless, always-on-top, click-through window that shows a `Surface` with
/// per-pixel transparency. Never takes focus, and stays out of the taskbar and Alt-Tab.
pub struct SpriteWindow {
    hwnd: HWND,
    dc: isize,
    dib: isize,
    bits: *mut c_void,
    previous: isize,
    width: i32,
    height: i32,
    visible: bool,
}

impl SpriteWindow {
    pub fn new() -> Self {
        let hwnd = plain_window(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
        );
        // Visible on screen, missing from screenshots and screen captures.
        // IDLEWALKER_TEST leaves them capturable, so a test can look at them.
        if std::env::var_os("IDLEWALKER_TEST").is_none() {
            unsafe {
                if SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE) == 0 {
                    SetWindowDisplayAffinity(hwnd, WDA_MONITOR);
                }
            }
        }
        SpriteWindow {
            hwnd,
            dc: 0,
            dib: 0,
            bits: std::ptr::null_mut(),
            previous: 0,
            width: 0,
            height: 0,
            visible: false,
        }
    }

    fn ensure_bitmap(&mut self, width: i32, height: i32) {
        if width == self.width && height == self.height && self.dib != 0 {
            return;
        }
        self.release_bitmap();
        unsafe {
            if self.dc == 0 {
                self.dc = CreateCompatibleDC(0);
            }
            let header = BITMAPINFOHEADER {
                size: 40,
                width,
                height: -height,
                planes: 1,
                bits: 32,
                ..Default::default()
            };
            self.dib = CreateDIBSection(0, &header, 0, &mut self.bits, 0, 0);
            if self.dib != 0 {
                self.previous = SelectObject(self.dc, self.dib);
                self.width = width;
                self.height = height;
            }
        }
    }

    fn release_bitmap(&mut self) {
        if self.dib != 0 {
            unsafe {
                SelectObject(self.dc, self.previous);
                DeleteObject(self.dib);
            }
            self.dib = 0;
        }
    }

    /// Shows the surface with its top-left corner at (x, y). Only the part that has
    /// content is uploaded and shown; the window shrinks to fit it.
    pub fn present(&mut self, surface: Surface, x: i32, y: i32) {
        let (sw, sh) = (surface.width(), surface.height());
        self.ensure_bitmap(sw, sh);
        if self.dib == 0 {
            return;
        }
        let mut b = surface.bounds();
        if b.is_empty() {
            b = Rectangle::new(0, 0, 1, 1); // a single transparent pixel
        }
        unsafe {
            let start = (b.top() * sw) as usize;
            let n = (b.height * sw) as usize;
            std::ptr::copy_nonoverlapping(
                surface.pixels.as_slice()[start..start + n].as_ptr(),
                (self.bits as *mut i32).add(start),
                n,
            );
            let dst = POINT {
                x: x + b.left(),
                y: y + b.top(),
            };
            let src = POINT {
                x: b.left(),
                y: b.top(),
            };
            let size = SIZE {
                cx: b.width,
                cy: b.height,
            };
            let blend = BLEND {
                op: 0,
                flags: 0,
                alpha: 255,
                format: 1,
            };
            UpdateLayeredWindow(
                self.hwnd, 0, &dst, &size, self.dc, &src, 0, &blend, ULW_ALPHA,
            );
            if !self.visible {
                ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
                self.visible = true;
                self.stay_on_top();
            }
        }
    }

    /// Re-asserts topmost; the taskbar is topmost too and likes to win.
    pub fn stay_on_top(&self) {
        unsafe {
            SetWindowPos(
                self.hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            )
        };
    }

    pub fn hide(&mut self) {
        if self.visible {
            unsafe { ShowWindow(self.hwnd, SW_HIDE) };
            self.visible = false;
        }
    }
}

impl Drop for SpriteWindow {
    fn drop(&mut self) {
        self.release_bitmap();
        unsafe {
            if self.dc != 0 {
                DeleteDC(self.dc);
            }
            DestroyWindow(self.hwnd);
        }
    }
}

/// The tray backend on this OS is `tray-icon`; see app/tray.rs.
pub fn tray_init_thread() {}
