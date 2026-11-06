// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

//
// A small software canvas. The character is drawn into its own layer so
// it can be given a soft edge, then laid over whatever is behind it
// (dust, its shadow). Pixels are premultiplied ARGB, ready for a
// layered window.
//
// The canvas is mostly empty, so it keeps track of the rectangle that
// actually has something in it (`Bounds`) and does all its
// clearing, outlining and uploading on that rectangle only.
//
#[derive(Clone, Default)]
pub struct SurfaceData {
    pub pixels: Arr<i32>,
    pub layer: Arr<i32>,
    pub rim: Arr<bool>,
    pub column_gap: Arr<i32>,
    // The region with content, as [x0, x1) x [y0, y1); empty when x1 <= x0.
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub width_v: i32,
    pub height_v: i32,
}
rt::handle!(Surface, SurfaceData);

impl Surface {
    fn alloc__() -> Surface {
        let mut this = Surface::from_data(SurfaceData::default());
        this.pixels = Arr::new(0);
        this.layer = Arr::new(0);
        this.rim = Arr::new(0);
        this.column_gap = Arr::new(0);
        this
    }
    pub fn new() -> Surface {
        Self::alloc__()
    }
    // A translucent dark halo one pixel wide keeps the figure readable on
    // a busy or pale desktop without looking like a drawn outline.
    pub const HALO: i32 = ((0x78101018) as i32);
    pub fn width(mut self) -> i32 {
        self.width_v
    }
    pub fn set_width(mut self, v: i32) {
        self.width_v = v;
    }
    pub fn height(mut self) -> i32 {
        self.height_v
    }
    pub fn set_height(mut self, v: i32) {
        self.height_v = v;
    }
    // The part of the canvas that has anything drawn in it.
    pub fn bounds(mut self) -> Rectangle {
        return (if ((self.x1 > self.x0) && (self.y1 > self.y0)) {
            Rectangle::from_ltrb(self.x0, self.y0, self.x1, self.y1)
        } else {
            Rectangle::default()
        });
    }
    // The character layer, premultiplied ARGB, Width pixels per row.
    pub fn layer(mut self) -> Arr<i32> {
        return self.layer.clone();
    }
    pub fn reset(mut self, mut width: i32, mut height: i32) {
        let mut needed: i32 = (width * height);
        if needed > self.pixels.len() {
            // Grow once: a pooled canvas keeps its arrays for good, so a
            // bird's frame after a Minion's costs no allocation.
            self.pixels = Arr::new(needed);
            self.layer = Arr::new(needed);
            self.rim = Arr::new(needed);
        } else if (width != self.width()) || (height != self.height()) {
            // Same arrays, new row stride: whatever was drawn last time
            // lies scattered under the new layout, so clear the whole of
            // the new canvas (a few microseconds).
            self.pixels.clear_range(0, needed);
            self.layer.clear_range(0, needed);
            self.rim.clear_range(0, needed);
        } else if (self.x1 > self.x0) && (self.y1 > self.y0) {
            // Only the rows that were drawn on last time need clearing.
            let mut start: i32 = (self.y0 * self.width());
            let mut length: i32 = ((self.y1 - self.y0) * self.width());
            self.pixels.clear_range(start, length);
            self.layer.clear_range(start, length);
            self.rim.clear_range(start, length);
        }
        self.set_width(width);
        self.set_height(height);
        self.x0 = self.width();
        self.y0 = self.height();
        self.x1 = 0;
        self.y1 = 0;
    }
    // Notes that [left, right) x [top, bottom) may have been drawn on.
    pub fn touch(mut self, mut left: i32, mut top: i32, mut right: i32, mut bottom: i32) {
        if left < 0 {
            left = 0;
        }
        if top < 0 {
            top = 0;
        }
        if right > self.width() {
            right = self.width();
        }
        if bottom > self.height() {
            bottom = self.height();
        }
        if (left >= right) || (top >= bottom) {
            return;
        }
        if left < self.x0 {
            self.x0 = left;
        }
        if top < self.y0 {
            self.y0 = top;
        }
        if right > self.x1 {
            self.x1 = right;
        }
        if bottom > self.y1 {
            self.y1 = bottom;
        }
    }
    pub fn fill(
        mut self,
        mut target: Arr<i32>,
        mut left: i32,
        mut top: i32,
        mut right: i32,
        mut bottom: i32,
        mut color: i32,
    ) {
        if left < 0 {
            left = 0;
        }
        if top < 0 {
            top = 0;
        }
        if right > self.width() {
            right = self.width();
        }
        if bottom > self.height() {
            bottom = self.height();
        }
        if (left >= right) || (top >= bottom) {
            return;
        }
        {
            let mut y: i32 = top;
            'l1: while y < bottom {
                let mut row: i32 = (y * self.width());
                {
                    let mut x: i32 = left;
                    'l2: while x < right {
                        (*target.at((row + x))) = color;
                        x += 1;
                    }
                }
                y += 1;
            }
        }
        self.touch(left, top, right, bottom);
    }
    // A translucent square of dust resting on (x, y).
    pub fn puff(mut self, mut x: i32, mut y: i32, mut size: i32, mut alpha: i32) {
        if alpha <= 0 {
            return;
        }
        let mut color: i32 = ((((alpha << (24)) | (((214 * alpha) / 255) << (16)))
            | (((204 * alpha) / 255) << (8)))
            | ((190 * alpha) / 255));
        self.fill(
            self.pixels.clone(),
            (x - (size / 2)),
            (y - size),
            ((x - (size / 2)) + size),
            y,
            color,
        );
    }
    //
    // A soft, dark ellipse on the canvas: the shadow under the feet.
    // Centred on (cx, cy) with radii rx, ry and `alpha` (0..1) at the centre.
    //
    pub fn shadow(mut self, mut cx: f64, mut cy: f64, mut rx: f64, mut ry: f64, mut alpha: f64) {
        if ((alpha <= 0.01) || (rx <= 0.0)) || (ry <= 0.0) {
            return;
        }
        let mut left: i32 = rt::max_i32(0, (((cx - rx).floor()) as i32));
        let mut right: i32 = rt::min_i32(self.width(), (((cx + rx).ceil()) as i32));
        let mut top: i32 = rt::max_i32(0, (((cy - ry).floor()) as i32));
        let mut bottom: i32 = rt::min_i32(self.height(), (((cy + ry).ceil()) as i32));
        if (left >= right) || (top >= bottom) {
            return;
        }
        {
            let mut y: i32 = top;
            'l1: while y < bottom {
                let mut dy: f64 = (((((y) as f64) + 0.5) - cy) / ry);
                let mut row: i32 = (y * self.width());
                {
                    let mut x: i32 = left;
                    'l2: while x < right {
                        'c3: {
                            let mut dx: f64 = (((((x) as f64) + 0.5) - cx) / rx);
                            let mut t: f64 = ((dx * dx) + (dy * dy));
                            if t >= 1.0 {
                                break 'c3;
                            }
                            let mut f: f64 = (1.0 - t);
                            let mut a: i32 = (((((alpha * f) * f) * 255.0) + 0.5) as i32);
                            if a <= 0 {
                                break 'c3;
                            }
                            let mut color: i32 = ((((a << (24)) | (((0x14 * a) / 255) << (16)))
                                | (((0x10 * a) / 255) << (8)))
                                | ((0x18 * a) / 255));
                            (*self.pixels.at((row + x))) =
                                Surface::over_premul(color, (*self.pixels.at((row + x))));
                        }
                        x += 1;
                    }
                }
                y += 1;
            }
        }
        self.touch(left, top, right, bottom);
    }
    //
    // Softens the edge of the character layer with a thin dark halo and
    // lays the layer over the canvas.
    //
    pub fn commit_character(mut self, mut thickness: i32) {
        if (self.x1 <= self.x0) || (self.y1 <= self.y0) {
            return;
        }
        // The halo can reach `thickness` beyond the drawn region.
        let mut left: i32 = rt::max_i32(0, (self.x0 - thickness));
        let mut top: i32 = rt::max_i32(0, (self.y0 - thickness));
        let mut right: i32 = rt::min_i32(self.width(), (self.x1 + thickness));
        let mut bottom: i32 = rt::min_i32(self.height(), (self.y1 + thickness));
        // A transparent pixel is rim if a character pixel lies within
        // `thickness` pixels of it along its row or its column.
        {
            let mut y: i32 = top;
            'l1: while y < bottom {
                let mut row: i32 = (y * self.width());
                let mut gap: i32 = thickness;
                {
                    let mut x: i32 = left;
                    'l2: while x < right {
                        if (*self.layer.at((row + x))) != 0 {
                            gap = 0;
                        } else if gap < thickness {
                            gap += 1;
                            (*self.rim.at((row + x))) = true;
                        }
                        x += 1;
                    }
                }
                gap = thickness;
                {
                    let mut x: i32 = (right - 1);
                    'l3: while x >= left {
                        if (*self.layer.at((row + x))) != 0 {
                            gap = 0;
                        } else if gap < thickness {
                            gap += 1;
                            (*self.rim.at((row + x))) = true;
                        }
                        x -= 1;
                    }
                }
                y += 1;
            }
        }
        // The columns, swept a row at a time (each column keeping its own
        // count) so memory is read in order rather than a row apart.
        if self.column_gap.len() < self.width() {
            self.column_gap = Arr::new(self.width());
        }
        let mut gaps: Arr<i32> = self.column_gap.clone();
        {
            let mut x: i32 = left;
            'l4: while x < right {
                (*gaps.at(x)) = thickness;
                x += 1;
            }
        }
        {
            let mut y: i32 = top;
            'l5: while y < bottom {
                let mut row: i32 = (y * self.width());
                {
                    let mut x: i32 = left;
                    'l6: while x < right {
                        let mut i: i32 = (row + x);
                        if (*self.layer.at(i)) != 0 {
                            (*gaps.at(x)) = 0;
                        } else if (*gaps.at(x)) < thickness {
                            (*gaps.at(x)) += 1;
                            (*self.rim.at(i)) = true;
                        }
                        x += 1;
                    }
                }
                y += 1;
            }
        }
        {
            let mut x: i32 = left;
            'l7: while x < right {
                (*gaps.at(x)) = thickness;
                x += 1;
            }
        }
        {
            let mut y: i32 = (bottom - 1);
            'l8: while y >= top {
                let mut row: i32 = (y * self.width());
                {
                    let mut x: i32 = left;
                    'l9: while x < right {
                        let mut i: i32 = (row + x);
                        if (*self.layer.at(i)) != 0 {
                            (*gaps.at(x)) = 0;
                        } else if (*gaps.at(x)) < thickness {
                            (*gaps.at(x)) += 1;
                            (*self.rim.at(i)) = true;
                        }
                        x += 1;
                    }
                }
                y -= 1;
            }
        }
        {
            let mut y: i32 = top;
            'l10: while y < bottom {
                let mut row: i32 = (y * self.width());
                {
                    let mut x: i32 = left;
                    'l11: while x < right {
                        let mut i: i32 = (row + x);
                        let mut src: i32 = (*self.layer.at(i));
                        if src != 0 {
                            // Anti-aliased edges get the halo behind them too.
                            let mut alpha: i32 = ((((src) as u32) >> (24)) as i32);
                            (*self.pixels.at(i)) = Surface::over_premul(
                                src,
                                (if (alpha < 255) {
                                    Surface::over_premul(Surface::HALO, (*self.pixels.at(i)))
                                } else {
                                    (*self.pixels.at(i))
                                }),
                            );
                        } else if *self.rim.at(i) {
                            (*self.pixels.at(i)) =
                                Surface::over_premul(Surface::HALO, (*self.pixels.at(i)));
                        }
                        x += 1;
                    }
                }
                y += 1;
            }
        }
        self.x0 = left;
        self.y0 = top;
        self.x1 = right;
        self.y1 = bottom;
    }
    // src over dst, both premultiplied.
    pub fn over_premul(mut src: i32, mut dst: i32) -> i32 {
        if dst == 0 {
            return src;
        }
        let mut sa: i32 = ((((src) as u32) >> (24)) as i32);
        if sa >= 255 {
            return src;
        }
        let mut keep: i32 = (255 - sa);
        let mut a: i32 = (sa + (((((((dst) as u32) >> (24)) as i32) * keep) + 127) / 255));
        let mut r: i32 = (((src >> (16)) & 255) + (((((dst >> (16)) & 255) * keep) + 127) / 255));
        let mut g: i32 = (((src >> (8)) & 255) + (((((dst >> (8)) & 255) * keep) + 127) / 255));
        let mut b: i32 = ((src & 255) + ((((dst & 255) * keep) + 127) / 255));
        if a > 255 {
            a = 255;
        }
        if r > 255 {
            r = 255;
        }
        if g > 255 {
            g = 255;
        }
        if b > 255 {
            b = 255;
        }
        return ((((a << (24)) | (r << (16))) | (g << (8))) | b);
    }
}
