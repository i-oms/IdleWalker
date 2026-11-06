// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(i32)]
pub enum Shape {
    #[default]
    Round,
    Triangle,
    Egg,
    Square,
}
impl Shape {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => Shape::Round,
            1 => Shape::Triangle,
            2 => Shape::Egg,
            3 => Shape::Square,
            _ => Self::default(),
        }
    }
}
impl std::fmt::Display for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

//
// Paints an Angry Bird into a `Surface` the way the Minions
// are painted: as lit solids with anti-aliased edges, composited back to
// front (see `SolidPainter`). Faithful to the classic
// 2009-2013 game designs: a LIMBLESS ball of feathers - no wings, no
// legs, no feet - with a lighter chest on most of them, two big front
// eyes, a beak that opens, crest feathers and tail feathers. They hop
// and roll instead of walking, so all motion reads through squash,
// stretch, lean and bobbing.
//
// What tells them apart, kind by kind (the classic art, not the films):
//  Red      round, red; peach chest; two red crest feathers; three black
//           tail feathers; thick black brows.
//  Blues    small (three fifths of Red), light blue; no chest patch, no
//           brows; two blue crest feathers; two black tail feathers;
//           orange lower eyelids.
//  Chuck    a yellow triangle, rounded corners, face set high; white
//           chest; red brows; four black crest feathers swept back;
//           three tail feathers angled up; small long beak.
//  Bomb     bigger than Red, black; dark grey chest; red brows; a grey
//           fuse with a yellow tip instead of a crest.
//  Matilda  a tall white egg; large oval beak; yellow cheek spots; three
//           short black crest and tail feathers; lashes.
//  Hal      green; pure white chest; a huge orange beak; three black
//           crest feathers; four spiky tail feathers; medium brows.
//  Terence  half again Red's size, dark crimson; darker red crest; red-
//           violet spots about the face; two forehead wrinkles; a
//           smaller face for his size.
//  Bubbles  the smallest, orange; dark brown chest and eye rings; maroon
//           brows; two long black round-ended crest feathers of
//           different lengths.
//  Stella   pink; light pink chest; three pink crest feathers with light
//           round tips; large light-blue eyes with lashes; small wide
//           beak.
//  Silver   squarish grey (Angry Birds 2); long dark head feathers
//           pointing back; a red beak tip pointing down; one pointed
//           lash per eye; pale chest.
//
// Model space is the Minions': u across in units, positive the way it
// faces; v up from the soles (its round bottom). Red is
// `Height` units tall; the others are scaled from him by
// `SizeOf` when drawn.
//
#[derive(Clone, Default)]
pub struct BirdPainterData {
    pub pose: Pose,
    pub face: Expression,
    pub kind: BirdKind,
    pub shape: Shape,
    pub yaw: f64,
    pub sin_y: f64,
    pub cos_y: f64,
    pub lean: f64,
    pub grow: f64,
    // The body for this kind.
    pub body_v: f64,
    pub body_rx: f64,
    pub body_ry: f64,
    pub egg_taper: f64,
    pub tri_base: f64,
    pub tri_top: f64,
    pub tri_v0: f64,
    pub tri_v1: f64,
    pub tri_round: f64,
    pub face_lift: f64,
    // Colours for this kind.
    pub body_r: f64,
    pub body_g: f64,
    pub body_b: f64,
    pub belly_r: f64,
    pub belly_g: f64,
    pub belly_b: f64,
    pub beak_r: f64,
    pub beak_g: f64,
    pub beak_b: f64,
    pub beak_r2: f64,
    pub beak_g2: f64,
    pub beak_b2: f64,
    pub tail_r: f64,
    pub tail_g: f64,
    pub tail_b: f64,
    pub crest_r: f64,
    pub crest_g: f64,
    pub crest_b: f64,
    pub brow_r: f64,
    pub brow_g: f64,
    pub brow_b: f64,
    pub iris_r: f64,
    pub iris_g: f64,
    pub iris_b: f64,
    // Shape numbers for this kind.
    pub has_belly: bool,
    pub brow_weight: f64,
    pub eye_rx: f64,
    pub eye_ry: f64,
    pub beak_scale: f64,
    pub beak_long: f64,
    pub tail_count: i32,
    pub tail_lift: f64,
    // Worked out once per frame.
    pub eye_v: f64,
    pub brow_v: f64,
    pub beak_v: f64,
    pub crest_v: f64,
    pub belly_u: f64,
    pub belly_rx: f64,
    pub eye_alpha: Arr<f64>,
    pub eye_u: Arr<f64>,
    pub eye_k: Arr<f64>,
    pub eye_vis: Arr<f64>,
    pub first_eye: i32,
    pub decorated: bool,
    pub top_v: f64,
    pub beak_u: f64,
    pub beak_w: f64,
    pub crest_u: f64,
    // Fixed for the frame: the cosines and sines of the eyes' angles,
    // where each iris sits, the tail's root, the sway of tail and crest.
    pub eye_cos: Arr<f64>,
    pub eye_sin: Arr<f64>,
    pub gaze_u: Arr<f64>,
    pub gaze_v: Arr<f64>,
    pub tail_x: f64,
    pub tail_sway: f64,
    pub crest_sway: f64,
    // The body's width depends on the row only: the last row's answers.
    pub egg_at: f64,
    pub egg_rx: f64,
    pub half_at: f64,
    pub half: f64,
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
rt::handle!(BirdPainter, BirdPainterData);

thread_local! { static BIRDPAINTER_CACHED: std::cell::Cell<BirdPainter> = std::cell::Cell::new(Default::default()); }

impl BirdPainter {
    fn alloc__() -> BirdPainter {
        let mut this = BirdPainter::from_data(BirdPainterData::default());
        this.eye_alpha = Arr::new(2);
        this.eye_u = Arr::new(2);
        this.eye_k = Arr::new(2);
        this.eye_vis = Arr::new(2);
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
    pub fn new() -> BirdPainter {
        Self::alloc__()
    }
    pub const HEIGHT: f64 = 14.0;
    // Where the features sit on Red, in units. Taller kinds lift the face.
    pub const BODY_BOTTOM: f64 = 1.2;
    pub const BELLY_V: f64 = 5.3;
    pub const BELLY_RY: f64 = 2.9;
    pub const BELLY_OFF: f64 = 1.6;
    pub const EYE_V: f64 = 9.7;
    pub const EYE_ANGLE: f64 = 17.0;
    pub const IRIS_R: f64 = 0.42;
    pub const BROW_V: f64 = 10.9;
    pub const BEAK_V: f64 = 7.6;
    pub const CREST_V: f64 = 11.1;
    pub fn init(mut self, mut pose: Pose, mut look: Look, mut face_left: bool, mut px: f64) {
        self.setup(face_left, px, 5.0);
        self.half_at = f64::NAN;
        self.egg_at = self.half_at;
        self.pose = pose;
        self.face = pose.face;
        self.kind = look.kind;
        self.lean = pose.lean;
        let mut puff: f64 = (if (pose.puff_up < 0.0) {
            0.0
        } else {
            (if (pose.puff_up > 1.0) {
                1.0
            } else {
                pose.puff_up
            })
        });
        self.grow = (1.0 + (0.35 * puff));
        self.yaw = (BirdPainter::wrap(pose.yaw) * SolidPainter::DEG);
        self.sin_y = (self.yaw).sin();
        self.cos_y = (self.yaw).cos();
        // ---- The kind: body, colours, features --------------------------------
        self.shape = Shape::Round;
        self.body_rx = 4.9;
        self.body_ry = 5.1;
        self.egg_taper = 0.0;
        self.tri_base = 0.0;
        self.tri_top = 0.0;
        self.tri_v0 = 0.0;
        self.tri_v1 = 0.0;
        self.tri_round = 0.0;
        self.face_lift = 0.0;
        self.has_belly = true;
        self.brow_weight = 1.0;
        self.eye_rx = 1.0;
        self.eye_ry = 0.92;
        self.beak_scale = 1.0;
        self.beak_long = 1.0;
        self.tail_count = 3;
        self.tail_lift = 0.0;
        self.iris_r = 0.08;
        self.iris_g = 0.07;
        self.iris_b = 0.08;
        'sw1: {
            match self.kind {
                BirdKind::Blues => {
                    BirdPainter::rgb(
                        0.36,
                        0.64,
                        0.94,
                        rt::mref(&mut self.body_r),
                        rt::mref(&mut self.body_g),
                        rt::mref(&mut self.body_b),
                    );
                    BirdPainter::rgb(
                        0.36,
                        0.64,
                        0.94,
                        rt::mref(&mut self.belly_r),
                        rt::mref(&mut self.belly_g),
                        rt::mref(&mut self.belly_b),
                    );
                    self.has_belly = false;
                    BirdPainter::rgb(
                        0.98,
                        0.64,
                        0.12,
                        rt::mref(&mut self.beak_r),
                        rt::mref(&mut self.beak_g),
                        rt::mref(&mut self.beak_b),
                    );
                    BirdPainter::rgb(
                        0.90,
                        0.50,
                        0.08,
                        rt::mref(&mut self.beak_r2),
                        rt::mref(&mut self.beak_g2),
                        rt::mref(&mut self.beak_b2),
                    );
                    BirdPainter::rgb(
                        0.10,
                        0.10,
                        0.12,
                        rt::mref(&mut self.tail_r),
                        rt::mref(&mut self.tail_g),
                        rt::mref(&mut self.tail_b),
                    );
                    BirdPainter::rgb(
                        0.22,
                        0.46,
                        0.84,
                        rt::mref(&mut self.crest_r),
                        rt::mref(&mut self.crest_g),
                        rt::mref(&mut self.crest_b),
                    );
                    BirdPainter::rgb(
                        0.0,
                        0.0,
                        0.0,
                        rt::mref(&mut self.brow_r),
                        rt::mref(&mut self.brow_g),
                        rt::mref(&mut self.brow_b),
                    );
                    self.brow_weight = 0.0;
                    self.tail_count = 2;
                    self.eye_rx = 1.05;
                    self.eye_ry = 1.0;
                }
                BirdKind::Chuck => {
                    self.shape = Shape::Triangle;
                    self.tri_base = 4.9;
                    self.tri_top = 0.55;
                    self.tri_v0 = 0.9;
                    self.tri_v1 = 12.9;
                    self.tri_round = 1.1;
                    self.face_lift = 0.7;
                    BirdPainter::rgb(
                        0.98,
                        0.80,
                        0.12,
                        rt::mref(&mut self.body_r),
                        rt::mref(&mut self.body_g),
                        rt::mref(&mut self.body_b),
                    );
                    BirdPainter::rgb(
                        0.98,
                        0.97,
                        0.93,
                        rt::mref(&mut self.belly_r),
                        rt::mref(&mut self.belly_g),
                        rt::mref(&mut self.belly_b),
                    );
                    BirdPainter::rgb(
                        0.96,
                        0.56,
                        0.10,
                        rt::mref(&mut self.beak_r),
                        rt::mref(&mut self.beak_g),
                        rt::mref(&mut self.beak_b),
                    );
                    BirdPainter::rgb(
                        0.88,
                        0.45,
                        0.07,
                        rt::mref(&mut self.beak_r2),
                        rt::mref(&mut self.beak_g2),
                        rt::mref(&mut self.beak_b2),
                    );
                    BirdPainter::rgb(
                        0.10,
                        0.09,
                        0.09,
                        rt::mref(&mut self.tail_r),
                        rt::mref(&mut self.tail_g),
                        rt::mref(&mut self.tail_b),
                    );
                    BirdPainter::rgb(
                        0.10,
                        0.09,
                        0.09,
                        rt::mref(&mut self.crest_r),
                        rt::mref(&mut self.crest_g),
                        rt::mref(&mut self.crest_b),
                    );
                    BirdPainter::rgb(
                        0.80,
                        0.12,
                        0.10,
                        rt::mref(&mut self.brow_r),
                        rt::mref(&mut self.brow_g),
                        rt::mref(&mut self.brow_b),
                    );
                    self.eye_rx = 0.88;
                    self.eye_ry = 0.92;
                    self.beak_scale = 0.9;
                    self.beak_long = 1.35;
                    self.tail_lift = 1.1;
                }
                BirdKind::Bomb => {
                    self.body_rx = 5.0;
                    self.body_ry = 5.2;
                    BirdPainter::rgb(
                        0.12,
                        0.12,
                        0.14,
                        rt::mref(&mut self.body_r),
                        rt::mref(&mut self.body_g),
                        rt::mref(&mut self.body_b),
                    );
                    BirdPainter::rgb(
                        0.42,
                        0.42,
                        0.45,
                        rt::mref(&mut self.belly_r),
                        rt::mref(&mut self.belly_g),
                        rt::mref(&mut self.belly_b),
                    );
                    BirdPainter::rgb(
                        0.99,
                        0.78,
                        0.12,
                        rt::mref(&mut self.beak_r),
                        rt::mref(&mut self.beak_g),
                        rt::mref(&mut self.beak_b),
                    );
                    BirdPainter::rgb(
                        0.93,
                        0.60,
                        0.08,
                        rt::mref(&mut self.beak_r2),
                        rt::mref(&mut self.beak_g2),
                        rt::mref(&mut self.beak_b2),
                    );
                    BirdPainter::rgb(
                        0.26,
                        0.26,
                        0.30,
                        rt::mref(&mut self.tail_r),
                        rt::mref(&mut self.tail_g),
                        rt::mref(&mut self.tail_b),
                    );
                    BirdPainter::rgb(
                        0.45,
                        0.40,
                        0.34,
                        rt::mref(&mut self.crest_r),
                        rt::mref(&mut self.crest_g),
                        rt::mref(&mut self.crest_b),
                    );
                    BirdPainter::rgb(
                        0.80,
                        0.12,
                        0.10,
                        rt::mref(&mut self.brow_r),
                        rt::mref(&mut self.brow_g),
                        rt::mref(&mut self.brow_b),
                    );
                }
                BirdKind::Matilda => {
                    self.shape = Shape::Egg;
                    self.body_rx = 4.4;
                    self.body_ry = 5.7;
                    self.egg_taper = 0.42;
                    self.face_lift = 0.9;
                    BirdPainter::rgb(
                        0.96,
                        0.96,
                        0.95,
                        rt::mref(&mut self.body_r),
                        rt::mref(&mut self.body_g),
                        rt::mref(&mut self.body_b),
                    );
                    BirdPainter::rgb(
                        0.96,
                        0.96,
                        0.95,
                        rt::mref(&mut self.belly_r),
                        rt::mref(&mut self.belly_g),
                        rt::mref(&mut self.belly_b),
                    );
                    self.has_belly = false;
                    BirdPainter::rgb(
                        0.99,
                        0.74,
                        0.14,
                        rt::mref(&mut self.beak_r),
                        rt::mref(&mut self.beak_g),
                        rt::mref(&mut self.beak_b),
                    );
                    BirdPainter::rgb(
                        0.93,
                        0.58,
                        0.10,
                        rt::mref(&mut self.beak_r2),
                        rt::mref(&mut self.beak_g2),
                        rt::mref(&mut self.beak_b2),
                    );
                    BirdPainter::rgb(
                        0.12,
                        0.11,
                        0.11,
                        rt::mref(&mut self.tail_r),
                        rt::mref(&mut self.tail_g),
                        rt::mref(&mut self.tail_b),
                    );
                    BirdPainter::rgb(
                        0.12,
                        0.11,
                        0.11,
                        rt::mref(&mut self.crest_r),
                        rt::mref(&mut self.crest_g),
                        rt::mref(&mut self.crest_b),
                    );
                    BirdPainter::rgb(
                        0.12,
                        0.10,
                        0.10,
                        rt::mref(&mut self.brow_r),
                        rt::mref(&mut self.brow_g),
                        rt::mref(&mut self.brow_b),
                    );
                    self.brow_weight = 0.55;
                    self.eye_rx = 0.82;
                    self.eye_ry = 0.84;
                    self.beak_scale = 1.25;
                }
                BirdKind::Hal => {
                    self.body_rx = 4.8;
                    self.body_ry = 5.0;
                    BirdPainter::rgb(
                        0.30,
                        0.64,
                        0.30,
                        rt::mref(&mut self.body_r),
                        rt::mref(&mut self.body_g),
                        rt::mref(&mut self.body_b),
                    );
                    BirdPainter::rgb(
                        0.97,
                        0.97,
                        0.95,
                        rt::mref(&mut self.belly_r),
                        rt::mref(&mut self.belly_g),
                        rt::mref(&mut self.belly_b),
                    );
                    BirdPainter::rgb(
                        0.97,
                        0.55,
                        0.10,
                        rt::mref(&mut self.beak_r),
                        rt::mref(&mut self.beak_g),
                        rt::mref(&mut self.beak_b),
                    );
                    BirdPainter::rgb(
                        0.88,
                        0.42,
                        0.07,
                        rt::mref(&mut self.beak_r2),
                        rt::mref(&mut self.beak_g2),
                        rt::mref(&mut self.beak_b2),
                    );
                    BirdPainter::rgb(
                        0.08,
                        0.12,
                        0.08,
                        rt::mref(&mut self.tail_r),
                        rt::mref(&mut self.tail_g),
                        rt::mref(&mut self.tail_b),
                    );
                    BirdPainter::rgb(
                        0.08,
                        0.12,
                        0.08,
                        rt::mref(&mut self.crest_r),
                        rt::mref(&mut self.crest_g),
                        rt::mref(&mut self.crest_b),
                    );
                    BirdPainter::rgb(
                        0.07,
                        0.07,
                        0.07,
                        rt::mref(&mut self.brow_r),
                        rt::mref(&mut self.brow_g),
                        rt::mref(&mut self.brow_b),
                    );
                    self.brow_weight = 0.75;
                    self.beak_scale = 1.3;
                    self.beak_long = 1.9;
                    self.tail_count = 4;
                    self.tail_lift = 0.9;
                }
                BirdKind::Terence => {
                    self.body_rx = 5.0;
                    self.body_ry = 5.2;
                    BirdPainter::rgb(
                        0.62,
                        0.10,
                        0.10,
                        rt::mref(&mut self.body_r),
                        rt::mref(&mut self.body_g),
                        rt::mref(&mut self.body_b),
                    );
                    BirdPainter::rgb(
                        0.84,
                        0.66,
                        0.55,
                        rt::mref(&mut self.belly_r),
                        rt::mref(&mut self.belly_g),
                        rt::mref(&mut self.belly_b),
                    );
                    BirdPainter::rgb(
                        0.99,
                        0.78,
                        0.12,
                        rt::mref(&mut self.beak_r),
                        rt::mref(&mut self.beak_g),
                        rt::mref(&mut self.beak_b),
                    );
                    BirdPainter::rgb(
                        0.93,
                        0.60,
                        0.08,
                        rt::mref(&mut self.beak_r2),
                        rt::mref(&mut self.beak_g2),
                        rt::mref(&mut self.beak_b2),
                    );
                    BirdPainter::rgb(
                        0.10,
                        0.07,
                        0.07,
                        rt::mref(&mut self.tail_r),
                        rt::mref(&mut self.tail_g),
                        rt::mref(&mut self.tail_b),
                    );
                    BirdPainter::rgb(
                        0.40,
                        0.05,
                        0.05,
                        rt::mref(&mut self.crest_r),
                        rt::mref(&mut self.crest_g),
                        rt::mref(&mut self.crest_b),
                    );
                    BirdPainter::rgb(
                        0.07,
                        0.05,
                        0.05,
                        rt::mref(&mut self.brow_r),
                        rt::mref(&mut self.brow_g),
                        rt::mref(&mut self.brow_b),
                    );
                    self.eye_rx = 0.9;
                    self.eye_ry = 0.84;
                    self.beak_scale = 0.9;
                }
                BirdKind::Bubbles => {
                    self.body_rx = 4.9;
                    self.body_ry = 5.0;
                    BirdPainter::rgb(
                        0.97,
                        0.55,
                        0.10,
                        rt::mref(&mut self.body_r),
                        rt::mref(&mut self.body_g),
                        rt::mref(&mut self.body_b),
                    );
                    BirdPainter::rgb(
                        0.46,
                        0.23,
                        0.08,
                        rt::mref(&mut self.belly_r),
                        rt::mref(&mut self.belly_g),
                        rt::mref(&mut self.belly_b),
                    );
                    BirdPainter::rgb(
                        0.99,
                        0.80,
                        0.16,
                        rt::mref(&mut self.beak_r),
                        rt::mref(&mut self.beak_g),
                        rt::mref(&mut self.beak_b),
                    );
                    BirdPainter::rgb(
                        0.93,
                        0.62,
                        0.10,
                        rt::mref(&mut self.beak_r2),
                        rt::mref(&mut self.beak_g2),
                        rt::mref(&mut self.beak_b2),
                    );
                    BirdPainter::rgb(
                        0.10,
                        0.08,
                        0.07,
                        rt::mref(&mut self.tail_r),
                        rt::mref(&mut self.tail_g),
                        rt::mref(&mut self.tail_b),
                    );
                    BirdPainter::rgb(
                        0.10,
                        0.08,
                        0.07,
                        rt::mref(&mut self.crest_r),
                        rt::mref(&mut self.crest_g),
                        rt::mref(&mut self.crest_b),
                    );
                    BirdPainter::rgb(
                        0.50,
                        0.10,
                        0.10,
                        rt::mref(&mut self.brow_r),
                        rt::mref(&mut self.brow_g),
                        rt::mref(&mut self.brow_b),
                    );
                    self.eye_rx = 1.05;
                    self.eye_ry = 1.0;
                    self.beak_scale = 0.85;
                }
                BirdKind::Stella => {
                    self.body_rx = 4.8;
                    self.body_ry = 5.0;
                    BirdPainter::rgb(
                        0.97,
                        0.52,
                        0.70,
                        rt::mref(&mut self.body_r),
                        rt::mref(&mut self.body_g),
                        rt::mref(&mut self.body_b),
                    );
                    BirdPainter::rgb(
                        0.99,
                        0.84,
                        0.90,
                        rt::mref(&mut self.belly_r),
                        rt::mref(&mut self.belly_g),
                        rt::mref(&mut self.belly_b),
                    );
                    BirdPainter::rgb(
                        0.99,
                        0.80,
                        0.20,
                        rt::mref(&mut self.beak_r),
                        rt::mref(&mut self.beak_g),
                        rt::mref(&mut self.beak_b),
                    );
                    BirdPainter::rgb(
                        0.93,
                        0.62,
                        0.12,
                        rt::mref(&mut self.beak_r2),
                        rt::mref(&mut self.beak_g2),
                        rt::mref(&mut self.beak_b2),
                    );
                    BirdPainter::rgb(
                        0.12,
                        0.09,
                        0.11,
                        rt::mref(&mut self.tail_r),
                        rt::mref(&mut self.tail_g),
                        rt::mref(&mut self.tail_b),
                    );
                    BirdPainter::rgb(
                        0.94,
                        0.42,
                        0.62,
                        rt::mref(&mut self.crest_r),
                        rt::mref(&mut self.crest_g),
                        rt::mref(&mut self.crest_b),
                    );
                    BirdPainter::rgb(
                        0.30,
                        0.14,
                        0.20,
                        rt::mref(&mut self.brow_r),
                        rt::mref(&mut self.brow_g),
                        rt::mref(&mut self.brow_b),
                    );
                    self.brow_weight = 0.45;
                    self.eye_rx = 1.18;
                    self.eye_ry = 1.12;
                    BirdPainter::rgb(
                        0.46,
                        0.74,
                        0.96,
                        rt::mref(&mut self.iris_r),
                        rt::mref(&mut self.iris_g),
                        rt::mref(&mut self.iris_b),
                    );
                    self.beak_scale = 0.8;
                    self.beak_long = 1.25;
                }
                BirdKind::Silver => {
                    self.shape = Shape::Square;
                    self.body_rx = 4.8;
                    self.body_ry = 5.0;
                    BirdPainter::rgb(
                        0.74,
                        0.75,
                        0.78,
                        rt::mref(&mut self.body_r),
                        rt::mref(&mut self.body_g),
                        rt::mref(&mut self.body_b),
                    );
                    BirdPainter::rgb(
                        0.90,
                        0.90,
                        0.92,
                        rt::mref(&mut self.belly_r),
                        rt::mref(&mut self.belly_g),
                        rt::mref(&mut self.belly_b),
                    );
                    BirdPainter::rgb(
                        0.99,
                        0.78,
                        0.12,
                        rt::mref(&mut self.beak_r),
                        rt::mref(&mut self.beak_g),
                        rt::mref(&mut self.beak_b),
                    );
                    BirdPainter::rgb(
                        0.93,
                        0.60,
                        0.08,
                        rt::mref(&mut self.beak_r2),
                        rt::mref(&mut self.beak_g2),
                        rt::mref(&mut self.beak_b2),
                    );
                    BirdPainter::rgb(
                        0.30,
                        0.32,
                        0.38,
                        rt::mref(&mut self.tail_r),
                        rt::mref(&mut self.tail_g),
                        rt::mref(&mut self.tail_b),
                    );
                    BirdPainter::rgb(
                        0.30,
                        0.32,
                        0.40,
                        rt::mref(&mut self.crest_r),
                        rt::mref(&mut self.crest_g),
                        rt::mref(&mut self.crest_b),
                    );
                    BirdPainter::rgb(
                        0.20,
                        0.20,
                        0.24,
                        rt::mref(&mut self.brow_r),
                        rt::mref(&mut self.brow_g),
                        rt::mref(&mut self.brow_b),
                    );
                    self.brow_weight = 0.6;
                    self.eye_rx = 1.05;
                    self.eye_ry = 1.0;
                }
                _ => {
                    BirdPainter::rgb(
                        0.84,
                        0.15,
                        0.12,
                        rt::mref(&mut self.body_r),
                        rt::mref(&mut self.body_g),
                        rt::mref(&mut self.body_b),
                    );
                    BirdPainter::rgb(
                        0.94,
                        0.82,
                        0.68,
                        rt::mref(&mut self.belly_r),
                        rt::mref(&mut self.belly_g),
                        rt::mref(&mut self.belly_b),
                    );
                    BirdPainter::rgb(
                        0.99,
                        0.78,
                        0.12,
                        rt::mref(&mut self.beak_r),
                        rt::mref(&mut self.beak_g),
                        rt::mref(&mut self.beak_b),
                    );
                    BirdPainter::rgb(
                        0.93,
                        0.60,
                        0.08,
                        rt::mref(&mut self.beak_r2),
                        rt::mref(&mut self.beak_g2),
                        rt::mref(&mut self.beak_b2),
                    );
                    BirdPainter::rgb(
                        0.10,
                        0.08,
                        0.08,
                        rt::mref(&mut self.tail_r),
                        rt::mref(&mut self.tail_g),
                        rt::mref(&mut self.tail_b),
                    );
                    BirdPainter::rgb(
                        0.60,
                        0.09,
                        0.08,
                        rt::mref(&mut self.crest_r),
                        rt::mref(&mut self.crest_g),
                        rt::mref(&mut self.crest_b),
                    );
                    BirdPainter::rgb(
                        0.07,
                        0.06,
                        0.06,
                        rt::mref(&mut self.brow_r),
                        rt::mref(&mut self.brow_g),
                        rt::mref(&mut self.brow_b),
                    );
                }
            }
        }
        self.body_v = (if (self.shape == Shape::Triangle) {
            ((self.tri_v0 + self.tri_v1) / 2.0)
        } else {
            (BirdPainter::BODY_BOTTOM + self.body_ry)
        });
        self.eye_v = (BirdPainter::EYE_V + self.face_lift);
        self.brow_v = (BirdPainter::BROW_V + self.face_lift);
        self.beak_v = (BirdPainter::BEAK_V + self.face_lift);
        self.crest_v = (if (self.shape == Shape::Triangle) {
            (self.tri_v1 - (self.tri_round * 0.6))
        } else {
            (if (self.shape == Shape::Egg) {
                ((self.body_v + self.body_ry) - 0.6)
            } else {
                BirdPainter::CREST_V
            })
        });
        // Features narrow with the local body width so nothing floats off
        // Chuck's triangle or Matilda's egg.
        let mut spread: f64 =
            BirdPainter::clamp01((self.half_width_at(self.eye_v) / 2.6), 0.35, 1.0);
        self.place_eye(0, (-(BirdPainter::EYE_ANGLE) * SolidPainter::DEG), spread);
        self.place_eye(1, (BirdPainter::EYE_ANGLE * SolidPainter::DEG), spread);
        self.first_eye = (if ((*self.eye_alpha.at(0)).cos() < (*self.eye_alpha.at(1)).cos()) {
            0
        } else {
            1
        });
        let mut reach: f64 = ((self.eye_rx - BirdPainter::IRIS_R) - 0.05);
        {
            let mut i: i32 = 0;
            'l2: while i < 2 {
                (*self.eye_cos.at(i)) = (*self.eye_alpha.at(i)).cos();
                (*self.eye_sin.at(i)) = (*self.eye_alpha.at(i)).sin();
                let mut ox: f64 = (((pose.gaze_x * reach) * 0.9) + (self.face.pupil_dx * 0.4));
                let mut oy: f64 = (((pose.gaze_y * (self.eye_ry - (BirdPainter::IRIS_R * 0.6)))
                    * 0.8)
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
        self.tail_x = (-(self.half_width_at(7.2)) * 0.85);
        self.tail_sway = (if pose.walking {
            (((pose.walk_phase * 2.0) * std::f64::consts::PI).sin() * 0.25)
        } else {
            0.0
        });
        self.crest_sway = (if pose.walking {
            ((((pose.walk_phase * 2.0) * std::f64::consts::PI) + 1.0).sin() * 0.15)
        } else {
            0.0
        });
        let mut beak_half: f64 = self.half_width_at(self.beak_v);
        self.beak_u = (((beak_half + 0.4) * self.sin_y)
            + ((0.5 * ((self.beak_scale * self.beak_long) - 1.0)) * self.sin_y));
        self.beak_w = (((((0.95 + (self.face.mouth_width * 0.12))
            * (0.5 + (0.5 * (self.sin_y).abs())))
            * self.beak_scale)
            * self.beak_long)
            * BirdPainter::clamp01((beak_half / 2.8), 0.55, 1.0));
        let mut belly_half: f64 = self.half_width_at(BirdPainter::BELLY_V);
        self.belly_u = (rt::min_f64(BirdPainter::BELLY_OFF, (belly_half * 0.45)) * self.sin_y);
        self.belly_rx = (if self.has_belly {
            rt::min_f64(2.5, ((belly_half - (self.belly_u).abs()) - 0.15))
        } else {
            0.0
        });
        self.crest_u = ((if (self.shape == Shape::Triangle) {
            0.3
        } else {
            0.8
        }) * self.sin_y);
        self.decorated = ((((((pose.sweat > 0.0) || pose.note) || pose.zzz) || pose.bubble)
            || pose.heart)
            || pose.blast_flash);
        // Only the rows the figure can reach are shaded: the crest top
        // (the longest crest is Bubbles', the tallest body Chuck's), or the
        // floating decorations, which go no higher than the "!" overhead.
        self.top_v = ((if self.decorated {
            (BirdPainter::HEIGHT + 7.0)
        } else {
            rt::max_f64((BirdPainter::HEIGHT + 1.2), (self.crest_v + 3.0))
        }) + (pose.body_dy).abs());
    }
    pub fn rgb(mut r: f64, mut g: f64, mut b: f64, or: &mut f64, og: &mut f64, ob: &mut f64) {
        (*or) = r;
        (*og) = g;
        (*ob) = b;
    }
    // How big a kind is, relative to Red: the Blues are small, Terence is huge.
    pub fn size_of(mut kind: BirdKind) -> f64 {
        'sw1: {
            match kind {
                BirdKind::Blues => {
                    return 0.62;
                }
                BirdKind::Bubbles => {
                    return 0.72;
                }
                BirdKind::Stella => {
                    return 0.95;
                }
                BirdKind::Chuck => {
                    return 1.0;
                }
                BirdKind::Matilda => {
                    return 1.05;
                }
                BirdKind::Bomb => {
                    return 1.15;
                }
                BirdKind::Terence => {
                    return 1.45;
                }
                _ => {
                    return 1.0;
                }
            }
        }
        #[allow(unreachable_code)]
        unreachable!()
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
    pub fn clamp01(mut x: f64, mut lo: f64, mut hi: f64) -> f64 {
        return (if (x < lo) {
            lo
        } else {
            (if (x > hi) { hi } else { x })
        });
    }
    // ---- The body ------------------------------------------------------------
    // The egg's half-width scale at height v: 1 at the middle, narrower towards the top.
    pub fn egg_rx(mut self, mut v: f64) -> f64 {
        if v == self.egg_at {
            return self.egg_rx;
        }
        let mut t: f64 = ((v - self.body_v) / (self.body_ry * self.grow));
        if t < -1.0 {
            t = -1.0;
        } else if t > 1.0 {
            t = 1.0;
        }
        self.egg_at = v;
        self.egg_rx = ((self.body_rx * self.grow)
            * ((1.0 - (((self.egg_taper * (t + 1.0)) / 2.0) * 0.9)) + (self.egg_taper * 0.25)));
        return self.egg_rx;
    }
    // Half the body width at height v, puffed up.
    pub fn half_width_at(mut self, mut v: f64) -> f64 {
        if v == self.half_at {
            return self.half;
        }
        self.half = self.half_width(v);
        self.half_at = v;
        return self.half;
    }
    pub fn half_width(mut self, mut v: f64) -> f64 {
        if self.shape == Shape::Triangle {
            let mut wb: f64 = (self.tri_base * self.grow);
            let mut wt: f64 = (self.tri_top * self.grow);
            if v <= (self.tri_v0 + self.tri_round) {
                return ((wb - self.tri_round)
                    + (rt::max_f64(
                        0.0,
                        ((self.tri_round * self.tri_round)
                            - (((self.tri_v0 + self.tri_round) - v)
                                * ((self.tri_v0 + self.tri_round) - v))),
                    ))
                    .sqrt());
            }
            if v >= self.tri_v1 {
                return 0.0;
            }
            if v >= (self.tri_v1 - self.tri_round) {
                return rt::max_f64(0.1, ((wt * (self.tri_v1 - v)) / self.tri_round));
            }
            return (wb + (((wt - wb) * (v - self.tri_v0)) / (self.tri_v1 - self.tri_v0)));
        }
        let mut ry: f64 = (self.body_ry * self.grow);
        let mut dy: f64 = ((v - self.body_v) / ry);
        let mut inside: f64 = (1.0 - (dy * dy));
        if inside <= 0.0 {
            return 0.0;
        }
        let mut rx: f64 = (if (self.shape == Shape::Egg) {
            self.egg_rx(v)
        } else {
            (self.body_rx * self.grow)
        });
        let mut w: f64 = (rx * (inside).sqrt());
        // The square body keeps its width longer and turns the corner late.
        if self.shape == Shape::Square {
            let mut r: f64 = 2.8;
            let mut ady: f64 = (v - self.body_v).abs();
            if ady <= (ry - r) {
                return rx;
            }
            let mut k: f64 = (ady - (ry - r));
            return rt::max_f64(
                w,
                ((rx - r) + (rt::max_f64(0.0, ((r * r) - (k * k)))).sqrt()),
            );
        }
        return w;
    }
    // Signed distance to the body silhouette.
    pub fn body_sdf(mut self, mut u: f64, mut v: f64) -> f64 {
        let mut rx: f64 = (self.body_rx * self.grow);
        let mut ry: f64 = (self.body_ry * self.grow);
        'sw1: {
            match self.shape {
                Shape::Triangle => {
                    // A trapezoid shrunk by the rounding, then grown back by it:
                    // rounded corners at the base and a rounded apex.
                    let mut r: f64 = self.tri_round;
                    let mut wb: f64 = ((self.tri_base * self.grow) - r);
                    let mut wt: f64 = rt::max_f64(0.05, ((self.tri_top * self.grow) - (r * 0.5)));
                    let mut v0: f64 = (self.tri_v0 + r);
                    let mut v1: f64 = (self.tri_v1 - r);
                    let mut t: f64 = ((v - v0) / (v1 - v0));
                    let mut tc: f64 = (if (t < 0.0) {
                        0.0
                    } else {
                        (if (t > 1.0) { 1.0 } else { t })
                    });
                    let mut w: f64 = (wb + ((wt - wb) * tc));
                    let mut slope: f64 = ((wb - wt) / (v1 - v0));
                    let mut side: f64 = (((u).abs() - w) / (1.0 + (slope * slope)).sqrt());
                    let mut d: f64 = rt::max_f64(side, rt::max_f64((v0 - v), (v - v1)));
                    if d > 0.0 {
                        // Outside: the corner distances are Euclidean.
                        let mut dx: f64 = rt::max_f64(0.0, ((u).abs() - w));
                        let mut dy: f64 = rt::max_f64(0.0, rt::max_f64((v0 - v), (v - v1)));
                        if dy > 0.0 {
                            d = ((dx * dx) + (dy * dy)).sqrt();
                        }
                    }
                    return (d - r);
                }
                Shape::Egg => {
                    return BirdPainter::ellipse(u, (v - self.body_v), self.egg_rx(v), ry);
                }
                Shape::Square => {
                    return BirdPainter::round_rect(u, (v - self.body_v), rx, ry, 2.8);
                }
                _ => {
                    return BirdPainter::ellipse(u, (v - self.body_v), rx, ry);
                }
            }
        }
        #[allow(unreachable_code)]
        unreachable!()
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
    pub fn place_eye(mut self, mut i: i32, mut phi: f64, mut spread: f64) {
        let mut a: f64 = (phi + self.yaw);
        (*self.eye_alpha.at(i)) = a;
        (*self.eye_u.at(i)) = (((4.3 * spread) + (0.2 * (self.eye_rx - 1.0))) * (a).sin());
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
    // ---- Entry point ---------------------------------------------------------
    pub fn draw(
        mut surface: Surface,
        mut pose: Pose,
        mut look: Look,
        mut face_left: bool,
        mut centre_x: f64,
        mut sole_y: f64,
        mut scale: f64,
    ) {
        let mut size: f64 = BirdPainter::size_of(look.kind);
        let mut sx: f64 = ((scale * size) * pose.stretch_x);
        let mut sy: f64 = ((scale * size) * pose.stretch_y);
        let mut px: f64 = (1.0 / rt::min_f64(sx, sy));
        let mut painter: BirdPainter = BIRDPAINTER_CACHED.with(|c__| {
            if c__.get().is_null() {
                c__.set(BirdPainter::new());
            }
            c__.get()
        });
        painter.init(pose, look, face_left, px);
        let mut left: i32 = rt::max_i32(0, (((centre_x - (9.5 * sx)).floor()) as i32));
        let mut right: i32 =
            rt::min_i32(surface.width(), (((centre_x + (9.5 * sx)).ceil()) as i32));
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
        let mut ub: f64 = (u - ((self.lean * 0.12) * rt::max_f64(0.0, (v - 4.0))));
        let mut body_dist: f64 = self.fluff(ub, vb, self.body_sdf(ub, vb));
        // Depth of the body's surface here for the (rare) sorted prop.
        let mut rx: f64 = (self.body_rx * self.grow);
        let mut bulge: f64 = (((rx * rx) - (ub * ub)) - ((vb - self.body_v) * (vb - self.body_v)));
        self.body_z = (if (bulge > 0.0) { (bulge).sqrt() } else { 0.0 });
        // Back to front: tail, body, chest, crest, face. No limbs at all.
        self.tail(ub, vb);
        self.body(ub, vb, body_dist);
        self.belly(ub, vb, body_dist);
        self.crest(ub, vb);
        if (vb - self.eye_v).abs() < 3.6 {
            self.marks(ub, vb, body_dist);
            self.eye(ub, vb, self.first_eye);
            self.eye(ub, vb, (1 - self.first_eye));
            if self.brow_weight > 0.0 {
                self.brows(ub, vb);
            }
            self.beak(ub, vb);
        }
        self.lay_front();
        if self.pose.egg {
            self.egg_prop(ub, vb);
        }
        if self.pose.blast_flash {
            self.blast(ub, vb);
        }
        if self.decorated {
            if self.pose.sweat > 0.0 {
                self.sweat(
                    ub,
                    vb,
                    self.pose.sweat,
                    -(self.body_rx + 0.6),
                    (self.eye_v + 1.9),
                );
            }
            if self.pose.note {
                self.note(ub, vb, self.pose.decor_phase, self.beak_u, self.beak_v);
            }
            if self.pose.zzz {
                self.zzz(
                    u,
                    v,
                    self.pose.decor_phase,
                    2.4,
                    (BirdPainter::HEIGHT + 1.2),
                );
            }
            if self.pose.bubble {
                self.bubble(
                    ub,
                    vb,
                    (BirdPainter::HEIGHT + 0.4),
                    (BirdPainter::HEIGHT + 1.3),
                    (BirdPainter::HEIGHT + 2.8),
                    (BirdPainter::HEIGHT + 5.4),
                );
            }
            if self.pose.heart {
                self.heart(ub, vb, self.pose.decor_phase, self.beak_u, self.beak_v);
            }
        }
        return self.finish();
    }
    pub fn body(mut self, mut u: f64, mut v: f64, mut dist: f64) {
        let mut cov: f64 = self.cov(dist);
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        let mut rx: f64 = (self.body_rx * self.grow);
        let mut ry: f64 = (self.body_ry * self.grow);
        if self.shape == Shape::Triangle {
            // A rounded cone: curved across its width, leaning back towards the apex.
            let mut half: f64 = self.half_width_at(v);
            let mut across: f64 = (if (half > 0.2) { (u / half) } else { 0.0 });
            if across > 1.0 {
                across = 1.0;
            } else if across < -1.0 {
                across = -1.0;
            }
            BirdPainter::normal(
                (across * 0.95),
                0.32,
                1.0,
                rt::mref(&mut nx),
                rt::mref(&mut ny),
                rt::mref(&mut nz),
            );
        } else {
            let mut ex: f64 = (if (self.shape == Shape::Egg) {
                self.egg_rx(v)
            } else {
                rx
            });
            BirdPainter::normal(
                ((u / ex) * 3.0),
                (((v - self.body_v) / ry) * 3.0),
                3.0,
                rt::mref(&mut nx),
                rt::mref(&mut ny),
                rt::mref(&mut nz),
            );
        }
        let mut lr: f64 = <f64>::default();
        let mut lg: f64 = <f64>::default();
        let mut lb: f64 = <f64>::default();
        self.lit(
            self.body_r,
            self.body_g,
            self.body_b,
            nx,
            ny,
            nz,
            BirdPainter::PLUMAGE_SPEC,
            BirdPainter::PLUMAGE_SHINE,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.plumage(
            u,
            v,
            ny,
            nz,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.over(lr, lg, lb, cov);
        // Soft shade tucked under the chest so the round reads.
        self.over(
            (self.body_r * 0.72),
            (self.body_g * 0.72),
            (self.body_b * 0.72),
            ((BirdPainter::clamp(((3.6 - v) / 2.4)) * 0.35) * cov),
        );
    }
    // Feathers take the light softly: a low, broad sheen rather than a hard
    // highlight.
    pub const PLUMAGE_SPEC: f64 = 0.10;
    pub const PLUMAGE_SHINE: f64 = 7.0;
    // How much fine feather detail the screen can show: none below about four
    // pixels per feather, full from six or so. `px` is units per pixel.
    pub fn detail(mut self) -> f64 {
        return BirdPainter::clamp(((0.45 - self.px) / 0.2));
    }
    // A silhouette of soft down rather than a clean edge; rougher when puffed
    // up. Only gentle waves, several pixels long, so the edge stays smooth.
    pub fn fluff(mut self, mut u: f64, mut v: f64, mut dist: f64) -> f64 {
        if (dist).abs() > 0.7 {
            return dist;
        }
        let mut a: f64 = (v - self.body_v).atan2(u);
        let mut amp: f64 =
            ((0.03 * self.detail()) + ((self.grow - 1.0) * (0.15 + (0.15 * self.detail()))));
        return (dist + (amp * ((a * 9.0).sin() + (0.6 * ((a * 14.0) + 1.3).sin()))));
    }
    // Soft scallops of overlapping feathers across the lit colour, plus a faint
    // glow where the down catches the sky at the rim. The additive part is
    // what shows on a black bird. The scallops are smooth and fade out at
    // small sizes, so nothing turns into pixel noise.
    pub fn plumage(
        mut self,
        mut u: f64,
        mut v: f64,
        mut ny: f64,
        mut nz: f64,
        lr: &mut f64,
        lg: &mut f64,
        lb: &mut f64,
    ) {
        let mut tau: f64 = (2.0 * std::f64::consts::PI);
        let mut wave: f64 = (0.5 * ((u - self.belly_u) * (tau / 2.2)).cos());
        let mut under: f64 = (0.5 + (0.5 * ((v * (tau / 1.8)) + wave).cos()));
        // Fades towards the rim, where the rows are too foreshortened to see.
        let mut face_on: f64 = ((if (nz > 0.0) { (nz * nz) } else { 0.0 }) * self.detail());
        let mut tone: f64 = (1.0 - ((0.06 * under) * face_on));
        let mut sheen: f64 = ((0.025 * (1.0 - under)) * face_on);
        let mut edge: f64 = (1.0 - (if (nz > 0.0) { nz } else { 0.0 }));
        let mut rim: f64 = (((edge * edge) * edge) * BirdPainter::clamp((0.6 + (0.4 * ny))) * 0.22);
        (*lr) = ((((*lr) * tone) + sheen) + (((0.72 + (0.28 * (*lr))) - (*lr)) * rim));
        (*lg) = ((((*lg) * tone) + sheen) + (((0.78 + (0.22 * (*lg))) - (*lg)) * rim));
        (*lb) = ((((*lb) * tone) + sheen) + (((0.88 + (0.12 * (*lb))) - (*lb)) * rim));
    }
    // The lighter chest patch, clipped to the silhouette. Its edge feathers into
    // the body colour instead of being cut out.
    pub fn belly(mut self, mut u: f64, mut v: f64, mut body_dist: f64) {
        if self.belly_rx <= 0.2 {
            return;
        }
        let mut edge: f64 = (BirdPainter::ellipse(
            (u - self.belly_u),
            (v - BirdPainter::BELLY_V),
            self.belly_rx,
            BirdPainter::BELLY_RY,
        ) + (0.15 * ((u * 2.4) + (v * 1.3)).sin()));
        let mut cov: f64 = BirdPainter::clamp((0.5 - (edge / 0.8)));
        if cov <= 0.0 {
            return;
        }
        cov = rt::min_f64(cov, self.cov(body_dist));
        if cov <= 0.0 {
            return;
        }
        let mut nx: f64 = <f64>::default();
        let mut ny: f64 = <f64>::default();
        let mut nz: f64 = <f64>::default();
        BirdPainter::normal(
            (((u - self.belly_u) / self.belly_rx) * 2.0),
            (((v - BirdPainter::BELLY_V) / BirdPainter::BELLY_RY) * 2.0),
            2.0,
            rt::mref(&mut nx),
            rt::mref(&mut ny),
            rt::mref(&mut nz),
        );
        let mut lr: f64 = <f64>::default();
        let mut lg: f64 = <f64>::default();
        let mut lb: f64 = <f64>::default();
        self.lit(
            self.belly_r,
            self.belly_g,
            self.belly_b,
            nx,
            ny,
            nz,
            (BirdPainter::PLUMAGE_SPEC * 0.8),
            BirdPainter::PLUMAGE_SHINE,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.plumage(
            u,
            v,
            ny,
            nz,
            rt::mref(&mut lr),
            rt::mref(&mut lg),
            rt::mref(&mut lb),
        );
        self.over(lr, lg, lb, cov);
    }
    pub fn tail(mut self, mut u: f64, mut v: f64) {
        let mut bx: f64 = self.tail_x;
        if ((((u - bx) + 1.2).abs() > 3.6) || (v < 4.0)) || (v > 11.2) {
            return;
        }
        let mut sway: f64 = self.tail_sway;
        let mut lift: f64 = self.tail_lift;
        if self.tail_count == 2 {
            self.tube(
                u,
                v,
                bx,
                6.6,
                (bx - 1.7),
                (6.0 + sway),
                0.38,
                self.tail_r,
                self.tail_g,
                self.tail_b,
                0.1,
                8.0,
            );
            self.tube(
                u,
                v,
                bx,
                8.0,
                (bx - 1.8),
                (8.6 + sway),
                0.38,
                self.tail_r,
                self.tail_g,
                self.tail_b,
                0.1,
                8.0,
            );
            return;
        }
        // Three (or four) feathers fanning out behind; spiky kinds angle them upwards.
        self.tube(
            u,
            v,
            bx,
            6.3,
            (bx - 1.7),
            ((5.4 + (lift * 0.8)) + sway),
            0.38,
            self.tail_r,
            self.tail_g,
            self.tail_b,
            0.1,
            8.0,
        );
        self.tube(
            u,
            v,
            bx,
            7.4,
            (bx - 1.9),
            ((7.4 + lift) + sway),
            0.40,
            self.tail_r,
            self.tail_g,
            self.tail_b,
            0.1,
            8.0,
        );
        self.tube(
            u,
            v,
            bx,
            8.5,
            (bx - 1.7),
            ((9.4 + (lift * 1.2)) + sway),
            0.38,
            self.tail_r,
            self.tail_g,
            self.tail_b,
            0.1,
            8.0,
        );
        if self.tail_count >= 4 {
            self.tube(
                u,
                v,
                (bx + 0.2),
                9.4,
                (bx - 1.2),
                ((10.6 + lift) + sway),
                0.34,
                self.tail_r,
                self.tail_g,
                self.tail_b,
                0.1,
                8.0,
            );
        }
    }
    // One crest feather: a tube in the crest colour, optionally with a light round tip.
    pub fn feather(
        mut self,
        mut u: f64,
        mut v: f64,
        mut x0: f64,
        mut y0: f64,
        mut x1: f64,
        mut y1: f64,
        mut r: f64,
    ) {
        self.tube(
            u,
            v,
            x0,
            y0,
            x1,
            y1,
            r,
            self.crest_r,
            self.crest_g,
            self.crest_b,
            0.15,
            10.0,
        );
    }
    pub fn crest(mut self, mut u: f64, mut v: f64) {
        let mut cy0: f64 = self.crest_v;
        let mut cx0: f64 = self.crest_u;
        if ((((u - cx0) + 0.4).abs() > 2.6) || (v < (cy0 - 0.6))) || (v > (cy0 + 3.0)) {
            return;
        }
        let mut sway: f64 = self.crest_sway;
        'sw1: {
            match self.kind {
                BirdKind::Bomb => {
                    // A fuse: a grey stalk with a yellow sparking tip.
                    self.tube(
                        u,
                        v,
                        cx0,
                        cy0,
                        (cx0 + 0.2),
                        (cy0 + 1.3),
                        0.16,
                        self.crest_r,
                        self.crest_g,
                        self.crest_b,
                        0.2,
                        10.0,
                    );
                    self.ball(
                        u,
                        v,
                        (cx0 + 0.2),
                        (cy0 + 1.45),
                        0.22,
                        1.0,
                        0.82,
                        0.18,
                        0.6,
                        20.0,
                    );
                    self.ball(
                        u,
                        v,
                        (cx0 + 0.28),
                        (cy0 + 1.38),
                        0.08,
                        1.0,
                        1.0,
                        1.0,
                        0.8,
                        24.0,
                    );
                    return;
                }
                BirdKind::Blues => {
                    // Two blue feathers standing tall.
                    self.feather(
                        u,
                        v,
                        (cx0 - 0.3),
                        cy0,
                        ((cx0 - 0.7) + sway),
                        (cy0 + 1.9),
                        0.40,
                    );
                    self.feather(
                        u,
                        v,
                        (cx0 + 0.3),
                        cy0,
                        ((cx0 + 0.6) + sway),
                        (cy0 + 1.7),
                        0.38,
                    );
                    return;
                }
                BirdKind::Silver => {
                    // Long feathers pointing behind the head.
                    self.feather(u, v, cx0, cy0, ((cx0 - 1.3) + sway), (cy0 + 1.3), 0.30);
                    self.feather(
                        u,
                        v,
                        (cx0 + 0.2),
                        (cy0 - 0.1),
                        ((cx0 - 1.8) + sway),
                        (cy0 + 0.7),
                        0.28,
                    );
                    self.feather(
                        u,
                        v,
                        (cx0 + 0.3),
                        (cy0 - 0.3),
                        ((cx0 - 1.9) + sway),
                        (cy0 + 0.1),
                        0.24,
                    );
                    return;
                }
                BirdKind::Chuck => {
                    // Four pointy black feathers swept back off the apex.
                    self.feather(
                        u,
                        v,
                        (cx0 + 0.1),
                        cy0,
                        ((cx0 - 0.3) + sway),
                        (cy0 + 1.9),
                        0.22,
                    );
                    self.feather(
                        u,
                        v,
                        (cx0 + 0.1),
                        (cy0 - 0.1),
                        ((cx0 - 0.9) + sway),
                        (cy0 + 1.6),
                        0.22,
                    );
                    self.feather(
                        u,
                        v,
                        cx0,
                        (cy0 - 0.3),
                        ((cx0 - 1.4) + sway),
                        (cy0 + 1.0),
                        0.22,
                    );
                    self.feather(
                        u,
                        v,
                        (cx0 - 0.1),
                        (cy0 - 0.5),
                        ((cx0 - 1.7) + sway),
                        (cy0 + 0.3),
                        0.20,
                    );
                    return;
                }
                BirdKind::Stella => {
                    // Three pink feathers with light, round tips.
                    self.feather(
                        u,
                        v,
                        (cx0 - 0.3),
                        cy0,
                        ((cx0 - 0.8) + sway),
                        (cy0 + 1.6),
                        0.30,
                    );
                    self.feather(
                        u,
                        v,
                        (cx0 + 0.1),
                        cy0,
                        ((cx0 + 0.1) + sway),
                        (cy0 + 1.9),
                        0.30,
                    );
                    self.feather(
                        u,
                        v,
                        (cx0 + 0.5),
                        cy0,
                        ((cx0 + 0.9) + sway),
                        (cy0 + 1.4),
                        0.28,
                    );
                    self.ball(
                        u,
                        v,
                        ((cx0 - 0.8) + sway),
                        (cy0 + 1.6),
                        0.34,
                        0.99,
                        0.78,
                        0.86,
                        0.3,
                        14.0,
                    );
                    self.ball(
                        u,
                        v,
                        ((cx0 + 0.1) + sway),
                        (cy0 + 1.9),
                        0.34,
                        0.99,
                        0.78,
                        0.86,
                        0.3,
                        14.0,
                    );
                    self.ball(
                        u,
                        v,
                        ((cx0 + 0.9) + sway),
                        (cy0 + 1.4),
                        0.32,
                        0.99,
                        0.78,
                        0.86,
                        0.3,
                        14.0,
                    );
                    return;
                }
                BirdKind::Matilda | BirdKind::Hal => {
                    // Three short black feathers.
                    self.feather(
                        u,
                        v,
                        (cx0 - 0.35),
                        cy0,
                        ((cx0 - 0.7) + sway),
                        (cy0 + 1.4),
                        0.21,
                    );
                    self.feather(u, v, cx0, cy0, (cx0 + sway), (cy0 + 1.6), 0.21);
                    self.feather(
                        u,
                        v,
                        (cx0 + 0.35),
                        cy0,
                        ((cx0 + 0.6) + sway),
                        (cy0 + 1.3),
                        0.21,
                    );
                    return;
                }
                BirdKind::Bubbles => {
                    // Two long round-ended feathers, one longer than the other.
                    self.feather(
                        u,
                        v,
                        (cx0 - 0.2),
                        cy0,
                        ((cx0 - 0.6) + sway),
                        (cy0 + 2.6),
                        0.26,
                    );
                    self.feather(
                        u,
                        v,
                        (cx0 + 0.3),
                        cy0,
                        ((cx0 + 0.7) + sway),
                        (cy0 + 1.9),
                        0.26,
                    );
                    return;
                }
                _ => {
                    // Red and Terence: two feathers, leaning with the hop.
                    self.feather(
                        u,
                        v,
                        (cx0 - 0.25),
                        cy0,
                        ((cx0 - 0.55) + sway),
                        (cy0 + 1.5),
                        0.32,
                    );
                    self.feather(
                        u,
                        v,
                        (cx0 + 0.25),
                        cy0,
                        ((cx0 + 0.45) + sway),
                        (cy0 + 1.3),
                        0.30,
                    );
                    return;
                }
            }
        }
    }
    // What sits on the face besides eyes, brows and beak: Terence's
    // spots and wrinkles, Matilda's cheek spots, Bubbles' eye rings.
    pub fn marks(mut self, mut u: f64, mut v: f64, mut body_dist: f64) {
        let mut clip: f64 = self.cov(body_dist);
        if clip <= 0.0 {
            return;
        }
        'sw1: {
            match self.kind {
                BirdKind::Terence => {
                    // Five red-violet spots about the face, two wrinkles on the forehead.
                    let mut fu: f64 = (self.belly_u * 0.8);
                    self.spot(
                        u,
                        v,
                        (fu - 2.6),
                        (self.eye_v - 1.3),
                        0.24,
                        0.46,
                        0.10,
                        0.26,
                        clip,
                    );
                    self.spot(
                        u,
                        v,
                        (fu + 2.7),
                        (self.eye_v - 1.1),
                        0.22,
                        0.46,
                        0.10,
                        0.26,
                        clip,
                    );
                    self.spot(
                        u,
                        v,
                        (fu - 2.2),
                        (self.eye_v + 1.1),
                        0.20,
                        0.46,
                        0.10,
                        0.26,
                        clip,
                    );
                    self.spot(
                        u,
                        v,
                        (fu + 2.4),
                        (self.eye_v + 1.3),
                        0.20,
                        0.46,
                        0.10,
                        0.26,
                        clip,
                    );
                    self.spot(
                        u,
                        v,
                        (fu + 0.1),
                        (self.eye_v - 2.2),
                        0.20,
                        0.46,
                        0.10,
                        0.26,
                        clip,
                    );
                    let mut qx: f64 = <f64>::default();
                    let mut qy: f64 = <f64>::default();
                    let mut t: f64 = <f64>::default();
                    let mut w1: f64 = (BirdPainter::seg(
                        u,
                        v,
                        (fu - 0.9),
                        (self.brow_v + 1.1),
                        (fu - 0.1),
                        (self.brow_v + 1.35),
                        rt::mref(&mut qx),
                        rt::mref(&mut qy),
                        rt::mref(&mut t),
                    ) - 0.08);
                    let mut w2: f64 = (BirdPainter::seg(
                        u,
                        v,
                        (fu + 0.1),
                        (self.brow_v + 1.35),
                        (fu + 0.9),
                        (self.brow_v + 1.1),
                        rt::mref(&mut qx),
                        rt::mref(&mut qy),
                        rt::mref(&mut t),
                    ) - 0.08);
                    self.over(
                        0.34,
                        0.05,
                        0.05,
                        ((self.cov(rt::min_f64(w1, w2)) * 0.8) * clip),
                    );
                }
                BirdKind::Matilda => {
                    // Yellow spots on the cheeks (her make-up).
                    let mut off: f64 = 2.0;
                    self.spot(
                        u,
                        v,
                        (self.belly_u - off),
                        (self.beak_v - 0.9),
                        0.42,
                        0.98,
                        0.78,
                        0.20,
                        (clip * 0.85),
                    );
                    self.spot(
                        u,
                        v,
                        (self.belly_u + off),
                        (self.beak_v - 0.9),
                        0.42,
                        0.98,
                        0.78,
                        0.20,
                        (clip * 0.85),
                    );
                }
                BirdKind::Bubbles => {
                    // Dark brown rings round the eyes, behind the whites.
                    {
                        let mut i: i32 = 0;
                        'l2: while i < 2 {
                            'c3: {
                                if (*self.eye_vis.at(i)) <= 0.0 {
                                    break 'c3;
                                }
                                let mut dx: f64 = (((u - (*self.eye_u.at(i)))
                                    / (*self.eye_k.at(i)))
                                    / (self.eye_rx + 0.38));
                                let mut dy: f64 = ((v - self.eye_v) / (self.eye_ry + 0.38));
                                self.over(
                                    self.belly_r,
                                    self.belly_g,
                                    self.belly_b,
                                    ((self.cov(
                                        ((((dx * dx) + (dy * dy)).sqrt() - 1.0)
                                            * (self.eye_ry + 0.38)),
                                    ) * (*self.eye_vis.at(i)))
                                        * clip),
                                );
                            }
                            i += 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    pub fn spot(
        mut self,
        mut u: f64,
        mut v: f64,
        mut cx: f64,
        mut cy: f64,
        mut radius: f64,
        mut r: f64,
        mut g: f64,
        mut b: f64,
        mut clip: f64,
    ) {
        let mut dx: f64 = (u - cx);
        let mut dy: f64 = (v - cy);
        if ((dx).abs() > (radius + 0.6)) || ((dy).abs() > (radius + 0.6)) {
            return;
        }
        self.over(
            r,
            g,
            b,
            ((self.cov((((dx * dx) + (dy * dy)).sqrt() - radius)) * 0.8) * clip),
        );
    }
    pub fn eye(mut self, mut u: f64, mut v: f64, mut i: i32) {
        let mut vis: f64 = (*self.eye_vis.at(i));
        if vis <= 0.0 {
            return;
        }
        let mut k: f64 = (*self.eye_k.at(i));
        let mut near: bool = (i != self.first_eye);
        let mut fu: f64 = (*self.eye_u.at(i));
        let mut fv: f64 = self.eye_v;
        let mut dx: f64 = ((u - fu) / k);
        let mut dy: f64 = (v - fv);
        if ((dx).abs() > (self.eye_rx + 0.6)) || ((dy).abs() > (self.eye_ry + 0.6)) {
            return;
        }
        let mut ex: f64 = (dx / self.eye_rx);
        let mut ey: f64 = (dy / self.eye_ry);
        let mut white: f64 = ((((ex * ex) + (ey * ey)).sqrt() - 1.0) * self.eye_ry);
        let mut cov_white: f64 = (self.cov(white) * vis);
        // A soft hollow in the feathers around the eye, so it sits in the head
        // rather than on it.
        self.over(
            (self.body_r * 0.5),
            (self.body_g * 0.5),
            (self.body_b * 0.5),
            ((self.cov((white - 0.2)) * 0.3) * vis),
        );
        if cov_white > 0.0 {
            let mut nx: f64 = <f64>::default();
            let mut ny: f64 = <f64>::default();
            let mut nz: f64 = <f64>::default();
            BirdPainter::normal(
                dx,
                dy,
                (self.eye_rx * 1.6),
                rt::mref(&mut nx),
                rt::mref(&mut ny),
                rt::mref(&mut nz),
            );
            BirdPainter::turn_2(
                rt::mref(&mut nx),
                rt::mref(&mut nz),
                (*self.eye_cos.at(i)),
                (*self.eye_sin.at(i)),
            );
            let mut r: f64 = <f64>::default();
            let mut g: f64 = <f64>::default();
            let mut b: f64 = <f64>::default();
            self.lit(
                1.0,
                1.0,
                1.0,
                nx,
                ny,
                nz,
                0.35,
                22.0,
                rt::mref(&mut r),
                rt::mref(&mut g),
                rt::mref(&mut b),
            );
            let mut ix: f64 = (dx - (*self.gaze_u.at(i)));
            let mut iy: f64 = (dy - (*self.gaze_v.at(i)));
            let mut cov_iris: f64 = self.cov(BirdPainter::squeezed(ix, iy, k, BirdPainter::IRIS_R));
            if cov_iris > 0.0 {
                r += ((self.iris_r - r) * cov_iris);
                g += ((self.iris_g - g) * cov_iris);
                b += ((self.iris_b - b) * cov_iris);
                let mut cov_pupil: f64 = self.cov(BirdPainter::squeezed(
                    ix,
                    iy,
                    k,
                    ((BirdPainter::IRIS_R * 0.55) * self.face.pupil_size),
                ));
                r += ((0.02 - r) * cov_pupil);
                g += ((0.02 - g) * cov_pupil);
                b += ((0.02 - b) * cov_pupil);
                let mut lx: f64 = (ix + ((self.flip * 0.3) * BirdPainter::IRIS_R));
                let mut ly: f64 = (iy - (0.3 * BirdPainter::IRIS_R));
                let mut cov_light: f64 = (self.cov(BirdPainter::squeezed(
                    lx,
                    ly,
                    k,
                    (BirdPainter::IRIS_R * 0.28),
                )) * 0.95);
                r += ((1.0 - r) * cov_light);
                g += ((1.0 - g) * cov_light);
                b += ((1.0 - b) * cov_light);
            }
            // Feather lids slide down over the eye.
            let mut upper: f64 =
                rt::max_f64(self.face.lid, (if near { self.face.wink } else { 0.0 }));
            let mut lower: f64 = self.face.squint;
            let mut s: f64 = (ex * ex);
            let mut upper_edge: f64 =
                ((fv + (self.eye_ry * (1.0 - (2.0 * upper)))) - ((0.25 * upper) * s));
            let mut lower_edge: f64 =
                ((fv - (self.eye_ry * (1.0 - (2.0 * lower)))) + ((0.25 * lower) * s));
            if (upper > 0.01) || (lower > 0.01) {
                let mut sr: f64 = <f64>::default();
                let mut sg: f64 = <f64>::default();
                let mut sb: f64 = <f64>::default();
                self.lit(
                    self.body_r,
                    self.body_g,
                    self.body_b,
                    nx,
                    ny,
                    nz,
                    0.3,
                    16.0,
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
            }
            self.over(r, g, b, cov_white);
            // The Blues' orange lower eyelids: a rim over the bottom of each eye.
            if self.kind == BirdKind::Blues {
                let mut rim: f64 = (((self.cov(white) * self.cov(((white).abs() - 0.26)))
                    * self.cov((dy + 0.1)))
                    * vis);
                self.over(0.96, 0.52, 0.14, (rim * 0.9));
            }
        }
        // Eyelashes for the ladies; Silver has a single pointed outer lash per eye.
        if ((self.kind == BirdKind::Matilda) || (self.kind == BirdKind::Stella))
            || (self.kind == BirdKind::Silver)
        {
            let mut qx: f64 = <f64>::default();
            let mut qy: f64 = <f64>::default();
            let mut t: f64 = <f64>::default();
            let mut outer: f64 = ((if (i == 0) { -1 } else { 1 }) as f64);
            let mut lash: f64 = (BirdPainter::seg(
                u,
                v,
                (fu + ((outer * 0.5) * self.eye_rx)),
                ((fv + self.eye_ry) + 0.2),
                (fu + ((outer * 0.85) * self.eye_rx)),
                ((fv + self.eye_ry) + 0.55),
                rt::mref(&mut qx),
                rt::mref(&mut qy),
                rt::mref(&mut t),
            ) - 0.07);
            self.over(0.1, 0.08, 0.08, (self.cov(lash) * vis));
            if self.kind != BirdKind::Silver {
                lash = (BirdPainter::seg(
                    u,
                    v,
                    (fu - ((outer * 0.5) * self.eye_rx)),
                    ((fv + self.eye_ry) + 0.2),
                    (fu - ((outer * 0.85) * self.eye_rx)),
                    ((fv + self.eye_ry) + 0.55),
                    rt::mref(&mut qx),
                    rt::mref(&mut qy),
                    rt::mref(&mut t),
                ) - 0.07);
                self.over(0.1, 0.08, 0.08, (self.cov(lash) * vis));
            }
        }
    }
    // Thick angry brows slanting down towards the middle, in the kind's colour.
    pub fn brows(mut self, mut u: f64, mut v: f64) {
        let mut lift: f64 = ((self.face.brow_raise * 0.4) - 0.1);
        let mut slant: f64 = (0.55 + (self.face.brow_angle * 0.35));
        if (v - (self.brow_v + lift)).abs() > 1.4 {
            return;
        }
        self.brow(
            u,
            v,
            0,
            -1.0,
            (self.brow_v + lift),
            slant,
            (*self.eye_vis.at(0)),
        );
        self.brow(
            u,
            v,
            1,
            1.0,
            ((self.brow_v + lift) + (self.face.brow_asym * 0.3)),
            slant,
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
        mut slant: f64,
        mut show: f64,
    ) {
        if show <= 0.0 {
            return;
        }
        let mut k: f64 = (*self.eye_cos.at(i));
        if k < 0.2 {
            return;
        }
        let mut ex: f64 = ((*self.eye_u.at(0))
            + (((*self.eye_u.at(1)) - (*self.eye_u.at(0)))
                * ((if (outward > 0.0) { 1 } else { 0 }) as f64)));
        // Inner end low (angry), outer end high.
        let mut ix: f64 = (ex - ((outward * 0.55) * k));
        let mut iy: f64 = (cy - slant);
        let mut ox: f64 = (ex + ((outward * 1.0) * k));
        let mut oy: f64 = (cy + (slant * 0.4));
        let mut qx: f64 = <f64>::default();
        let mut qy: f64 = <f64>::default();
        let mut t: f64 = <f64>::default();
        let mut d: f64 = (BirdPainter::seg(
            u,
            v,
            ix,
            iy,
            ox,
            oy,
            rt::mref(&mut qx),
            rt::mref(&mut qy),
            rt::mref(&mut t),
        ) - ((0.34 - (0.10 * t)) * self.brow_weight));
        self.over(self.brow_r, self.brow_g, self.brow_b, (self.cov(d) * show));
    }
    pub fn beak(mut self, mut u: f64, mut v: f64) {
        let mut vis: f64 = (if (self.cos_y > 0.1) {
            1.0
        } else {
            (if (self.cos_y < -0.2) {
                0.0
            } else {
                ((self.cos_y + 0.2) / 0.3)
            })
        });
        if vis <= 0.0 {
            return;
        }
        let mut mu: f64 = self.beak_u;
        let mut mv: f64 = self.beak_v;
        let mut w: f64 = self.beak_w;
        if w <= 0.15 {
            return;
        }
        if ((u - mu).abs() > (w + 1.2)) || ((v - mv).abs() > (2.2 * self.beak_scale)) {
            return;
        }
        let mut open: f64 = (self.face.mouth_open * 1.15);
        if open < 0.02 {
            open = 0.02;
        }
        // Upper beak; lower beak drops with the jaw.
        let mut upper_cy: f64 = (mv + 0.25);
        let mut lower_cy: f64 = ((mv - 0.35) - open);
        let mut ux: f64 = ((u - mu) / w);
        let mut uy: f64 = ((v - upper_cy) / (0.62 * self.beak_scale));
        let mut upper: f64 = ((((ux * ux) + (uy * uy)).sqrt() - 1.0) * 0.62);
        let mut cov_u: f64 = (self.cov(upper) * vis);
        if cov_u > 0.0 {
            let mut nx: f64 = <f64>::default();
            let mut ny: f64 = <f64>::default();
            let mut nz: f64 = <f64>::default();
            BirdPainter::normal(
                ux,
                uy,
                1.0,
                rt::mref(&mut nx),
                rt::mref(&mut ny),
                rt::mref(&mut nz),
            );
            let mut lr: f64 = <f64>::default();
            let mut lg: f64 = <f64>::default();
            let mut lb: f64 = <f64>::default();
            self.lit(
                self.beak_r,
                self.beak_g,
                self.beak_b,
                nx,
                ny,
                nz,
                0.45,
                20.0,
                rt::mref(&mut lr),
                rt::mref(&mut lg),
                rt::mref(&mut lb),
            );
            self.over(lr, lg, lb, cov_u);
        }
        let mut lx: f64 = ((u - mu) / (w * 0.75));
        let mut ly: f64 = ((v - lower_cy) / (0.45 * self.beak_scale));
        let mut lower: f64 = ((((lx * lx) + (ly * ly)).sqrt() - 1.0) * 0.45);
        let mut cov_l: f64 = ((self.cov(lower) * self.cov(((upper_cy - 0.1) - v))) * vis);
        if cov_l > 0.0 {
            let mut nx: f64 = <f64>::default();
            let mut ny: f64 = <f64>::default();
            let mut nz: f64 = <f64>::default();
            BirdPainter::normal(
                lx,
                ly,
                1.0,
                rt::mref(&mut nx),
                rt::mref(&mut ny),
                rt::mref(&mut nz),
            );
            let mut lr: f64 = <f64>::default();
            let mut lg: f64 = <f64>::default();
            let mut lb: f64 = <f64>::default();
            self.lit(
                self.beak_r2,
                self.beak_g2,
                self.beak_b2,
                nx,
                ny,
                nz,
                0.4,
                18.0,
                rt::mref(&mut lr),
                rt::mref(&mut lg),
                rt::mref(&mut lb),
            );
            self.over(lr, lg, lb, cov_l);
        }
        // Dark mouth inside when open, with a tongue.
        if self.face.mouth_open > 0.12 {
            let mut mx: f64 = ((u - mu) / (w * 0.6));
            let mut my: f64 = ((v - (mv - (open * 0.5))) / ((open * 0.6) + 0.05));
            let mut inside: f64 = ((((mx * mx) + (my * my)).sqrt() - 1.0) * 0.5);
            self.over(
                0.30,
                0.08,
                0.08,
                (((self.cov(inside) * self.cov((v - lower_cy))) * self.cov((upper_cy - v))) * vis),
            );
            if self.face.mouth_open > 0.3 {
                let mut tx: f64 = ((u - mu) / 0.4);
                let mut ty: f64 = ((v - (lower_cy + 0.15)) / 0.25);
                self.over(
                    0.75,
                    0.25,
                    0.25,
                    ((self.cov((((tx * tx) + (ty * ty)).sqrt() - 0.35)) * 0.9) * vis),
                );
            }
        }
        // The mouth line when shut.
        self.over(
            0.25,
            0.12,
            0.05,
            ((self.cov(((v - (mv - 0.05)).abs() - 0.06)) * self.cov(((u - mu).abs() - w))) * vis),
        );
        // Silver's red beak tip, pointing down at the end of the beak.
        if self.kind == BirdKind::Silver {
            let mut tx: f64 = (u - (mu + (w * 0.45)));
            let mut ty: f64 = (v - (mv - 0.5));
            self.over(
                0.80,
                0.12,
                0.12,
                (self.cov((((tx * tx) + (ty * ty)).sqrt() - 0.19)) * vis),
            );
        }
    }
    pub fn egg_prop(mut self, mut u: f64, mut v: f64) {
        let mut ex0: f64 = 2.9;
        let mut ey0: f64 = 0.95;
        if ((u - ex0).abs() > 1.6) || ((v - ey0).abs() > 1.8) {
            return;
        }
        let mut dx: f64 = ((u - ex0) / 0.8);
        let mut dy: f64 = ((v - ey0) / 1.0);
        let mut egg: f64 = ((((dx * dx) + (dy * dy)).sqrt() - 1.0) * 0.9);
        let mut cov: f64 = self.cov(egg);
        if cov > 0.0 {
            let mut nx: f64 = <f64>::default();
            let mut ny: f64 = <f64>::default();
            let mut nz: f64 = <f64>::default();
            BirdPainter::normal(
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
                0.96,
                0.93,
                0.88,
                nx,
                ny,
                nz,
                0.35,
                20.0,
                rt::mref(&mut lr),
                rt::mref(&mut lg),
                rt::mref(&mut lb),
            );
            self.over(lr, lg, lb, cov);
            // Speckles.
            let mut sx: f64 = (u - (ex0 - 0.25));
            let mut sy: f64 = (v - (ey0 + 0.3));
            self.over(
                0.55,
                0.45,
                0.35,
                self.cov((((sx * sx) + (sy * sy)).sqrt() - 0.09)),
            );
            sx = (u - (ex0 + 0.3));
            sy = (v - ey0);
            self.over(
                0.55,
                0.45,
                0.35,
                self.cov((((sx * sx) + (sy * sy)).sqrt() - 0.07)),
            );
        }
    }
    pub fn blast(mut self, mut u: f64, mut v: f64) {
        let mut cx0: f64 = 0.0;
        let mut cy0: f64 = self.body_v;
        let mut d: f64 = (((u - cx0) * (u - cx0)) + ((v - cy0) * (v - cy0))).sqrt();
        self.over(1.0, 0.98, 0.92, self.cov((d - 3.2)));
        self.over(1.0, 0.75, 0.25, (self.cov(((d - 3.6).abs() - 0.35)) * 0.95));
        self.over(0.35, 0.30, 0.28, (self.cov(((d - 4.3).abs() - 0.5)) * 0.7));
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
        BirdPainter::normal(
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
        let mut r: i32 = (((BirdPainter::clamp(self.cr) * 255.0) + 0.5) as i32);
        let mut g: i32 = (((BirdPainter::clamp(self.cg) * 255.0) + 0.5) as i32);
        let mut b: i32 = (((BirdPainter::clamp(self.cb) * 255.0) + 0.5) as i32);
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
            (spec * BirdPainter::pow(h, shine))
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
        BirdPainter::turn_2(&mut *nx, &mut *nz, (a).cos(), (a).sin());
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
        let mut dist: f64 = BirdPainter::seg(
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
        BirdPainter::normal(
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
        let mut dist: f64 = BirdPainter::seg(
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
        BirdPainter::normal(
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
            (BirdPainter::seg(
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
            (BirdPainter::seg(
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
            (BirdPainter::seg(
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
        let mut d: f64 = BirdPainter::seg(
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
            BirdPainter::seg(
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
            BirdPainter::seg(
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
        let mut bar: f64 = (BirdPainter::seg(
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
