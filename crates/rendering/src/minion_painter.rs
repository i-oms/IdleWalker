// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

//
// Paints a Minion into a `Surface`. There is no sprite
// sheet: the figure is a handful of solids - a capsule for the body,
// cylinders for the limbs, a sphere behind glass for each eye, a torus
// for each goggle rim. Every pixel gets a signed distance to each solid
// (for an anti-aliased edge) and a surface normal (for lighting), and
// the solids are composited back to front (the machinery is in
// `SolidPainter`). Because poses and faces are numbers
// rather than pictures, everything blends smoothly.
//
// Model space: u runs across in "units", 0 at the centre, positive
// towards where the Minion faces; v runs up from the soles. The figure
// is `Height` units tall, the same as the old pixel art, so
// one unit is one old art cell on screen.
//
// The figure can turn about its vertical axis (`Pose.Yaw`).
// The body is a capsule so its outline does not change, but everything
// attached to it - goggles, mouth, overalls, hair, limbs - sits at an
// angle round the axis and is swung round, foreshortened and hidden
// behind the body as it turns. The arms carry a depth as well as a
// position, and every pixel of them is sorted against the body's
// surface, so a hand slides behind the body or comes out in front of
// it continuously instead of the whole arm flipping over at once.
//
#[derive(Clone, Default)]
pub struct MinionPainterData {
    pub pose: Pose,
    pub look: Look,
    pub face: Expression,
    pub dress_r: f64,
    pub dress_g: f64,
    pub dress_b: f64,
    pub strap_r: f64,
    pub strap_g: f64,
    pub strap_b: f64,
    pub yaw: f64,
    pub sin_y: f64,
    pub cos_y: f64,
    pub lean: f64,
    // Eyes: angle from the viewer, screen centre, horizontal squeeze, visibility.
    pub eyes: i32,
    pub lens_r: f64,
    pub ring_r: f64,
    pub tube_r: f64,
    pub eye_alpha: Arr<f64>,
    pub eye_u: Arr<f64>,
    pub eye_k: Arr<f64>,
    pub eye_vis: Arr<f64>,
    pub eye_cos: Arr<f64>,
    pub eye_sin: Arr<f64>,
    pub gaze_u: Arr<f64>,
    pub gaze_v: Arr<f64>,
    pub brow_alpha: Arr<f64>,
    pub brow_cos: Arr<f64>,
    pub brow_sin: Arr<f64>,
    pub first_eye: i32,
    // Legs: hip and ankle of each, and which is behind.
    pub hip_u: Arr<f64>,
    pub ankle_u: Arr<f64>,
    pub ankle_v: Arr<f64>,
    pub leg_z: Arr<f64>,
    pub shoe_len: f64,
    pub toe: f64,
    // Arms: shoulder, elbow and hand of each, with the depth of each
    // joint (positive towards the viewer) and the screen box it all fits in.
    pub sh_u: Arr<f64>,
    pub sh_z: Arr<f64>,
    pub elb_u: Arr<f64>,
    pub elb_v: Arr<f64>,
    pub elb_z: Arr<f64>,
    pub hand_u: Arr<f64>,
    pub hand_v: Arr<f64>,
    pub hand_z: Arr<f64>,
    pub arm_lo: Arr<f64>,
    pub arm_hi: Arr<f64>,
    pub arm_vlo: Arr<f64>,
    pub arm_vhi: Arr<f64>,
    pub hair: Arr<f64>,
    pub hair_length: i32,
    pub hair_lo: f64,
    pub hair_hi: f64,
    pub hair_vlo: f64,
    pub decorated: bool,
    pub top_v: f64,
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
rt::handle!(MinionPainter, MinionPainterData);

thread_local! { static MINIONPAINTER_CACHED: std::cell::Cell<MinionPainter> = std::cell::Cell::new(Default::default()); }

pub static MINIONPAINTER_SPROUTS_AT: rt::Global<Arr<f64>> =
    rt::Global::new(|| Arr::from_vec(vec![-1.1, 0.1, 1.2]));

pub static MINIONPAINTER_SPROUTS_TILT: rt::Global<Arr<f64>> =
    rt::Global::new(|| Arr::from_vec(vec![-0.5, 0.1, 0.6]));

pub static MINIONPAINTER_COMBED_AT: rt::Global<Arr<f64>> =
    rt::Global::new(|| Arr::from_vec(vec![-2.6, -1.9, -1.2, -0.5, 0.2, 0.9]));

pub static MINIONPAINTER_PARTED_AT: rt::Global<Arr<f64>> =
    rt::Global::new(|| Arr::from_vec(vec![-0.4, -1.1, -1.8, 0.5, 1.2, 1.9]));

pub static MINIONPAINTER_SPIKY_AT: rt::Global<Arr<f64>> =
    rt::Global::new(|| Arr::from_vec(vec![-2.4, -1.3, -0.2, 0.9, 2.0]));

pub static MINIONPAINTER_QUIFF_AT: rt::Global<Arr<f64>> =
    rt::Global::new(|| Arr::from_vec(vec![1.8, 1.1, 0.4, -0.3, -1.0]));

pub static MINIONPAINTER_WILD_AT: rt::Global<Arr<f64>> =
    rt::Global::new(|| Arr::from_vec(vec![-2.6, -1.6, -0.7, 0.3, 1.2, 2.3]));

pub static MINIONPAINTER_WILD_TILT: rt::Global<Arr<f64>> =
    rt::Global::new(|| Arr::from_vec(vec![-1.1, -0.4, 0.5, -0.6, 0.9, 1.2]));

pub static MINIONPAINTER_WILD_LEN: rt::Global<Arr<f64>> =
    rt::Global::new(|| Arr::from_vec(vec![1.5, 2.1, 1.7, 2.3, 1.6, 1.9]));

// ---- Distance fields ----------------------------------------------------------
// One strap as a polyline in (|arc|, v): the bib corner, the shoulder
// front, the top of the shoulder just above the arm, down the back,
// and the waistband at the back. Stored per segment as start, span
// and 1 / span² so the distance needs one square root in all.
pub static MINIONPAINTER_STRAP_PATH: rt::Global<Arr<f64>> = rt::Global::new(|| {
    MinionPainter::build_strap(Arr::from_vec(vec![
        2.2,
        7.4,
        4.4,
        9.6,
        7.35,
        MinionPainter::STRAP_TOP,
        12.4,
        8.4,
        13.9,
        MinionPainter::WAIST_V,
    ]))
});

impl MinionPainter {
    fn alloc__() -> MinionPainter {
        let mut this = MinionPainter::from_data(MinionPainterData::default());
        this.eye_alpha = Arr::new(2);
        this.eye_u = Arr::new(2);
        this.eye_k = Arr::new(2);
        this.eye_vis = Arr::new(2);
        this.eye_cos = Arr::new(2);
        this.eye_sin = Arr::new(2);
        this.gaze_u = Arr::new(2);
        this.gaze_v = Arr::new(2);
        this.brow_alpha = Arr::new(2);
        this.brow_cos = Arr::new(2);
        this.brow_sin = Arr::new(2);
        this.hip_u = Arr::new(2);
        this.ankle_u = Arr::new(2);
        this.ankle_v = Arr::new(2);
        this.leg_z = Arr::new(2);
        this.sh_u = Arr::new(2);
        this.sh_z = Arr::new(2);
        this.elb_u = Arr::new(2);
        this.elb_v = Arr::new(2);
        this.elb_z = Arr::new(2);
        this.hand_u = Arr::new(2);
        this.hand_v = Arr::new(2);
        this.hand_z = Arr::new(2);
        this.arm_lo = Arr::new(2);
        this.arm_hi = Arr::new(2);
        this.arm_vlo = Arr::new(2);
        this.arm_vhi = Arr::new(2);
        this.hair = Arr::new(MinionPainter::MAX_HAIR);
        this.front_r = Arr::new(12);
        this.front_g = Arr::new(12);
        this.front_b = Arr::new(12);
        this.front_cov = Arr::new(12);
        this.column_u = Arr::new(0);
        this
    }
    pub fn new() -> MinionPainter {
        Self::alloc__()
    }
    pub const HEIGHT: f64 = 18.0;
    // Body: a capsule of radius BodyR with its axis from BodyBottom to BodyTop.
    pub const BODY_R: f64 = 5.4;
    pub const BODY_BOTTOM: f64 = 7.6;
    pub const BODY_TOP: f64 = 12.6;
    pub const WAIST_V: f64 = 4.6;
    pub const STRAP_R: f64 = 0.42;
    pub const STRAP_TOP: f64 = 11.0;
    pub const BUTTON_ARC: f64 = 2.2;
    pub const BUTTON_V: f64 = 7.3;
    pub const BUTTON_R: f64 = 0.4;
    pub const BAND_BOTTOM: f64 = 11.85;
    pub const BAND_TOP: f64 = 13.25;
    pub const EYE_V: f64 = 12.55;
    pub const MOUTH_V: f64 = 9.0;
    pub const EYE_ANGLE: f64 = 26.0;
    pub const GOGGLE_OUT: f64 = 0.4;
    pub const HIP_V: f64 = 4.3;
    pub const HIP_X: f64 = 1.7;
    pub const LEG_LEN: f64 = 3.0;
    pub const LEG_R: f64 = 0.95;
    // Shoulders sit on the body surface, ShoulderAngle round from the
    // front on each side: just forward of the widest point, so a hanging
    // arm shows beside the body when the Minion faces you and swings
    // round in front of it as the body turns away.
    pub const SHOULDER_ANGLE: f64 = 78.0;
    pub const SHOULDER_OUT: f64 = 0.15;
    pub const SHOULDER_V: f64 = 10.2;
    pub const UPPER_ARM: f64 = 1.75;
    pub const FOREARM: f64 = 1.85;
    pub const REST_SPREAD: f64 = 7.0;
    pub const ARM_R: f64 = 0.72;
    pub const FORE_R: f64 = 0.66;
    pub const GLOVE_R: f64 = 0.95;
    pub const FINGER_LEN: f64 = 0.55;
    pub const FINGER_R: f64 = 0.3;
    pub const STRAP_SHADE: f64 = 0.78;
    // Where the shoulders sit, and how far forward an arm can come.
    pub const ARM_RADIUS: f64 = (MinionPainter::BODY_R + MinionPainter::SHOULDER_OUT);
    pub fn init(
        mut self,
        mut pose: Pose,
        mut look: Look,
        mut dress: i32,
        mut face_left: bool,
        mut px: f64,
    ) {
        self.setup(face_left, px, MinionPainter::ARM_RADIUS);
        self.pose = pose;
        self.look = look;
        self.dress_r = ((((dress >> (16)) & 0xff) as f64) / 255.0);
        self.dress_g = ((((dress >> (8)) & 0xff) as f64) / 255.0);
        self.dress_b = (((dress & 0xff) as f64) / 255.0);
        self.strap_r = (self.dress_r * MinionPainter::STRAP_SHADE);
        self.strap_g = (self.dress_g * MinionPainter::STRAP_SHADE);
        self.strap_b = (self.dress_b * MinionPainter::STRAP_SHADE);
        self.face = pose.face;
        self.lean = pose.lean;
        // A spin can carry the yaw past a full turn; bring it back to
        // (-180, 180] so the body's arc lengths stay in range.
        let mut turn: f64 = (pose.yaw % 360.0);
        if turn > 180.0 {
            turn -= 360.0;
        } else if turn <= -180.0 {
            turn += 360.0;
        }
        self.yaw = (turn * SolidPainter::DEG);
        self.sin_y = (self.yaw).sin();
        self.cos_y = (self.yaw).cos();
        // Eyes sit on the surface at +-EyeAngle from the front (one eye: dead ahead).
        if look.one_eye {
            self.eyes = 1;
            self.lens_r = 2.35;
            self.ring_r = 2.65;
            self.tube_r = 0.36;
            self.place_eye(0, 0.0);
            self.first_eye = 0;
        } else {
            self.eyes = 2;
            self.lens_r = 1.85;
            self.ring_r = 2.12;
            self.tube_r = 0.3;
            self.place_eye(0, (-(MinionPainter::EYE_ANGLE) * SolidPainter::DEG));
            self.place_eye(1, (MinionPainter::EYE_ANGLE * SolidPainter::DEG));
            self.first_eye = (if ((*self.eye_alpha.at(0)).cos() < (*self.eye_alpha.at(1)).cos()) {
                0
            } else {
                1
            });
        }
        // Legs: hips either side of the axis, swung round by the yaw. The
        // stride is along the direction of travel, i.e. across the screen.
        let mut p: f64 = ((pose.walk_phase * 2.0) * std::f64::consts::PI);
        let mut swing: f64 = (if pose.walking {
            (38.0 * (p).sin())
        } else {
            0.0
        });
        let mut lift_a: f64 = (if pose.walking {
            (0.65 * rt::max_f64(0.0, (p).cos()))
        } else {
            0.0
        });
        let mut lift_b: f64 = (if pose.walking {
            (0.65 * rt::max_f64(0.0, -((p).cos())))
        } else {
            0.0
        });
        self.place_leg(0, -(MinionPainter::HIP_X), -swing, lift_b);
        self.place_leg(1, MinionPainter::HIP_X, swing, lift_a);
        self.shoe_len = (1.2 + (0.75 * (self.sin_y).abs())); // seen from the side a shoe is long
        self.toe = (0.3 + (0.7 * self.sin_y)); // and its toe points the way it is going
                                               // Arms: shoulders on the sides of the body, swung round likewise.
                                               // A hanging arm swings fore and aft - the way the Minion is going,
                                               // which brings the hand towards and away from the viewer as the
                                               // body turns - opposite to the leg on the same side. An explicitly
                                               // raised arm (waving, flailing overhead, hands on hips, ...) stays
                                               // where the pose puts it.
        let mut arm_swing: f64 = (if pose.walking {
            (32.0 * (p).sin())
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
        self.place_arm(0, -1.0, pose.arm_far, pose.fore_far, arm_swing, fore_swing);
        self.place_arm(
            1,
            1.0,
            pose.arm_near,
            pose.fore_near,
            -arm_swing,
            fore_swing,
        );
        // Where each iris sits in its lens, following the gaze; and the
        // brows' angles round the body. Fixed for the frame.
        let mut iris_r: f64 = (self.lens_r * 0.47);
        let mut reach: f64 = ((self.lens_r - iris_r) - 0.12);
        {
            let mut i: i32 = 0;
            'l1: while i < self.eyes {
                (*self.eye_cos.at(i)) = (*self.eye_alpha.at(i)).cos();
                (*self.eye_sin.at(i)) = (*self.eye_alpha.at(i)).sin();
                let mut ox: f64 = (((pose.gaze_x * reach) * 0.9) + self.face.pupil_dx);
                let mut oy: f64 = (((pose.gaze_y * reach) * 0.7) + self.face.pupil_dy);
                if self.eyes == 2 {
                    ox -= ((((if (i == 1) { 1 } else { -1 }) as f64) * self.face.cross_eye) * 0.75);
                }
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
        (*self.brow_alpha.at(0)) = (if (self.eyes == 2) {
            (*self.eye_alpha.at(0))
        } else {
            ((*self.eye_alpha.at(0)) - (14.0 * SolidPainter::DEG))
        });
        (*self.brow_alpha.at(1)) = (if (self.eyes == 2) {
            (*self.eye_alpha.at(1))
        } else {
            ((*self.eye_alpha.at(0)) + (14.0 * SolidPainter::DEG))
        });
        {
            let mut i: i32 = 0;
            'l2: while i < 2 {
                (*self.brow_cos.at(i)) = (*self.brow_alpha.at(i)).cos();
                (*self.brow_sin.at(i)) = (*self.brow_alpha.at(i)).sin();
                i += 1;
            }
        }
        self.hair_length = self.hair_for(look.hair);
        self.decorated =
            (((((pose.sweat > 0.0) || pose.note) || pose.zzz) || pose.bubble) || pose.heart);
        // How high the figure reaches: the dome, or the hair above it;
        // only the floating decorations go higher. Rows above this are
        // never shaded, which is most of the saving on an empty canvas.
        // The strands' box (with a pixel to spare for the soft edge)
        // spares the pixels beside the hair the strand-by-strand test.
        let mut hair_top: f64 = 0.0;
        self.hair_lo = f64::MAX;
        self.hair_hi = f64::MIN;
        self.hair_vlo = f64::MAX;
        {
            let mut i: i32 = 0;
            'l3: while i < self.hair_length {
                hair_top = rt::max_f64(
                    hair_top,
                    rt::max_f64(
                        ((*self.hair.at((i + 1))) + (*self.hair.at((i + 4)))),
                        ((*self.hair.at((i + 3))) + (*self.hair.at((i + 5)))),
                    ),
                );
                let mut r: f64 =
                    (rt::max_f64((*self.hair.at((i + 4))), (*self.hair.at((i + 5)))) + px);
                self.hair_lo = rt::min_f64(
                    self.hair_lo,
                    (rt::min_f64((*self.hair.at(i)), (*self.hair.at((i + 2)))) - r),
                );
                self.hair_hi = rt::max_f64(
                    self.hair_hi,
                    (rt::max_f64((*self.hair.at(i)), (*self.hair.at((i + 2)))) + r),
                );
                self.hair_vlo = rt::min_f64(
                    self.hair_vlo,
                    (rt::min_f64((*self.hair.at((i + 1))), (*self.hair.at((i + 3)))) - r),
                );
                i += 6;
            }
        }
        self.top_v = (if self.decorated {
            26.0
        } else {
            ((rt::max_f64((MinionPainter::BODY_TOP + MinionPainter::BODY_R), hair_top) + 1.0)
                + (pose.body_dy).abs())
        });
    }
    // Where an eye at `phi` round the body ends up once the body has turned.
    pub fn place_eye(mut self, mut i: i32, mut phi: f64) {
        let mut a: f64 = (phi + self.yaw);
        (*self.eye_alpha.at(i)) = a;
        (*self.eye_u.at(i)) = ((MinionPainter::BODY_R + MinionPainter::GOGGLE_OUT) * (a).sin());
        let mut c: f64 = (a).cos();
        (*self.eye_k.at(i)) = (if (c > 0.22) { c } else { 0.22 }); // the goggle is a barrel, never quite edge-on
        let mut away: f64 = ((a).abs() / SolidPainter::DEG);
        (*self.eye_vis.at(i)) = (if (away <= 78.0) {
            1.0
        } else {
            (if (away >= 100.0) {
                0.0
            } else {
                ((100.0 - away) / 22.0)
            })
        });
    }
    pub fn place_leg(mut self, mut i: i32, mut x: f64, mut angle: f64, mut lift: f64) {
        (*self.hip_u.at(i)) = (x * self.cos_y);
        (*self.leg_z.at(i)) = (-x * self.sin_y);
        let mut a: f64 = (angle * SolidPainter::DEG);
        let mut u: f64 = ((*self.hip_u.at(i)) + (MinionPainter::LEG_LEN * (a).sin()));
        let mut v: f64 = ((MinionPainter::HIP_V - (MinionPainter::LEG_LEN * (a).cos())) + lift);
        // Knees up: the foot comes back and up under the body.
        u += ((((*self.hip_u.at(i)) - 0.9) - u) * self.pose.leg_tuck);
        v += (((MinionPainter::HIP_V - 1.5) - v) * self.pose.leg_tuck);
        (*self.ankle_u.at(i)) = u;
        (*self.ankle_v.at(i)) = v;
    }
    //
    // Lays out one arm. `side` is -1 or 1. `raise` and `bend` are the
    // pose's angles, measured in the picture plane through the shoulder
    // (0 hanging, positive out to that arm's own side, 180 straight up),
    // so a raised hand reads the same from any angle. `swing` and
    // `foreSwing` are the walk, measured fore and aft of the body: a
    // forward swing carries the hand the way the Minion faces, which is
    // partly towards the viewer when it is turned.
    //
    pub fn place_arm(
        mut self,
        mut i: i32,
        mut side: f64,
        mut raise: f64,
        mut bend: f64,
        mut swing: f64,
        mut fore_swing: f64,
    ) {
        let mut at: f64 = (((side * MinionPainter::SHOULDER_ANGLE) * SolidPainter::DEG) + self.yaw);
        (*self.sh_u.at(i)) = (MinionPainter::ARM_RADIUS * (at).sin());
        (*self.sh_z.at(i)) = (MinionPainter::ARM_RADIUS * (at).cos());
        // A hanging arm angles out a little so the hand clears the hip.
        if raise == 0.0 {
            raise = (side * MinionPainter::REST_SPREAD);
        }
        let mut a: f64 = (raise * SolidPainter::DEG);
        let mut b: f64 = (swing * SolidPainter::DEG);
        let mut sa: f64 = (a).sin();
        let mut ca: f64 = (a).cos();
        let mut sb: f64 = (b).sin();
        let mut cb: f64 = (b).cos();
        (*self.elb_u.at(i)) =
            ((*self.sh_u.at(i)) + (MinionPainter::UPPER_ARM * (sa + ((ca * sb) * self.sin_y))));
        (*self.elb_v.at(i)) = (MinionPainter::SHOULDER_V - ((MinionPainter::UPPER_ARM * ca) * cb));
        (*self.elb_z.at(i)) =
            ((*self.sh_z.at(i)) + (((MinionPainter::UPPER_ARM * ca) * sb) * self.cos_y));
        a = ((raise + bend) * SolidPainter::DEG);
        b = ((swing + fore_swing) * SolidPainter::DEG);
        sa = (a).sin();
        ca = (a).cos();
        sb = (b).sin();
        cb = (b).cos();
        (*self.hand_u.at(i)) =
            ((*self.elb_u.at(i)) + (MinionPainter::FOREARM * (sa + ((ca * sb) * self.sin_y))));
        (*self.hand_v.at(i)) = ((*self.elb_v.at(i)) - ((MinionPainter::FOREARM * ca) * cb));
        (*self.hand_z.at(i)) =
            ((*self.elb_z.at(i)) + (((MinionPainter::FOREARM * ca) * sb) * self.cos_y));
        // The box everything fits in, with room for a pointing finger.
        let mut pad: f64 = (if ((side > 0.0) && self.pose.point_near) {
            (1.2 + MinionPainter::FINGER_LEN)
        } else {
            1.2
        });
        (*self.arm_lo.at(i)) = (rt::min_f64(
            rt::min_f64((*self.sh_u.at(i)), (*self.elb_u.at(i))),
            (*self.hand_u.at(i)),
        ) - pad);
        (*self.arm_hi.at(i)) = (rt::max_f64(
            rt::max_f64((*self.sh_u.at(i)), (*self.elb_u.at(i))),
            (*self.hand_u.at(i)),
        ) + pad);
        (*self.arm_vlo.at(i)) = (rt::min_f64(
            rt::min_f64(MinionPainter::SHOULDER_V, (*self.elb_v.at(i))),
            (*self.hand_v.at(i)),
        ) - pad);
        (*self.arm_vhi.at(i)) = (rt::max_f64(
            rt::max_f64(MinionPainter::SHOULDER_V, (*self.elb_v.at(i))),
            (*self.hand_v.at(i)),
        ) + pad);
    }
    pub fn dome_top(mut x: f64) -> f64 {
        let mut d: f64 = ((MinionPainter::BODY_R * MinionPainter::BODY_R) - (x * x));
        return (MinionPainter::BODY_TOP + (if (d > 0.0) { (d).sqrt() } else { 0.0 }));
    }
    // Hair strands are laid out as if seen from the front, then each is
    // swung round the dome to where the yaw puts its root.
    // (The strand roots and tilts are in the static tables below the method.)
    pub fn hair_for(mut self, mut style: HairStyle) -> i32 {
        let mut s: Arr<f64> = self.hair.clone();
        let mut length: i32 = <i32>::default();
        'sw1: {
            match style {
                HairStyle::Sprouts => {
                    let mut at: Arr<f64> = (*MINIONPAINTER_SPROUTS_AT).clone();
                    let mut tilt: Arr<f64> = (*MINIONPAINTER_SPROUTS_TILT).clone();
                    length = (at.len() * 12);
                    {
                        let mut i: i32 = 0;
                        'l2: while i < at.len() {
                            let mut u0: f64 = (*at.at(i));
                            let mut v0: f64 = (MinionPainter::dome_top((*at.at(i))) - 0.15);
                            let mut u1: f64 = (u0 + ((*tilt.at(i)) * 0.6));
                            let mut v1: f64 = (v0 + 1.25);
                            let mut u2: f64 = (u1 + ((*tilt.at(i)) * 1.2));
                            let mut v2: f64 = (v1 + 1.1);
                            MinionPainter::strand(s.clone(), (i * 12), u0, v0, u1, v1, 0.26, 0.17);
                            MinionPainter::strand(
                                s.clone(),
                                ((i * 12) + 6),
                                u1,
                                v1,
                                u2,
                                v2,
                                0.17,
                                0.06,
                            );
                            i += 1;
                        }
                    }
                }
                HairStyle::Combed => {
                    let mut at: Arr<f64> = (*MINIONPAINTER_COMBED_AT).clone();
                    length = (at.len() * 12);
                    {
                        let mut i: i32 = 0;
                        'l3: while i < at.len() {
                            let mut u0: f64 = (*at.at(i));
                            let mut u1: f64 = ((*at.at(i)) + 1.4);
                            let mut u2: f64 = ((*at.at(i)) + 2.5);
                            MinionPainter::strand(
                                s.clone(),
                                (i * 12),
                                u0,
                                (MinionPainter::dome_top(u0) - 0.1),
                                u1,
                                (MinionPainter::dome_top(u1) + 0.28),
                                0.24,
                                0.15,
                            );
                            MinionPainter::strand(
                                s.clone(),
                                ((i * 12) + 6),
                                u1,
                                (MinionPainter::dome_top(u1) + 0.28),
                                u2,
                                (MinionPainter::dome_top(u2) + 0.22),
                                0.15,
                                0.05,
                            );
                            i += 1;
                        }
                    }
                }
                HairStyle::Parted => {
                    let mut at: Arr<f64> = (*MINIONPAINTER_PARTED_AT).clone();
                    length = (at.len() * 12);
                    {
                        let mut i: i32 = 0;
                        'l4: while i < at.len() {
                            let mut side: f64 = ((if ((*at.at(i)) < 0.0) { -1 } else { 1 }) as f64);
                            let mut u0: f64 = (*at.at(i));
                            let mut v0: f64 = (MinionPainter::dome_top((*at.at(i))) - 0.15);
                            let mut u1: f64 = (u0 + (side * 0.5));
                            let mut v1: f64 = (v0 + 0.95);
                            let mut u2: f64 = (u1 + (side * 0.95));
                            let mut v2: f64 = (v1 + 0.45);
                            MinionPainter::strand(s.clone(), (i * 12), u0, v0, u1, v1, 0.25, 0.16);
                            MinionPainter::strand(
                                s.clone(),
                                ((i * 12) + 6),
                                u1,
                                v1,
                                u2,
                                v2,
                                0.16,
                                0.05,
                            );
                            i += 1;
                        }
                    }
                }
                HairStyle::Crew => {
                    let mut n: i32 = 9;
                    length = (n * 6);
                    {
                        let mut i: i32 = 0;
                        'l5: while i < n {
                            let mut u0: f64 = (-3.0 + (0.75 * ((i) as f64)));
                            let mut nx: f64 = (u0 / MinionPainter::BODY_R);
                            let mut ny: f64 = (rt::max_f64(0.0, (1.0 - (nx * nx)))).sqrt();
                            MinionPainter::strand(
                                s.clone(),
                                (i * 6),
                                u0,
                                (MinionPainter::dome_top(u0) - 0.15),
                                (u0 + (nx * 1.0)),
                                ((MinionPainter::dome_top(u0) - 0.15) + (ny * 1.0)),
                                0.24,
                                0.09,
                            );
                            i += 1;
                        }
                    }
                }
                HairStyle::Spiky => {
                    // Short, stiff spikes radiating straight out of the dome.
                    let mut at: Arr<f64> = (*MINIONPAINTER_SPIKY_AT).clone();
                    length = (at.len() * 6);
                    {
                        let mut i: i32 = 0;
                        'l6: while i < at.len() {
                            let mut u0: f64 = (*at.at(i));
                            let mut nx: f64 = (u0 / MinionPainter::BODY_R);
                            let mut ny: f64 = (rt::max_f64(0.0, (1.0 - (nx * nx)))).sqrt();
                            MinionPainter::strand(
                                s.clone(),
                                (i * 6),
                                u0,
                                (MinionPainter::dome_top(u0) - 0.15),
                                (u0 + (nx * 1.9)),
                                ((MinionPainter::dome_top(u0) - 0.15) + (ny * 1.9)),
                                0.3,
                                0.04,
                            );
                            i += 1;
                        }
                    }
                }
                HairStyle::Quiff => {
                    // A tall wave rising from the front and curling back.
                    let mut at: Arr<f64> = (*MINIONPAINTER_QUIFF_AT).clone();
                    length = (at.len() * 12);
                    {
                        let mut i: i32 = 0;
                        'l7: while i < at.len() {
                            let mut u0: f64 = (*at.at(i));
                            let mut v0: f64 = (MinionPainter::dome_top((*at.at(i))) - 0.15);
                            let mut u1: f64 = (u0 + 0.35);
                            let mut v1: f64 = ((v0 + 1.6) - (0.12 * ((i) as f64)));
                            let mut u2: f64 = (u1 - 0.9);
                            let mut v2: f64 = ((v1 + 0.9) - (0.1 * ((i) as f64)));
                            MinionPainter::strand(s.clone(), (i * 12), u0, v0, u1, v1, 0.3, 0.22);
                            MinionPainter::strand(
                                s.clone(),
                                ((i * 12) + 6),
                                u1,
                                v1,
                                u2,
                                v2,
                                0.22,
                                0.06,
                            );
                            i += 1;
                        }
                    }
                }
                HairStyle::Tuft => {
                    // One long curl: up from the crown, over, and back down.
                    length = (3 * 6);
                    let mut t0: f64 = 0.1;
                    let mut tv0: f64 = (MinionPainter::dome_top(0.1) - 0.15);
                    MinionPainter::strand(
                        s.clone(),
                        0,
                        t0,
                        tv0,
                        (t0 + 0.3),
                        (tv0 + 1.8),
                        0.34,
                        0.26,
                    );
                    MinionPainter::strand(
                        s.clone(),
                        6,
                        (t0 + 0.3),
                        (tv0 + 1.8),
                        (t0 + 1.4),
                        (tv0 + 2.5),
                        0.26,
                        0.18,
                    );
                    MinionPainter::strand(
                        s.clone(),
                        12,
                        (t0 + 1.4),
                        (tv0 + 2.5),
                        (t0 + 2.2),
                        (tv0 + 1.7),
                        0.18,
                        0.05,
                    );
                }
                HairStyle::Wild => {
                    // Long strands sticking out every which way, as if it had a fright.
                    let mut at: Arr<f64> = (*MINIONPAINTER_WILD_AT).clone();
                    let mut tilt: Arr<f64> = (*MINIONPAINTER_WILD_TILT).clone();
                    let mut len: Arr<f64> = (*MINIONPAINTER_WILD_LEN).clone();
                    length = (at.len() * 12);
                    {
                        let mut i: i32 = 0;
                        'l8: while i < at.len() {
                            let mut u0: f64 = (*at.at(i));
                            let mut v0: f64 = (MinionPainter::dome_top((*at.at(i))) - 0.15);
                            let mut u1: f64 = (u0 + ((*tilt.at(i)) * 0.5));
                            let mut v1: f64 = (v0 + ((*len.at(i)) * 0.55));
                            let mut u2: f64 = (u1
                                + (((*tilt.at(i)) * 0.9)
                                    * (if ((i % 2) == 0) { -0.5 } else { 1.0 })));
                            let mut v2: f64 = (v1 + ((*len.at(i)) * 0.45));
                            MinionPainter::strand(s.clone(), (i * 12), u0, v0, u1, v1, 0.24, 0.15);
                            MinionPainter::strand(
                                s.clone(),
                                ((i * 12) + 6),
                                u1,
                                v1,
                                u2,
                                v2,
                                0.15,
                                0.04,
                            );
                            i += 1;
                        }
                    }
                }
                _ => {
                    return 0;
                }
            }
        }
        // Swing each strand round: its root is at some angle round the
        // dome's circle of latitude; move it to that angle plus the yaw.
        {
            let mut i: i32 = 0;
            'l9: while i < length {
                let mut root: f64 = (*s.at(i));
                let mut root_v: f64 = (*s.at((i + 1)));
                let mut dv: f64 = (root_v - MinionPainter::BODY_TOP);
                let mut ring: f64 = (rt::max_f64(
                    0.3,
                    ((MinionPainter::BODY_R * MinionPainter::BODY_R) - (dv * dv)),
                ))
                .sqrt();
                let mut sin_phi: f64 = (root / ring);
                if sin_phi > 1.0 {
                    sin_phi = 1.0;
                } else if sin_phi < -1.0 {
                    sin_phi = -1.0;
                }
                let mut shift: f64 = ((ring * ((sin_phi).asin() + self.yaw).sin()) - root);
                (*s.at(i)) += shift;
                (*s.at((i + 2))) += shift;
                i += 6;
            }
        }
        return length;
    }
    // The longest style (six two-piece strands) sets the buffer's size.
    pub const MAX_HAIR: i32 = (6 * 12);
    pub fn strand(
        mut s: Arr<f64>,
        mut i: i32,
        mut u0: f64,
        mut v0: f64,
        mut u1: f64,
        mut v1: f64,
        mut r0: f64,
        mut r1: f64,
    ) {
        (*s.at(i)) = u0;
        (*s.at((i + 1))) = v0;
        (*s.at((i + 2))) = u1;
        (*s.at((i + 3))) = v1;
        (*s.at((i + 4))) = r0;
        (*s.at((i + 5))) = r1;
    }
    // ---- Entry point ---------------------------------------------------------
    //
    // Paints the Minion with its soles on soleY, centred on centreX, at
    // `scale` pixels per unit (before squash and stretch), in overalls of
    // colour `dress` (0xRRGGBB).
    //
    pub fn draw(
        mut surface: Surface,
        mut pose: Pose,
        mut look: Look,
        mut dress: i32,
        mut face_left: bool,
        mut centre_x: f64,
        mut sole_y: f64,
        mut scale: f64,
    ) {
        let mut sx: f64 = (scale * pose.stretch_x);
        let mut sy: f64 = (scale * pose.stretch_y);
        let mut px: f64 = (1.0 / rt::min_f64(sx, sy));
        let mut painter: MinionPainter = MINIONPAINTER_CACHED.with(|c__| {
            if c__.get().is_null() {
                c__.set(MinionPainter::new());
            }
            c__.get()
        });
        painter.init(pose, look, dress, face_left, px);
        let mut left: i32 = rt::max_i32(0, (((centre_x - (10.5 * sx)).floor()) as i32));
        let mut right: i32 =
            rt::min_i32(surface.width(), (((centre_x + (10.5 * sx)).ceil()) as i32));
        let mut top: i32 = rt::max_i32(0, (((sole_y - (painter.top_v * sy)).floor()) as i32));
        let mut bottom: i32 = rt::min_i32(surface.height(), (((sole_y + 0.5).ceil()) as i32));
        if (left >= right) || (top >= bottom) {
            return;
        }
        painter.raster(surface, left, top, right, bottom, centre_x, sole_y, sx, sy);
    }
    pub fn shade(mut self, mut u: f64, mut v: f64) -> i32 {
        self.begin();
        // The upper body bobs and leans; the legs stay planted.
        let mut vb: f64 = (v + self.pose.body_dy);
        let mut ub: f64 = (u - ((self.lean * 0.11) * rt::max_f64(0.0, (v - 2.0))));
        // The body's distance field: how far this pixel is from the
        // capsule's axis, and how far forward its surface bulges here.
        // Everything that dips into the body is sorted against this.
        let mut axis_v: f64 = (if (vb < MinionPainter::BODY_BOTTOM) {
            MinionPainter::BODY_BOTTOM
        } else {
            (if (vb > MinionPainter::BODY_TOP) {
                MinionPainter::BODY_TOP
            } else {
                vb
            })
        });
        let mut bdy: f64 = (vb - axis_v);
        let mut body_dist: f64 = ((ub * ub) + (bdy * bdy)).sqrt();
        let mut bulge: f64 =
            (((MinionPainter::BODY_R * MinionPainter::BODY_R) - (ub * ub)) - (bdy * bdy));
        self.body_z = (if (bulge > 0.0) { (bulge).sqrt() } else { 0.0 });
        // Back to front: whatever the turn has put behind the body first.
        // Each arm is painted once; the parts of it that are in front of
        // the body are held back and laid on after the face.
        let mut back_leg: i32 = (if ((*self.leg_z.at(0)) < (*self.leg_z.at(1))) {
            0
        } else {
            1
        });
        self.leg(u, v, back_leg);
        self.leg(u, v, (1 - back_leg));
        self.limb(ub, vb, 0);
        self.limb(ub, vb, 1);
        self.body(ub, vb, bdy, body_dist);
        if self.eyes == 2 {
            self.eye(ub, vb, self.first_eye);
        }
        self.eye(
            ub,
            vb,
            (if (self.eyes == 2) {
                (1 - self.first_eye)
            } else {
                0
            }),
        );
        self.brows(ub, vb);
        self.mouth(ub, vb);
        self.hair(ub, vb);
        self.lay_front();
        // Things floating about the figure, in front of everything: the
        // sweat beside the head, the note and the heart from the mouth,
        // the Zzz and the "!" above the dome.
        if self.decorated {
            if self.pose.sweat > 0.0 {
                self.sweat(
                    ub,
                    vb,
                    self.pose.sweat,
                    -(MinionPainter::BODY_R + 1.1),
                    15.0,
                );
            }
            if self.pose.note {
                self.note(
                    ub,
                    vb,
                    self.pose.decor_phase,
                    ((MinionPainter::BODY_R * 0.93) * self.sin_y),
                    MinionPainter::MOUTH_V,
                );
            }
            if self.pose.zzz {
                self.zzz(u, v, self.pose.decor_phase, 3.4, 19.8);
            }
            if self.pose.bubble {
                self.bubble(ub, vb, 18.5, 19.3, 20.8, 23.4);
            }
            if self.pose.heart {
                self.heart(
                    ub,
                    vb,
                    self.pose.decor_phase,
                    ((MinionPainter::BODY_R * 0.93) * self.sin_y),
                    MinionPainter::MOUTH_V,
                );
            }
        }
        return self.finish();
    }
    pub fn build_strap(mut points: Arr<f64>) -> Arr<f64> {
        let mut segments: i32 = ((points.len() / 2) - 1);
        let mut s: Arr<f64> = Arr::new((segments * 5));
        {
            let mut i: i32 = 0;
            'l1: while i < segments {
                let mut ax: f64 = (*points.at((2 * i)));
                let mut ay: f64 = (*points.at(((2 * i) + 1)));
                let mut dx: f64 = ((*points.at(((2 * i) + 2))) - ax);
                let mut dy: f64 = ((*points.at(((2 * i) + 3))) - ay);
                (*s.at((5 * i))) = ax;
                (*s.at(((5 * i) + 1))) = ay;
                (*s.at(((5 * i) + 2))) = dx;
                (*s.at(((5 * i) + 3))) = dy;
                (*s.at(((5 * i) + 4))) = (1.0 / ((dx * dx) + (dy * dy)));
                i += 1;
            }
        }
        return s.clone();
    }
    // Distance from (x, y) to the strap's centre line.
    pub fn strap_dist(mut x: f64, mut y: f64) -> f64 {
        let mut s: Arr<f64> = (*MINIONPAINTER_STRAP_PATH).clone();
        let mut best: f64 = f64::MAX;
        {
            let mut i: i32 = 0;
            'l1: while i < s.len() {
                let mut rx: f64 = (x - (*s.at(i)));
                let mut ry: f64 = (y - (*s.at((i + 1))));
                let mut t: f64 =
                    (((rx * (*s.at((i + 2)))) + (ry * (*s.at((i + 3))))) * (*s.at((i + 4))));
                if t < 0.0 {
                    t = 0.0;
                } else if t > 1.0 {
                    t = 1.0;
                }
                let mut dx: f64 = (rx - ((*s.at((i + 2))) * t));
                let mut dy: f64 = (ry - ((*s.at((i + 3))) * t));
                let mut d2: f64 = ((dx * dx) + (dy * dy));
                if d2 < best {
                    best = d2;
                }
                i += 5;
            }
        }
        return (best).sqrt();
    }
    // ---- The parts, back to front ---------------------------------------------------
    pub fn leg(mut self, mut u: f64, mut v: f64, mut i: i32) {
        let mut k: f64 =
            (0.78 + ((0.22 * (1.0 + ((*self.leg_z.at(i)) / MinionPainter::HIP_X))) / 2.0));
        if v > (MinionPainter::HIP_V + 1.5) {
            return;
        }
        let mut au: f64 = (*self.ankle_u.at(i));
        let mut av: f64 = (*self.ankle_v.at(i));
        if (u < (rt::min_f64((*self.hip_u.at(i)), au) - 2.5))
            || (u > (rt::max_f64((*self.hip_u.at(i)), au) + 2.5))
        {
            return;
        }
        self.tube(
            u,
            v,
            (*self.hip_u.at(i)),
            MinionPainter::HIP_V,
            au,
            av,
            MinionPainter::LEG_R,
            (0.17 * k),
            (0.35 * k),
            (0.72 * k),
            0.06,
            8.0,
        );
        // The shoe: a glossy black ellipsoid under the ankle, toe forward.
        let mut sx: f64 = (au + self.toe);
        let mut sy: f64 = (av - 0.5);
        let mut rx: f64 = self.shoe_len;
        let mut ry: f64 = 0.72;
        let mut dx: f64 = ((u - sx) / rx);
        let mut dy: f64 = ((v - sy) / ry);
        let mut e: f64 = ((dx * dx) + (dy * dy)).sqrt();
        let mut cov: f64 = self.cov(((e - 1.0) * ry));
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        MinionPainter::normal(
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
            (0.10 * k),
            (0.09 * k),
            (0.12 * k),
            nx,
            ny,
            nz,
            0.6,
            22.0,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.over(lr, lg, lb, cov);
    }
    // The body at (u, v): dy is the height above the nearest point of the capsule's axis, dist the distance from it.
    pub fn body(mut self, mut u: f64, mut v: f64, mut dy: f64, mut dist: f64) {
        let mut cov: f64 = self.cov((dist - MinionPainter::BODY_R));
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        MinionPainter::normal(
            u,
            dy,
            MinionPainter::BODY_R,
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        // Yellow skin, with the overalls painted on below the goggle strap
        // and the strap itself above: the two never share a row.
        let mut r: f64 = 0.98;
        let mut g: f64 = 0.84;
        let mut b: f64 = 0.11;
        let mut spec: f64 = 0.35;
        let mut shine: f64 = 26.0;
        if v < (MinionPainter::BAND_BOTTOM - 0.2) {
            // Where on the surface this is, measured round the body from the
            // front of the Minion (not the front as seen), as an arc length.
            // Only the overalls need it.
            let mut phi: f64 = ((nx).atan2(nz) - self.yaw);
            if phi > std::f64::consts::PI {
                phi -= (2.0 * std::f64::consts::PI);
            } else if phi < -(std::f64::consts::PI) {
                phi += (2.0 * std::f64::consts::PI);
            }
            let mut arc: f64 = (MinionPainter::BODY_R * phi);
            // Trousers all round, and the bib on the front.
            let mut above: f64 = self.cov((MinionPainter::WAIST_V - v));
            let mut denim: f64 = (1.0 - above);
            if v < 7.9 {
                denim = rt::max_f64(
                    denim,
                    self.cov(MinionPainter::round_rect(arc, (v - 6.1), 2.5, 1.5, 0.45)),
                );
            }
            MinionPainter::mix(
                rt::mref(&mut r),
                rt::mref(&mut g),
                rt::mref(&mut b),
                rt::mref(&mut spec),
                rt::mref(&mut shine),
                self.dress_r,
                self.dress_g,
                self.dress_b,
                0.06,
                8.0,
                denim,
            );
            // A strap on each side, from the bib corner over the shoulder
            // and down the back to the waistband, where it is cut flush;
            // the bib pocket in the same darker shade; a button on each
            // corner of the bib.
            if v > (MinionPainter::WAIST_V - 0.3) {
                let mut a: f64 = (arc).abs();
                let mut trim: f64 = rt::min_f64(
                    self.cov((MinionPainter::strap_dist(a, v) - MinionPainter::STRAP_R)),
                    above,
                );
                if v < 7.9 {
                    trim = rt::max_f64(
                        trim,
                        self.cov(MinionPainter::round_rect(arc, (v - 6.0), 0.85, 0.6, 0.2)),
                    );
                    MinionPainter::mix(
                        rt::mref(&mut r),
                        rt::mref(&mut g),
                        rt::mref(&mut b),
                        rt::mref(&mut spec),
                        rt::mref(&mut shine),
                        self.strap_r,
                        self.strap_g,
                        self.strap_b,
                        0.06,
                        8.0,
                        trim,
                    );
                    let mut bx: f64 = (a - MinionPainter::BUTTON_ARC);
                    let mut by: f64 = (v - MinionPainter::BUTTON_V);
                    let mut button: f64 =
                        self.cov((((bx * bx) + (by * by)).sqrt() - MinionPainter::BUTTON_R));
                    MinionPainter::mix(
                        rt::mref(&mut r),
                        rt::mref(&mut g),
                        rt::mref(&mut b),
                        rt::mref(&mut spec),
                        rt::mref(&mut shine),
                        0.08,
                        0.08,
                        0.10,
                        0.5,
                        20.0,
                        button,
                    );
                } else {
                    MinionPainter::mix(
                        rt::mref(&mut r),
                        rt::mref(&mut g),
                        rt::mref(&mut b),
                        rt::mref(&mut spec),
                        rt::mref(&mut shine),
                        self.strap_r,
                        self.strap_g,
                        self.strap_b,
                        0.06,
                        8.0,
                        trim,
                    );
                }
            }
        } else {
            let mut band: f64 = self.cov(rt::max_f64(
                (MinionPainter::BAND_BOTTOM - v),
                (v - MinionPainter::BAND_TOP),
            ));
            MinionPainter::mix(
                rt::mref(&mut r),
                rt::mref(&mut g),
                rt::mref(&mut b),
                rt::mref(&mut spec),
                rt::mref(&mut shine),
                0.30,
                0.31,
                0.34,
                0.2,
                10.0,
                band,
            );
        }
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
    pub fn eye(mut self, mut u: f64, mut v: f64, mut i: i32) {
        let mut vis: f64 = (*self.eye_vis.at(i));
        if vis <= 0.0 {
            return;
        }
        let mut k: f64 = (*self.eye_k.at(i));
        let mut ac: f64 = (*self.eye_cos.at(i));
        let mut asn: f64 = (*self.eye_sin.at(i));
        let mut near: bool = (i == (self.eyes - 1));
        // Work in the eye's own frame: squeezed horizontally as it turns away.
        let mut dx: f64 = ((u - (*self.eye_u.at(i))) / k);
        let mut dy: f64 = (v - MinionPainter::EYE_V);
        let mut dist: f64 = ((dx * dx) + (dy * dy)).sqrt();
        if dist > ((self.ring_r + self.tube_r) + 0.6) {
            return;
        }
        let mut cov_lens: f64 = (self.cov(MinionPainter::squeezed(dx, dy, k, self.lens_r)) * vis);
        if cov_lens > 0.0 {
            // The eyeball: a white sphere bulging out through the lens.
            let mut nx: f64 = <f64>::default();
            let mut ny: f64 = <f64>::default();
            let mut nz: f64 = <f64>::default();
            MinionPainter::normal(
                dx,
                dy,
                (self.lens_r * 1.3),
                rt::mref(&mut nx),
                rt::mref(&mut ny),
                rt::mref(&mut nz),
            );
            MinionPainter::turn_2(rt::mref(&mut nx), rt::mref(&mut nz), ac, asn);
            let mut r: f64 = <f64>::default();
            let mut g: f64 = <f64>::default();
            let mut b: f64 = <f64>::default();
            self.lit(
                0.96,
                0.96,
                0.94,
                nx,
                ny,
                nz,
                0.5,
                30.0,
                rt::mref(&mut r),
                rt::mref(&mut g),
                rt::mref(&mut b),
            );
            // Where the iris sits: following the gaze, kept inside the lens.
            let mut iris_r: f64 = (self.lens_r * 0.47);
            let mut ix: f64 = (dx - (*self.gaze_u.at(i)));
            let mut iy: f64 = (dy - (*self.gaze_v.at(i)));
            let mut id: f64 = ((ix * ix) + (iy * iy)).sqrt();
            let mut cov_iris: f64 = self.cov(MinionPainter::squeezed(ix, iy, k, iris_r));
            if cov_iris > 0.0 {
                let mut rgb: i32 = (if near {
                    self.look.near_iris
                } else {
                    self.look.far_iris
                });
                let mut t: f64 = (id / iris_r);
                let mut shade: f64 = (1.25 - (0.55 * t));
                if t > 0.82 {
                    shade *= (1.0 - ((0.55 * (t - 0.82)) / 0.18));
                }
                let mut ir: f64 = (((((rgb >> (16)) & 255) as f64) / 255.0) * shade);
                let mut ig: f64 = (((((rgb >> (8)) & 255) as f64) / 255.0) * shade);
                let mut ib: f64 = ((((rgb & 255) as f64) / 255.0) * shade);
                r += ((ir - r) * cov_iris);
                g += ((ig - g) * cov_iris);
                b += ((ib - b) * cov_iris);
                let mut cov_pupil: f64 = self.cov(MinionPainter::squeezed(
                    ix,
                    iy,
                    k,
                    ((iris_r * 0.5) * self.face.pupil_size),
                ));
                r += ((0.05 - r) * cov_pupil);
                g += ((0.04 - g) * cov_pupil);
                b += ((0.06 - b) * cov_pupil);
                // A catchlight from the window.
                let mut lx: f64 = (ix + ((self.flip * 0.36) * iris_r));
                let mut ly: f64 = (iy - (0.36 * iris_r));
                let mut cov_light: f64 =
                    (self.cov(MinionPainter::squeezed(lx, ly, k, (iris_r * 0.24))) * 0.9);
                r += ((1.0 - r) * cov_light);
                g += ((1.0 - g) * cov_light);
                b += ((1.0 - b) * cov_light);
            }
            // Lids: yellow skin sliding down (and up) over the glass.
            let mut upper: f64 =
                rt::max_f64(self.face.lid, (if near { self.face.wink } else { 0.0 }));
            let mut lower: f64 = self.face.squint;
            let mut s: f64 = ((dx / self.lens_r) * (dx / self.lens_r));
            let mut upper_edge: f64 = ((MinionPainter::EYE_V
                + (self.lens_r * (1.0 - (2.0 * upper))))
                - ((0.4 * upper) * s));
            let mut lower_edge: f64 = ((MinionPainter::EYE_V
                - (self.lens_r * (1.0 - (2.0 * lower))))
                + ((0.4 * lower) * s));
            if (upper > 0.01) || (lower > 0.01) {
                let mut bx: f64 = <f64>::default();
                let mut by: f64 = <f64>::default();
                let mut bz: f64 = <f64>::default();
                MinionPainter::normal(
                    u,
                    (v - (if (v < MinionPainter::BODY_BOTTOM) {
                        MinionPainter::BODY_BOTTOM
                    } else {
                        (if (v > MinionPainter::BODY_TOP) {
                            MinionPainter::BODY_TOP
                        } else {
                            v
                        })
                    })),
                    MinionPainter::BODY_R,
                    rt::mref(&mut bx),
                    rt::mref(&mut by),
                    rt::mref(&mut bz),
                );
                let mut sr: f64 = <f64>::default();
                let mut sg: f64 = <f64>::default();
                let mut sb: f64 = <f64>::default();
                self.lit(
                    0.93,
                    0.79,
                    0.10,
                    bx,
                    by,
                    bz,
                    0.3,
                    24.0,
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
                        self.cov(((v - upper_edge).abs() - 0.12))
                    } else {
                        0.0
                    }),
                    (if (lower > 0.01) {
                        self.cov(((v - lower_edge).abs() - 0.1))
                    } else {
                        0.0
                    }),
                ) * 0.75);
                r += ((0.16 - r) * lash);
                g += ((0.10 - g) * lash);
                b += ((0.08 - b) * lash);
            }
            // A soft reflection on the glass.
            let mut gx: f64 = ((dx + ((self.flip * 0.45) * self.lens_r)) / (0.5 * self.lens_r));
            let mut gy: f64 = ((dy - (0.45 * self.lens_r)) / (0.32 * self.lens_r));
            let mut gt: f64 = ((gx * gx) + (gy * gy));
            if gt < 1.0 {
                let mut w: f64 = (0.28 * (1.0 - gt));
                r += ((1.0 - r) * w);
                g += ((1.0 - g) * w);
                b += ((1.0 - b) * w);
            }
            self.over(r, g, b, cov_lens);
        }
        // The rim: a steel ring, lit as a torus.
        let mut ring_d: f64 = MinionPainter::squeezed(dx, dy, k, self.ring_r);
        let mut cov_ring: f64 = (self.cov(((ring_d).abs() - self.tube_r)) * vis);
        if cov_ring > 0.0 {
            let mut t: f64 = ((dist - self.ring_r) / self.tube_r);
            if t > 0.995 {
                t = 0.995;
            } else if t < -0.995 {
                t = -0.995;
            }
            let mut ux: f64 = (if (dist > 0.0) { (dx / dist) } else { 0.0 });
            let mut uy: f64 = (if (dist > 0.0) { (dy / dist) } else { 1.0 });
            let mut nx: f64 = (ux * t);
            let mut ny: f64 = (uy * t);
            let mut nz: f64 = (1.0 - (t * t)).sqrt();
            MinionPainter::turn_2(rt::mref(&mut nx), rt::mref(&mut nz), ac, asn);
            let mut tone: f64 = (0.86 + ((0.26 * uy) * t)); // a hint of sky above, ground below
            let mut lr: f64 = <f64>::default();
            let mut lg: f64 = <f64>::default();
            let mut lb: f64 = <f64>::default();
            self.lit(
                (0.60 * tone),
                (0.62 * tone),
                (0.66 * tone),
                nx,
                ny,
                nz,
                0.9,
                14.0,
                rt::mref(&mut lr),
                rt::mref(&mut lg),
                rt::mref(&mut lb),
            );
            self.over(lr, lg, lb, cov_ring);
        }
    }
    pub fn brows(mut self, mut u: f64, mut v: f64) {
        let mut show: f64 = ((((self.face.brow_angle).abs() * 1.2)
            + rt::max_f64(0.0, self.face.brow_raise))
            + self.face.brow_asym);
        if show <= 0.02 {
            return;
        }
        if show > 1.0 {
            show = 1.0;
        }
        let mut base_v: f64 = ((((MinionPainter::EYE_V + self.ring_r) + self.tube_r) + 0.45)
            + (self.face.brow_raise * 0.5));
        if (v - base_v).abs() > 2.5 {
            return;
        }
        if self.eyes == 2 {
            self.brow(u, v, 0, -1.0, base_v, (show * (*self.eye_vis.at(0))));
            self.brow(
                u,
                v,
                1,
                1.0,
                (base_v + (self.face.brow_asym * 0.6)),
                (show * (*self.eye_vis.at(1))),
            );
        } else {
            self.brow(u, v, 0, -1.0, base_v, show);
            self.brow(u, v, 1, 1.0, (base_v + (self.face.brow_asym * 0.6)), show);
        }
    }
    pub fn brow(
        mut self,
        mut u: f64,
        mut v: f64,
        mut j: i32,
        mut outward: f64,
        mut cy: f64,
        mut show: f64,
    ) {
        if show <= 0.0 {
            return;
        }
        let mut k: f64 = (*self.brow_cos.at(j));
        if k < 0.2 {
            return;
        }
        let mut ex: f64 = ((MinionPainter::BODY_R + 0.3) * (*self.brow_sin.at(j)));
        let mut ax: f64 = (ex - ((outward * 1.15) * k));
        let mut ay: f64 = (cy - (self.face.brow_angle * 0.55));
        let mut bx: f64 = (ex + ((outward * 1.15) * k));
        let mut by: f64 = (cy + (self.face.brow_angle * 0.55));
        let mut qx: f64 = <f64>::default();
        let mut qy: f64 = <f64>::default();
        let mut t: f64 = <f64>::default();
        let mut dist: f64 = MinionPainter::seg(
            u,
            v,
            ax,
            ay,
            bx,
            by,
            rt::mref(&mut qx),
            rt::mref(&mut qy),
            rt::mref(&mut t),
        );
        let mut cov: f64 = (self.cov((dist - (0.27 - (0.09 * t)))) * show);
        self.over(0.12, 0.09, 0.10, cov);
    }
    pub fn mouth(mut self, mut u: f64, mut v: f64) {
        // The mouth is on the front of the body, so it swings round and
        // narrows with the yaw like everything else, and goes out of
        // sight once the body has turned its back.
        let mut vis: f64 = (if (self.cos_y > 0.17) {
            1.0
        } else {
            (if (self.cos_y < -0.08) {
                0.0
            } else {
                ((self.cos_y + 0.08) / 0.25)
            })
        });
        if vis <= 0.0 {
            return;
        }
        let mut k: f64 = (if (self.cos_y > 0.35) {
            self.cos_y
        } else {
            0.35
        });
        let mut mu: f64 =
            (((MinionPainter::BODY_R * 0.93) * self.sin_y) + (self.face.mouth_dx * k));
        let mut w: f64 = (self.face.mouth_width * k);
        let mut hx: f64 = ((u - mu).abs() - w);
        if (hx > 0.8) || ((v - MinionPainter::MOUTH_V).abs() > 4.0) {
            return;
        }
        let mut t: f64 = ((u - mu) / w);
        if t < -1.0 {
            t = -1.0;
        } else if t > 1.0 {
            t = 1.0;
        }
        let mut line: f64 = ((MinionPainter::MOUTH_V
            + ((self.face.mouth_curve * 0.8) * ((t * t) - 0.45)))
            + ((self.face.wobble * 0.18) * (t * 8.5).sin()));
        let mut open: f64 = self.face.mouth_open;
        let mut lip: f64 = 0.26;
        if open < 0.03 {
            let mut ox: f64 = (if (hx > 0.0) { hx } else { 0.0 });
            let mut oy: f64 = (v - line);
            let mut cov: f64 = self.cov((((ox * ox) + (oy * oy)).sqrt() - (lip / 2.0)));
            self.over(0.16, 0.10, 0.09, (cov * vis));
            return;
        }
        let mut gape: f64 = ((open * 2.3) * rt::pow((1.0 - (t * t)), 0.6));
        let mut upper: f64 = line;
        let mut lower: f64 = (line - gape);
        let mut d: f64 = rt::max_f64(rt::max_f64((v - upper), (lower - v)), hx);
        let mut inside: f64 = self.cov(d);
        if inside > 0.0 {
            self.over(0.30, 0.07, 0.09, (inside * vis));
            let mut teeth_h: f64 = ((open * 0.7) * self.face.teeth);
            if teeth_h > 0.05 {
                let mut teeth: f64 = (self.cov(rt::max_f64(((upper - teeth_h) - v), d)) * inside);
                let mut gap: f64 = (((((u - mu) % 0.6) + 0.6) % 0.6) - 0.3).abs();
                let mut seam: f64 = (self.cov((gap - 0.03)) * 0.35);
                self.over(
                    (0.95 - (seam * 0.5)),
                    (0.94 - (seam * 0.5)),
                    (0.90 - (seam * 0.5)),
                    (teeth * vis),
                );
            }
        }
        let mut outline: f64 = self.cov(((d).abs() - (lip / 2.0)));
        self.over(0.16, 0.10, 0.09, (outline * vis));
        if self.face.tongue > 0.02 {
            self.tongue(u, v, mu, w, upper, lower, vis);
        }
    }
    // A pink tongue: lying in the bottom of an open mouth, or, the more
    // of it there is, hanging out over the lower lip.
    pub fn tongue(
        mut self,
        mut u: f64,
        mut v: f64,
        mut mu: f64,
        mut w: f64,
        mut upper: f64,
        mut lower: f64,
        mut vis: f64,
    ) {
        let mut tongue: f64 = self.face.tongue;
        let mut rx: f64 = rt::min_f64((w * 0.55), 0.8);
        let mut ry: f64 = (0.35 + (0.5 * tongue));
        let mut cy: f64 = ((lower + 0.35) - (0.8 * tongue));
        let mut dx: f64 = ((u - mu) / rx);
        let mut dy: f64 = ((v - cy) / ry);
        let mut e: f64 = ((dx * dx) + (dy * dy)).sqrt();
        let mut rim: f64 = ((e - 1.0) * ry);
        let mut d: f64 = rt::max_f64(rim, ((v - upper) + 0.15)); // never over the upper lip
        let mut cov: f64 = self.cov(d);
        if cov <= 0.0 {
            return;
        }
        let mut shade: f64 = (1.0 - (0.25 * e));
        self.over(
            (0.12 + (0.86 * shade)),
            (0.42 * shade),
            (0.52 * shade),
            (cov * vis),
        );
        let mut edge: f64 =
            ((self.cov(((rim).abs() - 0.09)) * self.cov(((v - upper) + 0.15))) * 0.7);
        self.over(0.16, 0.10, 0.09, (edge * vis));
    }
    pub fn hair(mut self, mut u: f64, mut v: f64) {
        if (((v < ((MinionPainter::BODY_TOP + MinionPainter::BODY_R) - 2.5))
            || (v < self.hair_vlo))
            || (u < self.hair_lo))
            || (u > self.hair_hi)
        {
            return;
        }
        {
            let mut i: i32 = 0;
            'l1: while i < self.hair_length {
                let mut qx: f64 = <f64>::default();
                let mut qy: f64 = <f64>::default();
                let mut t: f64 = <f64>::default();
                let mut dist: f64 = MinionPainter::seg(
                    u,
                    v,
                    (*self.hair.at(i)),
                    (*self.hair.at((i + 1))),
                    (*self.hair.at((i + 2))),
                    (*self.hair.at((i + 3))),
                    rt::mref(&mut qx),
                    rt::mref(&mut qy),
                    rt::mref(&mut t),
                );
                let mut radius: f64 = ((*self.hair.at((i + 4)))
                    + (((*self.hair.at((i + 5))) - (*self.hair.at((i + 4)))) * t));
                let mut cov: f64 = self.cov((dist - radius));
                if cov > 0.0 {
                    self.over(
                        (0.10 * (0.8 + (0.4 * t))),
                        (0.08 * (0.8 + (0.4 * t))),
                        (0.10 * (0.8 + (0.4 * t))),
                        cov,
                    );
                }
                i += 6;
            }
        }
    }
    //
    // One arm: two tubes and a glove, each sorted against the body
    // pixel by pixel. Where a piece is behind the body's surface it is
    // painted now and the body will cover it; where it is in front it
    // is kept for after the face. The arm's own depth matters too: a
    // far arm is a shade darker.
    //
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
            MinionPainter::SHOULDER_V,
            (*self.sh_z.at(i)),
            (*self.elb_u.at(i)),
            (*self.elb_v.at(i)),
            (*self.elb_z.at(i)),
            MinionPainter::ARM_R,
            0.98,
            0.84,
            0.11,
            0.3,
            24.0,
        );
        self.arm_tube(
            u,
            v,
            (*self.elb_u.at(i)),
            (*self.elb_v.at(i)),
            (*self.elb_z.at(i)),
            (*self.hand_u.at(i)),
            (*self.hand_v.at(i)),
            (*self.hand_z.at(i)),
            MinionPainter::FORE_R,
            0.98,
            0.84,
            0.11,
            0.3,
            24.0,
        );
        // Pointing: the index finger carries on past the glove in the
        // direction of the forearm.
        if (i == 1) && self.pose.point_near {
            let mut fx: f64 = ((*self.hand_u.at(i)) - (*self.elb_u.at(i)));
            let mut fy: f64 = ((*self.hand_v.at(i)) - (*self.elb_v.at(i)));
            let mut len: f64 = ((fx * fx) + (fy * fy)).sqrt();
            if len > 1e-6 {
                let mut reach: f64 = ((MinionPainter::GLOVE_R + MinionPainter::FINGER_LEN) / len);
                self.arm_tube(
                    u,
                    v,
                    (*self.hand_u.at(i)),
                    (*self.hand_v.at(i)),
                    (*self.hand_z.at(i)),
                    ((*self.hand_u.at(i)) + (fx * reach)),
                    ((*self.hand_v.at(i)) + (fy * reach)),
                    (*self.hand_z.at(i)),
                    MinionPainter::FINGER_R,
                    0.11,
                    0.10,
                    0.13,
                    0.7,
                    18.0,
                );
            }
        }
        let mut dx: f64 = (u - (*self.hand_u.at(i)));
        let mut dy: f64 = (v - (*self.hand_v.at(i)));
        let mut cov: f64 = self.cov((((dx * dx) + (dy * dy)).sqrt() - MinionPainter::GLOVE_R));
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        MinionPainter::normal(
            dx,
            dy,
            MinionPainter::GLOVE_R,
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        let mut k: f64 = self.depth((*self.hand_z.at(i)));
        let mut lr: f64 = <f64>::default();
        let mut lg: f64 = <f64>::default();
        let mut lb: f64 = <f64>::default();
        self.lit(
            (0.11 * k),
            (0.10 * k),
            (0.13 * k),
            nx,
            ny,
            nz,
            0.7,
            18.0,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.sort(
            lr,
            lg,
            lb,
            cov,
            ((*self.hand_z.at(i)) + (nz * MinionPainter::GLOVE_R)),
        );
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
        let mut r: i32 = (((MinionPainter::clamp(self.cr) * 255.0) + 0.5) as i32);
        let mut g: i32 = (((MinionPainter::clamp(self.cg) * 255.0) + 0.5) as i32);
        let mut b: i32 = (((MinionPainter::clamp(self.cb) * 255.0) + 0.5) as i32);
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
            (spec * MinionPainter::pow(h, shine))
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
        MinionPainter::turn_2(&mut *nx, &mut *nz, (a).cos(), (a).sin());
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
        let mut dist: f64 = MinionPainter::seg(
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
        MinionPainter::normal(
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
        let mut dist: f64 = MinionPainter::seg(
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
        MinionPainter::normal(
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
            (MinionPainter::seg(
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
            (MinionPainter::seg(
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
            (MinionPainter::seg(
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
        let mut d: f64 = MinionPainter::seg(
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
            MinionPainter::seg(
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
            MinionPainter::seg(
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
        let mut bar: f64 = (MinionPainter::seg(
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
