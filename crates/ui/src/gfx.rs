//! Small software drawing used by the cards and dialogs: text, rounded boxes and a
//! tiny immediate-mode set of controls. One bundled font, so the UI looks the same on
//! every OS and needs nothing installed. Pixels are premultiplied 0xAARRGGBB.
use std::collections::HashMap;

pub const BACK: u32 = 0xFF161822;
pub const PANEL: u32 = 0xFF1E212D;
pub const RIM: u32 = 0xFF3A3E4C;
pub const DIM: u32 = 0xFF8A8F9E;
pub const ACCENT: u32 = 0xFFFAD61C;
pub const WHITE: u32 = 0xFFFFFFFF;
pub const GREEN: u32 = 0xFF4CC36E;

pub struct Font {
    font: fontdue::Font,
    cache: HashMap<(char, u32), (fontdue::Metrics, Vec<u8>)>,
}

impl Font {
    pub fn new() -> Self {
        let font = fontdue::Font::from_bytes(
            epaint_default_fonts::UBUNTU_LIGHT,
            fontdue::FontSettings::default(),
        )
        .expect("bundled font");
        Font {
            font,
            cache: HashMap::new(),
        }
    }

    fn glyph(&mut self, c: char, size: f32) -> &(fontdue::Metrics, Vec<u8>) {
        let font = &self.font;
        self.cache
            .entry((c, size.to_bits()))
            .or_insert_with(|| font.rasterize(c, size))
    }

    pub fn width(&mut self, text: &str, size: f32) -> i32 {
        text.chars()
            .map(|c| self.glyph(c, size).0.advance_width)
            .sum::<f32>()
            .ceil() as i32
    }
}

/// A canvas over a pixel slice.
pub struct Canvas<'a> {
    pub px: &'a mut [u32],
    pub w: i32,
    pub h: i32,
}

#[inline]
fn over(dst: u32, r: u32, g: u32, b: u32, a: u32) -> u32 {
    // Source is straight colour with coverage a; destination is premultiplied.
    let keep = 255 - a;
    let mul = |c: u32| (c * a + 127) / 255;
    let mix = |d: u32, s: u32| s + (d * keep + 127) / 255;
    (mix(dst >> 24, a) << 24)
        | (mix((dst >> 16) & 255, mul(r)) << 16)
        | (mix((dst >> 8) & 255, mul(g)) << 8)
        | mix(dst & 255, mul(b))
}

impl Canvas<'_> {
    pub fn clear(&mut self, color: u32) {
        self.px.fill(color);
    }

    #[inline]
    fn plot(&mut self, x: i32, y: i32, color: u32, coverage: u32) {
        if x < 0 || y < 0 || x >= self.w || y >= self.h || coverage == 0 {
            return;
        }
        let a = ((color >> 24) * coverage + 127) / 255;
        let i = (y * self.w + x) as usize;
        self.px[i] = over(
            self.px[i],
            (color >> 16) & 255,
            (color >> 8) & 255,
            color & 255,
            a,
        );
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.plot(xx, yy, color, 255);
            }
        }
    }

    /// A filled rounded box with a smooth edge, and optionally a rim of `rim_w` pixels.
    pub fn round_box(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        radius: f64,
        fill: u32,
        rim: u32,
        rim_w: f64,
    ) {
        let (cx, cy) = (x as f64 + w as f64 / 2.0, y as f64 + h as f64 / 2.0);
        let (hx, hy) = (w as f64 / 2.0 - radius, h as f64 / 2.0 - radius);
        for yy in y..y + h {
            for xx in x..x + w {
                let qx = (xx as f64 + 0.5 - cx).abs() - hx;
                let qy = (yy as f64 + 0.5 - cy).abs() - hy;
                let (ox, oy) = (qx.max(0.0), qy.max(0.0));
                // A true signed distance: it keeps falling towards the middle, so
                // the rim stays at the edge whatever the corner radius.
                let d = (ox * ox + oy * oy).sqrt() + qx.max(qy).min(0.0) - radius;
                let edge = (0.5 - d).clamp(0.0, 1.0);
                if edge <= 0.0 {
                    continue;
                }
                self.plot(xx, yy, fill, (edge * 255.0) as u32);
                if rim_w > 0.0 {
                    let band = (0.5 - (d + rim_w / 2.0).abs() + rim_w / 2.0).clamp(0.0, 1.0);
                    self.plot(xx, yy, rim, (band * 255.0) as u32);
                }
            }
        }
    }

    /// Draws text with its top-left at (x, y); returns the width.
    pub fn text(
        &mut self,
        font: &mut Font,
        x: i32,
        y: i32,
        text: &str,
        size: f32,
        color: u32,
    ) -> i32 {
        let baseline = y + (size * 0.95) as i32;
        let mut pen = x as f32;
        for c in text.chars() {
            let (m, bitmap) = font.glyph(c, size).clone();
            let (gx, gy) = (pen as i32 + m.xmin, baseline - m.ymin - m.height as i32);
            for row in 0..m.height {
                for col in 0..m.width {
                    self.plot(
                        gx + col as i32,
                        gy + row as i32,
                        color,
                        bitmap[row * m.width + col] as u32,
                    );
                }
            }
            pen += m.advance_width;
        }
        (pen - x as f32).ceil() as i32
    }

    pub fn text_right(
        &mut self,
        font: &mut Font,
        right: i32,
        y: i32,
        text: &str,
        size: f32,
        color: u32,
    ) {
        let w = font.width(text, size);
        self.text(font, right - w, y, text, size, color);
    }

    /// Text cut with an ellipsis to fit a width.
    pub fn text_fit(
        &mut self,
        font: &mut Font,
        x: i32,
        y: i32,
        max: i32,
        text: &str,
        size: f32,
        color: u32,
    ) {
        if font.width(text, size) <= max {
            self.text(font, x, y, text, size, color);
            return;
        }
        let mut s: String = text.to_string();
        while !s.is_empty() && font.width(&s, size) + font.width("...", size) > max {
            s.pop();
        }
        s.push_str("...");
        self.text(font, x, y, &s, size, color);
    }
}

/// What the mouse did since the last paint.
#[derive(Default, Clone, Copy)]
pub struct Mouse {
    pub x: i32,
    pub y: i32,
    pub down: bool,
    pub clicked: bool,
    pub wheel: f32,
}

impl Mouse {
    pub fn inside(&self, x: i32, y: i32, w: i32, h: i32) -> bool {
        self.x >= x && self.y >= y && self.x < x + w && self.y < y + h
    }
}

/// A push button; true on the paint that saw its click.
pub fn button(
    c: &mut Canvas,
    f: &mut Font,
    m: &Mouse,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    s: f64,
    label: &str,
) -> bool {
    let hover = m.inside(x, y, w, h);
    c.round_box(
        x,
        y,
        w,
        h,
        5.0 * s,
        if hover { RIM } else { PANEL },
        RIM,
        1.0,
    );
    let size = (13.0 * s) as f32;
    let tw = f.width(label, size);
    c.text(
        f,
        x + (w - tw) / 2,
        y + (h - (size * 1.2) as i32) / 2,
        label,
        size,
        WHITE,
    );
    hover && m.clicked
}

/// A 0..100 slider, showing `value`. Returns the new value while being dragged here.
pub fn slider(
    c: &mut Canvas,
    m: &Mouse,
    dragging: bool,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    s: f64,
    value: i32,
    color: u32,
) -> Option<i32> {
    let track = ((4.0 * s) as i32).max(3);
    let ty = y + (h - track) / 2;
    let filled = (w as f64 * value as f64 / 100.0) as i32;
    c.round_box(x, ty, w, track, track as f64 / 2.0, RIM, 0, 0.0);
    if filled > 0 {
        c.round_box(x, ty, filled, track, track as f64 / 2.0, color, 0, 0.0);
    }
    let knob = (14.0 * s) as i32;
    c.round_box(
        x + filled - knob / 2,
        y + (h - knob) / 2,
        knob,
        knob,
        knob as f64 / 2.0,
        WHITE,
        0,
        0.0,
    );
    if dragging && m.down {
        return Some((((m.x - x) as f64 * 100.0 / w as f64).round() as i32).clamp(0, 100));
    }
    None
}
