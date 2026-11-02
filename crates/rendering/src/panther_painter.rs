// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

//
// Paints the Pink Panther into a `Surface` the way the other
// figures are painted: as lit solids with anti-aliased edges, composited
// back to front (see `SolidPainter`). The DePatie-Freleng
// cartoon cat: tall, thin and all pink - long legs with paws, a slim
// waist, a long neck, a small head with a long tapering snout ending
// in a big black nose, small pointed ears set back, large half-lidded
// yellow eyes with round black pupils, thin whiskers, a feline smile
// with a fang when the mouth opens, and a long thin tail curling at
// the tip. No lighter markings anywhere: the cartoon has none.
//
// Model space is the Minions': u across in units, positive the way he
// faces; v up from the soles. He is `Height` units tall.
// The body turns with `Pose.Yaw`; the head turns further by
// `Pose.HeadTurn` and nods by `Pose.HeadDip`.
//
#[derive(Clone, Default)]
pub struct PantherPainterData {
    pub pose: Pose,
    pub face: Expression,
    pub yaw: f64,
    pub sin_y: f64,
    pub cos_y: f64,
    pub head_yaw: f64,
    pub sin_h: f64,
    pub cos_h: f64,
    pub lean: f64,
    pub dip: f64,
    pub eye_alpha: Arr<f64>,
    pub eye_u: Arr<f64>,
    pub eye_k: Arr<f64>,
    pub eye_vis: Arr<f64>,
    pub first_eye: i32,
    pub ear_u: Arr<f64>,
    pub ear_z: Arr<f64>,
    pub hip_u: Arr<f64>,
    pub knee_u: Arr<f64>,
    pub knee_v: Arr<f64>,
    pub ankle_u: Arr<f64>,
    pub ankle_v: Arr<f64>,
    pub leg_z: Arr<f64>,
    pub paw_len: f64,
    pub toe_dir: f64,
    pub sh_u: Arr<f64>,
    pub sh_z: Arr<f64>,
    pub elb_u: Arr<f64>,
    pub elb_v: Arr<f64>,
    pub elb_z: Arr<f64>,
    pub wrist_u: Arr<f64>,
    pub wrist_v: Arr<f64>,
    pub wrist_z: Arr<f64>,
    pub hand_u: Arr<f64>,
    pub hand_v: Arr<f64>,
    pub hand_z: Arr<f64>,
    pub arm_lo: Arr<f64>,
    pub arm_hi: Arr<f64>,
    pub arm_vlo: Arr<f64>,
    pub arm_vhi: Arr<f64>,
    pub decorated: bool,
    pub top_v: f64,
    pub mouth_u: f64,
    pub snout_u0: f64,
    pub snout_v0: f64,
    pub snout_u1: f64,
    pub snout_v1: f64,
    pub tail_sway: f64,
    pub tail_u: Arr<f64>,
    pub tail_v: Arr<f64>,
    pub tail_lo: f64,
    pub tail_hi: f64,
    pub tail_vlo: f64,
    pub tail_vhi: f64,
    pub tail_du: Arr<f64>,
    pub tail_dv: Arr<f64>,
    pub tail_len2: Arr<f64>,
    pub knot_u: Arr<f64>,
    pub knot_v: Arr<f64>,
    // Fixed for the frame: the eyes' cosines and sines, where each pupil
    // sits, and the box the whiskers fit in.
    pub eye_cos: Arr<f64>,
    pub eye_sin: Arr<f64>,
    pub gaze_u: Arr<f64>,
    pub gaze_v: Arr<f64>,
    pub pupil_r: f64,
    pub whisk_lo: f64,
    pub whisk_hi: f64,
    pub row_v: f64,
    pub row_torso_r: f64,
    pub row_dy_eff: f64,
    pub row_cos_front: f64,
    pub row_cos_back: f64,
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
rt::handle!(PantherPainter, PantherPainterData);

thread_local! { static PANTHERPAINTER_CACHED: std::cell::Cell<PantherPainter> = std::cell::Cell::new(Default::default()); }

impl PantherPainter {
    fn alloc__() -> PantherPainter {
        let mut this = PantherPainter::from_data(PantherPainterData::default());
        this.eye_alpha = Arr::new(2);
        this.eye_u = Arr::new(2);
        this.eye_k = Arr::new(2);
        this.eye_vis = Arr::new(2);
        this.ear_u = Arr::new(2);
        this.ear_z = Arr::new(2);
        this.hip_u = Arr::new(2);
        this.knee_u = Arr::new(2);
        this.knee_v = Arr::new(2);
        this.ankle_u = Arr::new(2);
        this.ankle_v = Arr::new(2);
        this.leg_z = Arr::new(2);
        this.sh_u = Arr::new(2);
        this.sh_z = Arr::new(2);
        this.elb_u = Arr::new(2);
        this.elb_v = Arr::new(2);
        this.elb_z = Arr::new(2);
        this.wrist_u = Arr::new(2);
        this.wrist_v = Arr::new(2);
        this.wrist_z = Arr::new(2);
        this.hand_u = Arr::new(2);
        this.hand_v = Arr::new(2);
        this.hand_z = Arr::new(2);
        this.arm_lo = Arr::new(2);
        this.arm_hi = Arr::new(2);
        this.arm_vlo = Arr::new(2);
        this.arm_vhi = Arr::new(2);
        this.tail_u = Arr::new((PantherPainter::TAIL_SAMPLES + 1));
        this.tail_v = Arr::new((PantherPainter::TAIL_SAMPLES + 1));
        this.tail_du = Arr::new(PantherPainter::TAIL_SAMPLES);
        this.tail_dv = Arr::new(PantherPainter::TAIL_SAMPLES);
        this.tail_len2 = Arr::new(PantherPainter::TAIL_SAMPLES);
        this.knot_u = Arr::new(6);
        this.knot_v = Arr::new(6);
        this.eye_cos = Arr::new(2);
        this.eye_sin = Arr::new(2);
        this.gaze_u = Arr::new(2);
        this.gaze_v = Arr::new(2);
        this.front_r = Arr::new(12);
        this.front_g = Arr::new(12);
        this.front_b = Arr::new(12);
        this.front_cov = Arr::new(12);
        this.column_u = Arr::new(0);
        this
    }
    pub fn new() -> PantherPainter {
        Self::alloc__()
    }
    pub const HEIGHT: f64 = 26.0;
    // The DePatie-Freleng panther: all pink, nothing else - no lighter
    // chest, muzzle or belly - built long and thin: a slender torso on
    // long legs, a long neck, a small head with a long tapering snout
    // ending in a big black nose, small pointed ears set back, large
    // half-lidded yellow eyes with round black pupils, thin whiskers,
    // and a long thin tail curling at the tip.
    // Legs: hips either side of the axis, thigh + shin with a knee between.
    pub const HIP_V: f64 = 11.0;
    pub const HIP_X: f64 = 0.95;
    pub const THIGH: f64 = 5.2;
    pub const SHIN: f64 = 5.0;
    pub const THIGH_R: f64 = 0.78;
    pub const SHIN_R: f64 = 0.62;
    // Torso: a slim capsule from hips to shoulders, narrow at the waist.
    pub const TORSO_BOTTOM: f64 = 11.2;
    pub const TORSO_TOP: f64 = 17.6;
    pub const HIP_R: f64 = 1.85;
    pub const CHEST_R: f64 = 2.25;
    // Shoulders rise this far above TorsoTop before rounding off: a low
    // cap so the neck stands clear instead of lying along the chest.
    pub const SHOULDER_CAP: f64 = 0.8;
    pub const NECK_BOTTOM: f64 = 17.4;
    pub const NECK_TOP: f64 = 21.0;
    pub const NECK_R: f64 = 0.68;
    // Head: a small rounded skull; the snout reaches forward from it.
    pub const HEAD_V: f64 = 23.0;
    pub const HEAD_RX: f64 = 2.25;
    pub const HEAD_RY: f64 = 2.45;
    pub const EYE_V: f64 = 23.3;
    pub const EYE_ANGLE: f64 = 20.0;
    pub const EYE_RX: f64 = 0.72;
    pub const EYE_RY: f64 = 0.92;
    pub const IRIS_R: f64 = 0.44;
    pub const BROW_V: f64 = 24.4;
    pub const SNOUT_LEN: f64 = 2.6;
    pub const SNOUT_R_BASE: f64 = 1.05;
    pub const SNOUT_R_TIP: f64 = 0.55;
    pub const MUZZLE_V: f64 = 21.9;
    pub const SNOUT_DROP: f64 = 0.45;
    pub const NOSE_R0: f64 = 0.46;
    pub const MOUTH_V: f64 = 21.1;
    pub const EAR_BASE_V: f64 = 24.9;
    // Arms: shoulders on the chest surface.
    pub const SHOULDER_V: f64 = 17.6;
    pub const SHOULDER_ANGLE: f64 = 80.0;
    pub const ARM_RADIUS: f64 = 2.35;
    pub const UPPER_ARM: f64 = 3.3;
    pub const FOREARM: f64 = 3.1;
    pub const REST_SPREAD: f64 = 6.0;
    pub const SLEEVE_R: f64 = 0.55;
    pub const FORE_R: f64 = 0.48;
    pub const PAW_R: f64 = 0.58;
    pub const CLAW_LEN: f64 = 0.5;
    pub const CLAW_R: f64 = 0.16;
    // Colours (linear-ish 0..1): the cartoon's pastel pink and a deeper pink for shade.
    pub const FUR_R: f64 = 0.96;
    pub const FUR_G: f64 = 0.63;
    pub const FUR_B: f64 = 0.77;
    pub const FUR_DARK_R: f64 = 0.84;
    pub const FUR_DARK_G: f64 = 0.46;
    pub const FUR_DARK_B: f64 = 0.63;
    pub const CREAM_R: f64 = 0.98;
    pub const CREAM_G: f64 = 0.74;
    pub const CREAM_B: f64 = 0.85;
    pub const NOSE_R: f64 = 0.08;
    pub const NOSE_G: f64 = 0.06;
    pub const NOSE_B: f64 = 0.08;
    pub const IRIS_YR: f64 = 0.99;
    pub const IRIS_YG: f64 = 0.86;
    pub const IRIS_YB: f64 = 0.30;
    // Tail centreline resampled along a smooth spline: one flowing curve,
    // never straight rods. Filled in BuildTail once per pose.
    pub const TAIL_SPANS: i32 = 5;
    pub const TAIL_PER: i32 = 4;
    pub const TAIL_SAMPLES: i32 = (PantherPainter::TAIL_SPANS * PantherPainter::TAIL_PER);
    // The torso's cross-section depends on the row only: the last row's,
    // and the cosines of the angles round the body where the darker back
    // starts and ends (Torso only needs the exact angle in between).
    pub const BACK_ARC: f64 = 2.2;
    pub fn init(mut self, mut pose: Pose, mut face_left: bool, mut px: f64) {
        self.setup(face_left, px, PantherPainter::ARM_RADIUS);
        self.row_v = f64::NAN;
        self.pose = pose;
        self.face = pose.face;
        self.lean = pose.lean;
        self.dip = pose.head_dip;
        self.yaw = (PantherPainter::wrap(pose.yaw) * SolidPainter::DEG);
        self.sin_y = (self.yaw).sin();
        self.cos_y = (self.yaw).cos();
        self.head_yaw = (PantherPainter::wrap((pose.yaw + pose.head_turn)) * SolidPainter::DEG);
        self.sin_h = (self.head_yaw).sin();
        self.cos_h = (self.head_yaw).cos();
        // The snout leaves the face just below the eyes and reaches
        // forward the way the head faces: a long profile from the side,
        // a short bump from the front.
        let mut shift: f64 = ((self.dip * 0.5) * self.sin_h);
        self.snout_u0 = (((PantherPainter::HEAD_RX - 0.5) * self.sin_h) + shift);
        self.snout_v0 = (PantherPainter::MUZZLE_V - (self.dip * 0.8));
        self.snout_u1 =
            ((((PantherPainter::HEAD_RX - 0.5) + PantherPainter::SNOUT_LEN) * self.sin_h) + shift);
        self.snout_v1 =
            ((PantherPainter::MUZZLE_V - PantherPainter::SNOUT_DROP) - (self.dip * 1.1));
        self.mouth_u = ((self.snout_u0 + self.snout_u1) / 2.0);
        self.place_eye(0, (-(PantherPainter::EYE_ANGLE) * SolidPainter::DEG));
        self.place_eye(1, (PantherPainter::EYE_ANGLE * SolidPainter::DEG));
        self.first_eye = (if ((*self.eye_alpha.at(0)).cos() < (*self.eye_alpha.at(1)).cos()) {
            0
        } else {
            1
        });
        self.pupil_r = (((PantherPainter::IRIS_R * 0.62) * self.face.pupil_size) + 0.04);
        let mut reach: f64 = ((PantherPainter::EYE_RX - self.pupil_r) - 0.08);
        {
            let mut i: i32 = 0;
            'l1: while i < 2 {
                (*self.eye_cos.at(i)) = (*self.eye_alpha.at(i)).cos();
                (*self.eye_sin.at(i)) = (*self.eye_alpha.at(i)).sin();
                let mut ox: f64 = (((pose.gaze_x * reach) * 0.9) + (self.face.pupil_dx * 0.4));
                let mut oy: f64 = (((pose.gaze_y * (PantherPainter::EYE_RY - self.pupil_r)) * 0.8)
                    + (self.face.pupil_dy * 0.3));
                ox -= ((((if (i == 1) { 1 } else { -1 }) as f64) * self.face.cross_eye) * 0.3);
                let mut om: f64 = ((ox * ox) + (oy * oy)).sqrt();
                if om > reach {
                    ox *= (reach / om);
                    oy *= (reach / om);
                }
                (*self.gaze_u.at(i)) = ox;
                (*self.gaze_v.at(i)) = oy;
                i += 1;
            }
        }
        // The whiskers' box: every root and tip, padded by their
        // thickness and a pixel for the soft edge.
        {
            let mut root_u: f64 = ((self.snout_u0 + self.snout_u1) / 2.0);
            let mut k: f64 = (if (self.cos_h > 0.35) {
                self.cos_h
            } else {
                0.35
            });
            self.whisk_lo = f64::MAX;
            self.whisk_hi = f64::MIN;
            {
                let mut side: i32 = -1;
                'l2: while side <= 1 {
                    let mut ru: f64 = (root_u + ((((side) as f64) * 0.75) * k));
                    {
                        let mut w: i32 = -1;
                        'l3: while w <= 1 {
                            let mut tip_u: f64 = ((ru
                                + ((((side) as f64) * (2.0 + ((((w).abs()) as f64) * 0.2)))
                                    * (0.4 + (0.6 * k))))
                                - (0.6 * self.sin_h));
                            self.whisk_lo = rt::min_f64(self.whisk_lo, rt::min_f64(ru, tip_u));
                            self.whisk_hi = rt::max_f64(self.whisk_hi, rt::max_f64(ru, tip_u));
                            w += 1;
                        }
                    }
                    side += 2;
                }
            }
            self.whisk_lo -= (0.03 + px);
            self.whisk_hi += (0.03 + px);
        }
        {
            let mut i: i32 = 0;
            'l4: while i < 2 {
                let mut base_x: f64 = (if (i == 0) { -1.4 } else { 1.4 });
                (*self.ear_u.at(i)) = (((base_x * self.cos_h) * 0.85) + (0.2 * self.sin_h));
                (*self.ear_z.at(i)) = (-base_x * self.sin_h);
                i += 1;
            }
        }
        let mut p: f64 = ((pose.walk_phase * 2.0) * std::f64::consts::PI);
        let mut swing: f64 = (if pose.walking {
            (30.0 * (p).sin())
        } else {
            0.0
        });
        let mut bend_a: f64 = (if pose.walking {
            (55.0 * rt::max_f64(0.0, (p).cos()))
        } else {
            4.0
        });
        let mut bend_b: f64 = (if pose.walking {
            (55.0 * rt::max_f64(0.0, -((p).cos())))
        } else {
            4.0
        });
        let mut tuck: f64 = pose.leg_tuck;
        self.place_leg(
            0,
            -(PantherPainter::HIP_X),
            (-swing + (55.0 * tuck)),
            (bend_b + (95.0 * tuck)),
        );
        self.place_leg(
            1,
            PantherPainter::HIP_X,
            (swing + (55.0 * tuck)),
            (bend_a + (95.0 * tuck)),
        );
        self.paw_len = (1.15 + (0.75 * (self.sin_y).abs()));
        self.toe_dir = (0.25 + (0.8 * self.sin_y));
        let mut arm_swing: f64 = (if pose.walking {
            (30.0 * (p).sin())
        } else {
            0.0
        });
        let mut fore_swing: f64 = ((if pose.walking { 14 } else { 0 }) as f64);
        if (pose.arm_near != 0.0) || (pose.arm_far != 0.0) {
            arm_swing = 0.0;
        }
        if (pose.fore_near != 0.0) || (pose.fore_far != 0.0) {
            fore_swing = 0.0;
        }
        self.place_arm(
            0,
            -1.0,
            pose.arm_far,
            pose.fore_far,
            pose.reach_far,
            arm_swing,
            fore_swing,
        );
        self.place_arm(
            1,
            1.0,
            pose.arm_near,
            pose.fore_near,
            pose.reach_near,
            -arm_swing,
            fore_swing,
        );
        if pose.walking {
            self.tail_sway = ((p).sin() * 0.7);
        } else {
            self.tail_sway = ((pose.body_dy * 2.0) + (pose.lean * 0.5));
        }
        self.build_tail();
        self.decorated =
            (((((pose.sweat > 0.0) || pose.note) || pose.zzz) || pose.bubble) || pose.heart);
        self.top_v = (if self.decorated {
            37.0
        } else {
            (((PantherPainter::HEIGHT + 1.6) + (pose.body_dy).abs()) + rt::max_f64(0.0, -self.dip))
        });
    }
    pub fn wrap(mut degrees: f64) -> f64 {
        let mut turn: f64 = (degrees % 360.0);
        if turn > 180.0 {
            turn -= 360.0;
        } else if turn <= -180.0 {
            turn += 360.0;
        }
        return turn;
    }
    pub fn place_eye(mut self, mut i: i32, mut phi: f64) {
        let mut a: f64 = (phi + self.head_yaw);
        (*self.eye_alpha.at(i)) = a;
        (*self.eye_u.at(i)) = ((PantherPainter::HEAD_RX + 0.05) * (a).sin());
        let mut c: f64 = (a).cos();
        (*self.eye_k.at(i)) = (if (c > 0.25) { c } else { 0.25 });
        let mut away: f64 = ((a).abs() / SolidPainter::DEG);
        (*self.eye_vis.at(i)) = (if (away <= 72.0) {
            1.0
        } else {
            (if (away >= 95.0) {
                0.0
            } else {
                ((95.0 - away) / 23.0)
            })
        });
    }
    pub fn place_leg(mut self, mut i: i32, mut x: f64, mut swing: f64, mut bend: f64) {
        (*self.hip_u.at(i)) = (x * self.cos_y);
        (*self.leg_z.at(i)) = (-x * self.sin_y);
        let mut a: f64 = (swing * SolidPainter::DEG);
        (*self.knee_u.at(i)) = ((*self.hip_u.at(i)) + (PantherPainter::THIGH * (a).sin()));
        (*self.knee_v.at(i)) = (PantherPainter::HIP_V - (PantherPainter::THIGH * (a).cos()));
        let mut b: f64 = ((swing - bend) * SolidPainter::DEG);
        (*self.ankle_u.at(i)) = ((*self.knee_u.at(i)) + (PantherPainter::SHIN * (b).sin()));
        (*self.ankle_v.at(i)) = (((*self.knee_v.at(i)) - (PantherPainter::SHIN * (b).cos()))
            + (0.6 * self.pose.tiptoe));
    }
    //
    // Lays out the tail as a long hanging banana curve through six
    // control points: out from the rear, drooping down, then flicking up
    // at the thin tip. Every bend along the thick part turns 45 degrees
    // or less, so the tube's inner wall can never pinch into a crease;
    // only the thin tip hooks harder. Resampled with Catmull-Rom into
    // short links so no straight-rod joints ever show. Radius tapers
    // thin and even towards the tip. Drawn in profile at any yaw, the
    // way the cartoon always draws the tail.
    //
    pub fn build_tail(mut self) {
        let mut cu: Arr<f64> = self.knot_u.clone();
        let mut cv: Arr<f64> = self.knot_v.clone();
        let mut root_u: f64 = (-2.0 * self.sin_y);
        (*cu.at(0)) = root_u;
        (*cv.at(0)) = 11.5;
        (*cu.at(1)) = ((root_u - 3.0) + (self.tail_sway * 0.4));
        (*cv.at(1)) = 9.8;
        (*cu.at(2)) = ((root_u - 4.8) + (self.tail_sway * 0.7));
        (*cv.at(2)) = 7.6;
        (*cu.at(3)) = ((root_u - 5.4) + self.tail_sway);
        (*cv.at(3)) = 5.0;
        (*cu.at(4)) = ((root_u - 4.4) + (self.tail_sway * 1.3));
        (*cv.at(4)) = 3.2;
        (*cu.at(5)) = ((root_u - 3.2) + (self.tail_sway * 1.5));
        (*cv.at(5)) = 3.4;
        self.tail_hi = (*cu.at(0));
        self.tail_lo = self.tail_hi;
        self.tail_vhi = (*cv.at(0));
        self.tail_vlo = self.tail_vhi;
        {
            let mut j: i32 = 0;
            'l1: while j < PantherPainter::TAIL_SAMPLES {
                let mut span: i32 = (j / PantherPainter::TAIL_PER);
                let mut t: f64 =
                    (((j % PantherPainter::TAIL_PER) as f64) / ((PantherPainter::TAIL_PER) as f64));
                let mut p0: i32 = (if ((span - 1) < 0) { 0 } else { (span - 1) });
                let mut p3: i32 = (if ((span + 2) > 5) { 5 } else { (span + 2) });
                (*self.tail_u.at(j)) = PantherPainter::catmull(
                    (*cu.at(p0)),
                    (*cu.at(span)),
                    (*cu.at((span + 1))),
                    (*cu.at(p3)),
                    t,
                );
                (*self.tail_v.at(j)) = PantherPainter::catmull(
                    (*cv.at(p0)),
                    (*cv.at(span)),
                    (*cv.at((span + 1))),
                    (*cv.at(p3)),
                    t,
                );
                if (*self.tail_u.at(j)) < self.tail_lo {
                    self.tail_lo = (*self.tail_u.at(j));
                }
                if (*self.tail_u.at(j)) > self.tail_hi {
                    self.tail_hi = (*self.tail_u.at(j));
                }
                if (*self.tail_v.at(j)) < self.tail_vlo {
                    self.tail_vlo = (*self.tail_v.at(j));
                }
                if (*self.tail_v.at(j)) > self.tail_vhi {
                    self.tail_vhi = (*self.tail_v.at(j));
                }
                j += 1;
            }
        }
        (*self.tail_u.at(PantherPainter::TAIL_SAMPLES)) = (*cu.at(5));
        (*self.tail_v.at(PantherPainter::TAIL_SAMPLES)) = (*cv.at(5));
        {
            let mut j: i32 = 0;
            'l2: while j < PantherPainter::TAIL_SAMPLES {
                (*self.tail_du.at(j)) = ((*self.tail_u.at((j + 1))) - (*self.tail_u.at(j)));
                (*self.tail_dv.at(j)) = ((*self.tail_v.at((j + 1))) - (*self.tail_v.at(j)));
                (*self.tail_len2.at(j)) = (((*self.tail_du.at(j)) * (*self.tail_du.at(j)))
                    + ((*self.tail_dv.at(j)) * (*self.tail_dv.at(j))));
                j += 1;
            }
        }
    }
    pub fn catmull(mut p0: f64, mut p1: f64, mut p2: f64, mut p3: f64, mut t: f64) -> f64 {
        let mut t2: f64 = (t * t);
        let mut t3: f64 = (t2 * t);
        return (0.5
            * ((((2.0 * p1) + ((-p0 + p2) * t))
                + (((((2.0 * p0) - (5.0 * p1)) + (4.0 * p2)) - p3) * t2))
                + ((((-p0 + (3.0 * p1)) - (3.0 * p2)) + p3) * t3)));
    }
    pub fn place_arm(
        mut self,
        mut i: i32,
        mut side: f64,
        mut raise: f64,
        mut bend: f64,
        mut reach: f64,
        mut swing: f64,
        mut fore_swing: f64,
    ) {
        let mut at: f64 =
            (((side * PantherPainter::SHOULDER_ANGLE) * SolidPainter::DEG) + self.yaw);
        (*self.sh_u.at(i)) = (PantherPainter::ARM_RADIUS * (at).sin());
        (*self.sh_z.at(i)) = (PantherPainter::ARM_RADIUS * (at).cos());
        if raise == 0.0 {
            raise = (side * PantherPainter::REST_SPREAD);
        }
        if reach != 0.0 {
            swing = (if ((raise > 90.0) || (raise < -90.0)) {
                -reach
            } else {
                reach
            });
            fore_swing = -swing;
        }
        let mut a: f64 = (raise * SolidPainter::DEG);
        let mut b: f64 = (swing * SolidPainter::DEG);
        let mut sa: f64 = (a).sin();
        let mut ca: f64 = (a).cos();
        let mut sb: f64 = (b).sin();
        let mut cb: f64 = (b).cos();
        (*self.elb_u.at(i)) =
            ((*self.sh_u.at(i)) + (PantherPainter::UPPER_ARM * (sa + ((ca * sb) * self.sin_y))));
        (*self.elb_v.at(i)) =
            (PantherPainter::SHOULDER_V - ((PantherPainter::UPPER_ARM * ca) * cb));
        (*self.elb_z.at(i)) =
            ((*self.sh_z.at(i)) + (((PantherPainter::UPPER_ARM * ca) * sb) * self.cos_y));
        a = ((raise + bend) * SolidPainter::DEG);
        b = ((swing + fore_swing) * SolidPainter::DEG);
        sa = (a).sin();
        ca = (a).cos();
        sb = (b).sin();
        cb = (b).cos();
        (*self.hand_u.at(i)) =
            ((*self.elb_u.at(i)) + (PantherPainter::FOREARM * (sa + ((ca * sb) * self.sin_y))));
        (*self.hand_v.at(i)) = ((*self.elb_v.at(i)) - ((PantherPainter::FOREARM * ca) * cb));
        (*self.hand_z.at(i)) =
            ((*self.elb_z.at(i)) + (((PantherPainter::FOREARM * ca) * sb) * self.cos_y));
        let mut f: f64 = 0.82;
        (*self.wrist_u.at(i)) =
            ((*self.elb_u.at(i)) + (((*self.hand_u.at(i)) - (*self.elb_u.at(i))) * f));
        (*self.wrist_v.at(i)) =
            ((*self.elb_v.at(i)) + (((*self.hand_v.at(i)) - (*self.elb_v.at(i))) * f));
        (*self.wrist_z.at(i)) =
            ((*self.elb_z.at(i)) + (((*self.hand_z.at(i)) - (*self.elb_z.at(i))) * f));
        let mut pad: f64 = 1.3;
        if (side > 0.0) && self.pose.point_near {
            pad += PantherPainter::CLAW_LEN;
        }
        (*self.arm_lo.at(i)) = (rt::min_f64(
            rt::min_f64((*self.sh_u.at(i)), (*self.elb_u.at(i))),
            (*self.hand_u.at(i)),
        ) - pad);
        (*self.arm_hi.at(i)) = (rt::max_f64(
            rt::max_f64((*self.sh_u.at(i)), (*self.elb_u.at(i))),
            (*self.hand_u.at(i)),
        ) + pad);
        (*self.arm_vlo.at(i)) = (rt::min_f64(
            rt::min_f64(PantherPainter::SHOULDER_V, (*self.elb_v.at(i))),
            (*self.hand_v.at(i)),
        ) - pad);
        (*self.arm_vhi.at(i)) = (rt::max_f64(
            rt::max_f64(PantherPainter::SHOULDER_V, (*self.elb_v.at(i))),
            (*self.hand_v.at(i)),
        ) + pad);
    }
    pub fn draw(
        mut surface: Surface,
        mut pose: Pose,
        mut face_left: bool,
        mut centre_x: f64,
        mut sole_y: f64,
        mut scale: f64,
    ) {
        let mut sx: f64 = (scale * pose.stretch_x);
        let mut sy: f64 = (scale * pose.stretch_y);
        let mut px: f64 = (1.0 / rt::min_f64(sx, sy));
        let mut painter: PantherPainter = PANTHERPAINTER_CACHED.with(|c__| {
            if c__.get().is_null() {
                c__.set(PantherPainter::new());
            }
            c__.get()
        });
        painter.init(pose, face_left, px);
        let mut left: i32 = rt::max_i32(0, (((centre_x - (12.5 * sx)).floor()) as i32));
        let mut right: i32 =
            rt::min_i32(surface.width(), (((centre_x + (12.5 * sx)).ceil()) as i32));
        let mut top: i32 = rt::max_i32(0, (((sole_y - (painter.top_v * sy)).floor()) as i32));
        let mut bottom: i32 = rt::min_i32(surface.height(), (((sole_y + 0.5).ceil()) as i32));
        if (left >= right) || (top >= bottom) {
            return;
        }
        painter.raster(surface, left, top, right, bottom, centre_x, sole_y, sx, sy);
    }
    pub fn shade(mut self, mut u: f64, mut v: f64) -> i32 {
        self.begin();
        let mut vb: f64 = (v + self.pose.body_dy);
        let mut ub: f64 = (u - ((self.lean * 0.14) * rt::max_f64(0.0, (v - 10.0))));
        if vb != self.row_v {
            let mut axis_v: f64 = (if (vb < PantherPainter::TORSO_BOTTOM) {
                PantherPainter::TORSO_BOTTOM
            } else {
                (if (vb > PantherPainter::TORSO_TOP) {
                    PantherPainter::TORSO_TOP
                } else {
                    vb
                })
            });
            let mut t: f64 = ((axis_v - PantherPainter::TORSO_BOTTOM)
                / (PantherPainter::TORSO_TOP - PantherPainter::TORSO_BOTTOM));
            let mut r: f64 =
                (PantherPainter::HIP_R + ((PantherPainter::CHEST_R - PantherPainter::HIP_R) * t));
            // Narrow waist: pinch the middle in a little.
            r -= (0.35 * (t * std::f64::consts::PI).sin());
            // Above the shoulders the dome is squashed to ShoulderCap so the
            // neck stands clear of the chest instead of lying along it.
            let mut tdy: f64 = (vb - axis_v);
            self.row_dy_eff = (if (vb > PantherPainter::TORSO_TOP) {
                ((tdy * r) / PantherPainter::SHOULDER_CAP)
            } else {
                tdy
            });
            self.row_torso_r = r;
            self.row_cos_front = ((PantherPainter::BACK_ARC - (0.5 * self.px)) / r).cos();
            self.row_cos_back = ((PantherPainter::BACK_ARC + (0.5 * self.px)) / r).cos();
            self.row_v = vb;
        }
        let mut torso_r: f64 = self.row_torso_r;
        let mut dy_eff: f64 = self.row_dy_eff;
        let mut torso_dist: f64 = ((ub * ub) + (dy_eff * dy_eff)).sqrt();
        let mut bulge: f64 = ((torso_r * torso_r) - (torso_dist * torso_dist));
        self.body_z = (if (bulge > 0.0) { (bulge).sqrt() } else { 0.0 });
        let mut hdy: f64 = (vb - PantherPainter::HEAD_V);
        let mut hky: f64 = (PantherPainter::HEAD_RX / PantherPainter::HEAD_RY);
        let mut head_bulge: f64 = (((PantherPainter::HEAD_RX * PantherPainter::HEAD_RX)
            - (ub * ub))
            - (((hdy * hdy) * hky) * hky));
        if head_bulge > (self.body_z * self.body_z) {
            self.body_z = (head_bulge).sqrt();
        }
        let mut back_leg: i32 = (if ((*self.leg_z.at(0)) < (*self.leg_z.at(1))) {
            0
        } else {
            1
        });
        self.tail(ub, vb);
        self.leg(ub, vb, back_leg);
        self.leg(ub, vb, (1 - back_leg));
        self.limb(ub, vb, 0);
        self.limb(ub, vb, 1);
        // The neck behind the torso: its base stays buried in the
        // shoulders, so no tube shows through the chest - only the part
        // standing clear above them is ever seen.
        self.neck(ub, vb);
        self.torso(ub, vb, dy_eff, torso_dist, torso_r);
        if (*self.ear_z.at(0)) <= 0.0 {
            self.ear(ub, vb, 0);
        }
        if (*self.ear_z.at(1)) <= 0.0 {
            self.ear(ub, vb, 1);
        }
        if ((hdy).abs() < (PantherPainter::HEAD_RY + 1.6))
            && ((ub).abs() < ((PantherPainter::HEAD_RX + PantherPainter::SNOUT_LEN) + 0.8))
        {
            self.head(ub, vb, hdy);
            if (*self.ear_z.at(0)) > 0.0 {
                self.ear(ub, vb, 0);
            }
            if (*self.ear_z.at(1)) > 0.0 {
                self.ear(ub, vb, 1);
            }
            self.snout(ub, vb);
            self.eye(ub, vb, self.first_eye);
            self.eye(ub, vb, (1 - self.first_eye));
            self.brows(ub, vb);
            self.nose(ub, vb);
            self.mouth(ub, vb);
            self.whiskers(ub, vb);
        }
        self.lay_front();
        if self.decorated {
            if self.pose.sweat > 0.0 {
                self.sweat(
                    ub,
                    vb,
                    self.pose.sweat,
                    -(PantherPainter::HEAD_RX + 0.9),
                    (PantherPainter::HEAD_V + 1.2),
                );
            }
            if self.pose.note {
                self.note(
                    ub,
                    vb,
                    self.pose.decor_phase,
                    self.mouth_u,
                    PantherPainter::MOUTH_V,
                );
            }
            if self.pose.zzz {
                self.zzz(
                    u,
                    v,
                    self.pose.decor_phase,
                    2.6,
                    (PantherPainter::HEIGHT + 1.2),
                );
            }
            if self.pose.bubble {
                self.bubble(
                    ub,
                    vb,
                    (PantherPainter::HEIGHT + 0.4),
                    (PantherPainter::HEIGHT + 1.3),
                    (PantherPainter::HEIGHT + 2.8),
                    (PantherPainter::HEIGHT + 5.4),
                );
            }
            if self.pose.heart {
                self.heart(
                    ub,
                    vb,
                    self.pose.decor_phase,
                    self.mouth_u,
                    PantherPainter::MOUTH_V,
                );
            }
        }
        return self.finish();
    }
    pub fn ellipse(mut dx: f64, mut dy: f64, mut rx: f64, mut ry: f64) -> f64 {
        let mut ex: f64 = (dx / rx);
        let mut ey: f64 = (dy / ry);
        let mut e: f64 = ((ex * ex) + (ey * ey)).sqrt();
        if e < 1e-6 {
            return -(rt::min_f64(rx, ry));
        }
        let mut grad: f64 = ((((ex * ex) / (rx * rx)) + ((ey * ey) / (ry * ry))).sqrt() / e);
        return ((e - 1.0) / grad);
    }
    pub fn leg(mut self, mut u: f64, mut v: f64, mut i: i32) {
        if v > (PantherPainter::HIP_V + 1.5) {
            return;
        }
        let mut lo: f64 = (rt::min_f64(
            rt::min_f64((*self.hip_u.at(i)), (*self.knee_u.at(i))),
            (*self.ankle_u.at(i)),
        ) - 2.4);
        let mut hi: f64 = (rt::max_f64(
            rt::max_f64((*self.hip_u.at(i)), (*self.knee_u.at(i))),
            (*self.ankle_u.at(i)),
        ) + 2.4);
        if (u < lo) || (u > hi) {
            return;
        }
        let mut k: f64 =
            (0.82 + ((0.18 * (1.0 + ((*self.leg_z.at(i)) / PantherPainter::HIP_X))) / 2.0));
        self.tube(
            u,
            v,
            (*self.hip_u.at(i)),
            PantherPainter::HIP_V,
            (*self.knee_u.at(i)),
            (*self.knee_v.at(i)),
            PantherPainter::THIGH_R,
            (PantherPainter::FUR_R * k),
            (PantherPainter::FUR_G * k),
            (PantherPainter::FUR_B * k),
            0.28,
            16.0,
        );
        self.tube(
            u,
            v,
            (*self.knee_u.at(i)),
            (*self.knee_v.at(i)),
            (*self.ankle_u.at(i)),
            (*self.ankle_v.at(i)),
            PantherPainter::SHIN_R,
            (PantherPainter::FUR_R * k),
            (PantherPainter::FUR_G * k),
            (PantherPainter::FUR_B * k),
            0.28,
            16.0,
        );
        // Hind paw: a sleek feline foot, toe forward.
        let mut tip: f64 = self.pose.tiptoe;
        let mut sx: f64 = (((*self.ankle_u.at(i)) + self.toe_dir) + (0.25 * tip));
        let mut sy: f64 = (((*self.ankle_v.at(i)) - 0.5) - (0.2 * tip));
        let mut dx: f64 = ((u - sx) / self.paw_len);
        let mut dy: f64 = ((v - sy) / 0.55);
        let mut e: f64 = ((dx * dx) + (dy * dy)).sqrt();
        let mut cov: f64 = self.cov(((e - 1.0) * 0.55));
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        PantherPainter::normal(
            dx,
            dy,
            1.0,
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        let mut lr: f64 = <f64>::default();
        let mut lg: f64 = <f64>::default();
        let mut lb: f64 = <f64>::default();
        self.lit(
            (PantherPainter::FUR_R * k),
            (PantherPainter::FUR_G * k),
            (PantherPainter::FUR_B * k),
            nx,
            ny,
            nz,
            0.35,
            18.0,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.over(lr, lg, lb, cov);
        // Toe beans: three small darker pads along the front.
        {
            let mut toe: i32 = -1;
            'l1: while toe <= 1 {
                let mut tx: f64 = ((sx + 0.45) + (((toe) as f64) * 0.05));
                let mut ty: f64 = ((sy + 0.08) + ((((toe).abs()) as f64) * -0.08));
                let mut ox: f64 = (u - tx);
                let mut oy: f64 = (v - ty);
                self.over(
                    (PantherPainter::FUR_DARK_R * k),
                    (PantherPainter::FUR_DARK_G * k),
                    (PantherPainter::FUR_DARK_B * k),
                    ((self.cov((((ox * ox) + (oy * oy)).sqrt() - 0.10)) * 0.5) * cov),
                );
                toe += 1;
            }
        }
        // Main pad highlight on top of the paw.
        {
            let mut hx: f64 = (u - (sx - 0.2));
            let mut hy: f64 = (v - (sy + 0.22));
            self.over(
                1.0,
                0.85,
                0.9,
                ((self.cov((((hx * hx) + (hy * hy)).sqrt() - 0.14)) * 0.35) * cov),
            );
        }
    }
    pub fn torso(mut self, mut u: f64, mut v: f64, mut dy: f64, mut dist: f64, mut r: f64) {
        let mut cov: f64 = self.cov((dist - r));
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        PantherPainter::normal(
            u,
            dy,
            r,
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        // Sleek pink coat, plain as the cartoon's: no chest patch, no
        // sheen; a slightly deeper pink down the back so the round reads.
        // The back starts BackArc round from the front. The cosine of
        // the angle round the body says at once whether a pixel is well
        // in front of that (none of it) or well behind (all of it); only
        // across the soft edge between is the angle itself worked out.
        let mut back: f64 = <f64>::default();
        let mut n: f64 = ((nx * nx) + (nz * nz)).sqrt();
        let mut c: f64 = ((nx * self.sin_y) + (nz * self.cos_y));
        if c > ((n * self.row_cos_front) + 1e-9) {
            back = 0.0;
        } else if c < ((n * self.row_cos_back) - 1e-9) {
            back = 1.0;
        } else {
            let mut phi: f64 = ((nx).atan2(nz) - self.yaw);
            if phi > std::f64::consts::PI {
                phi -= (2.0 * std::f64::consts::PI);
            } else if phi < -(std::f64::consts::PI) {
                phi += (2.0 * std::f64::consts::PI);
            }
            let mut arc: f64 = (r * phi);
            back = self.cov((PantherPainter::BACK_ARC - (arc).abs()));
        }
        let mut cr: f64 = PantherPainter::FUR_R;
        let mut cg: f64 = PantherPainter::FUR_G;
        let mut cb: f64 = PantherPainter::FUR_B;
        let mut spec: f64 = 0.30;
        let mut shine: f64 = 18.0;
        PantherPainter::mix(
            rt::mref(&mut cr),
            rt::mref(&mut cg),
            rt::mref(&mut cb),
            rt::mref(&mut spec),
            rt::mref(&mut shine),
            PantherPainter::FUR_DARK_R,
            PantherPainter::FUR_DARK_G,
            PantherPainter::FUR_DARK_B,
            0.28,
            16.0,
            ((back * 0.35) * cov),
        );
        let mut lr: f64 = <f64>::default();
        let mut lg: f64 = <f64>::default();
        let mut lb: f64 = <f64>::default();
        self.lit(
            cr,
            cg,
            cb,
            nx,
            ny,
            nz,
            spec,
            shine,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.over(lr, lg, lb, cov);
    }
    pub fn neck(mut self, mut u: f64, mut v: f64) {
        if (((u).abs() > (PantherPainter::NECK_R + 0.6))
            || (v < (PantherPainter::NECK_BOTTOM - 0.3)))
            || (v > (PantherPainter::NECK_TOP + 0.4))
        {
            return;
        }
        let mut qx: f64 = <f64>::default();
        let mut qy: f64 = <f64>::default();
        let mut tt: f64 = <f64>::default();
        let mut dist: f64 = PantherPainter::seg(
            u,
            v,
            0.0,
            PantherPainter::NECK_BOTTOM,
            0.0,
            PantherPainter::NECK_TOP,
            rt::mref(&mut qx),
            rt::mref(&mut qy),
            rt::mref(&mut tt),
        );
        let mut cov: f64 = self.cov((dist - PantherPainter::NECK_R));
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        PantherPainter::normal(
            (u - qx),
            (v - qy),
            PantherPainter::NECK_R,
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        let mut lr: f64 = <f64>::default();
        let mut lg: f64 = <f64>::default();
        let mut lb: f64 = <f64>::default();
        self.lit(
            (PantherPainter::FUR_R * 0.97),
            (PantherPainter::FUR_G * 0.97),
            (PantherPainter::FUR_B * 0.97),
            nx,
            ny,
            nz,
            0.30,
            18.0,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.over(lr, lg, lb, cov);
    }
    pub fn ear(mut self, mut u: f64, mut v: f64, mut i: i32) {
        let mut cx: f64 = (*self.ear_u.at(i));
        let mut cy: f64 = (PantherPainter::EAR_BASE_V - (self.dip * 0.7));
        // A small pointed ear: a tapering tube from a round base to a
        // tip leaning outward, set back on the skull.
        let mut lean_out: f64 = (if (i == 0) { -0.4 } else { 0.4 });
        let mut tx: f64 = (cx + lean_out);
        let mut ty: f64 = (cy + 1.25);
        if (((u - cx).abs() > 1.2) || (v < (cy - 0.7))) || (v > (ty + 0.5)) {
            return;
        }
        let mut qx: f64 = <f64>::default();
        let mut qy: f64 = <f64>::default();
        let mut t: f64 = <f64>::default();
        let mut d: f64 = (PantherPainter::seg(
            u,
            v,
            cx,
            (cy + 0.2),
            tx,
            ty,
            rt::mref(&mut qx),
            rt::mref(&mut qy),
            rt::mref(&mut t),
        ) - (0.62 - (0.42 * t)));
        let mut cov: f64 = self.cov(d);
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        PantherPainter::normal(
            (u - qx),
            (v - qy),
            (0.62 - (0.42 * t)),
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        let mut lr: f64 = <f64>::default();
        let mut lg: f64 = <f64>::default();
        let mut lb: f64 = <f64>::default();
        self.lit(
            PantherPainter::FUR_R,
            PantherPainter::FUR_G,
            PantherPainter::FUR_B,
            nx,
            ny,
            nz,
            0.30,
            18.0,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.over(lr, lg, lb, cov);
        // Inner ear: a lighter pink hollow, only when the ear faces the viewer.
        if (*self.ear_z.at(i)) > 0.0 {
            let mut ix: f64 = (u - (cx + (lean_out * 0.3)));
            let mut iy: f64 = (v - (cy + 0.5));
            self.over(
                PantherPainter::CREAM_R,
                PantherPainter::CREAM_G,
                PantherPainter::CREAM_B,
                ((self.cov((((ix * ix) + ((iy * iy) * 2.2)).sqrt() - 0.3)) * 0.85) * cov),
            );
        }
    }
    pub fn head(mut self, mut u: f64, mut v: f64, mut dy: f64) {
        let mut cov: f64 = self.cov(PantherPainter::ellipse(
            u,
            dy,
            PantherPainter::HEAD_RX,
            PantherPainter::HEAD_RY,
        ));
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        PantherPainter::normal(
            ((u / PantherPainter::HEAD_RX) * 3.0),
            ((dy / PantherPainter::HEAD_RY) * 3.0),
            3.0,
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        let mut lr: f64 = <f64>::default();
        let mut lg: f64 = <f64>::default();
        let mut lb: f64 = <f64>::default();
        self.lit(
            PantherPainter::FUR_R,
            PantherPainter::FUR_G,
            PantherPainter::FUR_B,
            nx,
            ny,
            nz,
            0.32,
            20.0,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.over(lr, lg, lb, cov);
        // Soft cheek shading low on the sides so the skull reads round.
        self.over(
            PantherPainter::FUR_DARK_R,
            PantherPainter::FUR_DARK_G,
            PantherPainter::FUR_DARK_B,
            (((self.cov((((u).abs() - 1.9).abs() - 0.5)) * self.cov((v - 21.0))) * 0.25) * cov),
        );
        // Forehead highlight catching the light upper-left.
        {
            let mut hx: f64 = (u + 0.9);
            let mut hy: f64 = (v - (PantherPainter::HEAD_V + 1.1));
            self.over(
                1.0,
                0.85,
                0.9,
                ((self.cov((((hx * hx) + (hy * hy)).sqrt() - 0.55)) * 0.30) * cov),
            );
        }
    }
    pub fn eye(mut self, mut u: f64, mut v: f64, mut i: i32) {
        let mut vis: f64 = (*self.eye_vis.at(i));
        if vis <= 0.0 {
            return;
        }
        let mut k: f64 = (*self.eye_k.at(i));
        let mut near: bool = (i != self.first_eye);
        let mut fu: f64 = ((*self.eye_u.at(i)) + ((self.dip * 0.5) * self.sin_h));
        let mut fv: f64 = (PantherPainter::EYE_V - (self.dip * 0.8));
        let mut dx: f64 = ((u - fu) / k);
        let mut dy: f64 = (v - fv);
        if ((dx).abs() > (PantherPainter::EYE_RX + 0.6))
            || ((dy).abs() > (PantherPainter::EYE_RY + 0.6))
        {
            return;
        }
        let mut ex: f64 = (dx / PantherPainter::EYE_RX);
        let mut ey: f64 = (dy / PantherPainter::EYE_RY);
        let mut white: f64 = ((((ex * ex) + (ey * ey)).sqrt() - 1.0) * PantherPainter::EYE_RY);
        let mut cov_white: f64 = (self.cov(white) * vis);
        if cov_white <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        PantherPainter::normal(
            dx,
            dy,
            (PantherPainter::EYE_RX * 1.6),
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        PantherPainter::turn_2(
            rt::mref(&mut nx),
            rt::mref(&mut nz),
            (*self.eye_cos.at(i)),
            (*self.eye_sin.at(i)),
        );
        let mut r: f64 = <f64>::default();
        let mut g: f64 = <f64>::default();
        let mut b: f64 = <f64>::default();
        // The cartoon's eye: the whole eye is yellow, with a round black
        // pupil, under a heavy half-closed lid - the cool look.
        self.lit(
            PantherPainter::IRIS_YR,
            PantherPainter::IRIS_YG,
            PantherPainter::IRIS_YB,
            nx,
            ny,
            nz,
            0.4,
            24.0,
            rt::mref(&mut r),
            rt::mref(&mut g),
            rt::mref(&mut b),
        );
        let mut ix: f64 = (dx - (*self.gaze_u.at(i)));
        let mut iy: f64 = (dy - (*self.gaze_v.at(i)));
        let mut cov_pupil: f64 = self.cov(PantherPainter::squeezed(ix, iy, k, self.pupil_r));
        if cov_pupil > 0.0 {
            r += ((0.04 - r) * cov_pupil);
            g += ((0.03 - g) * cov_pupil);
            b += ((0.04 - b) * cov_pupil);
            let mut lx: f64 = (ix + ((self.flip * 0.35) * self.pupil_r));
            let mut ly: f64 = (iy - (0.35 * self.pupil_r));
            let mut cov_light: f64 =
                (self.cov(PantherPainter::squeezed(lx, ly, k, (self.pupil_r * 0.3))) * 0.95);
            r += ((1.0 - r) * cov_light);
            g += ((1.0 - g) * cov_light);
            b += ((1.0 - b) * cov_light);
        }
        // The lids sit a third of the way down even when the face is at
        // rest; a wide-eyed face lifts them, a sleepy one drops them.
        let mut upper: f64 = rt::max_f64(
            rt::min_f64(
                0.95,
                ((self.face.lid + 0.3) - (0.35 * rt::max_f64(0.0, self.face.brow_raise))),
            ),
            (if near { self.face.wink } else { 0.0 }),
        );
        let mut lower: f64 = self.face.squint;
        let mut s: f64 = (ex * ex);
        let mut upper_edge: f64 =
            ((fv + (PantherPainter::EYE_RY * (1.0 - (2.0 * upper)))) - ((0.25 * upper) * s));
        let mut lower_edge: f64 =
            ((fv - (PantherPainter::EYE_RY * (1.0 - (2.0 * lower)))) + ((0.25 * lower) * s));
        if (upper > 0.01) || (lower > 0.01) {
            let mut bx: f64 = <f64>::default();
            let mut by: f64 = <f64>::default();
            let mut bz: f64 = <f64>::default();
            PantherPainter::normal(
                ((u / PantherPainter::HEAD_RX) * 3.0),
                (((v - PantherPainter::HEAD_V) / PantherPainter::HEAD_RY) * 3.0),
                3.0,
                rt::mref(&mut bx),
                rt::mref(&mut by),
                rt::mref(&mut bz),
            );
            let mut sr: f64 = <f64>::default();
            let mut sg: f64 = <f64>::default();
            let mut sb: f64 = <f64>::default();
            self.lit(
                PantherPainter::FUR_R,
                PantherPainter::FUR_G,
                PantherPainter::FUR_B,
                bx,
                by,
                bz,
                0.32,
                20.0,
                rt::mref(&mut sr),
                rt::mref(&mut sg),
                rt::mref(&mut sb),
            );
            let mut cov_lid: f64 = rt::max_f64(
                (if (upper > 0.01) {
                    self.cov((upper_edge - v))
                } else {
                    0.0
                }),
                (if (lower > 0.01) {
                    self.cov((v - lower_edge))
                } else {
                    0.0
                }),
            );
            r += ((sr - r) * cov_lid);
            g += ((sg - g) * cov_lid);
            b += ((sb - b) * cov_lid);
            let mut lash: f64 = (rt::max_f64(
                (if (upper > 0.01) {
                    self.cov(((v - upper_edge).abs() - 0.07))
                } else {
                    0.0
                }),
                (if (lower > 0.01) {
                    self.cov(((v - lower_edge).abs() - 0.06))
                } else {
                    0.0
                }),
            ) * 0.7);
            r += ((0.25 - r) * lash);
            g += ((0.12 - g) * lash);
            b += ((0.18 - b) * lash);
        }
        self.over(r, g, b, cov_white);
        let mut rim: f64 =
            ((self.cov(((white).abs() - 0.07)) * (if (dy > 0.1) { 0.8 } else { 0.4 })) * vis);
        self.over(0.30, 0.12, 0.20, rim);
    }
    pub fn brows(mut self, mut u: f64, mut v: f64) {
        let mut base_v: f64 =
            ((PantherPainter::BROW_V - (self.dip * 0.8)) + (self.face.brow_raise * 0.35));
        if (v - base_v).abs() > 1.2 {
            return;
        }
        self.brow(u, v, 0, -1.0, base_v, (*self.eye_vis.at(0)));
        self.brow(
            u,
            v,
            1,
            1.0,
            (base_v + (self.face.brow_asym * 0.3)),
            (*self.eye_vis.at(1)),
        );
    }
    pub fn brow(
        mut self,
        mut u: f64,
        mut v: f64,
        mut i: i32,
        mut outward: f64,
        mut cy: f64,
        mut show: f64,
    ) {
        if show <= 0.0 {
            return;
        }
        let mut k: f64 = (*self.eye_cos.at(i));
        if k < 0.2 {
            return;
        }
        let mut ex: f64 = (((PantherPainter::HEAD_RX + 0.1) * (*self.eye_sin.at(i)))
            + ((self.dip * 0.5) * self.sin_h));
        let mut ax: f64 = (ex - ((outward * 0.9) * k));
        let mut ay: f64 = ((cy - (self.face.brow_angle * 0.4)) - 0.06);
        let mut bx: f64 = (ex + ((outward * 0.9) * k));
        let mut by: f64 = ((cy + (self.face.brow_angle * 0.4)) - 0.06);
        let mut qx: f64 = <f64>::default();
        let mut qy: f64 = <f64>::default();
        let mut tt: f64 = <f64>::default();
        let mut d: f64 = (PantherPainter::seg(
            u,
            v,
            ax,
            ay,
            bx,
            by,
            rt::mref(&mut qx),
            rt::mref(&mut qy),
            rt::mref(&mut tt),
        ) - 0.16);
        self.over(0.35, 0.12, 0.22, ((self.cov(d) * show) * 0.9));
    }
    // The snout: a tapering tube out of the face, the full length in
    // profile and a short bump head-on, in the same pink as the rest.
    pub fn snout(mut self, mut u: f64, mut v: f64) {
        let mut vis: f64 = (if (self.cos_h > -0.5) {
            1.0
        } else {
            (if (self.cos_h < -0.8) {
                0.0
            } else {
                ((self.cos_h + 0.8) / 0.3)
            })
        });
        if vis <= 0.0 {
            return;
        }
        if (v > ((self.snout_v0 + PantherPainter::SNOUT_R_BASE) + 0.4))
            || (v < ((self.snout_v1 - PantherPainter::SNOUT_R_TIP) - 0.5))
        {
            return;
        }
        if (u < ((rt::min_f64(self.snout_u0, self.snout_u1) - PantherPainter::SNOUT_R_BASE) - 0.4))
            || (u
                > ((rt::max_f64(self.snout_u0, self.snout_u1) + PantherPainter::SNOUT_R_BASE)
                    + 0.4))
        {
            return;
        }
        let mut qx: f64 = <f64>::default();
        let mut qy: f64 = <f64>::default();
        let mut t: f64 = <f64>::default();
        let mut dist: f64 = PantherPainter::seg(
            u,
            v,
            self.snout_u0,
            self.snout_v0,
            self.snout_u1,
            self.snout_v1,
            rt::mref(&mut qx),
            rt::mref(&mut qy),
            rt::mref(&mut t),
        );
        let mut radius: f64 = (PantherPainter::SNOUT_R_BASE
            + ((PantherPainter::SNOUT_R_TIP - PantherPainter::SNOUT_R_BASE) * t));
        let mut cov: f64 = (self.cov((dist - radius)) * vis);
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        PantherPainter::normal(
            (u - qx),
            (v - qy),
            radius,
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        let mut lr: f64 = <f64>::default();
        let mut lg: f64 = <f64>::default();
        let mut lb: f64 = <f64>::default();
        self.lit(
            PantherPainter::FUR_R,
            PantherPainter::FUR_G,
            PantherPainter::FUR_B,
            nx,
            ny,
            nz,
            0.30,
            18.0,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.over(lr, lg, lb, cov);
        // A touch of shade under the snout where it meets the face.
        self.over(
            PantherPainter::FUR_DARK_R,
            PantherPainter::FUR_DARK_G,
            PantherPainter::FUR_DARK_B,
            (((self.cov((v - (self.snout_v0 - (radius * 0.3)))) * 0.25) * (1.0 - t)) * cov),
        );
    }
    // The big black nose on the tip of the snout, with a glint.
    pub fn nose(mut self, mut u: f64, mut v: f64) {
        let mut vis: f64 = (if (self.cos_h > -0.5) {
            1.0
        } else {
            (if (self.cos_h < -0.8) {
                0.0
            } else {
                ((self.cos_h + 0.8) / 0.3)
            })
        });
        if vis <= 0.0 {
            return;
        }
        let mut nu: f64 = (self.snout_u1 + (0.15 * self.sin_h));
        let mut nv: f64 = (self.snout_v1 + 0.1);
        let mut dx: f64 = (u - nu);
        let mut dy: f64 = (v - nv);
        if ((dx).abs() > (PantherPainter::NOSE_R0 + 0.5))
            || ((dy).abs() > (PantherPainter::NOSE_R0 + 0.5))
        {
            return;
        }
        let mut cov: f64 =
            (self.cov((((dx * dx) + ((dy * dy) * 1.3)).sqrt() - PantherPainter::NOSE_R0)) * vis);
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        PantherPainter::normal(
            dx,
            dy,
            PantherPainter::NOSE_R0,
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        let mut lr: f64 = <f64>::default();
        let mut lg: f64 = <f64>::default();
        let mut lb: f64 = <f64>::default();
        self.lit(
            PantherPainter::NOSE_R,
            PantherPainter::NOSE_G,
            PantherPainter::NOSE_B,
            nx,
            ny,
            nz,
            0.6,
            24.0,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.over(lr, lg, lb, cov);
        let mut hx: f64 = (u - (nu - (0.14 * self.flip)));
        let mut hy: f64 = (v - (nv + 0.14));
        self.over(
            0.9,
            0.9,
            0.95,
            ((self.cov((((hx * hx) + (hy * hy)).sqrt() - 0.09)) * 0.7) * cov),
        );
    }
    pub fn mouth(mut self, mut u: f64, mut v: f64) {
        let mut vis: f64 = (if (self.cos_h > 0.17) {
            1.0
        } else {
            (if (self.cos_h < -0.08) {
                0.0
            } else {
                ((self.cos_h + 0.08) / 0.25)
            })
        });
        if vis <= 0.0 {
            return;
        }
        let mut k: f64 = (if (self.cos_h > 0.35) {
            self.cos_h
        } else {
            0.35
        });
        // The mouth runs along the underside of the snout, from the
        // cheek towards the nose.
        let mut mu: f64 =
            (((self.snout_u0 + self.snout_u1) / 2.0) + ((self.face.mouth_dx * 0.5) * k));
        let mut mv: f64 =
            ((PantherPainter::MOUTH_V - (self.dip * 0.9)) - (0.25 * (self.sin_h).abs()));
        let mut w: f64 = (((0.8 + (self.face.mouth_width * 0.8)) * k) + (0.6 * (self.sin_h).abs()));
        let mut hx: f64 = ((u - mu).abs() - w);
        if (hx > 0.7) || ((v - mv).abs() > 2.2) {
            return;
        }
        let mut tt: f64 = ((u - mu) / w);
        if tt < -1.0 {
            tt = -1.0;
        } else if tt > 1.0 {
            tt = 1.0;
        }
        // Feline smile: dips at the centre (philtrum), curls up at the corners.
        let mut line: f64 = (((mv + ((self.face.mouth_curve * 0.55) * ((tt * tt) - 0.35)))
            - (0.18 * (1.0 - (tt * tt))))
            + ((self.face.wobble * 0.10) * (tt * 8.5).sin()));
        let mut open: f64 = (self.face.mouth_open * 1.2);
        let mut lip: f64 = 0.17;
        if open < 0.03 {
            let mut ox: f64 = (if (hx > 0.0) { hx } else { 0.0 });
            let mut oy: f64 = (v - line);
            let mut cov: f64 = self.cov((((ox * ox) + (oy * oy)).sqrt() - (lip / 2.0)));
            self.over(0.48, 0.16, 0.28, (cov * vis));
            return;
        }
        let mut gape: f64 = ((open * 1.4) * rt::pow((1.0 - (tt * tt)), 0.6));
        let mut upper: f64 = line;
        let mut lower: f64 = (line - gape);
        let mut d: f64 = rt::max_f64(rt::max_f64((v - upper), (lower - v)), hx);
        let mut inside: f64 = self.cov(d);
        if inside > 0.0 {
            self.over(0.30, 0.08, 0.14, (inside * vis));
            let mut teeth_h: f64 = ((open * 0.55) * self.face.teeth);
            if teeth_h > 0.04 {
                let mut teeth: f64 = (self.cov(rt::max_f64(((upper - teeth_h) - v), d)) * inside);
                self.over(0.98, 0.96, 0.94, (teeth * vis));
                // One fang each side.
                let mut fx: f64 = ((u - mu).abs() - (w * 0.55));
                let mut fy: f64 = (v - (upper - (teeth_h * 0.6)));
                self.over(
                    0.98,
                    0.96,
                    0.94,
                    ((self.cov((((fx * fx) + (fy * fy)).sqrt() - 0.14)) * teeth) * vis),
                );
            }
            if (self.face.tongue > 0.02) || (open > 0.35) {
                let mut rx: f64 = rt::min_f64((w * 0.5), 0.5);
                let mut ry: f64 = (0.20 + (0.30 * rt::max_f64(self.face.tongue, (open * 0.5))));
                let mut cy: f64 =
                    ((lower + 0.20) - (0.45 * rt::max_f64(self.face.tongue, (open * 0.4))));
                let mut tx2: f64 = ((u - mu) / rx);
                let mut ty2: f64 = ((v - cy) / ry);
                let mut ee: f64 = ((tx2 * tx2) + (ty2 * ty2)).sqrt();
                let mut td: f64 = rt::max_f64(((ee - 1.0) * ry), ((v - upper) + 0.1));
                let mut cov_t: f64 = self.cov(td);
                if cov_t > 0.0 {
                    self.over(0.85, 0.35, 0.48, (cov_t * vis));
                }
            }
        }
        let mut outline: f64 = self.cov(((d).abs() - (lip / 2.0)));
        self.over(0.48, 0.16, 0.28, (outline * vis));
    }
    pub fn whiskers(mut self, mut u: f64, mut v: f64) {
        let mut vis: f64 = (if (self.cos_h > 0.0) {
            1.0
        } else {
            (if (self.cos_h < -0.4) {
                0.0
            } else {
                ((self.cos_h + 0.4) / 0.4)
            })
        });
        if vis <= 0.0 {
            return;
        }
        // Three long thin whiskers a side, from the snout's cheeks; from
        // the side only the near ones show, sweeping back past the face.
        let mut root_v: f64 = (((self.snout_v0 + self.snout_v1) / 2.0) - 0.1);
        if (((v - root_v).abs() > 1.8) || (u < self.whisk_lo)) || (u > self.whisk_hi) {
            return;
        }
        let mut qx: f64 = <f64>::default();
        let mut qy: f64 = <f64>::default();
        let mut tt: f64 = <f64>::default();
        let mut root_u: f64 = ((self.snout_u0 + self.snout_u1) / 2.0);
        let mut k: f64 = (if (self.cos_h > 0.35) {
            self.cos_h
        } else {
            0.35
        });
        {
            let mut side: i32 = -1;
            'l1: while side <= 1 {
                let mut ru: f64 = (root_u + ((((side) as f64) * 0.75) * k));
                {
                    let mut w: i32 = -1;
                    'l2: while w <= 1 {
                        let mut tip_u: f64 = ((ru
                            + ((((side) as f64) * (2.0 + ((((w).abs()) as f64) * 0.2)))
                                * (0.4 + (0.6 * k))))
                            - (0.6 * self.sin_h));
                        let mut tip_v: f64 = ((root_v + (((w) as f64) * 0.5)) + 0.15);
                        let mut d: f64 = (PantherPainter::seg(
                            u,
                            v,
                            ru,
                            (root_v + (((w) as f64) * 0.16)),
                            tip_u,
                            tip_v,
                            rt::mref(&mut qx),
                            rt::mref(&mut qy),
                            rt::mref(&mut tt),
                        ) - 0.03);
                        self.over(0.25, 0.10, 0.16, ((self.cov(d) * 0.7) * vis));
                        w += 1;
                    }
                }
                side += 2;
            }
        }
    }
    pub fn limb(mut self, mut u: f64, mut v: f64, mut i: i32) {
        if (((u < (*self.arm_lo.at(i))) || (u > (*self.arm_hi.at(i))))
            || (v < (*self.arm_vlo.at(i))))
            || (v > (*self.arm_vhi.at(i)))
        {
            return;
        }
        self.arm_tube(
            u,
            v,
            (*self.sh_u.at(i)),
            PantherPainter::SHOULDER_V,
            (*self.sh_z.at(i)),
            (*self.elb_u.at(i)),
            (*self.elb_v.at(i)),
            (*self.elb_z.at(i)),
            PantherPainter::SLEEVE_R,
            PantherPainter::FUR_R,
            PantherPainter::FUR_G,
            PantherPainter::FUR_B,
            0.28,
            16.0,
        );
        self.arm_tube(
            u,
            v,
            (*self.elb_u.at(i)),
            (*self.elb_v.at(i)),
            (*self.elb_z.at(i)),
            (*self.wrist_u.at(i)),
            (*self.wrist_v.at(i)),
            (*self.wrist_z.at(i)),
            PantherPainter::FORE_R,
            PantherPainter::FUR_R,
            PantherPainter::FUR_G,
            PantherPainter::FUR_B,
            0.28,
            16.0,
        );
        if (i == 1) && self.pose.point_near {
            let mut fx: f64 = ((*self.hand_u.at(i)) - (*self.wrist_u.at(i)));
            let mut fy: f64 = ((*self.hand_v.at(i)) - (*self.wrist_v.at(i)));
            let mut len: f64 = ((fx * fx) + (fy * fy)).sqrt();
            if len > 1e-6 {
                let mut reach: f64 = ((PantherPainter::PAW_R + PantherPainter::CLAW_LEN) / len);
                self.arm_tube(
                    u,
                    v,
                    (*self.hand_u.at(i)),
                    (*self.hand_v.at(i)),
                    (*self.hand_z.at(i)),
                    ((*self.hand_u.at(i)) + (fx * reach)),
                    ((*self.hand_v.at(i)) + (fy * reach)),
                    (*self.hand_z.at(i)),
                    PantherPainter::CLAW_R,
                    PantherPainter::CREAM_R,
                    PantherPainter::CREAM_G,
                    PantherPainter::CREAM_B,
                    0.3,
                    16.0,
                );
            }
        }
        let mut dx: f64 = (u - (*self.hand_u.at(i)));
        let mut dy: f64 = (v - (*self.hand_v.at(i)));
        let mut cov: f64 = self.cov((((dx * dx) + (dy * dy)).sqrt() - PantherPainter::PAW_R));
        if cov <= 0.0 {
            return;
        }
        let mut hnx: f64 = <f64>::default();
        let mut hny: f64 = <f64>::default();
        let mut hnz: f64 = <f64>::default();
        PantherPainter::normal(
            dx,
            dy,
            PantherPainter::PAW_R,
            rt::mref(&mut hnx),
            rt::mref(&mut hny),
            rt::mref(&mut hnz),
        );
        let mut hk: f64 = self.depth((*self.hand_z.at(i)));
        let mut hr: f64 = <f64>::default();
        let mut hg: f64 = <f64>::default();
        let mut hb: f64 = <f64>::default();
        self.lit(
            (PantherPainter::FUR_R * hk),
            (PantherPainter::FUR_G * hk),
            (PantherPainter::FUR_B * hk),
            hnx,
            hny,
            hnz,
            0.32,
            20.0,
            rt::mref(&mut hr),
            rt::mref(&mut hg),
            rt::mref(&mut hb),
        );
        self.sort(
            hr,
            hg,
            hb,
            cov,
            ((*self.hand_z.at(i)) + (hnz * PantherPainter::PAW_R)),
        );
        // Paw pads: one main + toes when the paw is held up.
        let mut px0: f64 = (u - (*self.hand_u.at(i)));
        let mut py0: f64 = (v - ((*self.hand_v.at(i)) - 0.15));
        self.sort(
            (PantherPainter::FUR_DARK_R * hk),
            (PantherPainter::FUR_DARK_G * hk),
            (PantherPainter::FUR_DARK_B * hk),
            ((self.cov((((px0 * px0) + (py0 * py0)).sqrt() - 0.16)) * 0.5) * cov),
            ((*self.hand_z.at(i)) + PantherPainter::PAW_R),
        );
    }
    pub fn tail(mut self, mut u: f64, mut v: f64) {
        // One flowing ribbon, shaded once: distance to the whole sampled
        // centreline, then a single lighting pass - so neither straight
        // joints nor caterpillar bands can show.
        if (((u < (self.tail_lo - 1.2)) || (u > (self.tail_hi + 1.2)))
            || (v < (self.tail_vlo - 1.2)))
            || (v > (self.tail_vhi + 1.2))
        {
            return;
        }
        // Squared distances to each link (the spans worked out per
        // frame), one square root for the nearest.
        let mut best: f64 = f64::MAX;
        let mut qx: f64 = 0.0;
        let mut qy: f64 = 0.0;
        let mut qt: f64 = 0.0;
        let mut hit: i32 = 0;
        {
            let mut i: i32 = 0;
            'l1: while i < PantherPainter::TAIL_SAMPLES {
                let mut ax: f64 = (*self.tail_u.at(i));
                let mut ay: f64 = (*self.tail_v.at(i));
                let mut abx: f64 = (*self.tail_du.at(i));
                let mut aby: f64 = (*self.tail_dv.at(i));
                let mut len2: f64 = (*self.tail_len2.at(i));
                let mut st: f64 = (if (len2 > 0.0) {
                    ((((u - ax) * abx) + ((v - ay) * aby)) / len2)
                } else {
                    0.0
                });
                if st < 0.0 {
                    st = 0.0;
                } else if st > 1.0 {
                    st = 1.0;
                }
                let mut sx: f64 = (ax + (abx * st));
                let mut sy: f64 = (ay + (aby * st));
                let mut dx: f64 = (u - sx);
                let mut dy: f64 = (v - sy);
                let mut d2: f64 = ((dx * dx) + (dy * dy));
                if d2 < best {
                    best = d2;
                    qx = sx;
                    qy = sy;
                    qt = st;
                    hit = i;
                }
                i += 1;
            }
        }
        best = (best).sqrt();
        let mut g: f64 = ((((hit) as f64) + qt) / ((PantherPainter::TAIL_SAMPLES) as f64));
        let mut radius: f64 = (0.42 - (0.22 * g));
        let mut cov: f64 = self.cov((best - radius));
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        PantherPainter::normal(
            (u - qx),
            (v - qy),
            radius,
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        let mut k: f64 = (0.96 + (0.06 * g));
        let mut lr: f64 = <f64>::default();
        let mut lg: f64 = <f64>::default();
        let mut lb: f64 = <f64>::default();
        self.lit(
            (PantherPainter::FUR_R * k),
            (PantherPainter::FUR_G * k),
            (PantherPainter::FUR_B * k),
            nx,
            ny,
            nz,
            (0.28 + (0.04 * g)),
            (16.0 + (4.0 * g)),
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.over(lr, lg, lb, cov);
        // Tuft highlight on the curl.
        {
            let mut dx: f64 = (u - (*self.tail_u.at(PantherPainter::TAIL_SAMPLES)));
            let mut dy: f64 = (v - (*self.tail_v.at(PantherPainter::TAIL_SAMPLES)));
            self.over(
                1.0,
                0.85,
                0.9,
                (self.cov((((dx * dx) + (dy * dy)).sqrt() - 0.22)) * 0.4),
            );
        }
    }
    pub fn ball(
        mut self,
        mut u: f64,
        mut v: f64,
        mut cx: f64,
        mut cy: f64,
        mut radius: f64,
        mut r: f64,
        mut g: f64,
        mut b: f64,
        mut spec: f64,
        mut shine: f64,
    ) {
        let mut dx: f64 = (u - cx);
        let mut dy: f64 = (v - cy);
        let mut cov: f64 = self.cov((((dx * dx) + (dy * dy)).sqrt() - radius));
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        PantherPainter::normal(
            dx,
            dy,
            radius,
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        let mut lr: f64 = <f64>::default();
        let mut lg: f64 = <f64>::default();
        let mut lb: f64 = <f64>::default();
        self.lit(
            r,
            g,
            b,
            nx,
            ny,
            nz,
            spec,
            shine,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.over(lr, lg, lb, cov);
    }
    //
    // Sets the painter up for one frame. Each painter is kept per thread
    // and set up afresh every frame, so a frame allocates nothing.
    //
    pub fn setup(mut self, mut face_left: bool, mut px: f64, mut depth_radius: f64) {
        self.front_count = 0;
        self.flip = ((if face_left { -1 } else { 1 }) as f64);
        self.px = px;
        self.depth_radius = depth_radius;
        self.depth_blend = (2.5 * px);
    }
    //
    // Shades [left, right) x [top, bottom) of the surface's character
    // layer, the figure's soles on soleY and centred on centreX at sx, sy
    // pixels per unit, and marks as touched only the part that got
    // anything, so the halo and the upload skip the empty margins.
    //
    pub fn raster(
        mut self,
        mut surface: Surface,
        mut left: i32,
        mut top: i32,
        mut right: i32,
        mut bottom: i32,
        mut centre_x: f64,
        mut sole_y: f64,
        mut sx: f64,
        mut sy: f64,
    ) {
        // u depends on the column only: work it out once per column, not per pixel.
        if self.column_u.len() < right {
            self.column_u = Arr::new((right + 64));
        }
        {
            let mut x: i32 = left;
            'l1: while x < right {
                (*self.column_u.at(x)) = ((((((x) as f64) + 0.5) - centre_x) / sx) * self.flip);
                x += 1;
            }
        }
        let mut layer: Arr<i32> = surface.layer();
        let mut width: i32 = surface.width();
        let mut min_x: i32 = right;
        let mut max_x: i32 = (left - 1);
        let mut min_y: i32 = bottom;
        let mut max_y: i32 = (top - 1);
        {
            let mut y: i32 = top;
            'l2: while y < bottom {
                let mut v: f64 = ((sole_y - (((y) as f64) + 0.5)) / sy);
                let mut row: i32 = (y * width);
                {
                    let mut x: i32 = left;
                    'l3: while x < right {
                        'c4: {
                            let mut argb: i32 = self.shade((*self.column_u.at(x)), v);
                            if argb == 0 {
                                break 'c4;
                            }
                            (*layer.at((row + x))) = argb;
                            if x < min_x {
                                min_x = x;
                            }
                            if x > max_x {
                                max_x = x;
                            }
                            if y < min_y {
                                min_y = y;
                            }
                            max_y = y;
                        }
                        x += 1;
                    }
                }
                y += 1;
            }
        }
        surface.touch(min_x, min_y, (max_x + 1), (max_y + 1));
    }
    // ---- The pixel ---------------------------------------------------------------
    // Starts a pixel: nothing painted yet, no limb pieces held back.
    pub fn begin(mut self) {
        self.cb = 0.0;
        self.cg = self.cb;
        self.cr = self.cg;
        self.ca = self.cr;
        self.front_count = 0;
    }
    // Lays on the limb pieces that were in front of the body.
    pub fn lay_front(mut self) {
        {
            let mut i: i32 = 0;
            'l1: while i < self.front_count {
                self.over(
                    (*self.front_r.at(i)),
                    (*self.front_g.at(i)),
                    (*self.front_b.at(i)),
                    (*self.front_cov.at(i)),
                );
                i += 1;
            }
        }
    }
    // The finished pixel as premultiplied ARGB, 0 if nothing was painted.
    pub fn finish(mut self) -> i32 {
        if self.ca <= 0.003 {
            return 0;
        }
        if self.ca > 1.0 {
            self.ca = 1.0;
        }
        let mut a: i32 = (((self.ca * 255.0) + 0.5) as i32);
        let mut r: i32 = (((PantherPainter::clamp(self.cr) * 255.0) + 0.5) as i32);
        let mut g: i32 = (((PantherPainter::clamp(self.cg) * 255.0) + 0.5) as i32);
        let mut b: i32 = (((PantherPainter::clamp(self.cb) * 255.0) + 0.5) as i32);
        return ((((a << (24)) | (r << (16))) | (g << (8))) | b);
    }
    // ---- Compositing and lighting ---------------------------------------------
    pub fn cov(mut self, mut d: f64) -> f64 {
        let mut c: f64 = (0.5 - (d / self.px));
        return (if (c < 0.0) {
            0.0
        } else {
            (if (c > 1.0) { 1.0 } else { c })
        });
    }
    pub fn clamp(mut x: f64) -> f64 {
        return (if (x < 0.0) {
            0.0
        } else {
            (if (x > 1.0) { 1.0 } else { x })
        });
    }
    pub fn over(mut self, mut r: f64, mut g: f64, mut b: f64, mut cov: f64) {
        if cov <= 0.0 {
            return;
        }
        let mut keep: f64 = (1.0 - cov);
        self.cr = ((r * cov) + (self.cr * keep));
        self.cg = ((g * cov) + (self.cg * keep));
        self.cb = ((b * cov) + (self.cb * keep));
        self.ca = (cov + (self.ca * keep));
    }
    // Lights an albedo with a normal (nx right, ny up, nz towards the viewer).
    pub fn lit(
        mut self,
        mut r: f64,
        mut g: f64,
        mut b: f64,
        mut nx: f64,
        mut ny: f64,
        mut nz: f64,
        mut spec: f64,
        mut shine: f64,
        lr: &mut f64,
        lg: &mut f64,
        lb: &mut f64,
    ) {
        nx *= self.flip;
        let mut diffuse: f64 =
            (((nx * SolidPainter::LX) + (ny * SolidPainter::LY)) + (nz * SolidPainter::LZ));
        if diffuse < 0.0 {
            diffuse = 0.0;
        }
        let mut k: f64 = ((0.40 + (0.62 * diffuse))
            * (0.6 + (0.4 * (if (nz < 0.0) { 0.0 } else { nz }).sqrt())));
        let mut h: f64 =
            (((nx * SolidPainter::HX) + (ny * SolidPainter::HY)) + (nz * SolidPainter::HZ));
        let mut s: f64 = (if (h > 0.0) {
            (spec * PantherPainter::pow(h, shine))
        } else {
            0.0
        });
        (*lr) = ((r * k) + s);
        (*lg) = ((g * k) + s);
        (*lb) = ((b * k) + s);
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
        PantherPainter::turn_2(&mut *nx, &mut *nz, (a).cos(), (a).sin());
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
    // ---- Solids -------------------------------------------------------------------
    // A cylinder between two points, lit as a tube.
    pub fn tube(
        mut self,
        mut x: f64,
        mut y: f64,
        mut ax: f64,
        mut ay: f64,
        mut bx: f64,
        mut by: f64,
        mut radius: f64,
        mut r: f64,
        mut g: f64,
        mut b: f64,
        mut spec: f64,
        mut shine: f64,
    ) {
        let mut qx: f64 = <f64>::default();
        let mut qy: f64 = <f64>::default();
        let mut t: f64 = <f64>::default();
        let mut dist: f64 = PantherPainter::seg(
            x,
            y,
            ax,
            ay,
            bx,
            by,
            rt::mref(&mut qx),
            rt::mref(&mut qy),
            rt::mref(&mut t),
        );
        let mut cov: f64 = self.cov((dist - radius));
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        PantherPainter::normal(
            (x - qx),
            (y - qy),
            radius,
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        let mut lr: f64 = <f64>::default();
        let mut lg: f64 = <f64>::default();
        let mut lb: f64 = <f64>::default();
        self.lit(
            r,
            g,
            b,
            nx,
            ny,
            nz,
            spec,
            shine,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.over(lr, lg, lb, cov);
    }
    //
    // A limb segment: a tube whose ends have a depth, sorted against the
    // body pixel by pixel, and a shade darker the further back it is.
    //
    pub fn arm_tube(
        mut self,
        mut x: f64,
        mut y: f64,
        mut ax: f64,
        mut ay: f64,
        mut az: f64,
        mut bx: f64,
        mut by: f64,
        mut bz: f64,
        mut radius: f64,
        mut r: f64,
        mut g: f64,
        mut b: f64,
        mut spec: f64,
        mut shine: f64,
    ) {
        let mut qx: f64 = <f64>::default();
        let mut qy: f64 = <f64>::default();
        let mut t: f64 = <f64>::default();
        let mut dist: f64 = PantherPainter::seg(
            x,
            y,
            ax,
            ay,
            bx,
            by,
            rt::mref(&mut qx),
            rt::mref(&mut qy),
            rt::mref(&mut t),
        );
        let mut cov: f64 = self.cov((dist - radius));
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        PantherPainter::normal(
            (x - qx),
            (y - qy),
            radius,
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        let mut z: f64 = (az + ((bz - az) * t));
        let mut k: f64 = self.depth(z);
        let mut lr: f64 = <f64>::default();
        let mut lg: f64 = <f64>::default();
        let mut lb: f64 = <f64>::default();
        self.lit(
            (r * k),
            (g * k),
            (b * k),
            nx,
            ny,
            nz,
            spec,
            shine,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.sort(lr, lg, lb, cov, (z + (nz * radius)));
    }
    // How much darker something gets with distance from the viewer.
    pub fn depth(mut self, mut z: f64) -> f64 {
        let mut k: f64 = (0.89 + ((0.11 * z) / self.depth_radius));
        return (if (k < 0.78) {
            0.78
        } else {
            (if (k > 1.0) { 1.0 } else { k })
        });
    }
    //
    // Composites a limb piece at depth z against the body: the part
    // behind the body's surface now, the part in front later. Across
    // the surface the two blend over a band a couple of pixels deep,
    // which reads as the arm sinking into the body.
    //
    pub fn sort(mut self, mut r: f64, mut g: f64, mut b: f64, mut cov: f64, mut z: f64) {
        let mut f: f64 = (0.5 + ((z - self.body_z) / self.depth_blend));
        if f <= 0.0 {
            self.over(r, g, b, cov);
            return;
        }
        if f < 1.0 {
            self.over(r, g, b, (cov * (1.0 - f)));
        } else {
            f = 1.0;
        }
        let mut n: i32 = {
            let t__ = self.front_count;
            self.front_count += 1;
            t__
        };
        (*self.front_r.at(n)) = r;
        (*self.front_g.at(n)) = g;
        (*self.front_b.at(n)) = b;
        (*self.front_cov.at(n)) = (cov * f);
    }
    // ---- Things floating about the figure ---------------------------------------------
    //
    // A drop of sweat beside the head at x, sliding down from y0 and
    // fading as `slid` goes 0..1.
    //
    pub fn sweat(mut self, mut u: f64, mut v: f64, mut slid: f64, mut x: f64, mut y0: f64) {
        let mut s: f64 = slid;
        let mut y: f64 = (y0 - (2.6 * s));
        if ((u - x).abs() > 1.0) || (((v - y) - 0.3).abs() > 1.3) {
            return;
        }
        let mut qx: f64 = <f64>::default();
        let mut qy: f64 = <f64>::default();
        let mut t: f64 = <f64>::default();
        let mut dx: f64 = (u - x);
        let mut dy: f64 = (v - y);
        let mut d: f64 = rt::min_f64(
            (((dx * dx) + (dy * dy)).sqrt() - 0.42),
            (PantherPainter::seg(
                u,
                v,
                x,
                (y + 1.0),
                x,
                y,
                rt::mref(&mut qx),
                rt::mref(&mut qy),
                rt::mref(&mut t),
            ) - (0.04 + (0.36 * t))),
        );
        let mut a: f64 = (if (s < 0.75) { 1.0 } else { ((1.0 - s) / 0.25) });
        self.over(0.10, 0.22, 0.42, ((self.cov(((d).abs() - 0.07)) * 0.5) * a));
        self.over(0.55, 0.78, 0.98, (self.cov(d) * a));
        let mut hx: f64 = (dx + 0.15);
        let mut hy: f64 = (dy - 0.12);
        self.over(
            1.0,
            1.0,
            1.0,
            ((self.cov((((hx * hx) + (hy * hy)).sqrt() - 0.12)) * 0.9) * a),
        );
    }
    // A music note drifting up and forward from the mouth at (mouthU, mouthV).
    pub fn note(mut self, mut u: f64, mut v: f64, mut ph: f64, mut mouth_u: f64, mut mouth_v: f64) {
        let mut x: f64 = ((mouth_u + 2.2) + (1.2 * ph));
        let mut y: f64 = ((mouth_v + 1.0) + (5.0 * ph));
        if (((u - x) - 0.3).abs() > 1.4) || (((v - y) - 0.8).abs() > 1.6) {
            return;
        }
        let mut s: f64 = 0.85;
        let mut dx: f64 = ((u - x) / (0.5 * s));
        let mut dy: f64 = ((v - y) / (0.38 * s));
        let mut d: f64 = (((((dx * dx) + (dy * dy)).sqrt() - 1.0) * 0.38) * s); // the head
        let mut qx: f64 = <f64>::default();
        let mut qy: f64 = <f64>::default();
        let mut t: f64 = <f64>::default();
        let mut sx: f64 = (x + (0.42 * s));
        d = rt::min_f64(
            d,
            (PantherPainter::seg(
                u,
                v,
                sx,
                y,
                sx,
                (y + (1.7 * s)),
                rt::mref(&mut qx),
                rt::mref(&mut qy),
                rt::mref(&mut t),
            ) - (0.1 * s)),
        ); // the stem
        d = rt::min_f64(
            d,
            (PantherPainter::seg(
                u,
                v,
                sx,
                (y + (1.7 * s)),
                (sx + (0.6 * s)),
                (y + (1.15 * s)),
                rt::mref(&mut qx),
                rt::mref(&mut qy),
                rt::mref(&mut t),
            ) - (0.12 * s)),
        ); // the flag
        let mut a: f64 = (if (ph < 0.75) {
            1.0
        } else {
            ((1.0 - ph) / 0.25)
        });
        self.over(0.98, 0.98, 1.0, ((self.cov((d - 0.22)) * 0.85) * a));
        self.over(0.12, 0.10, 0.18, (self.cov(d) * a));
    }
    //
    // Two "z"s floating up and away from (x0, y0) above the head, growing
    // as they go; the second is half a drift behind the first.
    //
    pub fn zzz(mut self, mut u: f64, mut v: f64, mut ph: f64, mut x0: f64, mut y0: f64) {
        if ((v < (y0 - 2.3)) || (u < (x0 - 2.4))) || (u > (x0 + 6.1)) {
            return;
        }
        self.z_glyph(u, v, ph, x0, y0);
        self.z_glyph(
            u,
            v,
            (if (ph >= 0.5) { (ph - 0.5) } else { (ph + 0.5) }),
            x0,
            y0,
        );
    }
    pub fn z_glyph(mut self, mut u: f64, mut v: f64, mut ph: f64, mut x0: f64, mut y0: f64) {
        let mut s: f64 = (1.0 + (0.6 * ph));
        let mut x: f64 = (x0 + (2.4 * ph));
        let mut y: f64 = (y0 + (3.2 * ph));
        let mut h: f64 = (s * 0.5);
        if ((u - x).abs() > (h + 0.6)) || ((v - y).abs() > (h + 0.6)) {
            return;
        }
        let mut qx: f64 = <f64>::default();
        let mut qy: f64 = <f64>::default();
        let mut t: f64 = <f64>::default();
        let mut d: f64 = PantherPainter::seg(
            u,
            v,
            (x - h),
            (y + h),
            (x + h),
            (y + h),
            rt::mref(&mut qx),
            rt::mref(&mut qy),
            rt::mref(&mut t),
        );
        d = rt::min_f64(
            d,
            PantherPainter::seg(
                u,
                v,
                (x + h),
                (y + h),
                (x - h),
                (y - h),
                rt::mref(&mut qx),
                rt::mref(&mut qy),
                rt::mref(&mut t),
            ),
        );
        d = rt::min_f64(
            d,
            PantherPainter::seg(
                u,
                v,
                (x - h),
                (y - h),
                (x + h),
                (y - h),
                rt::mref(&mut qx),
                rt::mref(&mut qy),
                rt::mref(&mut t),
            ),
        );
        let mut r: f64 = (0.08 + (0.1 * s));
        let mut a: f64 = (if (ph < 0.8) { 1.0 } else { ((1.0 - ph) / 0.2) });
        self.over(0.12, 0.10, 0.14, ((self.cov(((d - r) - 0.25)) * 0.8) * a));
        self.over(0.95, 0.96, 1.0, (self.cov((d - r)) * a));
    }
    // The "!" overhead: a dot at dotV under a bar from barBottom to barTop; nothing below floorV.
    pub fn bubble(
        mut self,
        mut u: f64,
        mut v: f64,
        mut floor_v: f64,
        mut dot_v: f64,
        mut bar_bottom: f64,
        mut bar_top: f64,
    ) {
        if (v < floor_v) || ((u).abs() > 1.5) {
            return;
        }
        let mut qx: f64 = <f64>::default();
        let mut qy: f64 = <f64>::default();
        let mut t: f64 = <f64>::default();
        let mut bar: f64 = (PantherPainter::seg(
            u,
            v,
            0.0,
            bar_bottom,
            0.0,
            bar_top,
            rt::mref(&mut qx),
            rt::mref(&mut qy),
            rt::mref(&mut t),
        ) - (0.78 - (0.18 * (1.0 - t))));
        let mut dot: f64 = (((u * u) + ((v - dot_v) * (v - dot_v))).sqrt() - 0.75);
        let mut d: f64 = rt::min_f64(bar, dot);
        self.over(0.12, 0.10, 0.14, (self.cov((d - 0.3)) * 0.85));
        self.over(0.98, 0.98, 0.98, self.cov(d));
    }
    //
    // A small pink heart drifting up and forward from the mouth at
    // (mouthU, mouthV), like the note: two round lobes over a point.
    //
    pub fn heart(
        mut self,
        mut u: f64,
        mut v: f64,
        mut ph: f64,
        mut mouth_u: f64,
        mut mouth_v: f64,
    ) {
        let mut x: f64 = ((mouth_u + 2.4) + (1.6 * ph));
        let mut y: f64 = ((mouth_v + 1.2) + (5.0 * ph));
        if ((u - x).abs() > 1.4) || ((v - y).abs() > 1.4) {
            return;
        }
        let mut s: f64 = 0.8;
        let mut hx: f64 = ((u - x).abs() / s);
        let mut hy: f64 = ((v - y) / s);
        let mut lx: f64 = (hx - 0.45);
        let mut ly: f64 = (hy - 0.3);
        let mut lobe: f64 = (((lx * lx) + (ly * ly)).sqrt() - 0.5);
        // The point: inside the triangle between the lobes' centres and the tip.
        let mut edge: f64 = (((1.2 * hx) - (0.95 * (hy + 0.85))) / 1.53);
        let mut tri: f64 = rt::max_f64(edge, (hy - 0.3));
        let mut d: f64 = (rt::min_f64(lobe, tri) * s);
        let mut a: f64 = (if (ph < 0.75) {
            1.0
        } else {
            ((1.0 - ph) / 0.25)
        });
        self.over(0.14, 0.08, 0.12, ((self.cov((d - 0.22)) * 0.85) * a));
        self.over(0.96, 0.28, 0.42, (self.cov(d) * a));
        let mut gx: f64 = (((u - x) / s) + 0.35);
        let mut gy: f64 = (hy - 0.5);
        self.over(
            1.0,
            0.85,
            0.9,
            ((self.cov((((gx * gx) + (gy * gy)).sqrt() - 0.14)) * 0.8) * a),
        );
    }
}
