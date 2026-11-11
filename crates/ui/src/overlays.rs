//! The focus-free cards: the hot-corner ripple, the click highlight, the About card and
//! the "Display N" badges. Click-through sprite windows like the walkers, drawn in software.
use super::gfx::{self, Canvas, Font};
use crate::prelude::*;
use idlewalker_application::{MouseButtons, Screen};
use std::time::Instant;

fn pixels(s: &mut Surface, w: i32, h: i32) -> &'static mut [u32] {
    let slice = s.pixels.as_mut_slice();
    unsafe { std::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut u32, (w * h) as usize) }
}

// ---- The ripple: two expanding rings and a soft central flash ---------------------------

const DURATION_MS: f64 = 650.0;
const SECOND_RING_DELAY: f64 = 0.12;
const MAX_RADIUS: f64 = 56.0;
const RING_WIDTH: f64 = 3.0;

pub struct CornerRipple {
    window: Box<dyn SpriteSink>,
    surface: Surface,
    start: Option<Instant>,
    size: i32,
    factor: f64,
    pos: (i32, i32),
}

fn ease_out(t: f64) -> f64 {
    let u = 1.0 - t;
    1.0 - u * u
}

impl CornerRipple {
    pub fn new(sprites: &SpriteFactory) -> Self {
        CornerRipple {
            window: sprites(),
            surface: Surface::new(),
            start: None,
            size: 32,
            factor: 1.0,
            pos: (0, 0),
        }
    }

    pub fn active(&self) -> bool {
        self.start.is_some()
    }

    /// Starts (or restarts) the ripple centred on (x, y), at the monitor's sprite scale.
    pub fn show(&mut self, x: i32, y: i32, scale: i32) {
        self.factor = (scale as f64 / Settings::BASE_PIXEL_SIZE as f64).max(0.5);
        self.size = ((2.0 * (MAX_RADIUS * self.factor + 6.0)).ceil() as i32).max(32);
        self.pos = (x - self.size / 2, y - self.size / 2);
        self.start = Some(Instant::now());
        self.tick();
    }

    /// Wall-clock timing: a slow frame never makes it sticky.
    pub fn tick(&mut self) {
        let Some(start) = self.start else { return };
        let t = start.elapsed().as_secs_f64() * 1000.0 / DURATION_MS;
        if t >= 1.0 {
            self.start = None;
            self.window.hide();
            return;
        }
        self.render(t);
        self.window.present(self.surface, self.pos.0, self.pos.1);
        self.window.stay_on_top();
    }

    fn render(&mut self, t: f64) {
        let size = self.size;
        self.surface.reset(size, size);
        let c = size as f64 / 2.0;
        let f = self.factor;
        let max_r = MAX_RADIUS * f;
        let w = (RING_WIDTH * f).max(2.0);
        let px = pixels(&mut self.surface, size, size);
        let mut canvas = Canvas {
            px,
            w: size,
            h: size,
        };

        // Soft central flash, shrinking and fading fast.
        let flash_r = 14.0 * f * (1.0 - 0.4 * t);
        let flash_a = (140.0 * (1.0 - t) * (1.0 - t)) as u32;
        disc(&mut canvas, c, flash_r, flash_a);

        // Inner ring, starting a beat later and stopping shorter.
        let t2 = (t - SECOND_RING_DELAY) / (1.0 - SECOND_RING_DELAY);
        if t2 > 0.0 {
            let r2 = 8.0 * f + (max_r * 0.7 - 8.0 * f) * ease_out(t2);
            ring(&mut canvas, c, r2, w, (190.0 * (1.0 - t2).powf(1.6)) as u32);
        }

        // Outer ring, leading the way to the full radius.
        let r1 = 10.0 * f + (max_r - 10.0 * f) * ease_out(t);
        ring(&mut canvas, c, r1, w, (230.0 * (1.0 - t).powf(1.6)) as u32);
        self.surface.touch(0, 0, size, size);
    }
}

fn ring(c: &mut Canvas, center: f64, radius: f64, width: f64, alpha: u32) {
    let color = 0xFF50AAFF;
    for y in 0..c.h {
        let dy = y as f64 + 0.5 - center;
        for x in 0..c.w {
            let dx = x as f64 + 0.5 - center;
            let cov = 1.0 - ((dx * dx + dy * dy).sqrt() - radius).abs() / width;
            if cov > 0.0 {
                c.rect(
                    x,
                    y,
                    1,
                    1,
                    ((alpha as f64 * cov.min(1.0)) as u32) << 24 | (color & 0xFFFFFF),
                );
            }
        }
    }
}

fn disc(c: &mut Canvas, center: f64, radius: f64, alpha: u32) {
    for y in 0..c.h {
        let dy = y as f64 + 0.5 - center;
        for x in 0..c.w {
            let dx = x as f64 + 0.5 - center;
            let cov = 1.0 - ((dx * dx + dy * dy).sqrt() - radius) / 2.0;
            if cov > 0.0 {
                c.rect(
                    x,
                    y,
                    1,
                    1,
                    ((alpha as f64 * cov.min(1.0)) as u32) << 24 | 0xFFFFFF,
                );
            }
        }
    }
}

// ---- The click highlight: a coloured spot under the cursor at every click ---------------

/// The spot's radius at 100% display scaling.
const SPOT_RADIUS: f64 = 20.0;
const SPOT_OPACITY: f64 = 0.65;
/// How long the spot stays after the button is let go, and how long it then takes to fade.
const SPOT_HOLD_MS: f64 = 500.0;
const SPOT_FADE_MS: f64 = 250.0;
const SPOT_LEFT: u32 = 0xFFE100;
const SPOT_RIGHT: u32 = 0x2D7DFF;

struct Spot {
    /// Whether the left button made it; otherwise the right.
    left: bool,
    at: Point,
    /// Display scaling of the monitor it is on.
    scale: f64,
    /// When its button was let go; `None` while it is held and the spot follows the cursor.
    released: Option<Instant>,
}

/// Shows where the mouse was clicked: yellow for the left button, blue for the right.
/// The spot follows a drag, stays where the button was let go, then fades. It is sized
/// by the display scaling of whichever monitor the cursor is on.
pub struct ClickHighlight {
    window: Box<dyn SpriteSink>,
    surface: Surface,
    spot: Option<Spot>,
    held: MouseButtons,
    /// What the surface holds: colour, radius in pixels and alpha.
    drawn: Option<(u32, i32, u32)>,
    /// Where the window was last shown.
    shown_at: Option<(i32, i32)>,
}

impl ClickHighlight {
    pub fn new(sprites: &SpriteFactory) -> Self {
        ClickHighlight {
            window: sprites(),
            surface: Surface::new(),
            spot: None,
            held: MouseButtons::default(),
            drawn: None,
            shown_at: None,
        }
    }

    /// A spot is on screen: it needs a tick every frame.
    pub fn active(&self) -> bool {
        self.spot.is_some()
    }

    /// Once a frame, with the buttons as they are now. `scale_at` is the display scaling
    /// at a point of the desktop.
    pub fn tick(&mut self, cursor: Point, buttons: MouseButtons, scale_at: impl Fn(Point) -> f64) {
        // The button pressed last owns the spot; a press restarts one that was fading.
        let pressed = if buttons.left && !self.held.left {
            Some(true)
        } else if buttons.right && !self.held.right {
            Some(false)
        } else {
            None
        };
        self.held = buttons;
        if let Some(left) = pressed {
            self.spot = Some(Spot {
                left,
                at: cursor,
                scale: 1.0,
                released: None,
            });
            self.window.stay_on_top();
        }
        let Some(spot) = &mut self.spot else { return };
        if spot.released.is_none() {
            let own = if spot.left {
                buttons.left
            } else {
                buttons.right
            };
            if own {
                spot.at = cursor;
            } else if buttons.left || buttons.right {
                // Let go while the other button is still down: that one takes over.
                spot.left = buttons.left;
                spot.at = cursor;
            } else {
                spot.released = Some(Instant::now());
            }
            spot.scale = scale_at(spot.at);
        }
        // Wall-clock timing: a slow frame never makes it sticky.
        let opacity = match spot.released {
            None => 1.0,
            Some(at) => {
                let ms = at.elapsed().as_secs_f64() * 1000.0;
                1.0 - ((ms - SPOT_HOLD_MS) / SPOT_FADE_MS).clamp(0.0, 1.0)
            }
        };
        if opacity <= 0.0 {
            self.spot = None;
            self.drawn = None;
            self.shown_at = None;
            self.window.hide();
            return;
        }
        let colour = if spot.left { SPOT_LEFT } else { SPOT_RIGHT };
        let radius = ((SPOT_RADIUS * spot.scale).round() as i32).max(4);
        let alpha = (255.0 * SPOT_OPACITY * opacity).round() as u32;
        let size = 2 * (radius + 2);
        let at = (spot.at.x - size / 2, spot.at.y - size / 2);
        let look = Some((colour, radius, alpha));
        if look != self.drawn {
            self.drawn = look;
            self.shown_at = None;
            self.surface.reset(size, size);
            let mut canvas = Canvas {
                px: pixels(&mut self.surface, size, size),
                w: size,
                h: size,
            };
            spot_disc(&mut canvas, radius as f64, alpha << 24 | colour);
            self.surface.touch(0, 0, size, size);
        }
        if self.shown_at != Some(at) {
            self.shown_at = Some(at);
            self.window.present(self.surface, at.0, at.1);
        }
    }
}

/// A filled circle in the middle of the canvas, with a smooth edge.
fn spot_disc(c: &mut Canvas, radius: f64, color: u32) {
    let center = c.w as f64 / 2.0;
    let alpha = (color >> 24) as f64;
    for y in 0..c.h {
        let dy = y as f64 + 0.5 - center;
        for x in 0..c.w {
            let dx = x as f64 + 0.5 - center;
            let cov = (radius + 0.5 - (dx * dx + dy * dy).sqrt()).clamp(0.0, 1.0);
            if cov > 0.0 {
                c.rect(
                    x,
                    y,
                    1,
                    1,
                    ((alpha * cov) as u32) << 24 | (color & 0xFFFFFF),
                );
            }
        }
    }
}

// ---- Find cursor: a key pressed twice puts a spotlight on the cursor -----------------------

/// A press held longer than this is not a tap, in seconds.
const TAP_MAX: f64 = 0.3;
/// The second press must come within this of the first being let go.
const TAP_GAP: f64 = 0.4;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Tap {
    #[default]
    Idle,
    /// The first press, since when.
    Down(f64),
    /// The first press was let go, when.
    Up(f64),
    /// Nothing more can come of this press: wait for the key to be let go.
    Spent,
}

/// Recognises a modifier key pressed twice in a row with nothing else in between, so
/// Ctrl+C twice, or the key held for a shortcut, is not taken for it.
#[derive(Default)]
struct DoubleTap(Tap);

impl DoubleTap {
    /// True on the frame the second press arrives. `other` is any other key or button.
    fn step(&mut self, now: f64, key: bool, other: bool) -> bool {
        let spent = if key { Tap::Spent } else { Tap::Idle };
        let (next, fired) = match self.0 {
            _ if other => (spent, false),
            Tap::Idle if key => (Tap::Down(now), false),
            Tap::Idle => (Tap::Idle, false),
            // Held too long to be a tap: nothing more comes of it.
            Tap::Down(since) if now - since > TAP_MAX => (spent, false),
            Tap::Down(_) if !key => (Tap::Up(now), false),
            Tap::Down(since) => (Tap::Down(since), false),
            Tap::Up(_) if key => (Tap::Spent, true),
            Tap::Up(since) if now - since > TAP_GAP => (Tap::Idle, false),
            Tap::Up(since) => (Tap::Up(since), false),
            Tap::Spent => (spent, false),
        };
        self.0 = next;
        fired
    }
}

/// The clear middle and the reach of the dark around it, at 100% display scaling.
const BEAM_HOLE: f64 = 90.0;
const BEAM_OUTER: f64 = 280.0;
/// How much wider the middle starts, before it closes in on the cursor.
const BEAM_ZOOM: f64 = 1.2;
const BEAM_DARK: f64 = 0.62;
const BEAM_RING: u32 = 0xFAD61C;
const BEAM_OPEN_MS: f64 = 350.0;
const BEAM_CLOSE_MS: f64 = 200.0;
/// It goes away by itself once the cursor has rested this long.
const BEAM_REST_SECONDS: f64 = 3.0;

struct Beam {
    since: Instant,
    at: Point,
    moved: Instant,
    closing: Option<Instant>,
}

/// Shows where the cursor is when the chosen key is pressed twice: a dark halo closes in on it,
/// leaving it in a clear, ringed middle. The halo follows the cursor over every monitor,
/// sized by each one's display scaling, and goes at a click, a key, or a few seconds' rest.
pub struct FindCursor {
    window: Box<dyn SpriteSink>,
    surface: Surface,
    taps: DoubleTap,
    key_was: bool,
    beam: Option<Beam>,
    /// What the surface holds: the middle's and the halo's radius in pixels, and strength.
    drawn: Option<(i32, i32, u32)>,
    shown_at: Option<(i32, i32)>,
}

impl FindCursor {
    pub fn new(sprites: &SpriteFactory) -> Self {
        FindCursor {
            window: sprites(),
            surface: Surface::new(),
            taps: DoubleTap::default(),
            key_was: false,
            beam: None,
            drawn: None,
            shown_at: None,
        }
    }

    /// The halo is on screen: it needs a tick every frame.
    pub fn active(&self) -> bool {
        self.beam.is_some()
    }

    /// A press is being followed or the halo is up: other keys matter now.
    pub fn watching(&self) -> bool {
        self.taps.0 != Tap::Idle || self.beam.is_some()
    }

    /// Once a frame. `now` is in seconds; `key` is the chosen key, `other` any other, `button` any
    /// mouse button, and `scale_at` the display scaling at a point of the desktop.
    pub fn tick(
        &mut self,
        now: f64,
        cursor: Point,
        key: bool,
        other: bool,
        button: bool,
        scale_at: impl Fn(Point) -> f64,
    ) {
        let fired = self.taps.step(now, key, other || button);
        let pressed = key && !self.key_was;
        self.key_was = key;
        match &mut self.beam {
            // Up: any key or click sends it away.
            Some(b) if b.closing.is_none() => {
                if pressed || other || button {
                    b.closing = Some(Instant::now());
                }
            }
            _ if fired => {
                self.beam = Some(Beam {
                    since: Instant::now(),
                    at: cursor,
                    moved: Instant::now(),
                    closing: None,
                });
                self.window.stay_on_top();
            }
            _ => {}
        }
        let Some(b) = &mut self.beam else { return };
        if cursor != b.at {
            b.at = cursor;
            b.moved = Instant::now();
        } else if b.closing.is_none() && b.moved.elapsed().as_secs_f64() > BEAM_REST_SECONDS {
            b.closing = Some(Instant::now());
        }
        // Wall-clock timing: a slow frame never makes it sticky.
        let open = ease_out((b.since.elapsed().as_secs_f64() * 1000.0 / BEAM_OPEN_MS).min(1.0));
        let fade = match b.closing {
            Some(c) => 1.0 - c.elapsed().as_secs_f64() * 1000.0 / BEAM_CLOSE_MS,
            None => 1.0,
        };
        if fade <= 0.0 {
            self.beam = None;
            self.drawn = None;
            self.shown_at = None;
            self.window.hide();
            return;
        }
        let scale = scale_at(b.at).max(0.5);
        let outer = (BEAM_OUTER * scale).round() as i32;
        let hole =
            ((BEAM_HOLE * scale * (1.0 + BEAM_ZOOM * (1.0 - open))).round() as i32).min(outer - 4);
        let strength = (255.0 * (0.25 + 0.75 * open) * fade).round() as u32;
        let size = 2 * (outer + 1);
        let at = (b.at.x - size / 2, b.at.y - size / 2);
        let look = Some((hole, outer, strength));
        if look != self.drawn {
            self.drawn = look;
            self.shown_at = None;
            self.surface.reset(size, size);
            halo(
                pixels(&mut self.surface, size, size),
                size,
                hole as f64,
                outer as f64,
                (3.0 * scale).max(2.0),
                strength as f64 / 255.0,
            );
            self.surface.touch(0, 0, size, size);
        }
        if self.shown_at != Some(at) {
            self.shown_at = Some(at);
            self.window.present(self.surface, at.0, at.1);
        }
    }
}

/// The dark halo, premultiplied: clear inside `hole`, darkest just outside it and fading
/// to nothing at `outer`, with a bright ring on the edge so it shows on a dark screen too.
fn halo(px: &mut [u32], size: i32, hole: f64, outer: f64, ring_w: f64, strength: f64) {
    let center = size as f64 / 2.0;
    let (r, g, b) = (
        (BEAM_RING >> 16 & 255) as f64,
        (BEAM_RING >> 8 & 255) as f64,
        (BEAM_RING & 255) as f64,
    );
    for y in 0..size {
        let dy = y as f64 + 0.5 - center;
        let row = (y * size) as usize;
        for x in 0..size {
            let dx = x as f64 + 0.5 - center;
            let d2 = dx * dx + dy * dy;
            if d2 >= outer * outer {
                continue;
            }
            let d = d2.sqrt();
            let ring = (1.0 - ((d - hole).abs() - ring_w / 2.0)).clamp(0.0, 1.0) * strength;
            let reach = (1.0 - (d - hole) / (outer - hole)).clamp(0.0, 1.0);
            let dark = BEAM_DARK * reach * reach * (d - hole + 0.5).clamp(0.0, 1.0) * strength;
            if ring <= 0.0 && dark <= 0.0 {
                continue;
            }
            // The ring over the dark; the dark is black, so it only adds alpha.
            let alpha = ring + dark * (1.0 - ring);
            px[row + x as usize] = ((alpha * 255.0) as u32) << 24
                | ((r * ring) as u32) << 16
                | ((g * ring) as u32) << 8
                | (b * ring) as u32;
        }
    }
}

// ---- The About card --------------------------------------------------------------------

const HOLD_MS: f64 = 2000.0;
const FADE_MS: f64 = 220.0;

pub struct AboutToast {
    window: Box<dyn SpriteSink>,
    picture: Vec<u32>,
    surface: Surface,
    size: (i32, i32),
    pos: (i32, i32),
    start: Instant,
    fading_from: Option<Instant>,
}

impl AboutToast {
    /// A small dark panel with a waving Minion and the creator's name, centred on the
    /// cursor's monitor just above where the walkers walk.
    pub fn new(screen: Screen, font: &mut Font, sprites: &SpriteFactory) -> Self {
        let scale = screen.dpi_scale;
        let px = |v: f64| (v * scale).round_ties_even() as i32;
        let unit = 3.2 * scale;
        let (mw, mh) = ((22.0 * unit).ceil() as i32, (26.0 * unit).ceil() as i32);
        let (title_size, text_size) = ((15.0 * scale) as f32, (13.0 * scale) as f32);
        let (title, text) = ("IdleWalker", "Our creator is Sharafas OM");
        let (pad, gap) = (px(14.0), px(6.0));
        let text_left = pad + mw + px(10.0);
        let width = text_left
            + font
                .width(title, title_size)
                .max(font.width(text, text_size))
            + pad;
        let line = |s: f32| (s * 1.3) as i32;
        let height = mh.max(line(title_size) + gap + line(text_size)) + 2 * pad;

        let mut surface = Surface::new();
        surface.reset(width, height);
        let figure_top = (height - mh) / 2;
        {
            let mut c = Canvas {
                px: pixels(&mut surface, width, height),
                w: width,
                h: height,
            };
            c.round_box(
                0,
                0,
                width,
                height,
                px(10.0) as f64,
                gfx::BACK,
                gfx::RIM,
                1.0,
            );
            let top = (height - (line(title_size) + gap + line(text_size))) / 2;
            c.text(font, text_left, top, title, title_size, gfx::ACCENT);
            c.text(
                font,
                text_left,
                top + line(title_size) + gap,
                text,
                text_size,
                gfx::WHITE,
            );
        }
        // The figure, waving, drawn by the same painter as on screen.
        let mut pose = Pose::standing();
        pose.face = *EXPRESSION_WINKING;
        pose.gaze_x = 0.0;
        pose.arm_near = 150.0;
        pose.fore_near = -25.0;
        let look = PERSONALITY_ROSTER[0].look;
        FrameRenderer::draw_pose(
            surface,
            pose,
            look,
            SETTINGS_DRESS_COLORS[0],
            false,
            (pad + mw / 2) as f64,
            (figure_top + mh - 2) as f64,
            unit,
        );
        surface.touch(0, 0, width, height);

        let picture = pixels(&mut surface, width, height).to_vec();
        let area = screen.work;
        let left = area.left() + (area.width - width) / 2;
        let top = (area.bottom() - 52 * screen.cell() - height).max(area.top());
        AboutToast {
            window: sprites(),
            picture,
            surface,
            size: (width, height),
            pos: (left, top),
            start: Instant::now(),
            fading_from: None,
        }
    }

    /// False once it has faded away.
    pub fn tick(&mut self, cursor: Point, button_down: bool) -> bool {
        let inside =
            Rectangle::new(self.pos.0, self.pos.1, self.size.0, self.size.1).contains(cursor);
        if button_down && inside && self.fading_from.is_none() {
            self.fading_from = Some(Instant::now()); // a click dismisses it early
        }
        let elapsed = self.start.elapsed().as_secs_f64() * 1000.0;
        if self.fading_from.is_none() && elapsed >= FADE_MS + HOLD_MS {
            self.fading_from = Some(Instant::now());
        }
        let opacity = match self.fading_from {
            Some(from) => 1.0 - from.elapsed().as_secs_f64() * 1000.0 / FADE_MS,
            None => (elapsed / FADE_MS).min(1.0),
        };
        if opacity <= 0.0 {
            self.window.hide();
            return false;
        }
        let (w, h) = self.size;
        self.surface.reset(w, h);
        let k = (opacity * 256.0) as u32;
        let out = pixels(&mut self.surface, w, h);
        for (d, s) in out.iter_mut().zip(&self.picture) {
            let (rb, ag) = (s & 0x00FF00FF, (s >> 8) & 0x00FF00FF);
            *d = (((rb * k) >> 8) & 0x00FF00FF) | ((ag * k) & 0xFF00FF00);
        }
        self.surface.touch(0, 0, w, h);
        self.window.present(self.surface, self.pos.0, self.pos.1);
        self.window.stay_on_top();
        true
    }
}

// ---- The display badges -----------------------------------------------------------------------

/// Large "Display N" cards, one centred on every monitor, so the sliders in the
/// brightness panel can be matched to the screens they move. With a single monitor
/// there is nothing to tell apart, so no cards are shown.
pub struct DisplayBadges {
    screens: Vec<Screen>,
    windows: Vec<Box<dyn SpriteSink>>,
    lit: Vec<bool>,
    surface: Surface,
}

impl DisplayBadges {
    pub fn new(screens: Vec<Screen>, font: &mut Font, sprites: &SpriteFactory) -> Self {
        let n = if screens.len() > 1 { screens.len() } else { 0 };
        let mut b = DisplayBadges {
            windows: (0..n).map(|_| sprites()).collect(),
            lit: vec![false; n],
            screens,
            surface: Surface::new(),
        };
        for i in 0..n {
            b.draw(i, font);
        }
        b
    }

    /// Lights one card and dims the rest; -1 lights them all. Only cards that change are redrawn.
    pub fn highlight(&mut self, index: i32, font: &mut Font) {
        for i in 0..self.windows.len() {
            let on = index < 0 || i as i32 == index;
            if self.lit[i] != on {
                self.lit[i] = on;
                self.draw(i, font);
            }
        }
    }

    fn draw(&mut self, i: usize, font: &mut Font) {
        let screen = self.screens[i];
        let s = screen.dpi_scale;
        let size = (150.0 * s).round_ties_even() as i32;
        let on = self.lit[i];
        self.surface.reset(size, size);
        {
            let mut c = Canvas {
                px: pixels(&mut self.surface, size, size),
                w: size,
                h: size,
            };
            let rim_w = (if on { 3.0 } else { 1.0 } * s).max(1.0);
            c.round_box(
                0,
                0,
                size,
                size,
                18.0 * s,
                0xE6161822,
                if on { gfx::ACCENT } else { gfx::RIM },
                rim_w,
            );
            let (caption, digits) = ((15.0 * s) as f32, (76.0 * s) as f32);
            let w = font.width("Display", caption);
            c.text(
                font,
                (size - w) / 2,
                (14.0 * s) as i32,
                "Display",
                caption,
                if on { gfx::WHITE } else { gfx::DIM },
            );
            let number = (i + 1).to_string();
            let w = font.width(&number, digits);
            c.text(
                font,
                (size - w) / 2,
                (38.0 * s) as i32,
                &number,
                digits,
                if on { gfx::ACCENT } else { gfx::WHITE },
            );
        }
        self.surface.touch(0, 0, size, size);
        let b = screen.bounds;
        self.windows[i].present(
            self.surface,
            b.left() + (b.width - size) / 2,
            b.top() + (b.height - size) / 2,
        );
        self.windows[i].stay_on_top();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// What a sprite window was asked to do.
    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Shown {
        /// Top-left corner, canvas size and the pixel in the middle.
        At(i32, i32, i32, u32),
        Hidden,
    }

    struct Recorder(Arc<Mutex<Vec<Shown>>>);

    impl SpriteSink for Recorder {
        fn present(&mut self, surface: Surface, x: i32, y: i32) {
            let (w, h) = (surface.width(), surface.height());
            let middle = surface.pixels.as_slice()[(h / 2 * w + w / 2) as usize] as u32;
            self.0.lock().unwrap().push(Shown::At(x, y, w, middle));
        }
        fn stay_on_top(&self) {}
        fn hide(&mut self) {
            self.0.lock().unwrap().push(Shown::Hidden);
        }
    }

    fn highlight() -> (ClickHighlight, Arc<Mutex<Vec<Shown>>>) {
        let log = Arc::new(Mutex::new(Vec::new()));
        let sink = log.clone();
        let sprites: SpriteFactory =
            Arc::new(move || Box::new(Recorder(sink.clone())) as Box<dyn SpriteSink>);
        (ClickHighlight::new(&sprites), log)
    }

    const NONE: MouseButtons = MouseButtons {
        left: false,
        right: false,
    };
    const LEFT: MouseButtons = MouseButtons {
        left: true,
        right: false,
    };
    const RIGHT: MouseButtons = MouseButtons {
        left: false,
        right: true,
    };

    /// A second monitor at 200% to the right of one at 100%.
    fn scale_at(p: Point) -> f64 {
        if p.x >= 1920 {
            2.0
        } else {
            1.0
        }
    }

    #[test]
    fn nothing_is_shown_until_a_button_goes_down() {
        let (mut h, log) = highlight();
        h.tick(Point::new(100, 100), NONE, scale_at);
        assert!(!h.active());
        assert!(log.lock().unwrap().is_empty());
    }

    #[test]
    fn a_left_press_puts_a_yellow_spot_centred_on_the_cursor() {
        let (mut h, log) = highlight();
        h.tick(Point::new(100, 100), LEFT, scale_at);
        assert!(h.active());
        // Radius 20 and a 2 pixel margin; 65% of yellow, premultiplied.
        assert_eq!(
            log.lock().unwrap().as_slice(),
            [Shown::At(78, 78, 44, 0xA6A69200)]
        );
    }

    #[test]
    fn a_right_press_is_blue() {
        let (mut h, log) = highlight();
        h.tick(Point::new(100, 100), RIGHT, scale_at);
        let Shown::At(_, _, _, middle) = log.lock().unwrap()[0] else {
            panic!("not shown");
        };
        assert_eq!(middle, 0xA61D51A6);
    }

    #[test]
    fn it_follows_a_drag_onto_a_monitor_with_another_scaling() {
        let (mut h, log) = highlight();
        h.tick(Point::new(1900, 500), LEFT, scale_at);
        h.tick(Point::new(1900, 500), LEFT, scale_at); // held still: nothing to redraw
        h.tick(Point::new(2000, 500), LEFT, scale_at);
        let log = log.lock().unwrap();
        assert_eq!(log.len(), 2);
        // Twice the radius on the 200% monitor, still centred on the cursor.
        assert!(matches!(log[1], Shown::At(1958, 458, 84, _)));
    }

    #[test]
    fn it_stays_where_the_button_was_let_go_then_fades_away() {
        let (mut h, log) = highlight();
        h.tick(Point::new(100, 100), LEFT, scale_at);
        h.tick(Point::new(400, 400), NONE, scale_at);
        assert!(h.active());
        assert_eq!(log.lock().unwrap().len(), 1); // it did not follow the cursor
        std::thread::sleep(std::time::Duration::from_millis(
            (SPOT_HOLD_MS + SPOT_FADE_MS) as u64 + 50,
        ));
        h.tick(Point::new(400, 400), NONE, scale_at);
        assert!(!h.active());
        assert_eq!(log.lock().unwrap().last(), Some(&Shown::Hidden));
    }

    #[test]
    fn a_new_press_replaces_a_fading_spot() {
        let (mut h, log) = highlight();
        h.tick(Point::new(100, 100), LEFT, scale_at);
        h.tick(Point::new(100, 100), NONE, scale_at);
        h.tick(Point::new(300, 300), RIGHT, scale_at);
        assert!(matches!(
            log.lock().unwrap().last(),
            Some(Shown::At(278, 278, 44, 0xA61D51A6))
        ));
    }

    // ---- Find cursor ----

    /// Feeds the detector one frame every 10 ms; returns whether it ever fired.
    fn taps(frames: &[(bool, bool)]) -> bool {
        let mut t = DoubleTap::default();
        let mut fired = false;
        for (i, (ctrl, other)) in frames.iter().enumerate() {
            fired |= t.step(i as f64 * 0.01, *ctrl, *other);
        }
        fired
    }

    const UP: (bool, bool) = (false, false);
    const CTRL: (bool, bool) = (true, false);

    #[test]
    fn ctrl_twice_fires() {
        assert!(taps(&[UP, CTRL, CTRL, UP, UP, CTRL]));
    }

    #[test]
    fn ctrl_once_does_not() {
        assert!(!taps(&[UP, CTRL, CTRL, UP, UP, UP]));
    }

    #[test]
    fn a_shortcut_typed_twice_does_not() {
        let combo = (true, true);
        assert!(!taps(&[UP, CTRL, combo, CTRL, UP, CTRL, combo, CTRL, UP]));
        // Nor a key between two taps.
        assert!(!taps(&[UP, CTRL, UP, (false, true), UP, CTRL]));
    }

    #[test]
    fn a_long_press_or_a_long_wait_is_no_double_tap() {
        let mut held = vec![UP];
        held.extend([CTRL; 40]); // 0.4 s
        held.extend([UP, CTRL]);
        assert!(!taps(&held));
        let mut slow = vec![UP, CTRL];
        slow.extend([UP; 50]); // 0.5 s
        slow.push(CTRL);
        assert!(!taps(&slow));
    }

    fn finder() -> (FindCursor, Arc<Mutex<Vec<Shown>>>) {
        let log = Arc::new(Mutex::new(Vec::new()));
        let sink = log.clone();
        let sprites: SpriteFactory =
            Arc::new(move || Box::new(Recorder(sink.clone())) as Box<dyn SpriteSink>);
        (FindCursor::new(&sprites), log)
    }

    /// Ctrl twice, a frame every 10 ms; returns the time of the next frame.
    fn double_tap(f: &mut FindCursor, at: Point) -> f64 {
        for (i, ctrl) in [false, true, false, true].into_iter().enumerate() {
            f.tick(i as f64 * 0.01, at, ctrl, false, false, scale_at);
        }
        0.04
    }

    #[test]
    fn ctrl_twice_puts_a_halo_centred_on_the_cursor_with_a_clear_middle() {
        let (mut f, log) = finder();
        let now = double_tap(&mut f, Point::new(1000, 600));
        assert!(f.active());
        // 280 pixels of reach and one of margin on each side; nothing drawn in the middle.
        assert_eq!(
            log.lock().unwrap().first(),
            Some(&Shown::At(1000 - 281, 600 - 281, 562, 0))
        );
        // It follows the cursor onto the 200% monitor at twice the size.
        f.tick(now, Point::new(2500, 600), true, false, false, scale_at);
        assert_eq!(
            log.lock().unwrap().last(),
            Some(&Shown::At(2500 - 561, 600 - 561, 1122, 0))
        );
    }

    #[test]
    fn a_click_sends_the_halo_away() {
        let (mut f, log) = finder();
        let now = double_tap(&mut f, Point::new(1000, 600));
        f.tick(now, Point::new(1000, 600), false, false, true, scale_at);
        std::thread::sleep(std::time::Duration::from_millis(BEAM_CLOSE_MS as u64 + 50));
        f.tick(
            now + 0.3,
            Point::new(1000, 600),
            false,
            false,
            false,
            scale_at,
        );
        assert!(!f.active());
        assert_eq!(log.lock().unwrap().last(), Some(&Shown::Hidden));
    }

    #[test]
    fn nothing_shows_without_the_double_tap() {
        let (mut f, log) = finder();
        f.tick(0.0, Point::new(10, 10), false, false, false, scale_at);
        f.tick(0.01, Point::new(10, 10), true, false, false, scale_at);
        f.tick(0.02, Point::new(10, 10), false, false, false, scale_at);
        assert!(f.watching() && !f.active());
        assert!(log.lock().unwrap().is_empty());
    }
}
