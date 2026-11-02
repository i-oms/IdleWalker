//! The focus-free cards: the hot-corner ripple, the About card and the "Display N"
//! badges. Click-through sprite windows like the walkers, drawn in software.
use super::gfx::{self, Canvas, Font};
use crate::prelude::*;
use idlewalker_application::Screen;
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
