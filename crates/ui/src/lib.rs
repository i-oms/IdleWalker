//! The presentation layer: the winit event loop, the frame pump, the sprite windows,
//! the tray menu, the cards and the dialogs. What happens each frame is the
//! `Session`'s business; this crate shows it, and turns clicks into its commands. It
//! never names an OS service: they all arrive as ports from the composition root.
// Style lints only: the painters take their geometry as plain arguments, and the index
// loops mirror the rows they draw. Compiler warnings are not silenced.
#![allow(
    clippy::too_many_arguments,
    clippy::new_without_default,
    clippy::needless_range_loop
)]
pub mod dialogs;
pub mod gfx;
pub mod menu_panel;
pub mod overlays;
pub mod posh_dialog;
pub mod tray;
pub mod update;

pub(crate) mod prelude {
    pub use idlewalker_rendering::prelude::*;
}

use crate::prelude::*;
use dialogs::{
    BrainDialog, BrightnessPanel, ColourDialog, Dialog, Kind, Outcome, TrayPlace, UpdateDialog,
};
use gfx::Font;
use idlewalker_application::{
    Brains, BrightnessInfo, Desktop, Journal, Posh, Screen, Session, UpdateState, Updates,
};
use overlays::{AboutToast, CornerRipple, DisplayBadges};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tray::Tray;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::WindowId;

pub enum UserEvent {
    Tick,
    Menu(String),
    TrayLeftClick,
    Brightness(Vec<BrightnessInfo>),
    /// The answer of an update check; `asked` when the person started it from the menu.
    UpdateChecked {
        state: UpdateState,
        asked: bool,
    },
    /// A terminal prompt task has finished.
    Posh(Box<idlewalker_application::posh::Done>),
}

/// What the dialogs use besides the cast.
#[derive(Clone)]
pub struct Services {
    pub desktop: Arc<dyn Desktop>,
    pub brains: Brains,
    pub updates: Updates,
    pub posh: Posh,
}

/// Everything the presentation layer is handed by the composition root.
pub struct Wiring {
    /// Builds the session; runs once the event loop exists, so the OS is ready for it.
    pub session: Box<dyn FnOnce() -> Session>,
    pub sprites: SpriteFactory,
    pub updates: Updates,
    /// The terminal prompt use case.
    pub posh: Posh,
    /// For trouble before the session exists.
    pub journal: Arc<dyn Journal>,
    /// Runs first on each painter thread.
    pub on_painter_start: fn(),
    /// Background update checks; off for test runs.
    pub check_for_updates: bool,
    /// Menu items to fire at start-up, so a test can drive the menu.
    pub test_menu: Vec<String>,
}

// ---- The frame pump ----------------------------------------------------------------------

const WATCH_INTERVAL_MS: u64 = 30;
const MIN_TICK_MS: f64 = 9.0;
const HALF_RATE_MIN_TICK_MS: f64 = 20.0;
const MAX_FLUSHES_PER_TICK: i32 = 8;
const CLOCK_TICK_MS: f64 = 16.0;

#[derive(Default)]
struct PumpState {
    animating: AtomicBool,
    half_rate: AtomicBool,
    stopping: AtomicBool,
    pending: AtomicBool,
}

/// One tick per screen refresh while animating (never faster than about 100 Hz), a lazy
/// poll while idle: it also drives the hot corners, so it samples well within the
/// shortest dwell. Where there is no compositor wait, it paces by the clock.
fn spawn_pump(state: Arc<PumpState>, proxy: EventLoopProxy<UserEvent>, desktop: Arc<dyn Desktop>) {
    let _ = std::thread::Builder::new()
        .name("IdleWalker frame pump".into())
        .spawn(move || {
            while !state.stopping.load(Ordering::Relaxed) {
                if state.animating.load(Ordering::Relaxed) {
                    let frame = Instant::now();
                    let half = state.half_rate.load(Ordering::Relaxed);
                    let min_ms = if half {
                        HALF_RATE_MIN_TICK_MS
                    } else {
                        MIN_TICK_MS
                    };
                    let mut paced = false;
                    for _ in 0..MAX_FLUSHES_PER_TICK {
                        if !desktop.wait_vblank() {
                            break;
                        }
                        paced = true;
                        if frame.elapsed().as_secs_f64() * 1000.0 >= min_ms {
                            break;
                        }
                    }
                    if !paced || frame.elapsed().as_secs_f64() * 1000.0 < 3.0 {
                        let ms = if half {
                            HALF_RATE_MIN_TICK_MS * 1.6
                        } else {
                            CLOCK_TICK_MS
                        };
                        std::thread::sleep(Duration::from_secs_f64(ms / 1000.0));
                    }
                } else {
                    desktop.pacing_idle();
                    std::thread::sleep(Duration::from_millis(WATCH_INTERVAL_MS));
                }
                // Never queue a second tick while one is waiting.
                if state.pending.swap(true, Ordering::AcqRel) {
                    continue;
                }
                if proxy.send_event(UserEvent::Tick).is_err() {
                    return;
                }
            }
        });
}

// ---- The app ---------------------------------------------------------------------------------------

struct View {
    renderer: FrameRenderer,
    window: Box<dyn SpriteSink>,
}

struct App {
    proxy: EventLoopProxy<UserEvent>,
    pump: Arc<PumpState>,
    session: Session,
    services: Services,
    sprites: SpriteFactory,
    on_painter_start: fn(),
    check_for_updates: bool,
    test_menu: Vec<String>,
    ripple: Option<CornerRipple>,
    clock: Instant,
    views: Vec<View>,
    tray: Option<Tray>,
    paint: RenderThread,
    font: Font,
    last_tick: f64,
    next_topmost: f64,
    next_dialog_refresh: f64,
    paint_every: i32,
    tick: i32,
    load_since: f64,
    load_paint_ms: f64,
    /// The smoothed paint demand, in cores; negative until the first reading.
    load_demand: f64,
    /// Since when the demand would fit a higher draw rate comfortably.
    rate_room_since: Option<f64>,
    about: Option<AboutToast>,
    dialogs: HashMap<WindowId, Dialog>,
    brightness_closed: Option<Instant>,
    menu_closed: Option<Instant>,
    started: bool,
}

impl App {
    fn new(proxy: EventLoopProxy<UserEvent>, wiring: Wiring) -> Self {
        let session = (wiring.session)();
        let services = Services {
            desktop: session.desktop().clone(),
            brains: session.brains().clone(),
            updates: wiring.updates,
            posh: wiring.posh,
        };
        App {
            pump: Arc::new(PumpState::default()),
            session,
            services,
            sprites: wiring.sprites,
            on_painter_start: wiring.on_painter_start,
            check_for_updates: wiring.check_for_updates,
            test_menu: wiring.test_menu,
            ripple: None,
            clock: Instant::now(),
            views: Vec::new(),
            tray: None,
            paint: RenderThread::default(),
            font: Font::new(),
            proxy,
            last_tick: 0.0,
            next_topmost: 0.0,
            next_dialog_refresh: 0.0,
            paint_every: 1,
            tick: 0,
            load_since: 0.0,
            load_paint_ms: 0.0,
            load_demand: -1.0,
            rate_room_since: None,
            about: None,
            dialogs: HashMap::new(),
            brightness_closed: None,
            menu_closed: None,
            started: false,
        }
    }

    fn desktop(&self) -> &dyn Desktop {
        &*self.services.desktop
    }

    fn on_tick(&mut self, el: &ActiveEventLoop) {
        let now = self.clock.elapsed().as_secs_f64();
        let dt = (now - self.last_tick).min(0.05);
        self.last_tick = now;

        let report = self.session.tick(now, dt);
        if let Some(on_battery) = report.on_battery {
            self.pump.half_rate.store(on_battery, Ordering::Relaxed);
        }
        if let Some(fire) = report.fired {
            if fire.action != CornerAction::None {
                self.ripple_for(fire.monitor, fire.corner);
            }
        }

        // The walkers are re-raised once a second, except under the About card.
        let raise = now >= self.next_topmost && self.about.is_none();
        if raise {
            self.next_topmost = now + 1.0;
        }

        // The paint budget: over it, snapshots are taken every 2nd (3rd, 4th) tick, so
        // the cast keeps moving at full rate and is drawn at a lower, steady one. The
        // demand is smoothed over a few seconds, the rate drops only when that stays over
        // budget, and comes back only after several seconds of comfortable room, so a
        // demand that hovers near the budget does not flip the frame rate back and forth.
        self.load_paint_ms += self.paint.take_paint_ms();
        if now - self.load_since >= 1.0 {
            let sample =
                self.load_paint_ms / ((now - self.load_since) * 1000.0) * self.paint_every as f64;
            self.load_demand = if self.load_demand < 0.0 {
                sample
            } else {
                0.6 * self.load_demand + 0.4 * sample
            };
            let budget = self.paint.budget();
            let want =
                ((self.load_demand / budget).ceil() as i32).clamp(1, Settings::MAX_PAINT_EVERY);
            if want > self.paint_every {
                self.paint_every = want;
                self.rate_room_since = None;
            } else if want < self.paint_every && self.load_demand <= 0.7 * budget * want as f64 {
                let since = *self.rate_room_since.get_or_insert(now);
                if now - since >= 5.0 {
                    self.paint_every = want;
                    self.rate_room_since = None;
                }
            } else {
                self.rate_room_since = None;
            }
            self.load_since = now;
            self.load_paint_ms = 0.0;
        }
        self.tick = self.tick.wrapping_add(1);
        let snapshot = self.tick % self.paint_every == 0;

        // Snapshots are cheap and made here for every walker that changed, the render
        // thread turns them into pixels, and only the finished frames are shown.
        let walkers = self.session.troupe().walkers();
        for i in 0..walkers.len() as usize {
            while self.views.len() <= i {
                self.views.push(View {
                    renderer: FrameRenderer::new(),
                    window: (self.sprites)(),
                });
            }
            let view = &mut self.views[i];
            let walker = walkers[i as i32];
            if walker.is_hidden() {
                view.window.hide();
                view.renderer.invalidate();
                continue;
            }
            if snapshot {
                let mut job = RenderJob::default();
                if view.renderer.prepare(walker, &mut job) {
                    self.paint.request(i, job);
                }
            }
            if raise {
                view.window.stay_on_top();
            }
        }
        while let Some(r) = self.paint.try_take() {
            let i = r.walker as usize;
            let current = i < self.views.len()
                && (i as i32) < walkers.len()
                && !walkers[i as i32].is_hidden()
                && r.stamp == self.views[i].renderer.stamp();
            if current {
                self.views[i].window.present(r.surface, r.left, r.top); // otherwise stale: hid or moved on mid-paint
            }
            self.paint.release(r.surface);
        }

        self.close_menu_on_outside_press();

        // The cards tick here; the idle poll is frequent enough for them.
        if let Some(r) = &mut self.ripple {
            r.tick();
        }
        if let Some(a) = &mut self.about {
            if !a.tick(
                self.session.cursor(),
                self.services.desktop.mouse_button_down(),
            ) {
                self.about = None;
            }
        }
        if now >= self.next_dialog_refresh {
            self.next_dialog_refresh = now + 1.0;
            for d in self.dialogs.values() {
                if matches!(d.kind, Kind::Brains(_)) {
                    d.window.request_redraw();
                }
            }
        }

        let ripple_on = self.ripple.as_ref().is_some_and(|r| r.active());
        self.pump.animating.store(
            report.any_visible || ripple_on || self.about.is_some(),
            Ordering::Relaxed,
        );
        let _ = el;
    }

    /// Starts the ripple on the monitor corner that just fired.
    fn ripple_for(&mut self, fired: i32, corner: Corner) {
        let displays = self.session.displays();
        let screens = displays.screens();
        let s = screens
            .get(fired as usize)
            .copied()
            .unwrap_or_else(|| displays.screen_at(self.session.cursor()));
        let b = s.bounds;
        let (x, y) = match corner {
            Corner::TopRight => (b.right() - 1, b.top()),
            Corner::BottomLeft => (b.left(), b.bottom() - 1),
            Corner::BottomRight => (b.right() - 1, b.bottom() - 1),
            _ => (b.left(), b.top()),
        };
        let sprites = &self.sprites;
        self.ripple
            .get_or_insert_with(|| CornerRipple::new(sprites))
            .show(x, y, s.cell());
    }

    fn update_menu(&mut self) {
        let nodes = tray::build(self.session.choices(), &*self.services.desktop);
        for d in self.dialogs.values_mut() {
            if let Kind::Menu(p) = &mut d.kind {
                p.nodes = nodes;
                d.window.request_redraw();
                break;
            }
        }
    }

    fn on_menu(&mut self, id: &str, el: &ActiveEventLoop) {
        // The choices are the session's; what is left opens or closes something.
        if !self.session.apply(id) {
            match id {
                "dress-custom" => self.open_colour(el),
                "menu" => self.toggle_menu(el),
                "brightness" => self.toggle_brightness(el),
                "brains" => self.open_brains(el),
                "posh" => self.open_posh(el),
                "check-update" => self.check_for_update(el),
                "about" => {
                    if self.about.is_none() {
                        let screen = self
                            .session
                            .displays()
                            .screen_at(self.session.cursor_anchor());
                        self.about = Some(AboutToast::new(screen, &mut self.font, &self.sprites));
                        self.pump.animating.store(true, Ordering::Relaxed);
                    }
                }
                "exit" => el.exit(),
                _ => {}
            }
        }
        self.update_menu();
    }

    fn open_kind(
        &mut self,
        el: &ActiveEventLoop,
        title: &str,
        place: Option<TrayPlace>,
        size: (u32, u32),
        kind: Kind,
    ) {
        if let Some(d) = Dialog::open(el, title, place, size, kind, self.services.clone()) {
            self.dialogs.insert(d.window.id(), d);
        }
    }

    /// Display scaling of the screen the cursor rests on.
    fn scale_at_cursor(&self) -> f64 {
        self.session
            .displays()
            .screen_at(self.session.cursor_anchor())
            .dpi_scale
    }

    fn open_colour(&mut self, el: &ActiveEventLoop) {
        if let Some(d) = self
            .dialogs
            .values()
            .find(|d| matches!(d.kind, Kind::Colour(_)))
        {
            d.window.focus_window();
            return;
        }
        let scale = self.scale_at_cursor();
        let d = self.session.choices().dress;
        let start = if d == Settings::RANDOM_DRESS {
            SETTINGS_DRESS_COLORS[0]
        } else {
            d
        };
        let size = ((320.0 * scale) as u32, (290.0 * scale) as u32);
        self.open_kind(
            el,
            "Overalls colour",
            None,
            size,
            Kind::Colour(ColourDialog::new(start)),
        );
    }

    fn open_brains(&mut self, el: &ActiveEventLoop) {
        if let Some(d) = self
            .dialogs
            .values()
            .find(|d| matches!(d.kind, Kind::Brains(_)))
        {
            d.window.focus_window();
            return;
        }
        let scale = self.scale_at_cursor();
        let size = ((760.0 * scale) as u32, (470.0 * scale) as u32);
        self.open_kind(
            el,
            "Walker brains",
            None,
            size,
            Kind::Brains(BrainDialog::default()),
        );
    }

    fn open_posh(&mut self, el: &ActiveEventLoop) {
        if let Some(d) = self
            .dialogs
            .values()
            .find(|d| matches!(d.kind, Kind::Posh(_)))
        {
            d.window.focus_window();
            return;
        }
        let scale = self.scale_at_cursor();
        let (w, h) = posh_dialog::SIZE;
        let size = ((w * scale) as u32, (h * scale) as u32);
        let dialog = posh_dialog::PoshDialog::new(self.services.posh.clone(), self.proxy.clone());
        self.open_kind(
            el,
            "Terminal prompt",
            None,
            size,
            Kind::Posh(Box::new(dialog)),
        );
    }

    fn open_update(&mut self, el: &ActiveEventLoop, state: UpdateState) {
        let scale = self.scale_at_cursor();
        let tall = matches!(state, UpdateState::Available(_));
        let size = (
            (480.0 * scale) as u32,
            (if tall { 380.0 } else { 190.0 } * scale) as u32,
        );
        self.open_kind(
            el,
            "IdleWalker update",
            None,
            size,
            Kind::Update(UpdateDialog::new(state)),
        );
    }

    /// The menu's "Check for updates": shows the dialog at once, and the answer in it.
    fn check_for_update(&mut self, el: &ActiveEventLoop) {
        let checking = UpdateState::Checking;
        let open = self.dialogs.values_mut().find_map(|d| match &mut d.kind {
            Kind::Update(u) => Some((u, d.window.clone())),
            _ => None,
        });
        match open {
            Some((u, window)) => {
                window.focus_window();
                if u.state == checking {
                    return;
                }
                u.state = checking;
                window.request_redraw();
            }
            None => self.open_update(el, checking),
        }
        update::spawn_check(self.proxy.clone(), self.services.updates.clone());
    }

    fn on_update_checked(&mut self, el: &ActiveEventLoop, state: UpdateState, asked: bool) {
        let updates = self.services.updates.clone();
        let open = self.dialogs.values_mut().find_map(|d| match &mut d.kind {
            Kind::Update(u) => Some((u, d.window.clone())),
            _ => None,
        });
        if asked {
            // Closed while waiting: then the answer is no longer wanted.
            if let Some((u, window)) = open {
                if let UpdateState::Available(v) = &state {
                    updates.mark_announced(v);
                    // The first answer may need more room than "Checking" did.
                    let width = window.inner_size().width;
                    let scale = window.scale_factor();
                    let _ =
                        window.request_inner_size(PhysicalSize::new(width, (380.0 * scale) as u32));
                }
                u.state = state;
                window.request_redraw();
            }
            return;
        }
        // Found in the background: worth interrupting for once, and never over a full-screen app.
        if let UpdateState::Available(v) = &state {
            if open.is_none()
                && !updates.already_announced(v)
                && !self.services.desktop.is_fullscreen()
            {
                updates.mark_announced(v);
                self.open_update(el, state);
            }
        }
    }

    /// Opens the brightness panel, or closes it. A click on the tray icon first takes the
    /// focus from the panel, closing it, so a click just after that counts as "close".
    fn toggle_brightness(&mut self, el: &ActiveEventLoop) {
        let open: Option<WindowId> = self
            .dialogs
            .iter()
            .find(|(_, d)| matches!(d.kind, Kind::Brightness(_)))
            .map(|(k, _)| *k);
        if let Some(id) = open {
            self.close_dialog(id);
            return;
        }
        if self
            .brightness_closed
            .is_some_and(|t| t.elapsed().as_millis() < 300)
        {
            return;
        }
        let (screen, cursor_x) = self.screen_under_cursor();
        let s = screen.dpi_scale;
        let place = TrayPlace::new(&screen, cursor_x, 340.0);
        let size = ((340.0 * s) as u32, place.height((80.0 * s) as i32) as u32);
        let screens = self.session.displays().screens();
        let badges = DisplayBadges::new(screens, &mut self.font, &self.sprites);
        let panel = BrightnessPanel::new(badges, self.proxy.clone(), self.services.desktop.clone());
        self.open_kind(el, "Brightness", Some(place), size, Kind::Brightness(panel));
    }

    /// The screen under the cursor right now, and the cursor's x: the last tick's reading
    /// can be out of date when a click arrives.
    fn screen_under_cursor(&self) -> (Screen, i32) {
        let c = self.desktop().cursor_pos().unwrap_or(self.session.cursor());
        (self.session.displays().screen_at(c), c.x)
    }

    /// Opens the settings menu, or closes it. Pressing on the tray icon
    /// has already closed an open menu, so a click just after that counts as "close".
    fn toggle_menu(&mut self, el: &ActiveEventLoop) {
        let open = self
            .dialogs
            .iter()
            .find(|(_, d)| matches!(d.kind, Kind::Menu(_)))
            .map(|(k, _)| *k);
        if let Some(id) = open {
            self.close_dialog(id);
            return;
        }
        if self
            .menu_closed
            .is_some_and(|t| t.elapsed().as_millis() < 300)
        {
            return;
        }
        let (screen, cursor_x) = self.screen_under_cursor();
        let s = screen.dpi_scale;
        let place = TrayPlace::new(&screen, cursor_x, 300.0);
        let nodes = tray::build(self.session.choices(), self.desktop());
        let panel = menu_panel::MenuPanel::new(nodes);
        let size = (
            (300.0 * s) as u32,
            place.height(panel.content_height(s)) as u32,
        );
        self.open_kind(el, "Idle Walker", Some(place), size, Kind::Menu(panel));
    }

    /// A press outside dismisses the menu, even if the system never gave it the focus.
    fn close_menu_on_outside_press(&mut self) {
        if !self.desktop().mouse_button_down() {
            return;
        }
        let c = self.session.cursor();
        let outside: Option<WindowId> = self.dialogs.iter().find_map(|(id, d)| {
            if !matches!(d.kind, Kind::Menu(_)) {
                return None;
            }
            (!d.world_rect()?.contains(c)).then_some(*id)
        });
        if let Some(id) = outside {
            self.close_dialog(id);
        }
    }

    fn close_dialog(&mut self, id: WindowId) {
        if let Some(d) = self.dialogs.remove(&id) {
            if matches!(d.kind, Kind::Menu(_)) {
                self.menu_closed = Some(Instant::now());
            }
            if matches!(d.kind, Kind::Brightness(_)) {
                self.brightness_closed = Some(Instant::now());
            }
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.started {
            return;
        }
        self.started = true;
        self.tray = Tray::new(self.proxy.clone());
        if self.tray.is_none() {
            self.session
                .journal()
                .note("no tray icon: the desktop offers no notification area");
        }
        // Armed from the UI thread, which stays alive, so the request lasts the session.
        self.session.arm_keep_awake();
        self.paint.start(self.on_painter_start);
        spawn_pump(
            self.pump.clone(),
            self.proxy.clone(),
            self.services.desktop.clone(),
        );
        if self.check_for_updates {
            update::spawn_periodic(self.proxy.clone(), self.services.updates.clone());
        }
        el.set_control_flow(ControlFlow::Wait);
        for id in &self.test_menu {
            let _ = self.proxy.send_event(UserEvent::Menu(id.clone()));
        }
    }

    fn user_event(&mut self, el: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Tick => {
                self.pump.pending.store(false, Ordering::Release);
                self.on_tick(el);
            }
            UserEvent::Menu(id) => self.on_menu(&id, el),
            UserEvent::TrayLeftClick => self.toggle_brightness(el),
            UserEvent::Brightness(found) => {
                for d in self.dialogs.values_mut() {
                    if let Kind::Brightness(p) = &mut d.kind {
                        p.displays = Some(found);
                        d.window.request_redraw();
                        break;
                    }
                }
            }
            UserEvent::UpdateChecked { state, asked } => self.on_update_checked(el, state, asked),
            UserEvent::Posh(done) => {
                // Closed while waiting: then the answer is no longer wanted.
                for d in self.dialogs.values_mut() {
                    if let Kind::Posh(p) = &mut d.kind {
                        p.finish(*done);
                        d.window.request_redraw();
                        break;
                    }
                }
            }
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let troupe = self.session.troupe();
        let Some(d) = self.dialogs.get_mut(&id) else {
            return;
        };
        let outcome = d.event(&event, &mut self.font, troupe);
        d.fit();
        match outcome {
            Outcome::None => {}
            Outcome::Close => self.close_dialog(id),
            Outcome::Menu(item) => {
                if menu_panel::closes_menu(&item) {
                    self.close_dialog(id);
                }
                self.on_menu(&item, el);
            }
            Outcome::Dress(c) => {
                self.close_dialog(id);
                self.session.choose_dress(c);
                self.update_menu();
            }
        }
    }

    fn exiting(&mut self, _el: &ActiveEventLoop) {
        self.session.shut_down();
        self.pump.stopping.store(true, Ordering::Relaxed);
        self.paint.stop();
    }
}

/// Runs the message loop until the person exits.
pub fn run(wiring: Wiring) {
    let mut builder = EventLoop::<UserEvent>::with_user_event();
    #[cfg(target_os = "macos")]
    {
        // No Dock icon and no menu bar of its own: it lives in the status bar.
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        builder.with_activation_policy(ActivationPolicy::Accessory);
    }
    let event_loop = match builder.build() {
        Ok(el) => el,
        Err(e) => {
            wiring.journal.note(&format!("no event loop: {}", e));
            eprintln!("IdleWalker cannot start: {}", e);
            return;
        }
    };
    let mut app = App::new(event_loop.create_proxy(), wiring);
    let _ = event_loop.run_app(&mut app);
}
