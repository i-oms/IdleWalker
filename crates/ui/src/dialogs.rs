//! The windows people interact with: the brightness panel, the brains dashboard and the
//! colour picker. Ordinary winit windows, painted in software with the same small set
//! of controls on every OS.
use super::gfx::{self, button, slider, Canvas, Font, Mouse};
use super::overlays::DisplayBadges;
use super::{Services, UserEvent};
use crate::prelude::*;
use idlewalker_application::{BrightnessInfo, Desktop, Screen, UpdateState};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use winit::dpi::{PhysicalPosition, PhysicalSize, Position, Size};
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowLevel};

/// What a dialog asks of the app after an event.
pub enum Outcome {
    None,
    Close,
    Dress(i32),
    Menu(String),
}

/// Where a tray panel belongs: against the taskbar edge of the screen, whatever height it
/// has and however far up the screen the click came from (Windows keeps hidden tray icons
/// in a flyout well above the taskbar). On Windows that is always the bottom, even beside
/// a toolbar docked at the top. macOS and top-docked Linux panels hang from the top
/// instead, because that is where their tray icon is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrayPlace {
    x: i32,
    width: i32,
    top_limit: i32,
    bottom_limit: i32,
    from_top: bool,
}

impl TrayPlace {
    pub fn new(screen: &Screen, cursor_x: i32, width_dip: f64) -> Self {
        let (area, s) = (screen.work, screen.dpi_scale);
        let gap = (8.0 * s) as i32;
        let width = (width_dip * s) as i32;
        let left = area.left() + gap;
        let right = (area.right() - gap - width).max(left);
        TrayPlace {
            x: (cursor_x - width / 2).clamp(left, right),
            width,
            top_limit: area.top() + gap,
            bottom_limit: area.bottom() - gap,
            from_top: !cfg!(windows)
                && (cfg!(target_os = "macos")
                    || (area.top() > screen.bounds.top()
                        && area.bottom() >= screen.bounds.bottom())),
        }
    }

    /// The height to use for one that wants `wanted`: never more than the screen has.
    pub fn height(&self, wanted: i32) -> i32 {
        wanted.min(self.bottom_limit - self.top_limit)
    }

    /// The window's top at height `h`.
    fn top(&self, h: i32) -> i32 {
        if self.from_top {
            self.top_limit
        } else {
            (self.bottom_limit - h).max(self.top_limit)
        }
    }
}

/// After a tray panel is shown, ignore focus loss for this long: opening it via the status
/// item can cause a spurious focus change on macOS that must not dismiss the panel. Once
/// the grace has passed, losing focus means the user clicked outside, so the panel closes.
const PANEL_FOCUS_GRACE_MS: u64 = 300;
/// World pixels per pixel of this window. On macOS the world uses the largest backing scale
/// of any display, so on a mixed Retina and 1x setup a window's own pixels can differ from it.
#[cfg(target_os = "macos")]
fn world_per_physical(desktop: &dyn Desktop, window: &Window) -> f64 {
    desktop.world_per_point() / window.scale_factor()
}

#[cfg(not(target_os = "macos"))]
fn world_per_physical(_desktop: &dyn Desktop, _window: &Window) -> f64 {
    1.0
}

/// A new window's size and position from world pixels. macOS gets points, because winit
/// converts physical values at creation by the scale of whichever screen is "main".
#[cfg(target_os = "macos")]
fn new_window_geometry(desktop: &dyn Desktop, w: u32, h: u32, x: i32, y: i32) -> (Size, Position) {
    use winit::dpi::{LogicalPosition, LogicalSize};
    let f = desktop.world_per_point();
    (
        LogicalSize::new(w as f64 / f, h as f64 / f).into(),
        LogicalPosition::new(x as f64 / f, y as f64 / f).into(),
    )
}

#[cfg(not(target_os = "macos"))]
fn new_window_geometry(_desktop: &dyn Desktop, w: u32, h: u32, x: i32, y: i32) -> (Size, Position) {
    (
        PhysicalSize::new(w, h).into(),
        PhysicalPosition::new(x, y).into(),
    )
}

pub struct Dialog {
    pub window: Rc<Window>,
    /// Set for the bare tray panels, which `fit` keeps in place.
    place: Option<TrayPlace>,
    placed_top: i32,
    surface: softbuffer::Surface<Rc<Window>, Rc<Window>>,
    mouse: Mouse,
    drag: Option<i32>,
    pub kind: Kind,
    /// Instant the panel was shown, used to ignore spurious focus loss on macOS right after open.
    opened_at: Instant,
    services: Services,
}

pub enum Kind {
    Brightness(BrightnessPanel),
    Brains(BrainDialog),
    Colour(ColourDialog),
    Menu(super::menu_panel::MenuPanel),
    Update(UpdateDialog),
}

impl Dialog {
    pub fn open(
        el: &ActiveEventLoop,
        title: &str,
        place: Option<TrayPlace>,
        size: (u32, u32),
        kind: Kind,
        services: Services,
    ) -> Option<Dialog> {
        let bare = place.is_some();
        let placed_top = place.map_or(0, |p| p.top(size.1 as i32));
        let (inner, position) = new_window_geometry(
            &*services.desktop,
            size.0,
            size.1,
            place.map_or(0, |p| p.x),
            placed_top,
        );
        let mut attrs = Window::default_attributes()
            .with_title(title)
            .with_inner_size(inner)
            .with_resizable(false)
            .with_decorations(!bare)
            .with_window_level(WindowLevel::AlwaysOnTop);
        if bare {
            attrs = attrs.with_position(position);
        }
        #[cfg(windows)]
        {
            use winit::platform::windows::WindowAttributesExtWindows;
            attrs = attrs.with_skip_taskbar(bare);
        }
        let window = Rc::new(el.create_window(attrs).ok()?);
        let context = softbuffer::Context::new(window.clone()).ok()?;
        let surface = softbuffer::Surface::new(&context, window.clone()).ok()?;
        window.focus_window();
        Some(Dialog {
            window,
            place,
            placed_top,
            surface,
            mouse: Mouse::default(),
            drag: None,
            kind,
            opened_at: Instant::now(),
            services,
        })
    }

    /// Holds a tray panel to its place at the height its rows want. Checks the real window
    /// too: moving onto a monitor with other scaling makes the system resize and shift it.
    pub fn fit(&mut self) {
        let Some(place) = self.place else {
            return;
        };
        // The place is in world pixels; the rows and the window's calls are in its own.
        let k = world_per_physical(&*self.services.desktop, &self.window);
        let own = |world: i32| (world as f64 / k).round() as i32;
        let size = self.window.inner_size();
        let wanted = match &self.kind {
            Kind::Brightness(p) => p.wanted_height,
            Kind::Menu(p) => p.wanted_height,
            _ => 0,
        };
        let wanted = if wanted > 0 {
            wanted
        } else {
            size.height as i32
        };
        let h = place.height((wanted as f64 * k).round() as i32);
        let top = place.top(h);
        let target = PhysicalPosition::new(own(place.x), own(top));
        let target_size = PhysicalSize::new(own(place.width) as u32, own(h) as u32);
        let at = self.window.outer_position().ok();
        if at == Some(target) && size == target_size {
            return;
        }
        // Rows move with the window; where the cursor lies among them must follow at once,
        // or a quick second click would land on the row that used to be there.
        self.mouse.y -= own(top - self.placed_top);
        self.placed_top = top;
        self.window.set_outer_position(target);
        let _ = self.window.request_inner_size(target_size);
        self.window.request_redraw();
    }

    /// The window's outer rectangle in world pixels, to compare with the cursor.
    pub fn world_rect(&self) -> Option<Rectangle> {
        let k = world_per_physical(&*self.services.desktop, &self.window);
        let world = |v: f64| (v * k).round() as i32;
        let p = self.window.outer_position().ok()?;
        let z = self.window.outer_size();
        Some(Rectangle::new(
            world(p.x as f64),
            world(p.y as f64),
            world(z.width as f64),
            world(z.height as f64),
        ))
    }

    pub fn event(&mut self, ev: &WindowEvent, font: &mut Font, troupe: Troupe) -> Outcome {
        match ev {
            WindowEvent::CloseRequested => return Outcome::Close,
            WindowEvent::Focused(false)
                if matches!(self.kind, Kind::Brightness(_) | Kind::Menu(_)) =>
            {
                // On macOS a popup panel stays open right after it is shown (opening it via the
                // status item can cause a spurious focus change), so ignore focus loss during
                // the grace period. After that, losing focus means the user clicked outside:
                // dismiss the panel.
                if self.opened_at.elapsed().as_millis() < PANEL_FOCUS_GRACE_MS as u128 {
                    self.window.request_redraw();
                } else {
                    return Outcome::Close;
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                return Outcome::Close;
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse.x = position.x as i32;
                self.mouse.y = position.y as i32;
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.mouse.down = *state == ElementState::Pressed;
                if self.mouse.down {
                    self.mouse.clicked = true;
                } else {
                    self.drag = None;
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.mouse.wheel += match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 30.0,
                };
            }
            WindowEvent::Resized(_) => {}
            WindowEvent::RedrawRequested => return self.paint(font, troupe),
            _ => return Outcome::None,
        }
        self.window.request_redraw();
        Outcome::None
    }

    fn paint(&mut self, font: &mut Font, troupe: Troupe) -> Outcome {
        let size = self.window.inner_size();
        let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else {
            return Outcome::None;
        };
        if self.surface.resize(w, h).is_err() {
            return Outcome::None;
        }
        let Ok(mut buffer) = self.surface.buffer_mut() else {
            return Outcome::None;
        };
        let scale = self.window.scale_factor();
        let mut canvas = Canvas {
            px: &mut buffer,
            w: size.width as i32,
            h: size.height as i32,
        };
        let mouse = self.mouse;
        let outcome = match &mut self.kind {
            Kind::Brightness(p) => p.draw(&mut canvas, font, &mouse, &mut self.drag, scale),
            Kind::Brains(b) => b.draw(&mut canvas, font, &mouse, scale, troupe, &self.services),
            Kind::Colour(c) => c.draw(&mut canvas, font, &mouse, &mut self.drag, scale),
            Kind::Menu(p) => p.draw(&mut canvas, font, &mouse, scale),
            Kind::Update(u) => u.draw(&mut canvas, font, &mouse, scale, &self.services),
        };
        // Softbuffer's format is 0RGB.
        for p in buffer.iter_mut() {
            *p &= 0x00FFFFFF;
        }
        let _ = buffer.present();
        if self.mouse.clicked {
            self.mouse.clicked = false;
            self.window.request_redraw();
        }
        self.mouse.wheel = 0.0;
        outcome
    }
}

// ---- Brightness ---------------------------------------------------------------------------

/// Writes go out on a worker and coalesce: a slider dragged quickly sends only the
/// latest value once the previous write finishes, because a DDC/CI write is slow.
#[derive(Clone)]
struct Writer {
    desktop: Arc<dyn Desktop>,
    pending: Arc<Mutex<(HashMap<String, i32>, bool)>>,
}

impl Writer {
    fn set(&self, key: &str, percent: i32) {
        let mut p = self.pending.lock().unwrap();
        p.0.insert(key.to_string(), percent);
        if !p.1 {
            p.1 = true;
            let me = self.clone();
            std::thread::spawn(move || me.drain());
        }
    }
    fn drain(&self) {
        loop {
            let next = {
                let mut p = self.pending.lock().unwrap();
                let k = p.0.keys().next().cloned();
                match k {
                    Some(k) => {
                        let v = p.0.remove(&k).unwrap();
                        (k, v)
                    }
                    None => {
                        p.1 = false;
                        return;
                    }
                }
            };
            self.desktop.brightness_write(&next.0, next.1);
        }
    }
}

pub struct BrightnessPanel {
    pub displays: Option<Vec<BrightnessInfo>>,
    writer: Writer,
    pub badges: DisplayBadges,
    /// The panel's height as last laid out, so the app can resize it.
    pub wanted_height: i32,
}

impl BrightnessPanel {
    pub fn new(
        badges: DisplayBadges,
        proxy: EventLoopProxy<UserEvent>,
        desktop: Arc<dyn Desktop>,
    ) -> Self {
        // Slow: DDC/CI reads can take a while per monitor.
        let reader = desktop.clone();
        std::thread::spawn(move || {
            let found = reader.brightness_enumerate();
            let _ = proxy.send_event(UserEvent::Brightness(found));
        });
        BrightnessPanel {
            displays: None,
            writer: Writer {
                desktop,
                pending: Arc::default(),
            },
            badges,
            wanted_height: 0,
        }
    }

    fn draw(
        &mut self,
        c: &mut Canvas,
        f: &mut Font,
        m: &Mouse,
        drag: &mut Option<i32>,
        s: f64,
    ) -> Outcome {
        c.clear(gfx::BACK);
        c.round_box(0, 0, c.w, c.h, 1.0, gfx::BACK, gfx::RIM, 1.0);
        let px = |v: f64| (v * s) as i32;
        let (title, text) = ((15.0 * s) as f32, (13.0 * s) as f32);
        let left = px(14.0);
        let width = c.w - 2 * left;
        c.text(f, left, px(12.0), "Brightness", title, gfx::ACCENT);
        let mut y = px(44.0);
        let Some(displays) = &mut self.displays else {
            c.text(f, left, y, "Reading displays...", text, gfx::DIM);
            self.wanted_height = px(80.0);
            return Outcome::None;
        };
        let supported: Vec<usize> = (0..displays.len())
            .filter(|&i| displays[i].supported)
            .collect();
        let row_h = px(26.0);
        let mut light = None;
        // Mouse wheel over a display's label or slider nudges it, 5% per notch.
        let wheel_step = |value: i32, top: i32| -> Option<i32> {
            if m.wheel == 0.0 || !m.inside(left, top, width, px(20.0) + row_h) {
                return None;
            }
            let notches = if m.wheel.abs() < 1.0 {
                m.wheel.signum()
            } else {
                m.wheel.round()
            };
            Some((value + notches as i32 * 5).clamp(0, 100))
        };

        // All displays at once, only worth showing when there is more than one to move.
        if supported.len() > 1 {
            let mut avg = supported.iter().map(|&i| displays[i].percent).sum::<i32>()
                / supported.len() as i32;
            if let Some(v) = wheel_step(avg, y) {
                avg = v;
                for &i in &supported {
                    if v != displays[i].percent {
                        displays[i].percent = v;
                        self.writer.set(&displays[i].key, v);
                    }
                }
            }
            c.text(f, left, y, "All displays", text, gfx::WHITE);
            c.text_right(f, left + width, y, &format!("{}%", avg), text, gfx::DIM);
            y += px(20.0);
            if m.clicked && m.inside(left, y, width, row_h) {
                *drag = Some(-1);
            }
            if m.inside(left, y, width, row_h) || *drag == Some(-1) {
                light = Some(-1);
            }
            if let Some(v) = slider(
                c,
                m,
                *drag == Some(-1),
                left,
                y,
                width,
                row_h,
                s,
                avg,
                gfx::ACCENT,
            ) {
                for &i in &supported {
                    if v != displays[i].percent {
                        displays[i].percent = v;
                        self.writer.set(&displays[i].key, v);
                    }
                }
            }
            y += row_h + px(6.0);
        }
        for i in 0..displays.len() {
            let name = format!(
                "{}{}",
                displays[i].name,
                if displays[i].primary { " (main)" } else { "" }
            );
            c.text_fit(
                f,
                left,
                y,
                width - px(44.0),
                &name,
                text,
                if displays[i].supported {
                    gfx::WHITE
                } else {
                    gfx::DIM
                },
            );
            if !displays[i].supported {
                y += px(20.0);
                c.text_fit(f, left, y, width, "Not adjustable here", text, gfx::DIM);
                y += row_h;
                continue;
            }
            if let Some(v) = wheel_step(displays[i].percent, y) {
                if v != displays[i].percent {
                    displays[i].percent = v;
                    self.writer.set(&displays[i].key, v);
                }
            }
            c.text_right(
                f,
                left + width,
                y,
                &format!("{}%", displays[i].percent),
                text,
                gfx::DIM,
            );
            y += px(20.0);
            if m.clicked && m.inside(left, y, width, row_h) {
                *drag = Some(i as i32);
            }
            if m.inside(left, y, width, row_h) || *drag == Some(i as i32) {
                light = Some(i as i32);
            }
            if let Some(v) = slider(
                c,
                m,
                *drag == Some(i as i32),
                left,
                y,
                width,
                row_h,
                s,
                displays[i].percent,
                gfx::ACCENT,
            ) {
                if v != displays[i].percent {
                    displays[i].percent = v;
                    self.writer.set(&displays[i].key, v);
                }
            }
            y += row_h;
        }
        if let Some(i) = light {
            self.badges.highlight(i, f);
        }
        if supported.is_empty() {
            c.text_fit(
                f,
                left,
                y,
                width,
                "Turn on DDC/CI in the monitor's menu.",
                text,
                gfx::DIM,
            );
            y += px(30.0);
        }
        self.wanted_height = y + px(8.0);
        Outcome::None
    }
}

// ---- Updates ----------------------------------------------------------------------------------

/// The way from a finished download to the new version running, on this OS.
#[cfg(windows)]
const UPDATE_STEPS: [&str; 3] = [
    "1.  Click Download and wait for it to finish.",
    "2.  Click Exit IdleWalker below.",
    "3.  Run the downloaded file. Your settings are kept.",
];
#[cfg(target_os = "macos")]
const UPDATE_STEPS: [&str; 3] = [
    "1.  Click Download and wait for it to finish.",
    "2.  Click Exit IdleWalker below.",
    "3.  Open the download, drag IdleWalker to Applications, run it.",
];
#[cfg(target_os = "linux")]
const UPDATE_STEPS: [&str; 3] = [
    "1.  Click Download and wait for it to finish.",
    "2.  Click Exit IdleWalker below.",
    "3.  Make the file executable (chmod +x), then run it.",
];

pub struct UpdateDialog {
    pub state: UpdateState,
    download_opened: bool,
}

impl UpdateDialog {
    pub fn new(state: UpdateState) -> Self {
        UpdateDialog {
            state,
            download_opened: false,
        }
    }

    fn draw(
        &mut self,
        c: &mut Canvas,
        f: &mut Font,
        m: &Mouse,
        s: f64,
        services: &Services,
    ) -> Outcome {
        let update = &services.updates;
        c.clear(gfx::BACK);
        let px = |v: f64| (v * s) as i32;
        let (title, text) = ((18.0 * s) as f32, (13.0 * s) as f32);
        let left = px(18.0);
        let width = c.w - 2 * left;
        let (row_y, row_h) = (c.h - px(50.0), px(34.0));
        let running = update.current_version();
        let mut close_label = "Close";
        match self.state.clone() {
            UpdateState::Checking => {
                c.text(
                    f,
                    left,
                    px(18.0),
                    "Checking for updates...",
                    title,
                    gfx::ACCENT,
                );
                let line = format!("Your version: {running}");
                c.text_fit(f, left, px(54.0), width, &line, text, gfx::DIM);
            }
            UpdateState::UpToDate => {
                c.text(
                    f,
                    left,
                    px(18.0),
                    "You have the latest version",
                    title,
                    gfx::ACCENT,
                );
                let line = format!("Your version: {running}");
                c.text_fit(f, left, px(54.0), width, &line, text, gfx::WHITE);
            }
            UpdateState::Failed => {
                c.text(
                    f,
                    left,
                    px(18.0),
                    "Could not check for updates",
                    title,
                    gfx::ACCENT,
                );
                let line = "Check your connection, or look on the releases page.";
                c.text_fit(f, left, px(54.0), width, line, text, gfx::WHITE);
                if button(
                    c,
                    f,
                    m,
                    left,
                    px(84.0),
                    px(170.0),
                    row_h,
                    s,
                    "Open releases page",
                ) {
                    services.desktop.open_url(update.releases_url());
                }
            }
            UpdateState::Available(version) => {
                close_label = "Later";
                c.text(
                    f,
                    left,
                    px(16.0),
                    "A new version is available",
                    title,
                    gfx::ACCENT,
                );
                let line = format!("Your version: {running}");
                c.text_fit(f, left, px(50.0), width, &line, text, gfx::DIM);
                let line = format!("New version: {version}");
                c.text_fit(f, left, px(70.0), width, &line, text, gfx::GREEN);

                let download = format!("Download version {version}");
                let w = px(210.0);
                if button(c, f, m, left, px(98.0), w, row_h, s, &download) {
                    services.desktop.open_url(&update.download_url(&version));
                    self.download_opened = true;
                }
                if self.download_opened {
                    let (x, y) = (left + w + px(12.0), px(107.0));
                    c.text_fit(
                        f,
                        x,
                        y,
                        width - w - px(12.0),
                        "Sent to your browser",
                        text,
                        gfx::GREEN,
                    );
                }

                // Clear of everything else, because skipping this looks like a broken update:
                // a second copy quietly exits while this one is still running.
                let (top, pad) = (px(146.0), px(14.0));
                let box_h = px(176.0);
                c.round_box(
                    left,
                    top,
                    width,
                    box_h,
                    6.0 * s,
                    gfx::PANEL,
                    gfx::ACCENT,
                    (2.0 * s).max(1.0),
                );
                let inner = width - 2 * pad;
                let x = left + pad;
                let heading = "EXIT THIS APP BEFORE RUNNING THE NEW VERSION";
                c.text_fit(f, x, top + px(12.0), inner, heading, text, gfx::ACCENT);
                let why = [
                    "IdleWalker keeps running in the background, even with",
                    "no window open. The new version will not start until it exits.",
                ];
                let mut y = top + px(36.0);
                for line in why {
                    c.text_fit(f, x, y, inner, line, text, gfx::WHITE);
                    y += px(19.0);
                }
                y += px(8.0);
                for line in UPDATE_STEPS {
                    c.text_fit(f, x, y, inner, line, text, gfx::WHITE);
                    y += px(21.0);
                }
                if button(c, f, m, left, row_y, px(150.0), row_h, s, "Exit IdleWalker") {
                    return Outcome::Menu("exit".into());
                }
            }
        }
        let w = px(90.0);
        if button(c, f, m, left + width - w, row_y, w, row_h, s, close_label) {
            return Outcome::Close;
        }
        Outcome::None
    }
}

// ---- The brains dashboard ---------------------------------------------------------------------

struct Row {
    who: Walker,
    cells: [String; 10],
    nums: [f64; 10],
}

const COLUMNS: [(&str, i32); 10] = [
    ("Name", 85),
    ("Temperament", 96),
    ("Tastes", 50),
    ("Friends", 54),
    ("Best friend", 85),
    ("Rival", 85),
    ("Own walk", 72),
    ("Decisions", 66),
    ("Avg reward", 84),
    ("Net", 50),
];

#[derive(Default)]
pub struct BrainDialog {
    sort: Option<(usize, bool)>,
    selected: Option<Walker>,
    scroll: i32,
    confirm: bool,
}

fn row_of(mut w: Walker) -> Row {
    let (t, fr, exp) = (w.learned_entries(), w.friendships(), w.experience());
    let (avg, net) = (w.avg_reward(), w.mlp_weight_norm());
    Row {
        who: w,
        cells: [
            w.personality.name.clone(),
            w.personality.temperament.to_string(),
            t.to_string(),
            fr.to_string(),
            w.best_friend_name(),
            w.rival_name(),
            if w.has_own_walk() {
                "Yes".into()
            } else {
                "-".into()
            },
            exp.to_string(),
            format!("{:.2}", avg),
            format!("{:.2}", net),
        ],
        nums: [
            0.0, 0.0, t as f64, fr as f64, 0.0, 0.0, 0.0, exp as f64, avg, net,
        ],
    }
}

fn describe_size(bytes: i64) -> String {
    if bytes < 1024 {
        return format!("{} bytes", bytes);
    }
    let kb = bytes as f64 / 1024.0;
    if kb < 1024.0 {
        format!("{:.1} KB ({} bytes)", kb, bytes)
    } else {
        format!("{:.2} MB ({} bytes)", kb / 1024.0, bytes)
    }
}

impl BrainDialog {
    fn draw(
        &mut self,
        c: &mut Canvas,
        f: &mut Font,
        m: &Mouse,
        s: f64,
        mut troupe: Troupe,
        services: &Services,
    ) -> Outcome {
        c.clear(gfx::BACK);
        let px = |v: i32| (v as f64 * s) as i32;
        let text = (13.0 * s) as f32;
        let left = px(12);
        let width = c.w - 2 * left;
        troupe.ensure_cast();

        let path = services.brains.location();
        c.text_fit(
            f,
            left,
            px(10),
            px(560),
            &format!(
                "File: {}",
                path.as_ref()
                    .map_or("(unavailable)".to_string(), |p| p.display().to_string())
            ),
            text,
            gfx::WHITE,
        );
        let dt = DateTime::now();
        c.text_right(
            f,
            left + width,
            px(10),
            &format!("Live - {:02}:{:02}", dt.hour(), dt.minute()),
            text,
            gfx::GREEN,
        );
        let size = services.brains.size();
        c.text(
            f,
            left,
            px(30),
            &format!(
                "Size: {}",
                if size < 0 {
                    "nothing saved yet".to_string()
                } else {
                    describe_size(size)
                }
            ),
            text,
            gfx::WHITE,
        );

        let mut rows: Vec<Row> = troupe.walkers().items().map(row_of).collect();
        let (mut friends, mut tastes, mut decisions) = (0.0, 0.0, 0.0);
        for r in &rows {
            friends += r.nums[3];
            tastes += r.nums[2];
            decisions += r.nums[7];
        }
        c.text(
            f,
            left,
            px(52),
            &format!(
                "{} walkers  -  {} friendships  -  {} learned tastes  -  {} decisions",
                rows.len(),
                friends,
                tastes,
                decisions
            ),
            text,
            gfx::ACCENT,
        );
        c.text(
            f,
            left,
            px(72),
            "Live - they keep playing behind this card. Click a header to sort.",
            text,
            gfx::DIM,
        );

        if let Some((col, descending)) = self.sort {
            rows.sort_by(|a, b| {
                let o =
                    if a.nums[col] != 0.0 || b.nums[col] != 0.0 || matches!(col, 2 | 3 | 7 | 8 | 9)
                    {
                        a.nums[col]
                            .partial_cmp(&b.nums[col])
                            .unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        a.cells[col].cmp(&b.cells[col])
                    };
                if descending {
                    o.reverse()
                } else {
                    o
                }
            });
        }

        // The table.
        let top = px(96);
        let head_h = px(24);
        let row_h = px(22);
        let bottom = c.h - px(48);
        let body_h = bottom - top - head_h;
        c.rect(left, top, width, bottom - top, gfx::PANEL);
        let mut x = left;
        for (i, (name, w)) in COLUMNS.iter().enumerate() {
            let w = px(*w);
            if m.clicked && m.inside(x, top, w, head_h) {
                self.sort = match self.sort {
                    Some((col, false)) if col == i => Some((i, true)),
                    _ => Some((i, false)),
                };
            }
            let mark = match self.sort {
                Some((col, d)) if col == i => {
                    if d {
                        " v"
                    } else {
                        " ^"
                    }
                }
                _ => "",
            };
            c.text_fit(
                f,
                x + px(4),
                top + px(4),
                w - px(6),
                &format!("{}{}", name, mark),
                text,
                gfx::WHITE,
            );
            x += w;
        }
        let max_scroll = (rows.len() as i32 * row_h - body_h).max(0);
        self.scroll = (self.scroll - (m.wheel * row_h as f32 * 2.0) as i32).clamp(0, max_scroll);
        for (n, row) in rows.iter().enumerate() {
            let y = top + head_h + n as i32 * row_h - self.scroll;
            if y + row_h <= top + head_h || y >= bottom {
                continue;
            }
            if m.clicked && m.inside(left, y, width, row_h) && m.y >= top + head_h && m.y < bottom {
                self.selected = Some(row.who);
            }
            if self.selected == Some(row.who) {
                c.rect(left, y.max(top + head_h), width, row_h, 0xFF2C4A7A);
            } else if n % 2 == 1 {
                c.rect(left, y.max(top + head_h), width, row_h, 0xFF242836);
            }
            if y < top + head_h || y + row_h > bottom {
                continue;
            }
            let mut x = left;
            for (i, (_, w)) in COLUMNS.iter().enumerate() {
                let w = px(*w);
                let numeric = matches!(i, 2 | 3 | 7 | 8 | 9);
                if numeric {
                    c.text_right(f, x + w - px(6), y + px(3), &row.cells[i], text, gfx::WHITE);
                } else {
                    c.text_fit(
                        f,
                        x + px(4),
                        y + px(3),
                        w - px(6),
                        &row.cells[i],
                        text,
                        gfx::WHITE,
                    );
                }
                x += w;
            }
        }

        // The buttons.
        let by = c.h - px(38);
        let bh = px(28);
        if button(c, f, m, left, by, px(140), bh, s, "Open file location") {
            if let Some(p) = &path {
                services.desktop.reveal_file(p);
            }
        }
        if self.confirm {
            c.text(
                f,
                left + px(150),
                by + px(5),
                "Forget all they have learned?",
                text,
                gfx::ACCENT,
            );
            let bx = left + px(370);
            if button(c, f, m, bx, by, px(60), bh, s, "Yes") {
                services.brains.forget(troupe);
                self.confirm = false;
            }
            if button(c, f, m, bx + px(66), by, px(60), bh, s, "No") {
                self.confirm = false;
            }
        } else if button(
            c,
            f,
            m,
            left + px(148),
            by,
            px(140),
            bh,
            s,
            "Forget everything...",
        ) {
            self.confirm = true;
        }
        if button(c, f, m, left + width - px(80), by, px(80), bh, s, "Close") {
            return Outcome::Close;
        }
        Outcome::None
    }
}

// ---- The colour picker ------------------------------------------------------------------------

/// A small stand-in for the Windows colour dialog: there is no such thing on every OS.
pub struct ColourDialog {
    pub rgb: [i32; 3],
}

impl ColourDialog {
    pub fn new(start: i32) -> Self {
        ColourDialog {
            rgb: [(start >> 16) & 255, (start >> 8) & 255, start & 255],
        }
    }

    fn draw(
        &mut self,
        c: &mut Canvas,
        f: &mut Font,
        m: &Mouse,
        drag: &mut Option<i32>,
        s: f64,
    ) -> Outcome {
        c.clear(gfx::BACK);
        let px = |v: i32| (v as f64 * s) as i32;
        let text = (13.0 * s) as f32;
        let left = px(14);
        let width = c.w - 2 * left;
        let color = (self.rgb[0] << 16) | (self.rgb[1] << 8) | self.rgb[2];
        c.text(
            f,
            left,
            px(12),
            "Overalls colour",
            (15.0 * s) as f32,
            gfx::ACCENT,
        );
        let mut y = px(42);
        c.round_box(
            left,
            y,
            width,
            px(36),
            6.0 * s,
            0xFF000000 | color as u32,
            gfx::RIM,
            1.0,
        );
        c.text(
            f,
            left,
            y + px(44),
            &format!("#{:06X}", color),
            text,
            gfx::DIM,
        );
        y += px(66);
        let labels = ["Red", "Green", "Blue"];
        let colors = [0xFFE05050, 0xFF50C060, 0xFF5090E0];
        for i in 0..3 {
            c.text(f, left, y, labels[i], text, gfx::WHITE);
            c.text_right(f, left + width, y, &self.rgb[i].to_string(), text, gfx::DIM);
            y += px(20);
            let h = px(26);
            if m.clicked && m.inside(left, y, width, h) {
                *drag = Some(i as i32);
            }
            let shown = self.rgb[i] * 100 / 255;
            if slider(
                c,
                m,
                *drag == Some(i as i32),
                left,
                y,
                width,
                h,
                s,
                shown,
                colors[i],
            )
            .is_some()
            {
                self.rgb[i] =
                    (((m.x - left) as f64 * 255.0 / width as f64).round() as i32).clamp(0, 255);
            }
            y += h + px(4);
        }
        let by = c.h - px(40);
        if button(c, f, m, left + width - px(170), by, px(80), px(28), s, "OK") {
            return Outcome::Dress(color);
        }
        if button(
            c,
            f,
            m,
            left + width - px(80),
            by,
            px(80),
            px(28),
            s,
            "Cancel",
        ) {
            return Outcome::Close;
        }
        Outcome::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn taskbar_screen() -> Screen {
        Screen {
            bounds: Rectangle::new(0, 0, 1920, 1080),
            work: Rectangle::new(0, 0, 1920, 1032),
            dpi_scale: 1.0,
            primary: true,
        }
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn the_bottom_stays_on_the_taskbar_edge_at_any_height() {
        let place = TrayPlace::new(&taskbar_screen(), 1700, 300.0);
        for h in [80, 200, 600, 1016] {
            assert_eq!(place.top(place.height(h)) + place.height(h), 1024);
        }
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn a_panel_taller_than_the_screen_stays_on_it() {
        let place = TrayPlace::new(&taskbar_screen(), 1700, 300.0);
        let h = place.height(5000);
        assert_eq!((place.top(h), h), (8, 1016));
    }

    #[test]
    fn a_top_panel_decides_only_off_windows() {
        let mut screen = taskbar_screen();
        screen.work = Rectangle::new(0, 32, 1920, 1048);
        let place = TrayPlace::new(&screen, 1700, 300.0);
        let expected = if cfg!(windows) { 1072 - 200 } else { 40 };
        assert_eq!(place.top(200), expected);
    }

    #[test]
    fn it_stays_inside_the_screen_sideways() {
        let place = TrayPlace::new(&taskbar_screen(), 1915, 300.0);
        assert_eq!(place.x, 1920 - 8 - 300);
    }
}
