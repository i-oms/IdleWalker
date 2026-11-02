// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

//
// What the figure painters share. A figure is a handful of solids -
// capsules, spheres, tubes - and every pixel gets a signed distance to
// each (for an anti-aliased edge) and a surface normal (for lighting),
// the solids being composited back to front into the pixel being
// shaded. Here are the compositing, the lighting, the distance
// functions, the sorting of limbs against the body, and the things
// that float about a figure (sweat, notes, Zzz, "!", hearts), which
// each painter places where its own head and mouth are.
//
#[derive(Clone, Default)]
pub struct SolidPainterData {
    pub flip: f64,
    pub px: f64,
    // Depth of the body's surface at the pixel being shaded, and how far
    // in front of it a limb has to be to count as in front: a soft band
    // rather than a knife edge, so the join does not shimmer.
    pub body_z: f64,
    pub depth_blend: f64,
    pub depth_radius: f64,
    // Limb pieces that turned out to be in front of the body at this
    // pixel, held back until the face has been painted. Room for both
    // arms' every piece (sleeve, patch, cuff, finger, hand) at one pixel.
    pub front_r: Arr<f64>,
    pub front_g: Arr<f64>,
    pub front_b: Arr<f64>,
    pub front_cov: Arr<f64>,
    pub front_count: i32,
    // The pixel being shaded, premultiplied.
    pub ca: f64,
    pub cr: f64,
    pub cg: f64,
    pub cb: f64,
    // ---- The raster -----------------------------------------------------------------
    pub column_u: Arr<f64>,
}
rt::handle!(SolidPainter, SolidPainterData);

impl SolidPainter {
    // Light direction (towards the light), screen terms: upper left, in front.
    pub const LX: f64 = -0.45;
    pub const LY: f64 = 0.72;
    pub const LZ: f64 = 0.53;
    // Half vector between the light and the viewer, for highlights.
    pub const HX: f64 = -0.257;
    pub const HY: f64 = 0.411;
    pub const HZ: f64 = 0.874;
    pub const DEG: f64 = (std::f64::consts::PI / 180.0);
    pub fn clamp(mut x: f64) -> f64 {
        return (if (x < 0.0) {
            0.0
        } else {
            (if (x > 1.0) { 1.0 } else { x })
        });
    }
    //
    // x to the power n. The shines are nearly always whole numbers, and
    // for those a few multiplications beat the general Math.Pow by far;
    // a shine blended between two materials takes the general route.
    //
    pub fn pow(mut x: f64, mut n: f64) -> f64 {
        let mut k: i32 = ((n) as i32);
        if ((((k) as f64) != n) || (k < 1)) || (k > 64) {
            return rt::pow(x, n);
        }
        let mut result: f64 = 1.0;
        let mut b: f64 = x;
        'l1: while true {
            if (k & 1) != 0 {
                result *= b;
            }
            k = (k >> (1));
            if k == 0 {
                return result;
            }
            b *= b;
        }
        #[allow(unreachable_code)]
        unreachable!()
    }
    // Normal of a sphere (or a tube seen side on) from the offset to its axis.
    pub fn normal(
        mut dx: f64,
        mut dy: f64,
        mut radius: f64,
        nx: &mut f64,
        ny: &mut f64,
        nz: &mut f64,
    ) {
        (*nx) = (dx / radius);
        (*ny) = (dy / radius);
        let mut m: f64 = (((*nx) * (*nx)) + ((*ny) * (*ny)));
        if m > 0.995 {
            let mut f: f64 = (0.995 / m).sqrt();
            (*nx) *= f;
            (*ny) *= f;
            m = 0.995;
        }
        (*nz) = (1.0 - m).sqrt();
    }
    // Turns a normal about the vertical axis by angle a.
    pub fn turn(nx: &mut f64, nz: &mut f64, mut a: f64) {
        SolidPainter::turn_2(&mut *nx, &mut *nz, (a).cos(), (a).sin());
    }
    // Turns a normal about the vertical axis by the angle whose cosine and sine are c and s.
    pub fn turn_2(nx: &mut f64, nz: &mut f64, mut c: f64, mut s: f64) {
        let mut x: f64 = (((*nx) * c) + ((*nz) * s));
        (*nz) = ((-(*nx) * s) + ((*nz) * c));
        (*nx) = x;
    }
    pub fn mix(
        r: &mut f64,
        g: &mut f64,
        b: &mut f64,
        spec: &mut f64,
        shine: &mut f64,
        mut mr: f64,
        mut mg: f64,
        mut mb: f64,
        mut mspec: f64,
        mut mshine: f64,
        mut w: f64,
    ) {
        if w <= 0.0 {
            return;
        }
        (*r) += ((mr - (*r)) * w);
        (*g) += ((mg - (*g)) * w);
        (*b) += ((mb - (*b)) * w);
        (*spec) += ((mspec - (*spec)) * w);
        (*shine) += ((mshine - (*shine)) * w);
    }
    // ---- Distance fields ----------------------------------------------------------
    pub fn seg(
        mut x: f64,
        mut y: f64,
        mut ax: f64,
        mut ay: f64,
        mut bx: f64,
        mut by: f64,
        qx: &mut f64,
        qy: &mut f64,
        t: &mut f64,
    ) -> f64 {
        let mut abx: f64 = (bx - ax);
        let mut aby: f64 = (by - ay);
        let mut len2: f64 = ((abx * abx) + (aby * aby));
        (*t) = (if (len2 > 0.0) {
            ((((x - ax) * abx) + ((y - ay) * aby)) / len2)
        } else {
            0.0
        });
        if (*t) < 0.0 {
            (*t) = 0.0;
        } else if (*t) > 1.0 {
            (*t) = 1.0;
        }
        (*qx) = (ax + (abx * (*t)));
        (*qy) = (ay + (aby * (*t)));
        let mut dx: f64 = (x - (*qx));
        let mut dy: f64 = (y - (*qy));
        return ((dx * dx) + (dy * dy)).sqrt();
    }
    pub fn round_rect(mut x: f64, mut y: f64, mut hw: f64, mut hh: f64, mut r: f64) -> f64 {
        let mut qx: f64 = ((x).abs() - (hw - r));
        let mut qy: f64 = ((y).abs() - (hh - r));
        let mut ox: f64 = (if (qx > 0.0) { qx } else { 0.0 });
        let mut oy: f64 = (if (qy > 0.0) { qy } else { 0.0 });
        let mut inside: f64 = rt::min_f64(rt::max_f64(qx, qy), 0.0);
        return ((((ox * ox) + (oy * oy)).sqrt() + inside) - r);
    }
    //
    // Distance to a circle of radius r that has been squeezed to k of
    // its width: dx is already divided by k. First-order exact.
    //
    pub fn squeezed(mut dx: f64, mut dy: f64, mut k: f64, mut r: f64) -> f64 {
        let mut dist: f64 = ((dx * dx) + (dy * dy)).sqrt();
        if dist < 1e-6 {
            return -r;
        }
        let mut grad: f64 = ((((dx * dx) / (k * k)) + (dy * dy)).sqrt() / dist);
        return ((dist - r) / grad);
    }
}
