// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(i32)]
pub enum HairStyle {
    #[default]
    None, // bald
    Sprouts, // a few strands standing up
    Combed,  // swept flat over the top
    Parted,  // parted in the middle
    Crew,    // a short, even crew cut
    Spiky,   // short spikes standing straight out all over the top
    Quiff,   // a tall wave swept up and back from the front
    Tuft,    // one long curl on top
    Wild,    // long strands sticking out every which way, as if it had a fright
}
impl HairStyle {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => HairStyle::None,
            1 => HairStyle::Sprouts,
            2 => HairStyle::Combed,
            3 => HairStyle::Parted,
            4 => HairStyle::Crew,
            5 => HairStyle::Spiky,
            6 => HairStyle::Quiff,
            7 => HairStyle::Tuft,
            8 => HairStyle::Wild,
            _ => Self::default(),
        }
    }
}
impl std::fmt::Display for HairStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// What kind of figure a walker is: which painter draws it and which body it has.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(i32)]
pub enum Figure {
    #[default]
    Minion, // a yellow capsule in overalls and goggles
    Bird, // a round Angry Bird with a beak, crest and tail
}
impl Figure {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => Figure::Minion,
            1 => Figure::Bird,
            _ => Self::default(),
        }
    }
}
impl std::fmt::Display for Figure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Which Angry Bird: its colours and feathers come from this.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(i32)]
pub enum BirdKind {
    #[default]
    Red, // the red leader with black brows and two head feathers
    Blues,   // Jay, Jake and Jim: small, blue, no brows, two big head feathers
    Chuck,   // yellow and fast, with a black mohawk
    Bomb,    // black and round, with a fuse on top
    Matilda, // white and motherly, with a pink-cheeked head feather
    Hal,     // green with a long orange beak
    Terence, // huge and dark red with freckles
    Bubbles, // small and orange, blows up big
    Stella,  // pink and agile, with a feather bow
    Silver,  // grey falcon with a red-tipped beak and slicked-back crest
}
impl BirdKind {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => BirdKind::Red,
            1 => BirdKind::Blues,
            2 => BirdKind::Chuck,
            3 => BirdKind::Bomb,
            4 => BirdKind::Matilda,
            5 => BirdKind::Hal,
            6 => BirdKind::Terence,
            7 => BirdKind::Bubbles,
            8 => BirdKind::Stella,
            9 => BirdKind::Silver,
            _ => Self::default(),
        }
    }
}
impl std::fmt::Display for BirdKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

//
// The measurements of a kind of figure that the behaviour needs: how
// tall it is, where its eyes and shoulders are, how wide it stands,
// how far underground it hides. All in units (one unit is one sprite
// cell on screen).
//
#[derive(Clone, Default)]
pub struct AnatomyData {
    pub height: f64,
    pub eye_v: f64,
    pub mouth_v: f64,
    pub shoulder_v: f64,
    pub seat_v: f64,
    pub half_width: i32,
    pub hidden_depth: i32,
    pub peek_depth: i32,
    pub stride_cells: f64,
    pub canvas_rows: i32,
    pub shadow_radius: f64,
}
rt::handle!(Anatomy, AnatomyData);

// Short legs, quick steps, eyes in the middle of the body.
pub static ANATOMY_MINION: rt::Global<Anatomy> =
    rt::Global::new(|| Anatomy::new(18.0, 12.5, 8.0, 10.2, 3.0, 9, 20, 10, 10.0, 48, 6.4));

// Round and hoppy: a ball of feathers with its eyes up top and a beak in front.
pub static ANATOMY_BIRD: rt::Global<Anatomy> =
    rt::Global::new(|| Anatomy::new(14.0, 10.2, 7.6, 8.4, 2.5, 8, 16, 9, 8.0, 48, 5.2));

impl Anatomy {
    fn alloc__() -> Anatomy {
        let mut this = Anatomy::from_data(AnatomyData::default());
        this
    }
    pub fn new(
        mut height: f64,
        mut eye_v: f64,
        mut mouth_v: f64,
        mut shoulder_v: f64,
        mut seat_v: f64,
        mut half_width: i32,
        mut hidden_depth: i32,
        mut peek_depth: i32,
        mut stride_cells: f64,
        mut canvas_rows: i32,
        mut shadow_radius: f64,
    ) -> Anatomy {
        let mut this = Self::alloc__();
        this.ctor(
            height,
            eye_v,
            mouth_v,
            shoulder_v,
            seat_v,
            half_width,
            hidden_depth,
            peek_depth,
            stride_cells,
            canvas_rows,
            shadow_radius,
        );
        this
    }
    fn ctor(
        mut self,
        mut height: f64,
        mut eye_v: f64,
        mut mouth_v: f64,
        mut shoulder_v: f64,
        mut seat_v: f64,
        mut half_width: i32,
        mut hidden_depth: i32,
        mut peek_depth: i32,
        mut stride_cells: f64,
        mut canvas_rows: i32,
        mut shadow_radius: f64,
    ) {
        self.height = height;
        self.eye_v = eye_v;
        self.mouth_v = mouth_v;
        self.shoulder_v = shoulder_v;
        self.seat_v = seat_v;
        self.half_width = half_width;
        self.hidden_depth = hidden_depth;
        self.peek_depth = peek_depth;
        self.stride_cells = stride_cells;
        self.canvas_rows = canvas_rows;
        self.shadow_radius = shadow_radius;
    }
}

// What a walker is born with: its kind of figure and, for a Minion, how many eyes, their colour, its hair.
#[derive(Clone, Default)]
pub struct LookData {
    pub figure: Figure,
    pub one_eye: bool,
    pub hair: HairStyle,
    pub kind: BirdKind,
    // Iris colours (RGB) of the eye further from the viewer and the nearer one.
    pub far_iris: i32,
    pub near_iris: i32,
}
rt::handle!(Look, LookData);

impl Look {
    fn alloc__() -> Look {
        let mut this = Look::from_data(LookData::default());
        this
    }
    pub fn new(
        mut one_eye: bool,
        mut hair: HairStyle,
        mut far_iris: i32,
        mut near_iris: i32,
    ) -> Look {
        let mut this = Self::alloc__();
        this.ctor(one_eye, hair, far_iris, near_iris);
        this
    }
    fn ctor(
        mut self,
        mut one_eye: bool,
        mut hair: HairStyle,
        mut far_iris: i32,
        mut near_iris: i32,
    ) {
        self.figure = Figure::Minion;
        self.one_eye = one_eye;
        self.hair = hair;
        self.kind = BirdKind::Red;
        self.far_iris = far_iris;
        self.near_iris = near_iris;
    }
    pub fn new_2(mut kind: BirdKind) -> Look {
        let mut this = Self::alloc__();
        this.ctor_2(kind);
        this
    }
    fn ctor_2(mut self, mut kind: BirdKind) {
        self.figure = Figure::Bird;
        self.one_eye = false;
        self.hair = HairStyle::None;
        self.kind = kind;
        self.near_iris = Look::BROWN;
        self.far_iris = self.near_iris;
    }
    pub fn new_3(mut one_eye: bool, mut hair: HairStyle, mut iris: i32) -> Look {
        let mut this = Self::alloc__();
        this.ctor_3(one_eye, hair, iris);
        this
    }
    fn ctor_3(mut self, mut one_eye: bool, mut hair: HairStyle, mut iris: i32) {
        self.ctor(one_eye, hair, iris, iris);
    }
    pub fn new_4(mut one_eye: bool, mut hair: HairStyle) -> Look {
        let mut this = Self::alloc__();
        this.ctor_4(one_eye, hair);
        this
    }
    fn ctor_4(mut self, mut one_eye: bool, mut hair: HairStyle) {
        self.ctor(one_eye, hair, Look::BROWN, Look::BROWN);
    }
    pub const BROWN: i32 = 0x6b4423;
    pub const GREEN: i32 = 0x3e8e4b;
    pub const BLUE: i32 = 0x4a72b5;
    pub const AMBER: i32 = 0xb07a2a;
    // The body this figure has.
    pub fn anatomy(mut self) -> Anatomy {
        return (if (self.figure == Figure::Bird) {
            (*ANATOMY_BIRD)
        } else {
            (*ANATOMY_MINION)
        });
    }
    //
    // A random overalls colour (0xRRGGBB): any hue but yellow, strong enough to read
    // against the yellow skin and dark enough for the straps to show.
    //
    pub fn random_dress(mut rng: Random) -> i32 {
        // Hue sector, 0..6, skipping the yellows that vanish against the skin.
        let mut h: f64 = (1.3 + (rng.next_double() * 5.3));
        if h >= 6.0 {
            h -= 6.0;
        }
        let mut s: f64 = (0.55 + (rng.next_double() * 0.4)); // saturation
        let mut v: f64 = (0.45 + (rng.next_double() * 0.4)); // brightness
        let mut i: i32 = ((h) as i32);
        let mut f: f64 = (h - ((i) as f64));
        let mut p: f64 = (v * (1.0 - s));
        let mut q: f64 = (v * (1.0 - (s * f)));
        let mut t: f64 = (v * (1.0 - (s * (1.0 - f))));
        let mut r: f64 = <f64>::default();
        let mut g: f64 = <f64>::default();
        let mut b: f64 = <f64>::default();
        'sw1: {
            match i {
                0 => {
                    r = v;
                    g = t;
                    b = p;
                }
                1 => {
                    r = q;
                    g = v;
                    b = p;
                }
                2 => {
                    r = p;
                    g = v;
                    b = t;
                }
                3 => {
                    r = p;
                    g = q;
                    b = v;
                }
                4 => {
                    r = t;
                    g = p;
                    b = v;
                }
                _ => {
                    r = v;
                    g = p;
                    b = q;
                }
            }
        }
        return ((((((r * 255.0) + 0.5) as i32) << (16)) | ((((g * 255.0) + 0.5) as i32) << (8)))
            | (((b * 255.0) + 0.5) as i32));
    }
}
