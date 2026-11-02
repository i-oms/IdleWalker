// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

//
// Paints Mr. Bean into a `Surface` the way the Minions are
// painted: as lit solids with anti-aliased edges, composited back to
// front (see `SolidPainter`). A tall thin man: long legs
// with knees, a tapered torso in a brown tweed jacket over a white
// shirt and a thin red tie, a neck, and a round head with heavy
// eyebrows, a long nose, a wide mouth, ears, and dark hair swept back
// from a high forehead. Teddy appears in his hand when the pose says so.
//
// Model space is the Minions': u across in units, positive the way he
// faces; v up from the soles. He is `Height` units tall.
// The body turns with `Pose.Yaw`; the head turns further
// by `Pose.HeadTurn` and nods by `Pose.HeadDip`.
// The arms can come forward out of the picture plane
// (`Pose.ReachNear`), which is how a hand reaches the chin,
// the collar or the middle of the chest.
//
#[derive(Clone, Default)]
pub struct BeanPainterData {
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
    // Eyes: angle from the viewer, screen centre, horizontal squeeze, visibility.
    pub eye_alpha: Arr<f64>,
    pub eye_u: Arr<f64>,
    pub eye_k: Arr<f64>,
    pub eye_vis: Arr<f64>,
    pub eye_cos: Arr<f64>,
    pub eye_sin: Arr<f64>,
    pub gaze_u: Arr<f64>,
    pub gaze_v: Arr<f64>,
    // The props' moving parts: Teddy's waving paw and the wheel's spokes, from their roots.
    pub paw_u: f64,
    pub paw_v: f64,
    pub spoke_u: Arr<f64>,
    pub spoke_v: Arr<f64>,
    pub first_eye: i32,
    // Ears: screen centre and depth.
    pub ear_u: Arr<f64>,
    pub ear_z: Arr<f64>,
    // Legs: hip, knee and ankle of each, and which is behind.
    pub hip_u: Arr<f64>,
    pub knee_u: Arr<f64>,
    pub knee_v: Arr<f64>,
    pub ankle_u: Arr<f64>,
    pub ankle_v: Arr<f64>,
    pub leg_z: Arr<f64>,
    pub shoe_len: f64,
    pub toe: f64,
    // Arms: shoulder, elbow, cuff and hand of each, with depths, and the screen box it all fits in.
    pub sh_u: Arr<f64>,
    pub sh_z: Arr<f64>,
    pub elb_u: Arr<f64>,
    pub elb_v: Arr<f64>,
    pub elb_z: Arr<f64>,
    pub cuff_u: Arr<f64>,
    pub cuff_v: Arr<f64>,
    pub cuff_z: Arr<f64>,
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
rt::handle!(BeanPainter, BeanPainterData);

thread_local! { static BEANPAINTER_CACHED: std::cell::Cell<BeanPainter> = std::cell::Cell::new(Default::default()); }

impl BeanPainter {
    fn alloc__() -> BeanPainter {
        let mut this = BeanPainter::from_data(BeanPainterData::default());
        this.eye_alpha = Arr::new(2);
        this.eye_u = Arr::new(2);
        this.eye_k = Arr::new(2);
        this.eye_vis = Arr::new(2);
        this.eye_cos = Arr::new(2);
        this.eye_sin = Arr::new(2);
        this.gaze_u = Arr::new(2);
        this.gaze_v = Arr::new(2);
        this.spoke_u = Arr::new(3);
        this.spoke_v = Arr::new(3);
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
        this.cuff_u = Arr::new(2);
        this.cuff_v = Arr::new(2);
        this.cuff_z = Arr::new(2);
        this.hand_u = Arr::new(2);
        this.hand_v = Arr::new(2);
        this.hand_z = Arr::new(2);
        this.arm_lo = Arr::new(2);
        this.arm_hi = Arr::new(2);
        this.arm_vlo = Arr::new(2);
        this.arm_vhi = Arr::new(2);
        this.front_r = Arr::new(12);
        this.front_g = Arr::new(12);
        this.front_b = Arr::new(12);
        this.front_cov = Arr::new(12);
        this.column_u = Arr::new(0);
        this
    }
    pub fn new() -> BeanPainter {
        Self::alloc__()
    }
    pub const HEIGHT: f64 = 28.0;
    // Legs: hips either side of the axis, a thigh and a shin with a knee between.
    pub const HIP_V: f64 = 12.0;
    pub const HIP_X: f64 = 1.15;
    pub const THIGH: f64 = 5.6;
    pub const SHIN: f64 = 5.4;
    pub const THIGH_R: f64 = 1.0;
    pub const SHIN_R: f64 = 0.86;
    // Torso: a capsule from the hips to the shoulders, wider at the top.
    pub const TORSO_BOTTOM: f64 = 12.2;
    pub const TORSO_TOP: f64 = 19.4;
    pub const HIP_R: f64 = 2.4;
    pub const CHEST_R: f64 = 2.95;
    // Shoulders rise this far above TorsoTop before rounding off: a low
    // cap so the neck stands clear instead of sinking into a tall dome.
    pub const SHOULDER_CAP: f64 = 0.6;
    pub const HEM_V: f64 = 13.1;
    pub const V_BOTTOM: f64 = 14.6;
    pub const COLLAR_V: f64 = 19.9;
    pub const NECK_BOTTOM: f64 = 19.3;
    pub const NECK_TOP: f64 = 22.0;
    pub const NECK_R: f64 = 0.8;
    pub const COLLAR_TOP: f64 = 20.4;
    // Head: big, as a cartoon's, so the face reads at a few pixels per
    // unit, and taller than it is wide - a long face with a long chin.
    // The hair is a second ellipsoid sitting a little proud of the skull.
    pub const HEAD_V: f64 = 24.4;
    pub const HEAD_RX: f64 = 3.0;
    pub const HEAD_RY: f64 = 3.5;
    pub const HAIR_V: f64 = 24.85;
    pub const HAIR_RX: f64 = 3.2;
    pub const HAIR_RY: f64 = 3.55;
    pub const EYE_V: f64 = 24.9;
    pub const EYE_ANGLE: f64 = 19.0;
    pub const EYE_RX: f64 = 0.9;
    pub const EYE_RY: f64 = 0.66;
    pub const IRIS_R: f64 = 0.46;
    pub const BROW_V: f64 = 25.95;
    pub const NOSE_TOP_V: f64 = 24.7;
    pub const NOSE_TIP_V: f64 = 23.15;
    pub const MOUTH_V: f64 = 22.25;
    pub const EAR_V: f64 = 24.3;
    pub const EAR_R: f64 = 0.7;
    // Arms: shoulders on the chest's surface, ShoulderAngle round from the front.
    pub const SHOULDER_V: f64 = 19.4;
    pub const SHOULDER_ANGLE: f64 = 82.0;
    pub const ARM_RADIUS: f64 = (BeanPainter::CHEST_R + 0.1);
    pub const UPPER_ARM: f64 = 3.4;
    pub const FOREARM: f64 = 3.2;
    pub const REST_SPREAD: f64 = 4.0;
    pub const SLEEVE_R: f64 = 0.78;
    pub const FORE_R: f64 = 0.7;
    pub const CUFF_LEN: f64 = 0.5;
    pub const HAND_R: f64 = 0.72;
    pub const FINGER_LEN: f64 = 0.55;
    pub const FINGER_R: f64 = 0.2;
    pub const PATCH_R: f64 = 0.6;
    // Colours.
    pub const SKIN_R: f64 = 0.96;
    pub const SKIN_G: f64 = 0.81;
    pub const SKIN_B: f64 = 0.70;
    pub const HAIR_R: f64 = 0.10;
    pub const HAIR_G: f64 = 0.075;
    pub const HAIR_B: f64 = 0.06;
    pub const COAT_R: f64 = 0.50;
    pub const COAT_G: f64 = 0.37;
    pub const COAT_B: f64 = 0.24;
    pub const TROUSER_R: f64 = 0.32;
    pub const TROUSER_G: f64 = 0.24;
    pub const TROUSER_B: f64 = 0.17;
    pub const SHIRT_R: f64 = 0.95;
    pub const SHIRT_G: f64 = 0.95;
    pub const SHIRT_B: f64 = 0.91;
    pub const TIE_R: f64 = 0.66;
    pub const TIE_G: f64 = 0.11;
    pub const TIE_B: f64 = 0.14;
    pub const TEDDY_R: f64 = 0.76;
    pub const TEDDY_G: f64 = 0.56;
    pub const TEDDY_B: f64 = 0.33;
    pub fn init(mut self, mut pose: Pose, mut face_left: bool, mut px: f64) {
        self.setup(face_left, px, BeanPainter::ARM_RADIUS);
        self.pose = pose;
        self.face = pose.face;
        self.lean = pose.lean;
        self.dip = pose.head_dip;
        self.yaw = (BeanPainter::wrap(pose.yaw) * SolidPainter::DEG);
        self.sin_y = (self.yaw).sin();
        self.cos_y = (self.yaw).cos();
        self.head_yaw = (BeanPainter::wrap((pose.yaw + pose.head_turn)) * SolidPainter::DEG);
        self.sin_h = (self.head_yaw).sin();
        self.cos_h = (self.head_yaw).cos();
        self.mouth_u = ((BeanPainter::HEAD_RX * 0.95) * self.sin_h);
        self.place_eye(0, (-(BeanPainter::EYE_ANGLE) * SolidPainter::DEG));
        self.place_eye(1, (BeanPainter::EYE_ANGLE * SolidPainter::DEG));
        self.first_eye = (if ((*self.eye_alpha.at(0)).cos() < (*self.eye_alpha.at(1)).cos()) {
            0
        } else {
            1
        });
        // The irises follow the gaze, kept inside the whites: fixed for the frame.
        let mut reach: f64 = ((BeanPainter::EYE_RX - BeanPainter::IRIS_R) - 0.05);
        {
            let mut i: i32 = 0;
            'l1: while i < 2 {
                (*self.eye_cos.at(i)) = (*self.eye_alpha.at(i)).cos();
                (*self.eye_sin.at(i)) = (*self.eye_alpha.at(i)).sin();
                let mut ox: f64 = (((pose.gaze_x * reach) * 0.9) + (self.face.pupil_dx * 0.4));
                let mut oy: f64 =
                    (((pose.gaze_y * (BeanPainter::EYE_RY - (BeanPainter::IRIS_R * 0.6))) * 0.8)
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
        {
            let mut i: i32 = 0;
            'l2: while i < 2 {
                let mut a: f64 = ((((if (i == 0) { -90 } else { 90 }) as f64) * SolidPainter::DEG)
                    + self.head_yaw);
                (*self.ear_u.at(i)) = ((BeanPainter::HEAD_RX + 0.15) * (a).sin());
                (*self.ear_z.at(i)) = (BeanPainter::HEAD_RX * (a).cos());
                i += 1;
            }
        }
        // Legs. The thigh swings fore and aft with the stride; the knee
        // bends as the leg comes forward and straightens for the heel
        // strike and the push-off. Knees up for a jump.
        let mut p: f64 = ((pose.walk_phase * 2.0) * std::f64::consts::PI);
        let mut swing: f64 = (if pose.walking {
            (28.0 * (p).sin())
        } else {
            0.0
        });
        let mut bend_a: f64 = (if pose.walking {
            (55.0 * rt::max_f64(0.0, (p).cos()))
        } else {
            3.0
        });
        let mut bend_b: f64 = (if pose.walking {
            (55.0 * rt::max_f64(0.0, -((p).cos())))
        } else {
            3.0
        });
        let mut tuck: f64 = pose.leg_tuck;
        self.place_leg(
            0,
            -(BeanPainter::HIP_X),
            (-swing + (55.0 * tuck)),
            (bend_b + (95.0 * tuck)),
        );
        self.place_leg(
            1,
            BeanPainter::HIP_X,
            (swing + (55.0 * tuck)),
            (bend_a + (95.0 * tuck)),
        );
        self.shoe_len = (1.25 + (0.85 * (self.sin_y).abs()));
        self.toe = (0.25 + (0.8 * self.sin_y));
        // Arms, as for a Minion, plus the reach out of the picture plane.
        let mut arm_swing: f64 = (if pose.walking {
            (28.0 * (p).sin())
        } else {
            0.0
        });
        let mut fore_swing: f64 = ((if pose.walking { 15 } else { 0 }) as f64);
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
        let mut ang: f64 = ((25.0 + (85.0 * pose.teddy_wave)) * SolidPainter::DEG);
        self.paw_u = (0.85 * (ang).cos());
        self.paw_v = (0.85 * (ang).sin());
        let mut turn: f64 = (pose.wheel_turn * 0.9);
        {
            let mut s: i32 = 0;
            'l3: while s < 3 {
                let mut a: f64 = (turn + ((((s * 2) as f64) * std::f64::consts::PI) / 3.0));
                (*self.spoke_u.at(s)) = (1.05 * (a).cos());
                (*self.spoke_v.at(s)) = (1.05 * (a).sin());
                s += 1;
            }
        }
        self.decorated =
            (((((pose.sweat > 0.0) || pose.note) || pose.zzz) || pose.bubble) || pose.heart);
        self.top_v = (if self.decorated {
            37.0
        } else {
            (((BeanPainter::HEIGHT + 0.8) + (pose.body_dy).abs()) + rt::max_f64(0.0, -self.dip))
        });
    }
    // A spin can carry the yaw past a full turn; bring it back to (-180, 180].
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
        (*self.eye_u.at(i)) = ((BeanPainter::HEAD_RX + 0.05) * (a).sin());
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
        (*self.knee_u.at(i)) = ((*self.hip_u.at(i)) + (BeanPainter::THIGH * (a).sin()));
        (*self.knee_v.at(i)) = (BeanPainter::HIP_V - (BeanPainter::THIGH * (a).cos()));
        let mut b: f64 = ((swing - bend) * SolidPainter::DEG);
        (*self.ankle_u.at(i)) = ((*self.knee_u.at(i)) + (BeanPainter::SHIN * (b).sin()));
        (*self.ankle_v.at(i)) =
            (((*self.knee_v.at(i)) - (BeanPainter::SHIN * (b).cos())) + (0.65 * self.pose.tiptoe));
    }
    //
    // Lays out one arm like a Minion's (`raise` and `bend` in the picture
    // plane, `swing` fore and aft for the walk), with `reach` bringing
    // the upper arm forward out of the picture plane - towards where he
    // faces whether the arm hangs or is raised - and the forearm back
    // into it, so a bent arm can put a hand in front of the body.
    //
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
        let mut at: f64 = (((side * BeanPainter::SHOULDER_ANGLE) * SolidPainter::DEG) + self.yaw);
        (*self.sh_u.at(i)) = (BeanPainter::ARM_RADIUS * (at).sin());
        (*self.sh_z.at(i)) = (BeanPainter::ARM_RADIUS * (at).cos());
        if raise == 0.0 {
            raise = (side * BeanPainter::REST_SPREAD);
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
            ((*self.sh_u.at(i)) + (BeanPainter::UPPER_ARM * (sa + ((ca * sb) * self.sin_y))));
        (*self.elb_v.at(i)) = (BeanPainter::SHOULDER_V - ((BeanPainter::UPPER_ARM * ca) * cb));
        (*self.elb_z.at(i)) =
            ((*self.sh_z.at(i)) + (((BeanPainter::UPPER_ARM * ca) * sb) * self.cos_y));
        a = ((raise + bend) * SolidPainter::DEG);
        b = ((swing + fore_swing) * SolidPainter::DEG);
        sa = (a).sin();
        ca = (a).cos();
        sb = (b).sin();
        cb = (b).cos();
        (*self.hand_u.at(i)) =
            ((*self.elb_u.at(i)) + (BeanPainter::FOREARM * (sa + ((ca * sb) * self.sin_y))));
        (*self.hand_v.at(i)) = ((*self.elb_v.at(i)) - ((BeanPainter::FOREARM * ca) * cb));
        (*self.hand_z.at(i)) =
            ((*self.elb_z.at(i)) + (((BeanPainter::FOREARM * ca) * sb) * self.cos_y));
        // The shirt cuff: the last bit of the forearm.
        let mut f: f64 = (1.0 - (BeanPainter::CUFF_LEN / BeanPainter::FOREARM));
        (*self.cuff_u.at(i)) =
            ((*self.elb_u.at(i)) + (((*self.hand_u.at(i)) - (*self.elb_u.at(i))) * f));
        (*self.cuff_v.at(i)) =
            ((*self.elb_v.at(i)) + (((*self.hand_v.at(i)) - (*self.elb_v.at(i))) * f));
        (*self.cuff_z.at(i)) =
            ((*self.elb_z.at(i)) + (((*self.hand_z.at(i)) - (*self.elb_z.at(i))) * f));
        let mut pad: f64 = 1.3;
        if (side > 0.0) && self.pose.point_near {
            pad += BeanPainter::FINGER_LEN;
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
            rt::min_f64(BeanPainter::SHOULDER_V, (*self.elb_v.at(i))),
            (*self.hand_v.at(i)),
        ) - pad);
        (*self.arm_vhi.at(i)) = (rt::max_f64(
            rt::max_f64(BeanPainter::SHOULDER_V, (*self.elb_v.at(i))),
            (*self.hand_v.at(i)),
        ) + pad);
    }
    // ---- Entry point ---------------------------------------------------------
    // Paints Mr. Bean with his soles on soleY, centred on centreX, at `scale` pixels per unit.
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
        let mut painter: BeanPainter = BEANPAINTER_CACHED.with(|c__| {
            if c__.get().is_null() {
                c__.set(BeanPainter::new());
            }
            c__.get()
        });
        painter.init(pose, face_left, px);
        let mut left: i32 = rt::max_i32(0, (((centre_x - (11.5 * sx)).floor()) as i32));
        let mut right: i32 =
            rt::min_i32(surface.width(), (((centre_x + (11.5 * sx)).ceil()) as i32));
        let mut top: i32 = rt::max_i32(0, (((sole_y - (painter.top_v * sy)).floor()) as i32));
        let mut bottom: i32 = rt::min_i32(surface.height(), (((sole_y + 0.5).ceil()) as i32));
        if (left >= right) || (top >= bottom) {
            return;
        }
        painter.raster(surface, left, top, right, bottom, centre_x, sole_y, sx, sy);
    }
    pub fn shade(mut self, mut u: f64, mut v: f64) -> i32 {
        self.begin();
        // The upper body bobs and leans from the hips; the legs stay planted.
        let mut vb: f64 = (v + self.pose.body_dy);
        let mut ub: f64 = (u - ((self.lean * 0.14) * rt::max_f64(0.0, (v - 10.0))));
        // How far forward the body's surface is here: the torso's or the
        // head's, whichever bulges further. Limbs are sorted against it.
        // Above the shoulders the dome is squashed to ShoulderCap so the
        // neck stands clear of the chest instead of sinking into it.
        let mut axis_v: f64 = (if (vb < BeanPainter::TORSO_BOTTOM) {
            BeanPainter::TORSO_BOTTOM
        } else {
            (if (vb > BeanPainter::TORSO_TOP) {
                BeanPainter::TORSO_TOP
            } else {
                vb
            })
        });
        let mut t: f64 = ((axis_v - BeanPainter::TORSO_BOTTOM)
            / (BeanPainter::TORSO_TOP - BeanPainter::TORSO_BOTTOM));
        let mut torso_r: f64 =
            (BeanPainter::HIP_R + ((BeanPainter::CHEST_R - BeanPainter::HIP_R) * t));
        let mut tdy: f64 = (vb - axis_v);
        let mut dy_eff: f64 = (if (vb > BeanPainter::TORSO_TOP) {
            ((tdy * torso_r) / BeanPainter::SHOULDER_CAP)
        } else {
            tdy
        });
        let mut torso_dist: f64 = ((ub * ub) + (dy_eff * dy_eff)).sqrt();
        let mut bulge: f64 = ((torso_r * torso_r) - (torso_dist * torso_dist));
        self.body_z = (if (bulge > 0.0) { (bulge).sqrt() } else { 0.0 });
        let mut hdy: f64 = (vb - BeanPainter::HEAD_V);
        let mut hky: f64 = (BeanPainter::HEAD_RX / BeanPainter::HEAD_RY);
        let mut head_bulge: f64 = (((BeanPainter::HEAD_RX * BeanPainter::HEAD_RX) - (ub * ub))
            - (((hdy * hdy) * hky) * hky));
        if head_bulge > (self.body_z * self.body_z) {
            self.body_z = (head_bulge).sqrt();
        }
        // Back to front.
        let mut back_leg: i32 = (if ((*self.leg_z.at(0)) < (*self.leg_z.at(1))) {
            0
        } else {
            1
        });
        self.leg(u, v, back_leg);
        self.leg(u, v, (1 - back_leg));
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
        if ((hdy).abs() < (BeanPainter::HEAD_RY + 1.5))
            && ((ub).abs() < (BeanPainter::HEAD_RX + 1.5))
        {
            self.head(ub, vb, hdy);
            if (*self.ear_z.at(0)) > 0.0 {
                self.ear(ub, vb, 0);
            }
            if (*self.ear_z.at(1)) > 0.0 {
                self.ear(ub, vb, 1);
            }
            self.eye(ub, vb, self.first_eye);
            self.eye(ub, vb, (1 - self.first_eye));
            self.brows(ub, vb);
            self.nose(ub, vb);
            self.mouth(ub, vb);
        }
        self.lay_front();
        if self.pose.teddy {
            self.teddy(ub, vb);
        }
        if self.pose.wheel {
            self.wheel_prop(ub, vb);
        }
        if self.pose.camera {
            self.camera_prop(ub, vb);
        }
        // Things floating about him, in front of everything.
        if self.decorated {
            if self.pose.sweat > 0.0 {
                self.sweat(
                    ub,
                    vb,
                    self.pose.sweat,
                    -(BeanPainter::HEAD_RX + 0.9),
                    (BeanPainter::HEAD_V + 1.4),
                );
            }
            if self.pose.note {
                self.note(
                    ub,
                    vb,
                    self.pose.decor_phase,
                    self.mouth_u,
                    BeanPainter::MOUTH_V,
                );
            }
            if self.pose.zzz {
                self.zzz(
                    u,
                    v,
                    self.pose.decor_phase,
                    2.6,
                    (BeanPainter::HEIGHT + 1.6),
                );
            }
            if self.pose.bubble {
                self.bubble(
                    ub,
                    vb,
                    (BeanPainter::HEIGHT + 0.4),
                    (BeanPainter::HEIGHT + 1.3),
                    (BeanPainter::HEIGHT + 2.8),
                    (BeanPainter::HEIGHT + 5.4),
                );
            }
            if self.pose.heart {
                self.heart(
                    ub,
                    vb,
                    self.pose.decor_phase,
                    self.mouth_u,
                    BeanPainter::MOUTH_V,
                );
            }
        }
        return self.finish();
    }
    // ---- The parts, back to front ---------------------------------------------------
    pub fn leg(mut self, mut u: f64, mut v: f64, mut i: i32) {
        if v > (BeanPainter::HIP_V + 1.5) {
            return;
        }
        let mut lo: f64 = (rt::min_f64(
            rt::min_f64((*self.hip_u.at(i)), (*self.knee_u.at(i))),
            (*self.ankle_u.at(i)),
        ) - 2.6);
        let mut hi: f64 = (rt::max_f64(
            rt::max_f64((*self.hip_u.at(i)), (*self.knee_u.at(i))),
            (*self.ankle_u.at(i)),
        ) + 2.6);
        if (u < lo) || (u > hi) {
            return;
        }
        let mut k: f64 = (0.8 + ((0.2 * (1.0 + ((*self.leg_z.at(i)) / BeanPainter::HIP_X))) / 2.0)); // the far leg a shade darker
        self.tube(
            u,
            v,
            (*self.hip_u.at(i)),
            BeanPainter::HIP_V,
            (*self.knee_u.at(i)),
            (*self.knee_v.at(i)),
            BeanPainter::THIGH_R,
            (BeanPainter::TROUSER_R * k),
            (BeanPainter::TROUSER_G * k),
            (BeanPainter::TROUSER_B * k),
            0.06,
            8.0,
        );
        self.tube(
            u,
            v,
            (*self.knee_u.at(i)),
            (*self.knee_v.at(i)),
            (*self.ankle_u.at(i)),
            (*self.ankle_v.at(i)),
            BeanPainter::SHIN_R,
            (BeanPainter::TROUSER_R * k),
            (BeanPainter::TROUSER_G * k),
            (BeanPainter::TROUSER_B * k),
            0.06,
            8.0,
        );
        // Trouser crease: a faint pressed line down the front of the shin.
        {
            let mut qx: f64 = <f64>::default();
            let mut qy: f64 = <f64>::default();
            let mut t: f64 = <f64>::default();
            let mut crease: f64 = (BeanPainter::seg(
                u,
                v,
                ((*self.knee_u.at(i)) + 0.15),
                ((*self.knee_v.at(i)) - 0.3),
                ((*self.ankle_u.at(i)) + 0.15),
                ((*self.ankle_v.at(i)) + 0.9),
                rt::mref(&mut qx),
                rt::mref(&mut qy),
                rt::mref(&mut t),
            ) - 0.07);
            self.over(
                (0.42 * k),
                (0.33 * k),
                (0.25 * k),
                (self.cov(((crease).abs() - 0.05)) * 0.55),
            );
            // Turn-up cuff at the ankle, and the grey sock showing
            // between the trouser hem and the shoe.
            let mut cuff: f64 = self.cov(rt::max_f64(
                ((v - ((*self.ankle_v.at(i)) + 0.9)).abs() - 0.12),
                ((u - ((*self.ankle_u.at(i)) + 0.1)).abs() - 0.85),
            ));
            self.over((0.24 * k), (0.18 * k), (0.13 * k), (cuff * 0.7));
            let mut sock: f64 = self.cov(rt::max_f64(
                ((v - ((*self.ankle_v.at(i)) + 0.25)).abs() - 0.3),
                ((u - (*self.ankle_u.at(i))).abs() - (BeanPainter::SHIN_R + 0.02)),
            ));
            self.over((0.55 * k), (0.55 * k), (0.56 * k), sock);
        }
        // The shoe: a glossy black ellipsoid under the ankle, toe forward;
        // on tiptoe it tips so the toe stays down.
        let mut tip: f64 = self.pose.tiptoe;
        let mut sx: f64 = (((*self.ankle_u.at(i)) + self.toe) + (0.3 * tip));
        let mut sy: f64 = (((*self.ankle_v.at(i)) - 0.55) - (0.25 * tip));
        let mut rx: f64 = self.shoe_len;
        let mut ry: f64 = 0.58;
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
        BeanPainter::normal(
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
            (0.09 * k),
            (0.08 * k),
            (0.09 * k),
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
        // Sole: a dark strip along the bottom; laces: three dots along the top.
        self.over(
            0.02,
            0.02,
            0.025,
            ((self.cov(((v - ((sy - ry) + 0.12)).abs() - 0.07))
                * self.cov(((u - sx).abs() - (rx - 0.2))))
                * cov),
        );
        {
            let mut lace: i32 = -1;
            'l1: while lace <= 1 {
                let mut lx0: f64 = ((sx - 0.15) + (((lace) as f64) * 0.38));
                let mut ly0: f64 = ((sy + 0.30) - ((((lace).abs()) as f64) * 0.06));
                let mut ldx: f64 = (u - lx0);
                let mut ldy: f64 = (v - ly0);
                self.over(
                    0.16,
                    0.14,
                    0.15,
                    (self.cov((((ldx * ldx) + (ldy * ldy)).sqrt() - 0.09)) * 0.8),
                );
                lace += 1;
            }
        }
        // Toe-cap highlight.
        {
            let mut hx0: f64 = (u - (sx + (rx * 0.55)));
            let mut hy0: f64 = (v - (sy + 0.12));
            self.over(
                0.35,
                0.34,
                0.36,
                (self.cov((((hx0 * hx0) + (hy0 * hy0)).sqrt() - 0.16)) * 0.5),
            );
        }
    }
    // The torso at (u, v): dy is the height above the nearest point of the capsule's axis, dist the distance from it, r its radius there.
    pub fn torso(mut self, mut u: f64, mut v: f64, mut dy: f64, mut dist: f64, mut r: f64) {
        let mut cov: f64 = self.cov((dist - r));
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        BeanPainter::normal(
            u,
            dy,
            r,
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        let mut cr: f64 = <f64>::default();
        let mut cg: f64 = <f64>::default();
        let mut cb: f64 = <f64>::default();
        let mut spec: f64 = <f64>::default();
        let mut shine: f64 = <f64>::default();
        if v < BeanPainter::HEM_V {
            // Trousers below the jacket's hem.
            cr = BeanPainter::TROUSER_R;
            cg = BeanPainter::TROUSER_G;
            cb = BeanPainter::TROUSER_B;
            spec = 0.06;
            shine = 8.0;
            let mut hem: f64 = self.cov(((v - BeanPainter::HEM_V).abs() - 0.08));
            BeanPainter::mix(
                rt::mref(&mut cr),
                rt::mref(&mut cg),
                rt::mref(&mut cb),
                rt::mref(&mut spec),
                rt::mref(&mut shine),
                (BeanPainter::COAT_R * 0.7),
                (BeanPainter::COAT_G * 0.7),
                (BeanPainter::COAT_B * 0.7),
                0.05,
                6.0,
                hem,
            );
        } else {
            // Where on the surface this is, round from the front, as an
            // arc length. Only the jacket needs it.
            let mut phi: f64 = ((nx).atan2(nz) - self.yaw);
            if phi > std::f64::consts::PI {
                phi -= (2.0 * std::f64::consts::PI);
            } else if phi < -(std::f64::consts::PI) {
                phi += (2.0 * std::f64::consts::PI);
            }
            let mut arc: f64 = (r * phi);
            let mut a: f64 = (arc).abs();
            // The tweed: a herringbone weave - fine diagonal ribs that
            // reverse direction every few rows - one sine, not four,
            // and it reads as cloth rather than polka dots up close.
            let mut band: f64 = (v * 1.6).floor();
            let mut diag: f64 = (if ((((band) as i32) & 1) == 0) {
                (arc + v)
            } else {
                (arc - v)
            });
            let mut fleck: f64 = (0.03 * (diag * 11.0).sin());
            cr = (BeanPainter::COAT_R + fleck);
            cg = (BeanPainter::COAT_G + fleck);
            cb = (BeanPainter::COAT_B + (fleck * 0.6));
            spec = 0.05;
            shine = 6.0;
            // A breast pocket: a flap with a dark slit under it, on the far side of the chest.
            let mut pocket: f64 = self.cov(rt::max_f64(
                ((v - 17.6).abs() - 0.09),
                rt::max_f64((-2.0 - arc), (arc + 1.1)),
            ));
            BeanPainter::mix(
                rt::mref(&mut cr),
                rt::mref(&mut cg),
                rt::mref(&mut cb),
                rt::mref(&mut spec),
                rt::mref(&mut shine),
                (BeanPainter::COAT_R * 0.7),
                (BeanPainter::COAT_G * 0.7),
                (BeanPainter::COAT_B * 0.7),
                0.05,
                6.0,
                pocket,
            );
            let mut flap: f64 = self.cov(rt::max_f64(
                ((v - 18.05).abs() - 0.14),
                rt::max_f64((-1.95 - arc), (arc + 1.15)),
            ));
            BeanPainter::mix(
                rt::mref(&mut cr),
                rt::mref(&mut cg),
                rt::mref(&mut cb),
                rt::mref(&mut spec),
                rt::mref(&mut shine),
                (BeanPainter::COAT_R * 0.88),
                (BeanPainter::COAT_G * 0.88),
                (BeanPainter::COAT_B * 0.88),
                0.05,
                6.0,
                flap,
            );
            // Jacket front buttons below the V, down the centre front.
            if v <= (BeanPainter::V_BOTTOM - 0.4) {
                let mut b1x: f64 = a;
                let mut b1y: f64 = (v - 13.55);
                let mut button1: f64 = self.cov((((b1x * b1x) + (b1y * b1y)).sqrt() - 0.17));
                BeanPainter::mix(
                    rt::mref(&mut cr),
                    rt::mref(&mut cg),
                    rt::mref(&mut cb),
                    rt::mref(&mut spec),
                    rt::mref(&mut shine),
                    0.12,
                    0.09,
                    0.07,
                    0.5,
                    20.0,
                    button1,
                );
                let mut b2x: f64 = a;
                let mut b2y: f64 = (v - 14.05);
                let mut button2: f64 = self.cov((((b2x * b2x) + (b2y * b2y)).sqrt() - 0.15));
                BeanPainter::mix(
                    rt::mref(&mut cr),
                    rt::mref(&mut cg),
                    rt::mref(&mut cb),
                    rt::mref(&mut spec),
                    rt::mref(&mut shine),
                    0.12,
                    0.09,
                    0.07,
                    0.5,
                    20.0,
                    button2,
                );
            }
            if v > (BeanPainter::V_BOTTOM - 0.4) {
                // The jacket open in a V over the shirt, the lapels either
                // side of it, the collar spreading at the top.
                let mut hw: f64 = (if (v < BeanPainter::COLLAR_V) {
                    (0.3 + (0.95
                        * rt::min_f64(
                            1.0,
                            ((v - BeanPainter::V_BOTTOM)
                                / ((BeanPainter::COLLAR_V - BeanPainter::V_BOTTOM) - 0.2)),
                        )))
                } else {
                    (1.25 + ((v - BeanPainter::COLLAR_V) * 1.5))
                });
                let mut lapel: f64 = self.cov(rt::max_f64((hw - a), ((a - hw) - 0.6)));
                BeanPainter::mix(
                    rt::mref(&mut cr),
                    rt::mref(&mut cg),
                    rt::mref(&mut cb),
                    rt::mref(&mut spec),
                    rt::mref(&mut shine),
                    (BeanPainter::COAT_R * 0.82),
                    (BeanPainter::COAT_G * 0.82),
                    (BeanPainter::COAT_B * 0.82),
                    0.05,
                    6.0,
                    lapel,
                );
                // Lapel topstitch: a light edge where lapel meets jacket.
                let mut edge: f64 = self.cov((((a - hw) - 0.32).abs() - 0.045));
                BeanPainter::mix(
                    rt::mref(&mut cr),
                    rt::mref(&mut cg),
                    rt::mref(&mut cb),
                    rt::mref(&mut spec),
                    rt::mref(&mut shine),
                    (BeanPainter::COAT_R * 1.12),
                    (BeanPainter::COAT_G * 1.12),
                    (BeanPainter::COAT_B * 1.1),
                    0.08,
                    8.0,
                    (edge * lapel),
                );
                let mut shirt: f64 = self.cov((a - hw));
                BeanPainter::mix(
                    rt::mref(&mut cr),
                    rt::mref(&mut cg),
                    rt::mref(&mut cb),
                    rt::mref(&mut spec),
                    rt::mref(&mut shine),
                    BeanPainter::SHIRT_R,
                    BeanPainter::SHIRT_G,
                    BeanPainter::SHIRT_B,
                    0.1,
                    10.0,
                    shirt,
                );
                // The tie: thin, pointed at the bottom, a knot at the top, faint weave.
                let mut tie_hw: f64 = (if (v < (BeanPainter::V_BOTTOM + 0.6)) {
                    ((0.38 * (v - BeanPainter::V_BOTTOM)) / 0.6)
                } else {
                    (if (v > (BeanPainter::COLLAR_V - 0.5)) {
                        0.52
                    } else {
                        0.38
                    })
                });
                let mut tie: f64 = self.cov(rt::max_f64(
                    (a - tie_hw),
                    rt::max_f64(
                        (BeanPainter::V_BOTTOM - v),
                        ((v - BeanPainter::COLLAR_V) - 0.5),
                    ),
                ));
                let mut knot: f64 = (if (v > (BeanPainter::COLLAR_V - 0.5)) {
                    0.85
                } else {
                    1.0
                });
                BeanPainter::mix(
                    rt::mref(&mut cr),
                    rt::mref(&mut cg),
                    rt::mref(&mut cb),
                    rt::mref(&mut spec),
                    rt::mref(&mut shine),
                    (BeanPainter::TIE_R * knot),
                    (BeanPainter::TIE_G * knot),
                    (BeanPainter::TIE_B * knot),
                    0.3,
                    14.0,
                    tie,
                );
                let mut weave: f64 = (0.5 + (0.5 * (((arc * 5.0) + (v * 7.0)) * 2.0).sin()));
                let mut stripe: f64 = (tie * (0.10 + (0.10 * weave)));
                cr -= ((cr * stripe) * 0.4);
                cg -= ((cg * stripe) * 0.4);
                cb -= ((cb * stripe) * 0.2);
                // A button where the lapels meet.
                let mut bx: f64 = (a - 0.75);
                let mut by: f64 = ((v - BeanPainter::V_BOTTOM) + 0.1);
                let mut button: f64 = self.cov((((bx * bx) + (by * by)).sqrt() - 0.17));
                BeanPainter::mix(
                    rt::mref(&mut cr),
                    rt::mref(&mut cg),
                    rt::mref(&mut cb),
                    rt::mref(&mut spec),
                    rt::mref(&mut shine),
                    0.12,
                    0.09,
                    0.07,
                    0.5,
                    20.0,
                    button,
                );
            }
        }
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
    // The neck, with the shirt collar round its base.
    pub fn neck(mut self, mut u: f64, mut v: f64) {
        if (((u).abs() > (BeanPainter::NECK_R + 0.5)) || (v < (BeanPainter::NECK_BOTTOM - 0.3)))
            || (v > (BeanPainter::NECK_TOP + 0.3))
        {
            return;
        }
        self.tube(
            u,
            v,
            0.0,
            BeanPainter::NECK_BOTTOM,
            0.0,
            BeanPainter::COLLAR_TOP,
            (BeanPainter::NECK_R + 0.12),
            BeanPainter::SHIRT_R,
            BeanPainter::SHIRT_G,
            BeanPainter::SHIRT_B,
            0.1,
            10.0,
        );
        let mut qx: f64 = <f64>::default();
        let mut qy: f64 = <f64>::default();
        let mut t: f64 = <f64>::default();
        let mut dist: f64 = BeanPainter::seg(
            u,
            v,
            0.0,
            BeanPainter::COLLAR_TOP,
            0.0,
            BeanPainter::NECK_TOP,
            rt::mref(&mut qx),
            rt::mref(&mut qy),
            rt::mref(&mut t),
        );
        let mut cov: f64 = rt::min_f64(
            self.cov((dist - BeanPainter::NECK_R)),
            self.cov((BeanPainter::COLLAR_TOP - v)),
        ); // cut flush on the collar
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        BeanPainter::normal(
            (u - qx),
            (v - qy),
            BeanPainter::NECK_R,
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        let mut lr: f64 = <f64>::default();
        let mut lg: f64 = <f64>::default();
        let mut lb: f64 = <f64>::default();
        self.lit(
            (BeanPainter::SKIN_R * 0.92),
            (BeanPainter::SKIN_G * 0.9),
            (BeanPainter::SKIN_B * 0.9),
            nx,
            ny,
            nz,
            0.2,
            16.0,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.over(lr, lg, lb, cov);
    }
    pub fn ear(mut self, mut u: f64, mut v: f64, mut i: i32) {
        let mut dx: f64 = (u - (*self.ear_u.at(i)));
        let mut dy: f64 = (v - BeanPainter::EAR_V);
        let mut reach: f64 = ((BeanPainter::EAR_R + 0.08) + self.px);
        if (((dx > reach) || (dx < -reach)) || (dy > reach)) || (dy < -reach) {
            return;
        }
        let mut d: f64 = ((dx * dx) + (dy * dy)).sqrt();
        let mut cov: f64 = self.cov((d - (BeanPainter::EAR_R + 0.08)));
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        BeanPainter::normal(
            dx,
            dy,
            (BeanPainter::EAR_R + 0.08),
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        let mut lr: f64 = <f64>::default();
        let mut lg: f64 = <f64>::default();
        let mut lb: f64 = <f64>::default();
        self.lit(
            (BeanPainter::SKIN_R * 0.96),
            (BeanPainter::SKIN_G * 0.9),
            (BeanPainter::SKIN_B * 0.88),
            nx,
            ny,
            nz,
            0.25,
            18.0,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.over(lr, lg, lb, cov);
        // The hollow, a little towards the outside, with a rim ridge.
        let mut ox: f64 = (dx
            - (if ((*self.ear_u.at(i)) > 0.0) {
                0.1
            } else {
                -0.1
            }));
        let mut oy: f64 = (dy - 0.02);
        self.over(
            (BeanPainter::SKIN_R * 0.6),
            (BeanPainter::SKIN_G * 0.5),
            (BeanPainter::SKIN_B * 0.45),
            ((self.cov((((ox * ox) + (oy * oy)).sqrt() - 0.22)) * 0.6) * cov),
        );
        self.over(
            0.75,
            0.58,
            0.48,
            ((self.cov(((((ox * ox) + (oy * oy)).sqrt() - 0.34).abs() - 0.05)) * 0.5) * cov),
        );
    }
    // Signed distance to an ellipse of radii rx, ry, first-order exact.
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
    // The head: a long skin-coloured egg, and over it the hair - a
    // second ellipsoid standing a little proud of the skull, kept only
    // where its surface faces up and back: a high hairline across the
    // forehead, lower at the temples, sideburns in front of the ears,
    // swept across to one side from a parting, down to the nape behind.
    pub fn head(mut self, mut u: f64, mut v: f64, mut dy: f64) {
        let mut cov: f64 = self.cov(BeanPainter::ellipse(
            u,
            dy,
            BeanPainter::HEAD_RX,
            BeanPainter::HEAD_RY,
        ));
        if cov > 0.0 {
            let mut nx: f64 = <f64>::default();
            let mut ny: f64 = <f64>::default();
            let mut nz: f64 = <f64>::default();
            BeanPainter::normal(
                ((u / BeanPainter::HEAD_RX) * 3.0),
                ((dy / BeanPainter::HEAD_RY) * 3.0),
                3.0,
                rt::mref(&mut nx),
                rt::mref(&mut ny),
                rt::mref(&mut nz),
            );
            let mut lr: f64 = <f64>::default();
            let mut lg: f64 = <f64>::default();
            let mut lb: f64 = <f64>::default();
            self.lit(
                BeanPainter::SKIN_R,
                BeanPainter::SKIN_G,
                BeanPainter::SKIN_B,
                nx,
                ny,
                nz,
                0.25,
                18.0,
                rt::mref(&mut lr),
                rt::mref(&mut lg),
                rt::mref(&mut lb),
            );
            self.over(lr, lg, lb, cov);
        }
        let mut hdy: f64 = (v - BeanPainter::HAIR_V);
        let mut hair_cov: f64 = self.cov(BeanPainter::ellipse(
            u,
            hdy,
            BeanPainter::HAIR_RX,
            BeanPainter::HAIR_RY,
        ));
        if hair_cov <= 0.0 {
            return;
        }
        let mut mx: f64 = <f64>::default();
        let mut my: f64 = <f64>::default();
        let mut mz: f64 = <f64>::default();
        BeanPainter::normal(
            ((u / BeanPainter::HAIR_RX) * 3.0),
            ((hdy / BeanPainter::HAIR_RY) * 3.0),
            3.0,
            rt::mref(&mut mx),
            rt::mref(&mut my),
            rt::mref(&mut mz),
        );
        // The normal in the head's own frame: hz towards the face.
        let mut hx: f64 = ((mx * self.cos_h) - (mz * self.sin_h));
        let mut hz: f64 = ((mx * self.sin_h) + (mz * self.cos_h));
        let mut hy: f64 = my;
        let mut line: f64 = (0.10 + (0.52 * hz));
        line += ((0.10 * rt::max_f64(0.0, hz)) * (hx).abs()); // receding temples: higher at the corners
        line -= ((0.18 * rt::max_f64(0.0, -hx)) * rt::max_f64(0.0, hz)); // swept down over one side of the forehead
        if (((hx).abs() > 0.75) && (hz > 0.0)) && (hz < 0.5) {
            line = rt::min_f64(line, -0.15); // sideburns
        }
        let mut keep: f64 = self.cov((((line - hy) * BeanPainter::HAIR_RY) * 0.8));
        hair_cov *= keep;
        if hair_cov <= 0.0 {
            return;
        }
        let mut hr: f64 = BeanPainter::HAIR_R;
        let mut hg: f64 = BeanPainter::HAIR_G;
        let mut hb: f64 = BeanPainter::HAIR_B;
        // Combed strands: faint darker grooves running back from the hairline.
        let mut strand: f64 = (0.5 + (0.5 * (((hx * 22.0) + (hy * 6.0)) + (hz * 4.0)).sin()));
        hr *= (1.0 - ((0.12 * strand) * hair_cov));
        hg *= (1.0 - ((0.12 * strand) * hair_cov));
        hb *= (1.0 - ((0.10 * strand) * hair_cov));
        // The parting: a thin lighter line running back from the hairline.
        let mut part: f64 = ((self.cov(((hx - 0.32).abs() - 0.035))
            * self.cov(((line + 0.05) - hy)))
            * (if (hz > -0.2) { 0.65 } else { 0.0 }));
        hr += ((0.42 - hr) * part);
        hg += ((0.32 - hg) * part);
        hb += ((0.24 - hb) * part);
        let mut r: f64 = <f64>::default();
        let mut g: f64 = <f64>::default();
        let mut b: f64 = <f64>::default();
        self.lit(
            hr,
            hg,
            hb,
            mx,
            my,
            mz,
            0.35,
            12.0,
            rt::mref(&mut r),
            rt::mref(&mut g),
            rt::mref(&mut b),
        );
        self.over(r, g, b, hair_cov);
    }
    pub fn eye(mut self, mut u: f64, mut v: f64, mut i: i32) {
        let mut vis: f64 = (*self.eye_vis.at(i));
        if vis <= 0.0 {
            return;
        }
        let mut k: f64 = (*self.eye_k.at(i));
        let mut near: bool = (i != self.first_eye);
        // The eye's own frame: squeezed as it turns away, shifted down
        // and forward as the head nods.
        let mut fu: f64 = ((*self.eye_u.at(i)) + ((self.dip * 0.5) * self.sin_h));
        let mut fv: f64 = (BeanPainter::EYE_V - (self.dip * 0.8));
        let mut dx: f64 = ((u - fu) / k);
        let mut dy: f64 = (v - fv);
        if ((dx).abs() > (BeanPainter::EYE_RX + 0.5)) || ((dy).abs() > (BeanPainter::EYE_RY + 0.5))
        {
            return;
        }
        let mut ex: f64 = (dx / BeanPainter::EYE_RX);
        let mut ey: f64 = (dy / BeanPainter::EYE_RY);
        let mut white: f64 = ((((ex * ex) + (ey * ey)).sqrt() - 1.0) * BeanPainter::EYE_RY);
        let mut cov_white: f64 = (self.cov(white) * vis);
        if cov_white <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        BeanPainter::normal(
            dx,
            dy,
            (BeanPainter::EYE_RX * 1.6),
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        BeanPainter::turn_2(
            rt::mref(&mut nx),
            rt::mref(&mut nz),
            (*self.eye_cos.at(i)),
            (*self.eye_sin.at(i)),
        );
        let mut r: f64 = <f64>::default();
        let mut g: f64 = <f64>::default();
        let mut b: f64 = <f64>::default();
        self.lit(
            0.96,
            0.95,
            0.92,
            nx,
            ny,
            nz,
            0.4,
            24.0,
            rt::mref(&mut r),
            rt::mref(&mut g),
            rt::mref(&mut b),
        );
        // The iris follows the gaze, kept inside the white.
        let mut ix: f64 = (dx - (*self.gaze_u.at(i)));
        let mut iy: f64 = (dy - (*self.gaze_v.at(i)));
        let mut id: f64 = ((ix * ix) + (iy * iy)).sqrt();
        let mut cov_iris: f64 = self.cov(BeanPainter::squeezed(ix, iy, k, BeanPainter::IRIS_R));
        if cov_iris > 0.0 {
            let mut tt: f64 = (id / BeanPainter::IRIS_R);
            let mut shade: f64 = (1.25 - (0.55 * tt));
            if tt > 0.82 {
                shade *= (1.0 - ((0.55 * (tt - 0.82)) / 0.18));
            }
            r += (((0.30 * shade) - r) * cov_iris);
            g += (((0.17 * shade) - g) * cov_iris);
            b += (((0.08 * shade) - b) * cov_iris);
            let mut cov_pupil: f64 = self.cov(BeanPainter::squeezed(
                ix,
                iy,
                k,
                ((BeanPainter::IRIS_R * 0.5) * self.face.pupil_size),
            ));
            r += ((0.05 - r) * cov_pupil);
            g += ((0.04 - g) * cov_pupil);
            b += ((0.04 - b) * cov_pupil);
            let mut lx: f64 = (ix + ((self.flip * 0.35) * BeanPainter::IRIS_R));
            let mut ly: f64 = (iy - (0.35 * BeanPainter::IRIS_R));
            let mut cov_light: f64 = (self.cov(BeanPainter::squeezed(
                lx,
                ly,
                k,
                (BeanPainter::IRIS_R * 0.25),
            )) * 0.9);
            r += ((1.0 - r) * cov_light);
            g += ((1.0 - g) * cov_light);
            b += ((1.0 - b) * cov_light);
        }
        // Lids: skin sliding down (and up) over the eye.
        let mut upper: f64 = rt::max_f64(self.face.lid, (if near { self.face.wink } else { 0.0 }));
        let mut lower: f64 = self.face.squint;
        let mut s: f64 = (ex * ex);
        let mut upper_edge: f64 =
            ((fv + (BeanPainter::EYE_RY * (1.0 - (2.0 * upper)))) - ((0.25 * upper) * s));
        let mut lower_edge: f64 =
            ((fv - (BeanPainter::EYE_RY * (1.0 - (2.0 * lower)))) + ((0.25 * lower) * s));
        if (upper > 0.01) || (lower > 0.01) {
            let mut bx: f64 = <f64>::default();
            let mut by: f64 = <f64>::default();
            let mut bz: f64 = <f64>::default();
            BeanPainter::normal(
                ((u / BeanPainter::HEAD_RX) * 3.0),
                (((v - BeanPainter::HEAD_V) / BeanPainter::HEAD_RY) * 3.0),
                3.0,
                rt::mref(&mut bx),
                rt::mref(&mut by),
                rt::mref(&mut bz),
            );
            let mut sr: f64 = <f64>::default();
            let mut sg: f64 = <f64>::default();
            let mut sb: f64 = <f64>::default();
            self.lit(
                BeanPainter::SKIN_R,
                BeanPainter::SKIN_G,
                BeanPainter::SKIN_B,
                bx,
                by,
                bz,
                0.25,
                18.0,
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
            r += ((0.2 - r) * lash);
            g += ((0.12 - g) * lash);
            b += ((0.09 - b) * lash);
        }
        self.over(r, g, b, cov_white);
        // The rim of the eye: a fine dark line, heavier along the top;
        // a crease above it where the lid folds, and a shadow under it.
        let mut rim: f64 =
            ((self.cov(((white).abs() - 0.07)) * (if (dy > 0.1) { 0.85 } else { 0.4 })) * vis);
        self.over(0.22, 0.13, 0.1, rim);
        let mut crease: f64 = BeanPainter::ellipse(
            dx,
            (dy - 0.2),
            (BeanPainter::EYE_RX + 0.12),
            (BeanPainter::EYE_RY + 0.12),
        );
        self.over(
            0.60,
            0.42,
            0.32,
            ((self.cov(((crease).abs() - 0.05)) * (if (dy > 0.3) { 0.7 } else { 0.0 })) * vis),
        );
        let mut bag: f64 = BeanPainter::ellipse(
            dx,
            (dy + 0.12),
            (BeanPainter::EYE_RX + 0.05),
            (BeanPainter::EYE_RY + 0.1),
        );
        self.over(
            0.70,
            0.52,
            0.42,
            ((self.cov(((bag).abs() - 0.05)) * (if (dy < -0.35) { 0.6 } else { 0.0 })) * vis),
        );
        // Tired lower bags: a second, fainter fold under each eye.
        let mut bag2: f64 = BeanPainter::ellipse(
            dx,
            (dy + 0.32),
            (BeanPainter::EYE_RX + 0.02),
            (BeanPainter::EYE_RY + 0.12),
        );
        self.over(
            0.74,
            0.57,
            0.47,
            ((self.cov(((bag2).abs() - 0.045)) * (if (dy < -0.5) { 0.4 } else { 0.0 })) * vis),
        );
    }
    // Heavy dark brows above each eye, raised, angled and lifted one at
    // a time as the face says.
    pub fn brows(mut self, mut u: f64, mut v: f64) {
        let mut base_v: f64 =
            ((BeanPainter::BROW_V - (self.dip * 0.8)) + (self.face.brow_raise * 0.35));
        if (v - base_v).abs() > 1.3 {
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
        // Thick and dark, a little arched: heaviest over the inner half,
        // tapering to a point at the temple.
        let mut ex: f64 = (((BeanPainter::HEAD_RX + 0.15) * (*self.eye_sin.at(i)))
            + ((self.dip * 0.5) * self.sin_h));
        let mut ax: f64 = (ex - ((outward * 1.0) * k));
        let mut ay: f64 = ((cy - (self.face.brow_angle * 0.4)) - 0.08);
        let mut mx: f64 = (ex + ((outward * 0.15) * k));
        let mut my: f64 = (cy + 0.12);
        let mut bx: f64 = (ex + ((outward * 1.05) * k));
        let mut by: f64 = ((cy + (self.face.brow_angle * 0.4)) - 0.1);
        let mut qx: f64 = <f64>::default();
        let mut qy: f64 = <f64>::default();
        let mut t: f64 = <f64>::default();
        let mut d1: f64 = (BeanPainter::seg(
            u,
            v,
            ax,
            ay,
            mx,
            my,
            rt::mref(&mut qx),
            rt::mref(&mut qy),
            rt::mref(&mut t),
        ) - (0.38 + (0.07 * t)));
        let mut d2: f64 = (BeanPainter::seg(
            u,
            v,
            mx,
            my,
            bx,
            by,
            rt::mref(&mut qx),
            rt::mref(&mut qy),
            rt::mref(&mut t),
        ) - (0.44 - (0.32 * t)));
        let mut cov: f64 = (self.cov(rt::min_f64(d1, d2)) * show);
        self.over(0.11, 0.07, 0.055, cov);
        // Bushy lower edge: a softer, lighter fringe under the main mass.
        let mut fringe: f64 =
            ((self.cov(rt::min_f64((d1 + 0.22), (d2 + 0.22))) * (1.0 - cov)) * show);
        self.over(0.23, 0.15, 0.11, (fringe * 0.6));
    }
    // A long nose: a tube from the bridge to the tip, standing out from
    // the face, so it is a bump from the front and a profile from the side.
    pub fn nose(mut self, mut u: f64, mut v: f64) {
        let mut vis: f64 = (if (self.cos_h > -0.3) {
            1.0
        } else {
            (if (self.cos_h < -0.6) {
                0.0
            } else {
                ((self.cos_h + 0.6) / 0.3)
            })
        });
        if vis <= 0.0 {
            return;
        }
        let mut shift: f64 = ((self.dip * 0.5) * self.sin_h);
        let mut bu: f64 = (((BeanPainter::HEAD_RX + 0.1) * self.sin_h) + shift);
        let mut bv: f64 = (BeanPainter::NOSE_TOP_V - (self.dip * 0.7));
        let mut tu: f64 = (((BeanPainter::HEAD_RX + 0.9) * self.sin_h) + shift);
        let mut tv: f64 = (BeanPainter::NOSE_TIP_V - (self.dip * 0.9));
        if (((u - tu).abs() > 1.4) || (v > (bv + 0.6))) || (v < (tv - 0.9)) {
            return;
        }
        let mut qx: f64 = <f64>::default();
        let mut qy: f64 = <f64>::default();
        let mut t: f64 = <f64>::default();
        let mut dist: f64 = BeanPainter::seg(
            u,
            v,
            bu,
            bv,
            tu,
            tv,
            rt::mref(&mut qx),
            rt::mref(&mut qy),
            rt::mref(&mut t),
        );
        let mut radius: f64 = (0.27 + ((0.30 * t) * t)); // narrow at the bridge, bulbous at the tip
        let mut cov: f64 = (self.cov((dist - radius)) * vis);
        if cov > 0.0 {
            let mut nx: f64 = <f64>::default();
            let mut ny: f64 = <f64>::default();
            let mut nz: f64 = <f64>::default();
            BeanPainter::normal(
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
                (BeanPainter::SKIN_R * 0.98),
                (BeanPainter::SKIN_G * 0.88),
                (BeanPainter::SKIN_B * 0.84),
                nx,
                ny,
                nz,
                0.4,
                20.0,
                rt::mref(&mut lr),
                rt::mref(&mut lg),
                rt::mref(&mut lb),
            );
            self.over(lr, lg, lb, cov);
            // The shaded side of the bridge, so it stands off the face from the front.
            self.over(
                0.6,
                0.4,
                0.3,
                (((cov * 0.35) * rt::max_f64(0.0, (((u - qx) * self.flip) / radius))) * (1.0 - t)),
            );
            // A small highlight on the tip.
            let mut hx0: f64 = ((u - tu) + (0.12 * self.flip));
            let mut hy0: f64 = ((v - tv) - 0.12);
            self.over(
                1.0,
                0.9,
                0.82,
                ((self.cov((((hx0 * hx0) + (hy0 * hy0)).sqrt() - 0.12)) * 0.5) * vis),
            );
        }
        // The nostrils' shadows under the tip.
        let mut sx: f64 = (((u - tu) - (0.3 * self.cos_h)) + (0.1 * self.sin_h));
        let mut sy: f64 = ((v - tv) + 0.36);
        self.over(
            0.48,
            0.26,
            0.19,
            ((self.cov((((sx * sx) + (sy * sy)).sqrt() - 0.17)) * 0.6) * vis),
        );
        sx = (((u - tu) + (0.3 * self.cos_h)) + (0.1 * self.sin_h));
        self.over(
            0.48,
            0.26,
            0.19,
            ((self.cov((((sx * sx) + (sy * sy)).sqrt() - 0.17)) * 0.6) * vis),
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
        let mut mu: f64 =
            ((self.mouth_u + ((self.face.mouth_dx * 0.5) * k)) + ((self.dip * 0.5) * self.sin_h));
        let mut mv: f64 = (BeanPainter::MOUTH_V - (self.dip * 0.9));
        let mut w: f64 = ((self.face.mouth_width * 0.88) * k);
        let mut hx: f64 = ((u - mu).abs() - w);
        if (hx > 0.6) || ((v - mv).abs() > 2.5) {
            return;
        }
        let mut t: f64 = ((u - mu) / w);
        if t < -1.0 {
            t = -1.0;
        } else if t > 1.0 {
            t = 1.0;
        }
        let mut line: f64 = ((mv + ((self.face.mouth_curve * 0.55) * ((t * t) - 0.45)))
            + ((self.face.wobble * 0.12) * (t * 8.5).sin()));
        let mut open: f64 = self.face.mouth_open;
        let mut lip: f64 = 0.19;
        if open < 0.03 {
            let mut ox: f64 = (if (hx > 0.0) { hx } else { 0.0 });
            let mut oy: f64 = (v - line);
            let mut cov: f64 = self.cov((((ox * ox) + (oy * oy)).sqrt() - (lip / 2.0)));
            self.over(0.45, 0.2, 0.18, (cov * vis));
            // A thin lower lip catching the light under the line.
            self.over(
                0.9,
                0.62,
                0.55,
                ((self.cov((((ox * ox) + ((oy + 0.22) * (oy + 0.22))).sqrt() - 0.1)) * 0.5) * vis),
            );
            // Chin crease: a short shadow under the lip so the chin reads.
            let mut chx: f64 = (u - mu);
            let mut chy: f64 = ((v - line) + 0.62);
            self.over(
                0.68,
                0.50,
                0.42,
                ((self.cov(((((chx * chx) * 0.6) + (chy * chy)).sqrt() - 0.12)) * 0.45) * vis),
            );
            return;
        }
        let mut gape: f64 = ((open * 1.5) * rt::pow((1.0 - (t * t)), 0.6));
        let mut upper: f64 = line;
        let mut lower: f64 = (line - gape);
        let mut d: f64 = rt::max_f64(rt::max_f64((v - upper), (lower - v)), hx);
        let mut inside: f64 = self.cov(d);
        if inside > 0.0 {
            self.over(0.28, 0.08, 0.09, (inside * vis));
            let mut teeth_h: f64 = ((open * 0.5) * self.face.teeth);
            if teeth_h > 0.04 {
                let mut teeth: f64 = (self.cov(rt::max_f64(((upper - teeth_h) - v), d)) * inside);
                let mut gap: f64 = (((((u - mu) % 0.42) + 0.42) % 0.42) - 0.21).abs();
                let mut seam: f64 = (self.cov((gap - 0.025)) * 0.35);
                self.over(
                    (0.95 - (seam * 0.5)),
                    (0.94 - (seam * 0.5)),
                    (0.90 - (seam * 0.5)),
                    (teeth * vis),
                );
            }
        }
        let mut outline: f64 = self.cov(((d).abs() - (lip / 2.0)));
        self.over(0.45, 0.2, 0.18, (outline * vis));
        if self.face.tongue > 0.02 {
            let mut tongue: f64 = self.face.tongue;
            let mut rx: f64 = rt::min_f64((w * 0.55), 0.55);
            let mut ry: f64 = (0.22 + (0.35 * tongue));
            let mut cy: f64 = ((lower + 0.22) - (0.55 * tongue));
            let mut tx: f64 = ((u - mu) / rx);
            let mut ty: f64 = ((v - cy) / ry);
            let mut e: f64 = ((tx * tx) + (ty * ty)).sqrt();
            let mut rim: f64 = ((e - 1.0) * ry);
            let mut td: f64 = rt::max_f64(rim, ((v - upper) + 0.1));
            let mut cov: f64 = self.cov(td);
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
            self.over(
                0.45,
                0.2,
                0.18,
                (((self.cov(((rim).abs() - 0.07)) * self.cov(((v - upper) + 0.1))) * 0.7) * vis),
            );
        }
    }
    //
    // One arm: the sleeve in two tubes with an elbow patch, the shirt
    // cuff, and the hand, each sorted against the body pixel by pixel.
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
            BeanPainter::SHOULDER_V,
            (*self.sh_z.at(i)),
            (*self.elb_u.at(i)),
            (*self.elb_v.at(i)),
            (*self.elb_z.at(i)),
            BeanPainter::SLEEVE_R,
            BeanPainter::COAT_R,
            BeanPainter::COAT_G,
            BeanPainter::COAT_B,
            0.05,
            6.0,
        );
        self.arm_tube(
            u,
            v,
            (*self.elb_u.at(i)),
            (*self.elb_v.at(i)),
            (*self.elb_z.at(i)),
            (*self.cuff_u.at(i)),
            (*self.cuff_v.at(i)),
            (*self.cuff_z.at(i)),
            BeanPainter::FORE_R,
            BeanPainter::COAT_R,
            BeanPainter::COAT_G,
            BeanPainter::COAT_B,
            0.05,
            6.0,
        );
        // The elbow patch: a darker disc of suede on the sleeve.
        let mut ex: f64 = (u - (*self.elb_u.at(i)));
        let mut ey: f64 = (v - (*self.elb_v.at(i)));
        let mut patch: f64 = self.cov((((ex * ex) + (ey * ey)).sqrt() - BeanPainter::PATCH_R));
        if patch > 0.0 {
            let mut nx: f64 = <f64>::default();
            let mut ny: f64 = <f64>::default();
            let mut nz: f64 = <f64>::default();
            BeanPainter::normal(
                ex,
                ey,
                (BeanPainter::SLEEVE_R * 1.2),
                rt::mref(&mut nx),
                rt::mref(&mut ny),
                rt::mref(&mut nz),
            );
            let mut k: f64 = self.depth((*self.elb_z.at(i)));
            let mut lr: f64 = <f64>::default();
            let mut lg: f64 = <f64>::default();
            let mut lb: f64 = <f64>::default();
            self.lit(
                (0.38 * k),
                (0.27 * k),
                (0.17 * k),
                nx,
                ny,
                nz,
                0.04,
                5.0,
                rt::mref(&mut lr),
                rt::mref(&mut lg),
                rt::mref(&mut lb),
            );
            self.sort(
                lr,
                lg,
                lb,
                patch,
                ((*self.elb_z.at(i)) + (nz * BeanPainter::SLEEVE_R)),
            );
        }
        self.arm_tube(
            u,
            v,
            (*self.cuff_u.at(i)),
            (*self.cuff_v.at(i)),
            (*self.cuff_z.at(i)),
            (*self.hand_u.at(i)),
            (*self.hand_v.at(i)),
            (*self.hand_z.at(i)),
            (BeanPainter::FORE_R + 0.04),
            BeanPainter::SHIRT_R,
            BeanPainter::SHIRT_G,
            BeanPainter::SHIRT_B,
            0.1,
            10.0,
        );
        // The black digital watch on the near wrist: a dark band over the cuff.
        if i == 1 {
            let mut wx: f64 = (((*self.cuff_u.at(i)) + (*self.hand_u.at(i))) / 2.0);
            let mut wy: f64 = (((*self.cuff_v.at(i)) + (*self.hand_v.at(i))) / 2.0);
            let mut wz: f64 = (((*self.cuff_z.at(i)) + (*self.hand_z.at(i))) / 2.0);
            let mut ax: f64 = ((*self.hand_u.at(i)) - (*self.cuff_u.at(i)));
            let mut ay: f64 = ((*self.hand_v.at(i)) - (*self.cuff_v.at(i)));
            let mut len: f64 = ((ax * ax) + (ay * ay)).sqrt();
            if len > 1e-6 {
                ax /= len;
                ay /= len;
                self.arm_tube(
                    u,
                    v,
                    (wx - (ax * 0.12)),
                    (wy - (ay * 0.12)),
                    wz,
                    (wx + (ax * 0.12)),
                    (wy + (ay * 0.12)),
                    wz,
                    (BeanPainter::FORE_R + 0.1),
                    0.10,
                    0.10,
                    0.11,
                    0.3,
                    14.0,
                );
            }
        }
        // Pointing: the index finger carries on past the hand.
        if (i == 1) && self.pose.point_near {
            let mut fx: f64 = ((*self.hand_u.at(i)) - (*self.cuff_u.at(i)));
            let mut fy: f64 = ((*self.hand_v.at(i)) - (*self.cuff_v.at(i)));
            let mut len: f64 = ((fx * fx) + (fy * fy)).sqrt();
            if len > 1e-6 {
                let mut reach: f64 = ((BeanPainter::HAND_R + BeanPainter::FINGER_LEN) / len);
                self.arm_tube(
                    u,
                    v,
                    (*self.hand_u.at(i)),
                    (*self.hand_v.at(i)),
                    (*self.hand_z.at(i)),
                    ((*self.hand_u.at(i)) + (fx * reach)),
                    ((*self.hand_v.at(i)) + (fy * reach)),
                    (*self.hand_z.at(i)),
                    BeanPainter::FINGER_R,
                    BeanPainter::SKIN_R,
                    BeanPainter::SKIN_G,
                    BeanPainter::SKIN_B,
                    0.25,
                    18.0,
                );
            }
        }
        let mut dx: f64 = (u - (*self.hand_u.at(i)));
        let mut dy: f64 = (v - (*self.hand_v.at(i)));
        let mut cov: f64 = self.cov((((dx * dx) + (dy * dy)).sqrt() - BeanPainter::HAND_R));
        if cov <= 0.0 {
            return;
        }
        let mut hnx: f64 = <f64>::default();
        let mut hny: f64 = <f64>::default();
        let mut hnz: f64 = <f64>::default();
        BeanPainter::normal(
            dx,
            dy,
            BeanPainter::HAND_R,
            rt::mref(&mut hnx),
            rt::mref(&mut hny),
            rt::mref(&mut hnz),
        );
        let mut hk: f64 = self.depth((*self.hand_z.at(i)));
        let mut hr: f64 = <f64>::default();
        let mut hg: f64 = <f64>::default();
        let mut hb: f64 = <f64>::default();
        self.lit(
            (BeanPainter::SKIN_R * hk),
            (BeanPainter::SKIN_G * hk),
            (BeanPainter::SKIN_B * hk),
            hnx,
            hny,
            hnz,
            0.25,
            18.0,
            rt::mref(&mut hr),
            rt::mref(&mut hg),
            rt::mref(&mut hb),
        );
        self.sort(
            hr,
            hg,
            hb,
            cov,
            ((*self.hand_z.at(i)) + (hnz * BeanPainter::HAND_R)),
        );
    }
    // Teddy, sitting in the near hand: a round body with a lighter belly,
    // a round head with two ears, a lighter muzzle, button eyes with a
    // glint, a stitched nose and smile, and one paw that waves.
    pub fn teddy(mut self, mut u: f64, mut v: f64) {
        let mut tx: f64 = ((*self.hand_u.at(1)) + 0.2);
        let mut ty: f64 = ((*self.hand_v.at(1)) + 0.55);
        if (((u - tx).abs() > 1.8) || (v < (ty - 1.2))) || (v > (ty + 2.4)) {
            return;
        }
        // The legs dangle below the body.
        self.tube(
            u,
            v,
            (tx - 0.35),
            (ty - 0.5),
            (tx - 0.55),
            (ty - 1.0),
            0.19,
            (BeanPainter::TEDDY_R * 0.92),
            (BeanPainter::TEDDY_G * 0.92),
            (BeanPainter::TEDDY_B * 0.92),
            0.1,
            8.0,
        );
        self.tube(
            u,
            v,
            (tx + 0.35),
            (ty - 0.5),
            (tx + 0.55),
            (ty - 1.0),
            0.19,
            (BeanPainter::TEDDY_R * 0.92),
            (BeanPainter::TEDDY_G * 0.92),
            (BeanPainter::TEDDY_B * 0.92),
            0.1,
            8.0,
        );
        // The paws: one held out, one waving.
        let mut px0: f64 = (tx + 0.55);
        let mut py0: f64 = (ty + 0.3);
        self.tube(
            u,
            v,
            px0,
            py0,
            (px0 + self.paw_u),
            (py0 + self.paw_v),
            0.2,
            BeanPainter::TEDDY_R,
            BeanPainter::TEDDY_G,
            BeanPainter::TEDDY_B,
            0.1,
            8.0,
        );
        self.tube(
            u,
            v,
            (tx - 0.55),
            (ty + 0.1),
            (tx - 1.1),
            (ty - 0.35),
            0.2,
            BeanPainter::TEDDY_R,
            BeanPainter::TEDDY_G,
            BeanPainter::TEDDY_B,
            0.1,
            8.0,
        );
        self.ball(
            u,
            v,
            tx,
            ty,
            0.85,
            BeanPainter::TEDDY_R,
            BeanPainter::TEDDY_G,
            BeanPainter::TEDDY_B,
            0.1,
            8.0,
        );
        // Belly patch.
        {
            let mut bx: f64 = ((u - tx) / 0.55);
            let mut by: f64 = ((v - (ty - 0.1)) / 0.6);
            let mut belly: f64 = ((((bx * bx) + (by * by)).sqrt() - 1.0) * 0.6);
            self.over(0.86, 0.71, 0.50, (self.cov(belly) * 0.9));
        }
        let mut hx: f64 = (tx + 0.05);
        let mut hy: f64 = (ty + 1.3);
        self.ball(
            u,
            v,
            (hx - 0.52),
            (hy + 0.45),
            0.23,
            (BeanPainter::TEDDY_R * 0.9),
            (BeanPainter::TEDDY_G * 0.9),
            (BeanPainter::TEDDY_B * 0.9),
            0.1,
            8.0,
        );
        self.ball(
            u,
            v,
            (hx + 0.52),
            (hy + 0.45),
            0.23,
            (BeanPainter::TEDDY_R * 0.9),
            (BeanPainter::TEDDY_G * 0.9),
            (BeanPainter::TEDDY_B * 0.9),
            0.1,
            8.0,
        );
        // Inner ears.
        {
            let mut ix: f64 = (u - (hx - 0.52));
            let mut iy: f64 = (v - (hy + 0.45));
            self.over(
                0.85,
                0.68,
                0.48,
                self.cov((((ix * ix) + (iy * iy)).sqrt() - 0.11)),
            );
            ix = (u - (hx + 0.52));
            self.over(
                0.85,
                0.68,
                0.48,
                self.cov((((ix * ix) + (iy * iy)).sqrt() - 0.11)),
            );
        }
        self.ball(
            u,
            v,
            hx,
            hy,
            0.65,
            BeanPainter::TEDDY_R,
            BeanPainter::TEDDY_G,
            BeanPainter::TEDDY_B,
            0.1,
            8.0,
        );
        self.ball(
            u,
            v,
            (hx + 0.08),
            (hy - 0.15),
            0.28,
            0.88,
            0.74,
            0.52,
            0.1,
            8.0,
        );
        // Button eyes with a glint, and a stitched nose.
        let mut ex: f64 = ((u - hx) + 0.22);
        let mut ey: f64 = ((v - hy) - 0.15);
        self.over(
            0.1,
            0.07,
            0.06,
            self.cov((((ex * ex) + (ey * ey)).sqrt() - 0.085)),
        );
        self.over(
            1.0,
            1.0,
            1.0,
            (self
                .cov(((((ex + 0.03) * (ex + 0.03)) + ((ey - 0.03) * (ey - 0.03))).sqrt() - 0.025))
                * 0.9),
        );
        ex = ((u - hx) - 0.26);
        self.over(
            0.1,
            0.07,
            0.06,
            self.cov((((ex * ex) + (ey * ey)).sqrt() - 0.085)),
        );
        self.over(
            1.0,
            1.0,
            1.0,
            (self
                .cov(((((ex + 0.03) * (ex + 0.03)) + ((ey - 0.03) * (ey - 0.03))).sqrt() - 0.025))
                * 0.9),
        );
        ex = ((u - hx) - 0.08);
        ey = ((v - hy) + 0.05);
        self.over(
            0.15,
            0.09,
            0.07,
            self.cov((((ex * ex) + (ey * ey)).sqrt() - 0.09)),
        );
        // Stitched smile under the nose.
        {
            let mut smile_y: f64 = (hy - 0.28);
            let mut qx: f64 = <f64>::default();
            let mut qy: f64 = <f64>::default();
            let mut t: f64 = <f64>::default();
            let mut d1: f64 = BeanPainter::seg(
                u,
                v,
                (hx - 0.08),
                smile_y,
                (hx - 0.28),
                (smile_y - 0.14),
                rt::mref(&mut qx),
                rt::mref(&mut qy),
                rt::mref(&mut t),
            );
            let mut d2: f64 = BeanPainter::seg(
                u,
                v,
                (hx - 0.08),
                smile_y,
                (hx + 0.14),
                (smile_y - 0.14),
                rt::mref(&mut qx),
                rt::mref(&mut qy),
                rt::mref(&mut t),
            );
            self.over(
                0.25,
                0.14,
                0.10,
                (self.cov((rt::min_f64(d1, d2) - 0.02)) * 0.8),
            );
        }
    }
    // The Mini's steering wheel, held in both hands: a dark rim with
    // three spokes round a hub, turned by WheelTurn.
    pub fn wheel_prop(mut self, mut u: f64, mut v: f64) {
        let mut wx: f64 = 0.5;
        let mut wy: f64 = 16.2;
        if ((u - wx).abs() > 2.6) || ((v - wy).abs() > 2.6) {
            return;
        }
        let mut dx: f64 = (u - wx);
        let mut dy: f64 = (v - wy);
        let mut ring: f64 = ((((dx * dx) + (dy * dy)).sqrt() - 1.15).abs() - 0.16);
        let mut cov: f64 = self.cov(ring);
        if cov > 0.0 {
            let mut nx: f64 = <f64>::default();
            let mut ny: f64 = <f64>::default();
            let mut nz: f64 = <f64>::default();
            BeanPainter::normal(
                dx,
                dy,
                1.15,
                rt::mref(&mut nx),
                rt::mref(&mut ny),
                rt::mref(&mut nz),
            );
            let mut lr: f64 = <f64>::default();
            let mut lg: f64 = <f64>::default();
            let mut lb: f64 = <f64>::default();
            self.lit(
                0.16,
                0.15,
                0.17,
                nx,
                ny,
                nz,
                0.5,
                20.0,
                rt::mref(&mut lr),
                rt::mref(&mut lg),
                rt::mref(&mut lb),
            );
            self.over(lr, lg, lb, cov);
        }
        {
            let mut s: i32 = 0;
            'l1: while s < 3 {
                let mut bx: f64 = (wx + (*self.spoke_u.at(s)));
                let mut by: f64 = (wy + (*self.spoke_v.at(s)));
                let mut qx: f64 = <f64>::default();
                let mut qy: f64 = <f64>::default();
                let mut t: f64 = <f64>::default();
                let mut spoke: f64 = (BeanPainter::seg(
                    u,
                    v,
                    wx,
                    wy,
                    bx,
                    by,
                    rt::mref(&mut qx),
                    rt::mref(&mut qy),
                    rt::mref(&mut t),
                ) - 0.09);
                self.over(0.22, 0.21, 0.23, (self.cov(spoke) * 0.95));
                s += 1;
            }
        }
        self.ball(u, v, wx, wy, 0.26, 0.25, 0.24, 0.26, 0.5, 20.0);
    }
    // A small box camera held up in both hands, with a lens and a flash
    // that fires white for one beat while pose.Flash is set.
    pub fn camera_prop(mut self, mut u: f64, mut v: f64) {
        let mut cx0: f64 = (((*self.hand_u.at(0)) + (*self.hand_u.at(1))) / 2.0);
        let mut cy0: f64 = ((((*self.hand_v.at(0)) + (*self.hand_v.at(1))) / 2.0) + 0.3);
        if ((u - cx0).abs() > 3.2) || ((v - cy0).abs() > 3.2) {
            return;
        }
        let mut body: f64 = BeanPainter::round_rect((u - cx0), (v - cy0), 0.85, 0.55, 0.16);
        let mut cov_body: f64 = self.cov(body);
        if cov_body > 0.0 {
            let mut nx: f64 = <f64>::default();
            let mut ny: f64 = <f64>::default();
            let mut nz: f64 = <f64>::default();
            BeanPainter::normal(
                (u - cx0),
                (v - cy0),
                0.9,
                rt::mref(&mut nx),
                rt::mref(&mut ny),
                rt::mref(&mut nz),
            );
            let mut lr: f64 = <f64>::default();
            let mut lg: f64 = <f64>::default();
            let mut lb: f64 = <f64>::default();
            self.lit(
                0.18,
                0.17,
                0.19,
                nx,
                ny,
                nz,
                0.55,
                22.0,
                rt::mref(&mut lr),
                rt::mref(&mut lg),
                rt::mref(&mut lb),
            );
            self.over(lr, lg, lb, cov_body);
            // Top strip and shutter button.
            self.over(
                0.32,
                0.31,
                0.33,
                ((self.cov((((v - cy0) - 0.42).abs() - 0.07)) * self.cov(((u - cx0).abs() - 0.7)))
                    * cov_body),
            );
            let mut shx: f64 = (u - (cx0 + 0.55));
            let mut shy: f64 = (v - (cy0 + 0.62));
            self.over(
                0.7,
                0.12,
                0.14,
                self.cov((((shx * shx) + (shy * shy)).sqrt() - 0.11)),
            );
        }
        // Lens: dark glass with a glint.
        {
            let mut lx: f64 = (u - cx0);
            let mut ly: f64 = (v - cy0);
            let mut lens: f64 = (((lx * lx) + (ly * ly)).sqrt() - 0.34);
            self.over(0.05, 0.07, 0.10, self.cov(lens));
            self.over(
                0.45,
                0.6,
                0.85,
                (self.cov(((((lx + 0.1) * (lx + 0.1)) + ((ly - 0.1) * (ly - 0.1))).sqrt() - 0.08))
                    * 0.8),
            );
            self.over(
                1.0,
                1.0,
                1.0,
                (self.cov(
                    ((((lx + 0.12) * (lx + 0.12)) + ((ly - 0.12) * (ly - 0.12))).sqrt() - 0.035),
                ) * 0.9),
            );
        }
        if self.pose.flash {
            let mut fx: f64 = (u - (cx0 - 0.55));
            let mut fy: f64 = (v - (cy0 + 0.25));
            let mut flash: f64 = ((fx * fx) + (fy * fy)).sqrt();
            self.over(1.0, 1.0, 1.0, self.cov((flash - 0.9)));
            self.over(
                1.0,
                1.0,
                1.0,
                (self.cov(((flash - 1.2).abs() - 0.12)) * 0.9),
            );
        }
    }
    // A lit sphere, in front of everything.
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
        BeanPainter::normal(
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
        let mut r: i32 = (((BeanPainter::clamp(self.cr) * 255.0) + 0.5) as i32);
        let mut g: i32 = (((BeanPainter::clamp(self.cg) * 255.0) + 0.5) as i32);
        let mut b: i32 = (((BeanPainter::clamp(self.cb) * 255.0) + 0.5) as i32);
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
            (spec * BeanPainter::pow(h, shine))
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
        BeanPainter::turn_2(&mut *nx, &mut *nz, (a).cos(), (a).sin());
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
        let mut dist: f64 = BeanPainter::seg(
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
        BeanPainter::normal(
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
        let mut dist: f64 = BeanPainter::seg(
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
        BeanPainter::normal(
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
            (BeanPainter::seg(
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
            (BeanPainter::seg(
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
            (BeanPainter::seg(
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
        let mut d: f64 = BeanPainter::seg(
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
            BeanPainter::seg(
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
            BeanPainter::seg(
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
        let mut bar: f64 = (BeanPainter::seg(
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
