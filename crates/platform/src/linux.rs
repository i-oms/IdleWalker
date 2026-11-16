//! Linux: X11 through a pure-Rust connection, so there is no C library to install.
//!
//! Sprites are override-redirect ARGB windows with an empty input shape. That needs a
//! compositing window manager, which every mainstream desktop has; without one no
//! sprites are shown rather than black boxes. Under Wayland the program runs through
//! XWayland: walkers and the brightness panel work, but Wayland does not let any
//! program read the global cursor, so the position only updates while the pointer is
//! over an X11 window (and starts out in the middle of the screen). `pointer_tracked`
//! says so, and idleness comes from the compositor's idle clock over D-Bus instead.
//! See "Platform differences" in docs/DEVELOPMENT.md.
use super::{BrightnessInfo, Modifier, MouseButtons, Screen};
use crate::prelude::*;
use crate::rt::{Point, Rectangle};
use idlewalker_application::debug_note;
use std::ffi::CString;
use std::path::PathBuf;
use std::sync::OnceLock;
use x11rb::connection::{Connection, RequestConnection as _};
use x11rb::protocol::randr::ConnectionExt as _;
use x11rb::protocol::shape::{self, ConnectionExt as _};
use x11rb::protocol::xproto::*;
use x11rb::protocol::xtest::ConnectionExt as _;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

struct X {
    conn: RustConnection,
    root: Window,
    visual: Option<(Visualid, Colormap)>,
    a_workarea: Atom,
    a_desktop: Atom,
    a_client_list: Atom,
    a_strut: Atom,
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
    let (conn, screen_num) = match RustConnection::connect(None) {
        Ok(c) => c,
        Err(e) => {
            debug_note!(
                "x11: cannot connect to DISPLAY={:?}: {}",
                std::env::var("DISPLAY").ok(),
                e
            );
            return None;
        }
    };
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
    debug_note!(
        "x11: connected to screen {} ({}x{}, depth {}), 32-bit visual {:?}, Xft.dpi scale {}, wayland session {}",
        screen_num,
        screen.width_in_pixels,
        screen.height_in_pixels,
        screen.root_depth,
        visual.map(|v| v.0),
        dpi_scale,
        wayland()
    );
    Some(X {
        root: screen.root,
        visual,
        a_workarea: atom(&conn, "_NET_WORKAREA"),
        a_desktop: atom(&conn, "_NET_CURRENT_DESKTOP"),
        a_client_list: atom(&conn, "_NET_CLIENT_LIST"),
        a_strut: atom(&conn, "_NET_WM_STRUT_PARTIAL"),
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
/// The running copy is then asked to open its settings menu: some desktops (GNOME without
/// the AppIndicator extension) have no tray at all, and launching the program again is
/// how the menu is reached there.
pub fn single_instance() -> bool {
    let Some(dir) = config_dir() else { return true };
    let _ = std::fs::create_dir_all(&dir);
    let Ok(path) = CString::new(dir.join("instance.lock").to_string_lossy().as_bytes()) else {
        return true;
    };
    let first = unsafe {
        let fd = libc::open(path.as_ptr(), libc::O_CREAT | libc::O_RDWR, 0o600);
        fd < 0 || libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) == 0
    };
    if !first {
        if let Some(socket) = relaunch_socket() {
            use std::io::Write;
            match std::os::unix::net::UnixStream::connect(&socket) {
                Ok(mut s) => {
                    let _ = s.write_all(b"menu\n");
                    debug_note!("instance: already running; asked it to open its menu");
                }
                Err(e) => debug_note!(
                    "instance: already running, but {} did not answer: {}",
                    socket.display(),
                    e
                ),
            }
        }
    }
    first
}

/// Where a second copy finds the first: the per-user runtime folder when there is one.
fn relaunch_socket() -> Option<PathBuf> {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|d| d.is_dir())
        .or_else(config_dir)
        .map(|d| d.join("idlewalker.sock"))
}

/// Calls `on_relaunch` whenever the program is started again while this copy runs.
pub fn watch_relaunch(on_relaunch: Box<dyn Fn() + Send>) {
    use std::io::Read;
    let Some(path) = relaunch_socket() else {
        return;
    };
    // Only the copy holding the instance lock gets here, so a file left there is stale.
    let _ = std::fs::remove_file(&path);
    let listener = match std::os::unix::net::UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) => {
            debug_note!("relaunch: cannot listen on {}: {}", path.display(), e);
            return;
        }
    };
    debug_note!("relaunch: listening on {}", path.display());
    let _ = std::thread::Builder::new()
        .name("IdleWalker relaunch".into())
        .spawn(move || {
            for mut stream in listener.incoming().flatten() {
                let mut asked = [0u8; 16];
                let n = stream.read(&mut asked).unwrap_or(0);
                debug_note!(
                    "relaunch: started again, asked {:?}",
                    String::from_utf8_lossy(&asked[..n])
                );
                if asked[..n].starts_with(b"menu") {
                    on_relaunch();
                }
            }
        });
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

/// How long a reading of the monitor layout is reused. Reading it is several round trips
/// to the X server, and it is asked for every frame; a plugged-in monitor or a moved
/// panel still shows within this.
const SCREENS_TTL: std::time::Duration = std::time::Duration::from_millis(500);

pub fn screens() -> Vec<Screen> {
    static CACHE: std::sync::Mutex<Option<(std::time::Instant, Vec<Screen>)>> =
        std::sync::Mutex::new(None);
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((at, list)) = cache.as_ref() {
        if at.elapsed() < SCREENS_TTL {
            return list.clone();
        }
    }
    let list = read_screens();
    if debug_enabled() && cache.as_ref().is_none_or(|(_, old)| *old != list) {
        debug_note!(
            "screens: {} (work area from {})",
            describe_screens(&list),
            WORK_SOURCE.lock().unwrap_or_else(|e| e.into_inner())
        );
    }
    // Nothing to keep while there is no X server to ask.
    if !list.is_empty() {
        *cache = Some((std::time::Instant::now(), list.clone()));
    }
    list
}

fn read_screens() -> Vec<Screen> {
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
    shorten_for_panels(x, &mut out);
    out.sort_by_key(|s| !s.primary);
    out
}

/// A window property that is a list of 32-bit values; empty when it is not set.
fn cardinals(x: &X, window: Window, property: Atom, kind: AtomEnum, max: u32) -> Vec<u32> {
    x.conn
        .get_property(false, window, property, kind, 0, max)
        .ok()
        .and_then(|r| r.reply().ok())
        .and_then(|r| r.value32().map(|v| v.collect()))
        .unwrap_or_default()
}

/// Ends a monitor's working area at `line`, if that is inside the monitor.
fn end_work_at(s: &mut Screen, line: i32) {
    if line < s.work.bottom() && line > s.bounds.top() {
        s.work = Rectangle::from_ltrb(s.work.left(), s.work.top(), s.work.right(), line);
    }
}

/// Shortens the working area of each monitor that has a panel along its bottom, so the
/// cast walks on the panel and not behind it. With monitors of different heights this has
/// to be known per monitor, so the sources are asked from the most to the least exact.
fn debug_enabled() -> bool {
    idlewalker_application::debug::enabled()
}

/// Which source the working areas last came from, for the trace and `--diagnose`.
static WORK_SOURCE: std::sync::Mutex<&'static str> = std::sync::Mutex::new("nothing yet");

fn work_source(source: &'static str) {
    *WORK_SOURCE.lock().unwrap_or_else(|e| e.into_inner()) = source;
}

fn describe_screens(list: &[Screen]) -> String {
    list.iter()
        .map(|s| {
            format!(
                "[{},{} {}x{} work {},{} {}x{} scale {}{}]",
                s.bounds.x,
                s.bounds.y,
                s.bounds.width,
                s.bounds.height,
                s.work.x,
                s.work.y,
                s.work.width,
                s.work.height,
                s.dpi_scale,
                if s.primary { " primary" } else { "" }
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn shorten_for_panels(x: &X, out: &mut [Screen]) {
    // GNOME publishes a working area for every monitor of the current workspace.
    let desktop = cardinals(x, x.root, x.a_desktop, AtomEnum::CARDINAL, 1)
        .first()
        .copied()
        .unwrap_or(0);
    let a_areas = atom(&x.conn, &format!("_GTK_WORKAREAS_D{}", desktop));
    let areas = if a_areas != 0 {
        cardinals(x, x.root, a_areas, AtomEnum::CARDINAL, 256)
    } else {
        Vec::new()
    };
    if !areas.is_empty() && areas.len() % 4 == 0 {
        for q in areas.chunks(4) {
            let area = Rectangle::new(q[0] as i32, q[1] as i32, q[2] as i32, q[3] as i32);
            let middle = Point::new(area.left() + area.width / 2, area.top() + area.height / 2);
            for s in out.iter_mut().filter(|s| s.bounds.contains(middle)) {
                end_work_at(s, area.bottom());
            }
        }
        work_source("_GTK_WORKAREAS (GNOME)");
        return;
    }

    // Elsewhere each panel reserves its own strip: so many pixels up from the bottom of
    // the whole desktop, between two x positions.
    let desktop_bottom = x
        .conn
        .get_geometry(x.root)
        .ok()
        .and_then(|r| r.reply().ok())
        .map(|g| g.height as i32);
    if let Some(desktop_bottom) = desktop_bottom {
        let clients = cardinals(x, x.root, x.a_client_list, AtomEnum::WINDOW, 4096);
        // All asked before any answer is read: one round trip, however many windows.
        let asked: Vec<_> = clients
            .iter()
            .filter_map(|&w| {
                x.conn
                    .get_property(false, w, x.a_strut, AtomEnum::CARDINAL, 0, 12)
                    .ok()
            })
            .collect();
        let mut any = false;
        for cookie in asked {
            let strut: Vec<u32> = cookie
                .reply()
                .ok()
                .and_then(|r| r.value32().map(|v| v.collect()))
                .unwrap_or_default();
            if strut.len() < 12 {
                continue;
            }
            any = true;
            if strut[3] == 0 {
                continue;
            }
            let line = desktop_bottom - strut[3] as i32;
            let (from, to) = (strut[10] as i32, strut[11] as i32);
            for s in out
                .iter_mut()
                .filter(|s| from < s.bounds.right() && to >= s.bounds.left())
            {
                end_work_at(s, line);
            }
        }
        if any {
            work_source("panel struts");
            return;
        }
    }

    // Last resort: _NET_WORKAREA, one rectangle over the whole desktop. It cannot say
    // which monitor a panel is on, so every monitor it cuts through is shortened.
    let area = cardinals(x, x.root, x.a_workarea, AtomEnum::CARDINAL, 4);
    if area.len() == 4 {
        let bottom = (area[1] + area[3]) as i32;
        for s in out.iter_mut() {
            end_work_at(s, bottom);
        }
        work_source("_NET_WORKAREA");
    } else {
        work_source("nothing: whole monitors");
    }
}

/// A Wayland session, where this program is an XWayland client.
pub fn wayland() -> bool {
    static WAYLAND: OnceLock<bool> = OnceLock::new();
    *WAYLAND.get_or_init(|| {
        std::env::var("XDG_SESSION_TYPE").is_ok_and(|t| t.eq_ignore_ascii_case("wayland"))
            || std::env::var_os("WAYLAND_DISPLAY").is_some()
    })
}

/// XWayland only sees the pointer, its buttons and the keys while an X11 window has them;
/// the rest of the time it reports where the pointer last was.
pub fn pointer_tracked() -> bool {
    !wayland()
}

/// The compositor's idle clock under Wayland, read on a thread of its own so a slow
/// D-Bus never holds up a frame. `None` on X11, where the cursor is tracked, and where
/// the compositor has no idle clock to ask.
pub fn input_idle_seconds() -> Option<f64> {
    if !wayland() {
        return None;
    }
    let watch = INPUT_WATCH.get_or_init(InputWatch::start);
    let last = *watch.last_input.lock().unwrap_or_else(|e| e.into_inner());
    last.map(|t| t.elapsed().as_secs_f64())
}

static INPUT_WATCH: OnceLock<InputWatch> = OnceLock::new();

struct InputWatch {
    /// When the person last did something; `None` until (or unless) the clock answers.
    last_input: std::sync::Arc<std::sync::Mutex<Option<std::time::Instant>>>,
}

/// One compositor idle clock: who to ask, and whether it answers in milliseconds.
struct IdleClock {
    service: &'static str,
    path: &'static str,
    interface: &'static str,
    method: &'static str,
    /// GNOME's answer is documented as milliseconds. KDE's freedesktop.ScreenSaver one is
    /// documented as seconds but has been milliseconds; only its going back is used.
    exact_ms: bool,
}

const IDLE_CLOCKS: [IdleClock; 2] = [
    IdleClock {
        service: "org.gnome.Mutter.IdleMonitor",
        path: "/org/gnome/Mutter/IdleMonitor/Core",
        interface: "org.gnome.Mutter.IdleMonitor",
        method: "GetIdletime",
        exact_ms: true,
    },
    IdleClock {
        service: "org.freedesktop.ScreenSaver",
        path: "/org/freedesktop/ScreenSaver",
        interface: "org.freedesktop.ScreenSaver",
        method: "GetSessionIdleTime",
        exact_ms: false,
    },
];

impl IdleClock {
    fn read(&self, bus: &zbus::blocking::Connection) -> Option<u64> {
        let reply = bus
            .call_method(
                Some(self.service),
                self.path,
                Some(self.interface),
                self.method,
                &(),
            )
            .ok()?;
        let body = reply.body();
        if self.exact_ms {
            body.deserialize::<u64>().ok()
        } else {
            body.deserialize::<u32>().ok().map(u64::from)
        }
    }
}

impl InputWatch {
    fn start() -> InputWatch {
        let last_input = std::sync::Arc::new(std::sync::Mutex::new(None));
        let shared = last_input.clone();
        let _ = std::thread::Builder::new()
            .name("IdleWalker input watch".into())
            .spawn(move || {
                let bus = match zbus::blocking::Connection::session() {
                    Ok(b) => b,
                    Err(e) => {
                        debug_note!("idle clock: no D-Bus session bus: {}", e);
                        return;
                    }
                };
                let Some(clock) = IDLE_CLOCKS.iter().find(|c| c.read(&bus).is_some()) else {
                    debug_note!(
                        "idle clock: neither {} nor {} answers; idleness follows the XWayland cursor only",
                        IDLE_CLOCKS[0].service,
                        IDLE_CLOCKS[1].service
                    );
                    return;
                };
                debug_note!("idle clock: using {}.{}", clock.interface, clock.method);
                let mut previous: Option<u64> = None;
                let mut next_report = std::time::Instant::now();
                loop {
                    let Some(value) = clock.read(&bus) else {
                        debug_note!("idle clock: {} stopped answering", clock.service);
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        continue;
                    };
                    if debug_enabled() && std::time::Instant::now() >= next_report {
                        next_report =
                            std::time::Instant::now() + std::time::Duration::from_secs(2);
                        debug_note!("idle clock: {} reads {}", clock.method, value);
                    }
                    let now = std::time::Instant::now();
                    let at = if clock.exact_ms {
                        now.checked_sub(std::time::Duration::from_millis(value))
                    } else if value == 0 || previous.is_some_and(|p| value < p) {
                        Some(now)
                    } else if previous.is_none() {
                        // Unknown units: until the clock goes back, count from now.
                        Some(now)
                    } else {
                        None
                    };
                    previous = Some(value);
                    if let Some(at) = at {
                        let mut last = shared.lock().unwrap_or_else(|e| e.into_inner());
                        if last.is_none_or(|l| at > l) {
                            *last = Some(at);
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            });
        InputWatch { last_input }
    }
}

pub fn cursor_pos() -> Option<Point> {
    let x = x()?;
    let r = x.conn.query_pointer(x.root).ok()?.reply().ok()?;
    if debug_enabled() {
        static NEXT: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
        let mut next = NEXT.lock().unwrap_or_else(|e| e.into_inner());
        let now = std::time::Instant::now();
        if next.is_none_or(|t| now >= t) {
            *next = Some(now + std::time::Duration::from_secs(1));
            debug_note!(
                "cursor: {},{} mask {:#06x} over child {:#x} tracked {} input idle {:?}",
                r.root_x,
                r.root_y,
                u16::from(r.mask),
                r.child,
                pointer_tracked(),
                input_idle_seconds().map(|s| (s * 10.0).round() / 10.0)
            );
        }
    }
    drain_x_events(x);
    Some(Point::new(r.root_x as i32, r.root_y as i32))
}

/// Reads what the server sent unasked. Nothing listens for events on this connection, but
/// the errors of requests sent without waiting arrive this way and would pile up; with
/// `--debug` the first 200 are written down.
fn drain_x_events(x: &X) {
    static SHOWN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    while let Ok(Some(event)) = x.conn.poll_for_event() {
        if let x11rb::protocol::Event::Error(e) = event {
            let n = SHOWN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if n < 200 {
                debug_note!(
                    "x11 error: {:?} on request {}.{} resource {:#x} (seq {})",
                    e.error_kind,
                    e.major_opcode,
                    e.minor_opcode,
                    e.bad_value,
                    e.sequence
                );
            } else if n == 200 {
                debug_note!("x11 error: more than 200, no longer written down");
            }
        }
    }
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

pub fn modifier_down(key: Modifier) -> bool {
    let bit = match key {
        Modifier::Shift => KeyButMask::SHIFT,
        Modifier::Ctrl => KeyButMask::CONTROL,
        Modifier::Alt => KeyButMask::MOD1,
    };
    pointer_mask() & u16::from(bit) != 0
}

/// More keys are held than the one the modifier accounts for.
pub fn other_key_down(key: Modifier) -> bool {
    let Some(x) = x() else { return false };
    let held: u32 = x
        .conn
        .query_keymap()
        .ok()
        .and_then(|r| r.reply().ok())
        .map_or(0, |r| r.keys.iter().map(|b| b.count_ones()).sum());
    held > modifier_down(key) as u32
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
    // XWayland's own clock only counts input that went to X11 windows.
    if let Some(idle) = input_idle_seconds() {
        return idle;
    }
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
    // `--autostart` tells a start at login from one by hand: only the latter opens the
    // settings menu when the desktop has no tray to show the icon in.
    let text = format!("[Desktop Entry]\nType=Application\nName=IdleWalker\nExec=\"{}\" --autostart\nX-GNOME-Autostart-enabled=true\n", exe.display());
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

// ---- Diagnostics -----------------------------------------------------------------------------------

/// `--diagnose`: everything the Linux backend depends on, read once and written out as text:
/// the session, the X server, the compositor, the monitors, the cursor, the trays, the
/// idle clocks, the helper programs and the files.
pub fn diagnose_report() -> String {
    use std::fmt::Write;
    let mut o = String::new();
    macro_rules! say {
        ($($arg:tt)*) => {{ let _ = writeln!(o, $($arg)*); }};
    }

    say!("\n== Session");
    if let Ok(release) = std::fs::read_to_string("/etc/os-release") {
        let name = release
            .lines()
            .find_map(|l| l.strip_prefix("PRETTY_NAME="))
            .unwrap_or("?")
            .trim_matches('"')
            .to_string();
        say!("distribution: {}", name);
    }
    for key in [
        "XDG_SESSION_TYPE",
        "XDG_CURRENT_DESKTOP",
        "XDG_SESSION_DESKTOP",
        "DESKTOP_SESSION",
        "WAYLAND_DISPLAY",
        "DISPLAY",
        "GDK_BACKEND",
        "XDG_RUNTIME_DIR",
    ] {
        say!(
            "{}: {}",
            key,
            std::env::var(key).unwrap_or_else(|_| "(unset)".into())
        );
    }
    say!(
        "wayland session: {}; cursor tracked: {}",
        wayland(),
        pointer_tracked()
    );

    say!("\n== X server");
    match x() {
        None => say!("cannot connect: no walkers, cursor or trays can work"),
        Some(x) => diagnose_x(x, &mut o),
    }

    say!("\n== D-Bus");
    diagnose_dbus(&mut o);

    say!("\n== Helper programs");
    for program in [
        "systemd-inhibit",
        "loginctl",
        "xdg-open",
        "xdg-screensaver",
        "xset",
        "ddcutil",
        "brightnessctl",
        "gnome-extensions",
    ] {
        say!(
            "{}: {}",
            program,
            find_in_path(program).map_or("not found".into(), |p| p.display().to_string())
        );
    }
    if find_in_path("gnome-extensions").is_some() {
        let enabled = super::output_of("gnome-extensions", &["list", "--enabled"]);
        let enabled = enabled.unwrap_or_default();
        let tray = enabled
            .lines()
            .filter(|l| l.contains("appindicator") || l.contains("tray"))
            .collect::<Vec<_>>();
        say!(
            "GNOME tray extensions enabled: {}",
            if tray.is_empty() {
                "none".into()
            } else {
                tray.join(", ")
            }
        );
    }

    say!("\n== Files and power");
    say!("config folder: {:?}", config_dir());
    let socket = relaunch_socket();
    say!(
        "relaunch socket: {:?} (exists: {})",
        socket,
        socket.as_ref().is_some_and(|p| p.exists())
    );
    let autostart = autostart_path();
    say!("autostart entry: {:?}", autostart);
    if let Some(text) = autostart.and_then(|p| std::fs::read_to_string(p).ok()) {
        for l in text.lines() {
            say!("    {}", l);
        }
    }
    say!("backlight: {:?}", backlight());
    say!("on battery: {}", on_battery());
    o
}

fn find_in_path(program: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|d| d.join(program))
        .find(|p| p.is_file())
}

/// The name a window gives itself: `_NET_WM_NAME`, else `WM_NAME`.
fn window_name(x: &X, w: Window) -> String {
    let utf8 = atom(&x.conn, "UTF8_STRING");
    let net_name = atom(&x.conn, "_NET_WM_NAME");
    for (prop, kind) in [
        (net_name, utf8),
        (AtomEnum::WM_NAME.into(), AtomEnum::STRING.into()),
    ] {
        if let Some(r) = x
            .conn
            .get_property(false, w, prop, kind, 0, 256)
            .ok()
            .and_then(|r| r.reply().ok())
        {
            if !r.value.is_empty() {
                return String::from_utf8_lossy(&r.value).into_owned();
            }
        }
    }
    "(no name)".into()
}

fn selection_owner(x: &X, name: &str) -> Window {
    let a = atom(&x.conn, name);
    if a == 0 {
        return 0;
    }
    x.conn
        .get_selection_owner(a)
        .ok()
        .and_then(|r| r.reply().ok())
        .map_or(0, |r| r.owner)
}

fn diagnose_x(x: &X, o: &mut String) {
    use std::fmt::Write;
    macro_rules! say {
        ($($arg:tt)*) => {{ let _ = writeln!(o, $($arg)*); }};
    }
    let setup = x.conn.setup();
    say!(
        "vendor: {} (release {}), protocol {}.{}, {} screen(s), image byte order {:?}",
        String::from_utf8_lossy(&setup.vendor),
        setup.release_number,
        setup.protocol_major_version,
        setup.protocol_minor_version,
        setup.roots.len(),
        setup.image_byte_order
    );
    for name in [
        "Composite",
        "SHAPE",
        "RANDR",
        "XTEST",
        "MIT-SCREEN-SAVER",
        "BIG-REQUESTS",
        "XFIXES",
        "XInputExtension",
    ] {
        let present = x
            .conn
            .query_extension(name.as_bytes())
            .ok()
            .and_then(|r| r.reply().ok())
            .is_some_and(|r| r.present);
        say!("extension {}: {}", name, present);
    }
    say!("largest request: {} bytes", x.conn.maximum_request_bytes());
    say!("32-bit visual for sprites: {:?}", x.visual.map(|v| v.0));
    say!("Xft.dpi scale: {}", x.dpi_scale);

    // The window manager names itself on a child of the root.
    let check = atom(&x.conn, "_NET_SUPPORTING_WM_CHECK");
    let wm = cardinals(x, x.root, check, AtomEnum::WINDOW, 1)
        .first()
        .copied();
    say!(
        "window manager: {}",
        wm.map_or("(none says so)".into(), |w| window_name(x, w))
    );
    let cm = if x.a_wm_cm != 0 {
        x.conn
            .get_selection_owner(x.a_wm_cm)
            .ok()
            .and_then(|r| r.reply().ok())
            .map_or(0, |r| r.owner)
    } else {
        0
    };
    say!(
        "compositing manager: {}",
        if cm == 0 {
            "NONE: sprites cannot be shown".into()
        } else {
            format!("window {:#x} ({})", cm, window_name(x, cm))
        }
    );

    say!("\n== Monitors");
    if let Some(r) = x
        .conn
        .randr_get_monitors(x.root, true)
        .ok()
        .and_then(|r| r.reply().ok())
    {
        for m in &r.monitors {
            let name = x
                .conn
                .get_atom_name(m.name)
                .ok()
                .and_then(|r| r.reply().ok())
                .map_or("?".into(), |r| {
                    String::from_utf8_lossy(&r.name).into_owned()
                });
            say!(
                "randr monitor {}: {}x{} at {},{}{}",
                name,
                m.width,
                m.height,
                m.x,
                m.y,
                if m.primary { " primary" } else { "" }
            );
        }
    } else {
        say!("randr monitors: cannot ask");
    }
    let list = read_screens();
    say!("as the program sees them: {}", describe_screens(&list));
    say!(
        "work area from: {}",
        WORK_SOURCE.lock().unwrap_or_else(|e| e.into_inner())
    );
    let workarea = cardinals(x, x.root, x.a_workarea, AtomEnum::CARDINAL, 4);
    say!("_NET_WORKAREA: {:?}", workarea);
    say!("full-screen app in front: {}", is_fullscreen());
    say!("window in front: {}", foreground_description());

    say!("\n== Cursor (move the mouse now: 10 readings over 2 s)");
    for _ in 0..10 {
        if let Some(r) = x
            .conn
            .query_pointer(x.root)
            .ok()
            .and_then(|r| r.reply().ok())
        {
            say!(
                "{},{} buttons/modifiers {:#06x} over child {:#x} same screen {}",
                r.root_x,
                r.root_y,
                u16::from(r.mask),
                r.child,
                r.same_screen
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    let middle = list.first().map(|s| {
        (
            s.bounds.x + s.bounds.width / 2,
            s.bounds.y + s.bounds.height / 2,
        )
    });
    say!(
        "middle of the first monitor: {:?} (a cursor stuck there means XWayland cannot see it)",
        middle
    );

    say!("\n== XEmbed system tray");
    let screen = setup
        .roots
        .iter()
        .position(|r| r.root == x.root)
        .unwrap_or(0);
    let tray = selection_owner(x, &format!("_NET_SYSTEM_TRAY_S{}", screen));
    if tray == 0 {
        say!("no XEmbed tray (_NET_SYSTEM_TRAY_S{} has no owner)", screen);
    } else {
        let visual = cardinals(
            x,
            tray,
            atom(&x.conn, "_NET_SYSTEM_TRAY_VISUAL"),
            AtomEnum::VISUALID,
            1,
        );
        say!(
            "owner: window {:#x} ({}), icon visual {:?}",
            tray,
            window_name(x, tray),
            visual.first()
        );
    }
}

fn diagnose_dbus(o: &mut String) {
    use std::fmt::Write;
    macro_rules! say {
        ($($arg:tt)*) => {{ let _ = writeln!(o, $($arg)*); }};
    }
    let bus = match zbus::blocking::Connection::session() {
        Ok(b) => b,
        Err(e) => {
            say!(
                "no session bus: {} (no StatusNotifierItem tray, no idle clock)",
                e
            );
            return;
        }
    };
    let has_owner = |name: &str| -> String {
        bus.call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "NameHasOwner",
            &(name,),
        )
        .ok()
        .and_then(|r| r.body().deserialize::<bool>().ok())
        .map_or("cannot ask".into(), |b| b.to_string())
    };
    for name in [
        "org.kde.StatusNotifierWatcher",
        "org.freedesktop.StatusNotifierWatcher",
        "org.gnome.Mutter.IdleMonitor",
        "org.freedesktop.ScreenSaver",
        "org.gnome.Shell",
        "org.kde.KWin",
    ] {
        say!("{}: {}", name, has_owner(name));
    }
    for property in [
        "IsStatusNotifierHostRegistered",
        "ProtocolVersion",
        "RegisteredStatusNotifierItems",
    ] {
        let value = bus
            .call_method(
                Some("org.kde.StatusNotifierWatcher"),
                "/StatusNotifierWatcher",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.kde.StatusNotifierWatcher", property),
            )
            .map_err(|e| e.to_string())
            .and_then(|r| {
                r.body()
                    .deserialize::<zbus::zvariant::OwnedValue>()
                    .map(|v| format!("{:?}", v))
                    .map_err(|e| e.to_string())
            });
        say!(
            "StatusNotifierWatcher {}: {}",
            property,
            match value {
                Ok(v) => v,
                Err(e) => format!("({})", e),
            }
        );
    }
    for clock in &IDLE_CLOCKS {
        say!(
            "idle clock {}.{}: {:?}",
            clock.interface,
            clock.method,
            clock.read(&bus)
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

/// How long a sprite window that could not be made waits before trying again: at the start
/// of a session the program can be up before the compositor is.
const SPRITE_RETRY: std::time::Duration = std::time::Duration::from_secs(2);

/// Writes why sprite windows cannot be made, once for each new reason; "" once it works.
fn sprite_trouble(why: &'static str) {
    static LAST: std::sync::Mutex<&'static str> = std::sync::Mutex::new("");
    let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
    if *last != why {
        *last = why;
        if !why.is_empty() {
            debug_note!("sprite: cannot make a window: {}; retrying every 2 s", why);
        }
    }
}

/// A rectangle as [left, right) x [top, bottom); empty when it has no area.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Span {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Span {
    fn of(r: Rectangle) -> Span {
        Span {
            left: r.left(),
            top: r.top(),
            right: r.right(),
            bottom: r.bottom(),
        }
    }
    fn is_empty(self) -> bool {
        self.right <= self.left || self.bottom <= self.top
    }
    fn width(self) -> i32 {
        self.right - self.left
    }
    fn height(self) -> i32 {
        self.bottom - self.top
    }
    /// The smallest span holding both; an empty one adds nothing.
    fn union(self, o: Span) -> Span {
        if self.is_empty() {
            return o;
        }
        if o.is_empty() {
            return self;
        }
        Span {
            left: self.left.min(o.left),
            top: self.top.min(o.top),
            right: self.right.max(o.right),
            bottom: self.bottom.max(o.bottom),
        }
    }
    /// The part inside a `w` by `h` canvas.
    fn clip(self, w: i32, h: i32) -> Span {
        let s = Span {
            left: self.left.max(0),
            top: self.top.max(0),
            right: self.right.min(w),
            bottom: self.bottom.min(h),
        };
        if s.is_empty() {
            Span::default()
        } else {
            s
        }
    }
}

/// A borderless, always-on-top, click-through window that shows a `Surface` with
/// per-pixel transparency. Never takes focus.
///
/// The window is as big as the whole canvas and keeps that size while the figure moves
/// inside it: resizing a window that a compositor redirects gives it a new, empty pixmap
/// that the compositor can show for a frame, which made the cast flicker when the window
/// shrank to the drawn part every frame. Each frame is uploaded to a pixmap on the server
/// first and then copied to the window in one request, so the compositor never catches a
/// half-drawn frame either.
pub struct SpriteWindow {
    win: Option<Window>,
    gc: Gcontext,
    /// The frame being shown, as the server keeps it.
    pixmap: Option<Pixmap>,
    /// The window's (and the pixmap's) size.
    size: (u16, u16),
    pos: (i32, i32),
    visible: bool,
    /// The part of the pixmap the last frame drew on, to clear with the next one.
    drawn: Span,
    /// After a failed attempt, when to try making the window again.
    next_try: Option<std::time::Instant>,
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
            pixmap: None,
            size: (0, 0),
            pos: (0, 0),
            visible: false,
            drawn: Span::default(),
            next_try: None,
            bytes: Vec::new(),
        };
        s.ensure_window();
        s
    }

    /// Makes the window if it is not there yet; false while that cannot be done.
    fn ensure_window(&mut self) -> bool {
        if self.win.is_some() {
            return true;
        }
        let now = std::time::Instant::now();
        if self.next_try.is_some_and(|t| now < t) {
            return false;
        }
        self.next_try = Some(now + SPRITE_RETRY);
        let Some(x) = x() else {
            sprite_trouble("no X server");
            return false;
        };
        // Without a 32-bit visual and a compositor the transparent parts would be black.
        let Some((visual, cm)) = x.visual else {
            sprite_trouble("the X server has no 32-bit visual");
            return false;
        };
        if !compositing(x) {
            sprite_trouble(
                "no compositing manager owns _NET_WM_CM_Sn: start or turn on the compositor",
            );
            return false;
        }
        let Ok(win) = x.conn.generate_id() else {
            sprite_trouble("out of X resource ids");
            return false;
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
            sprite_trouble("CreateWindow failed");
            return false;
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
        // Compositors give menus and tooltips shadows and fades, which show as a dark
        // box around the cast or make it blink; a drag-and-drop icon gets neither, and
        // `_COMPTON_SHADOW` turns the shadow off in picom and compton. The class lets a
        // person name these windows in their compositor's rules.
        let a_type = atom(&x.conn, "_NET_WM_WINDOW_TYPE");
        let a_dnd = atom(&x.conn, "_NET_WM_WINDOW_TYPE_DND");
        if a_type != 0 && a_dnd != 0 {
            let _ =
                x.conn
                    .change_property32(PropMode::REPLACE, win, a_type, AtomEnum::ATOM, &[a_dnd]);
        }
        let a_shadow = atom(&x.conn, "_COMPTON_SHADOW");
        if a_shadow != 0 {
            let _ = x.conn.change_property32(
                PropMode::REPLACE,
                win,
                a_shadow,
                AtomEnum::CARDINAL,
                &[0],
            );
        }
        let _ = x.conn.change_property8(
            PropMode::REPLACE,
            win,
            AtomEnum::WM_CLASS,
            AtomEnum::STRING,
            b"idlewalker\0IdleWalker\0",
        );
        // No exposure events from copies: nothing reads this connection's events, so
        // they would only pile up.
        if let Ok(gc) = x.conn.generate_id() {
            let aux = CreateGCAux::new().graphics_exposures(0);
            if x.conn.create_gc(gc, win, &aux).is_ok() {
                self.gc = gc;
            }
        }
        let _ = x.conn.flush();
        self.win = Some(win);
        sprite_trouble("");
        debug_note!("sprite: window {:#x} made", win);
        true
    }

    /// Shows the surface with its top-left corner at (x, y). Only the part that has
    /// content now, or had it in the last frame, is uploaded.
    pub fn present(&mut self, surface: Surface, px: i32, py: i32) {
        if !self.ensure_window() {
            return;
        }
        let (Some(win), Some(x)) = (self.win, x()) else {
            return;
        };
        let (sw, sh) = (surface.width(), surface.height());
        if sw <= 0 || sh <= 0 {
            self.hide();
            return;
        }
        let size = (
            sw.min(u16::MAX as i32) as u16,
            sh.min(u16::MAX as i32) as u16,
        );
        let (cw, ch) = (size.0 as i32, size.1 as i32);
        // Whatever the window shows must be copied again once it is mapped or resized.
        let mut whole = !self.visible;
        if size != self.size || self.pixmap.is_none() {
            if let Some(old) = self.pixmap.take() {
                let _ = x.conn.free_pixmap(old);
            }
            let Ok(pixmap) = x.conn.generate_id() else {
                return;
            };
            if x.conn
                .create_pixmap(32, pixmap, win, size.0, size.1)
                .is_err()
            {
                return;
            }
            // A new pixmap holds whatever was in that memory before.
            let _ = x.conn.change_gc(self.gc, &ChangeGCAux::new().foreground(0));
            let _ = x.conn.poly_fill_rectangle(
                pixmap,
                self.gc,
                &[x11rb::protocol::xproto::Rectangle {
                    x: 0,
                    y: 0,
                    width: size.0,
                    height: size.1,
                }],
            );
            debug_note!(
                "sprite: window {:#x} is now {}x{}, first at {},{}",
                win,
                size.0,
                size.1,
                px,
                py
            );
            self.pixmap = Some(pixmap);
            self.drawn = Span::default();
            whole = true;
        }
        let Some(pixmap) = self.pixmap else { return };

        // This frame's content, and whatever the last one left that it must clear: the
        // surface is empty outside its bounds.
        let content = if surface.bounds().width > 0 && surface.bounds().height > 0 {
            Span::of(surface.bounds()).clip(cw, ch)
        } else {
            Span::default()
        };
        let region = content.union(self.drawn).clip(cw, ch);
        self.drawn = content;
        if !region.is_empty() {
            let row_bytes = region.width() as usize * 4;
            let max_rows = (x.conn.maximum_request_bytes().saturating_sub(64) / row_bytes).max(1);
            let pixels = surface.pixels.as_slice();
            let mut row = region.top;
            while row < region.bottom {
                let rows = (max_rows.min(i32::MAX as usize) as i32).min(region.bottom - row);
                self.bytes.clear();
                self.bytes.reserve(rows as usize * row_bytes);
                for r in row..row + rows {
                    let start = (r * sw + region.left) as usize;
                    for v in &pixels[start..start + region.width() as usize] {
                        self.bytes.extend_from_slice(&v.to_le_bytes());
                    }
                }
                let _ = x.conn.put_image(
                    ImageFormat::Z_PIXMAP,
                    pixmap,
                    self.gc,
                    region.width() as u16,
                    rows as u16,
                    region.left as i16,
                    row as i16,
                    0,
                    32,
                    &self.bytes,
                );
                row += rows;
            }
        }

        if size != self.size || (px, py) != self.pos {
            let aux = ConfigureWindowAux::new()
                .x(px)
                .y(py)
                .width(size.0 as u32)
                .height(size.1 as u32);
            let _ = x.conn.configure_window(win, &aux);
            self.size = size;
            self.pos = (px, py);
        }
        if !self.visible {
            let _ = x.conn.map_window(win);
            self.visible = true;
        }
        let shown = if whole {
            Span {
                left: 0,
                top: 0,
                right: cw,
                bottom: ch,
            }
        } else {
            region
        };
        if !shown.is_empty() {
            let _ = x.conn.copy_area(
                pixmap,
                win,
                self.gc,
                shown.left as i16,
                shown.top as i16,
                shown.left as i16,
                shown.top as i16,
                shown.width() as u16,
                shown.height() as u16,
            );
        }
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
        let Some(x) = x() else { return };
        if let Some(pixmap) = self.pixmap.take() {
            let _ = x.conn.free_pixmap(pixmap);
        }
        if self.gc != 0 {
            let _ = x.conn.free_gc(self.gc);
        }
        if let Some(win) = self.win {
            let _ = x.conn.destroy_window(win);
        }
        let _ = x.conn.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::Span;

    fn span(left: i32, top: i32, right: i32, bottom: i32) -> Span {
        Span {
            left,
            top,
            right,
            bottom,
        }
    }

    #[test]
    fn a_frame_clears_what_the_last_one_drew_and_stays_on_the_canvas() {
        let last = span(10, 10, 30, 40);
        let now = span(20, 5, 50, 35);
        assert_eq!(now.union(last), span(10, 5, 50, 40));
        assert_eq!(span(-5, -5, 200, 20).clip(100, 50), span(0, 0, 100, 20));
        // Nothing drawn now: only the old part is cleared.
        assert_eq!(Span::default().union(last), last);
        assert!(span(120, 0, 140, 10).clip(100, 50).is_empty());
    }
}
