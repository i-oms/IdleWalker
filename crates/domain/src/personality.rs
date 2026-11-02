// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(i32)]
pub enum Temperament {
    // Minds its own business; nods at walkers it meets.
    #[default]
    Calm,
    // Waves at everyone, stops to greet.
    Friendly,
    // Walks fast, charges at others and shoves them.
    Aggressive,
    // Frightened of everything: hides, runs from the bullies, faints when pranked.
    Timid,
    // The clown: everything is a joke, and it copies whatever it sees.
    Goofy,
    // Mr. Bean: a child in a grown man's body - fidgets, sneaks, pokes, shows off Teddy.
    Eccentric,
    // An Angry Bird: short-fused and territorial - pecks, flaps, launches, rages, blows up.
    Feisty,
    // The Pink Panther: suave, sly and silent - slinks, prowls, grooms, charms.
    Suave,
}
impl Temperament {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => Temperament::Calm,
            1 => Temperament::Friendly,
            2 => Temperament::Aggressive,
            3 => Temperament::Timid,
            4 => Temperament::Goofy,
            5 => Temperament::Eccentric,
            6 => Temperament::Feisty,
            7 => Temperament::Suave,
            _ => Self::default(),
        }
    }
}
impl std::fmt::Display for Temperament {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// The things a walker gets up to when it stops walking for a moment.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(i32)]
pub enum Antic {
    #[default]
    Look, // stop and look around
    Wave,      // wave at the cursor or at company
    Rest,      // stand and breathe
    Dance,     // a jig on the spot, arms pumping, a note or two
    Twirl,     // arms out, one full spin, ta-da
    Point,     // point at the cursor and giggle
    Yawn,      // a big stretch and a yawn
    Doze,      // yawn, then fall asleep standing up until it jerks awake
    Scratch,   // scratch the side of the head, thinking, then: an idea!
    Whistle,   // whistle a tune, notes floating up
    Sneeze,    // wind up and sneeze
    Box,       // shadow boxing
    Taunt,     // tongue out, beckoning: come and have a go
    Stomp,     // a two-footed tantrum
    Flex,      // flex both arms and admire them
    Joy,       // two hops with the arms thrown up
    Laugh,     // hold the belly and laugh at the viewer
    Hiccups,   // a run of hiccups, each one a jolt
    Shrug,     // arms out: oh well
    Trip,      // stumble and fall flat while walking, then get up embarrassed
    Boo,       // shout at somebody standing quietly, then laugh at them
    Strum,     // air guitar, head banging, notes flying - and whoever is near wants to dance
    Pushups,   // drop and give me ten
    Jog,       // running on the spot, arms pumping
    Salute,    // stand to attention and salute the viewer
    Moonwalk,  // slide backwards, cool as you like
    Toot,      // a little puff from behind, then whistle innocently (the others notice)
    Peekaboo,  // hands over the eyes, then... peekaboo!
    Flap,      // elbows out, strutting like a chicken
    Hide,      // hands over the eyes, trembling, peeking through the fingers
    Fret,      // biting its nails, eyes darting, a drop of sweat
    Faint,     // keel over backwards, out cold, then come round
    Dig,       // dig at the ground in a cloud of dust, find nothing, shrug
    Sleepwalk, // wander forward fast asleep, arms out, Zzz, until it bumps into something
    Daydream,  // gaze off into the distance with a dreamy smile, then snap out of it
    Kiss,      // blow a kiss at the cursor, a heart floating off
    // Mr. Bean's own.
    Teddy,  // bring Teddy out of the jacket, gaze at him, make him wave at you, tuck him back
    Hello, // the awkward hello: arm straight up beside the head, hand wagging stiffly, up on the toes
    Waggle, // lean in, hands behind the back, and waggle the eyebrows at you
    Sneak, // look both ways, then creep forward on tiptoe, hands up, and freeze with an innocent face
    Fuss,  // straighten the tie, brush the lapels, pat the hair
    Ponder, // finger to the chin, eyes rolled up, then: an idea, finger in the air
    Jive,  // the Bean dance: elbows pumping, hips twisting, knees bouncing
    Drive, // mime driving his Mini: hands on the wheel, leaning into the corners, a honk
    Photo, // hold up a camera, take your picture (flash!), study it and show it off
    Poke,  // creep up and prod somebody (or the cursor) with one finger, then look innocent
    // The Angry Birds' own. Short, loud and physical: nothing like a Minion's habits.
    Peck,    // hop up and jab somebody (or the cursor) hard with the beak, twice
    Fly,     // flap off along the ground, wings beating, feet trailing
    Launch,  // slingshot launch: crouch, spring high across the sky and land running
    Rage,    // bristle up, shake and stamp, steam coming off the head
    Egg,     // strain, lay an egg on the ground, gaze at it proudly
    Inflate, // blow up huge and round, wobbling, then let it all out
    Blast,   // Bomb goes off: white flash, bang, smoke, ears ringing
}
impl Antic {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => Antic::Look,
            1 => Antic::Wave,
            2 => Antic::Rest,
            3 => Antic::Dance,
            4 => Antic::Twirl,
            5 => Antic::Point,
            6 => Antic::Yawn,
            7 => Antic::Doze,
            8 => Antic::Scratch,
            9 => Antic::Whistle,
            10 => Antic::Sneeze,
            11 => Antic::Box,
            12 => Antic::Taunt,
            13 => Antic::Stomp,
            14 => Antic::Flex,
            15 => Antic::Joy,
            16 => Antic::Laugh,
            17 => Antic::Hiccups,
            18 => Antic::Shrug,
            19 => Antic::Trip,
            20 => Antic::Boo,
            21 => Antic::Strum,
            22 => Antic::Pushups,
            23 => Antic::Jog,
            24 => Antic::Salute,
            25 => Antic::Moonwalk,
            26 => Antic::Toot,
            27 => Antic::Peekaboo,
            28 => Antic::Flap,
            29 => Antic::Hide,
            30 => Antic::Fret,
            31 => Antic::Faint,
            32 => Antic::Dig,
            33 => Antic::Sleepwalk,
            34 => Antic::Daydream,
            35 => Antic::Kiss,
            36 => Antic::Teddy,
            37 => Antic::Hello,
            38 => Antic::Waggle,
            39 => Antic::Sneak,
            40 => Antic::Fuss,
            41 => Antic::Ponder,
            42 => Antic::Jive,
            43 => Antic::Drive,
            44 => Antic::Photo,
            45 => Antic::Poke,
            46 => Antic::Peck,
            47 => Antic::Fly,
            48 => Antic::Launch,
            49 => Antic::Rage,
            50 => Antic::Egg,
            51 => Antic::Inflate,
            52 => Antic::Blast,
            _ => Self::default(),
        }
    }
}
impl std::fmt::Display for Antic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// One antic and how often, relative to the others, the walker picks it.
#[derive(Clone, Default, Copy)]
pub struct Habit {
    pub antic: Antic,
    pub weight: f64,
}

impl Habit {
    pub fn new(mut antic: Antic, mut weight: f64) -> Habit {
        let mut this = Habit::default();
        this.ctor(antic, weight);
        this
    }
    fn ctor(&mut self, mut antic: Antic, mut weight: f64) {
        self.antic = antic;
        self.weight = weight;
    }
}

// What makes one walker different from another.
#[derive(Clone, Default)]
pub struct PersonalityData {
    pub name: String,
    pub temperament: Temperament,
    pub look: Look,
    // The face it wears when nothing in particular is happening.
    pub base_face: Expression,
    // The face it settles into on a long rest.
    pub rest_face: Expression,
    // Multiplier on the base walking speed.
    pub speed_factor: f64,
    // How much the upper body bobs with each step, in units, and how far forward it leans.
    pub walk_bob: f64,
    pub walk_lean: f64,
    // Seconds between hops (random in this range) - and only while it has energy to spare.
    pub hop_min: f64,
    pub hop_max: f64,
    // Seconds between idle antics (random in this range).
    pub idle_min: f64,
    pub idle_max: f64,
    // How slowly it tires: 1 is normal, less tires quickly.
    pub stamina: f64,
    // How quickly it starts wanting company, 0..1.
    pub sociable: f64,
    // How quickly the cursor starts to intrigue it, 0..1.
    pub curious: f64,
    // How often it trips over its own feet: 0 never, 1 every minute or two of walking.
    pub clumsy: f64,
    // Its build: width and height relative to the standard Minion (1 is standard).
    pub build_x: f64,
    pub build_y: f64,
    // Copies whatever antic it sees somebody else start.
    pub mimic: bool,
    // What it does on reaching the cursor after walking over to inspect it.
    pub cursor_antic: Antic,
    // What it does with an idle moment, each weighted; the weights need not add up to anything.
    pub habits: Arr<Habit>,
}
rt::handle!(Personality, PersonalityData);

//
// Mr. Bean. Not one of the Minions: he comes out alongside them when
// switched on in the menu. Tall, thin, in his tweed jacket and red
// tie. Shows off Teddy (the Minions come to look), says hello with
// a stiff wave, waggles his eyebrows at you, creeps about on tiptoe,
// fusses with his tie and hair, ponders, does the Bean dance, mimes
// driving his Mini and takes your picture - and pokes anyone standing
// quietly, and your cursor, with one finger.
//
pub static PERSONALITY_BEAN: rt::Global<Personality> = rt::Global::new(|| {
    Personality::new(
        "Mr. Bean".to_string(),
        Temperament::Eccentric,
        (*LOOK_BEAN),
        (*EXPRESSION_GORMLESS),
        (*EXPRESSION_KNOWING),
        0.95,
        0.22,
        0.05,
        60.0,
        120.0,
        4.0,
        9.0,
        0.9,
        0.9,
        1.0,
        0.0,
        1.0,
        1.0,
        false,
        Antic::Poke,
        Arr::from_vec(vec![
            Personality::h(Antic::Teddy, 3.5),
            Personality::h(Antic::Hello, 2.0),
            Personality::h(Antic::Waggle, 2.0),
            Personality::h(Antic::Sneak, 2.0),
            Personality::h(Antic::Fuss, 2.0),
            Personality::h(Antic::Ponder, 1.5),
            Personality::h(Antic::Jive, 1.5),
            Personality::h(Antic::Drive, 2.0),
            Personality::h(Antic::Photo, 1.8),
            Personality::h(Antic::Look, 1.5),
            Personality::h(Antic::Rest, 1.0),
            Personality::h(Antic::Doze, 1.0),
            Personality::h(Antic::Whistle, 0.5),
            Personality::h(Antic::Hiccups, 0.5),
            Personality::h(Antic::Sneeze, 0.4),
        ]),
    )
});

//
// The Pink Panther. Not one of the Minions: he comes out alongside
// them when switched on in the menu. Tall, sleek and silent, all in
// pink with a long curling tail, yellow eyes and a dark pink nose.
// Slinks on tiptoe, prowls, grooms a paw, ponders with a finger to
// the chin, twirls, dances and mimes mischief - and pokes anyone
// standing quietly, and your cursor, with one claw.
//
pub static PERSONALITY_PANTHER: rt::Global<Personality> = rt::Global::new(|| {
    Personality::new(
        "Pink Panther".to_string(),
        Temperament::Suave,
        (*LOOK_PANTHER),
        (*EXPRESSION_COOL),
        (*EXPRESSION_KNOWING),
        1.0,
        0.30,
        0.10,
        18.0,
        40.0,
        4.0,
        9.0,
        1.1,
        0.8,
        0.9,
        0.0,
        1.0,
        1.0,
        false,
        Antic::Poke,
        Arr::from_vec(vec![
            Personality::h(Antic::Sneak, 3.0),
            Personality::h(Antic::Wave, 2.5),
            Personality::h(Antic::Dance, 2.0),
            Personality::h(Antic::Twirl, 2.0),
            Personality::h(Antic::Ponder, 2.0),
            Personality::h(Antic::Jive, 1.5),
            Personality::h(Antic::Poke, 1.5),
            Personality::h(Antic::Look, 2.5),
            Personality::h(Antic::Rest, 1.5),
            Personality::h(Antic::Scratch, 1.5),
            Personality::h(Antic::Joy, 1.2),
            Personality::h(Antic::Laugh, 1.0),
            Personality::h(Antic::Doze, 0.8),
            Personality::h(Antic::Whistle, 1.0),
            Personality::h(Antic::Sneeze, 0.4),
            Personality::h(Antic::Hiccups, 0.4),
        ]),
    )
});

//
// The Minions, in the order they join as the count goes up. Kevin is
// the first out.
//
pub static PERSONALITY_ROSTER: rt::Global<Arr<Personality>> = rt::Global::new(|| {
    Arr::from_vec(vec![
        Personality::new(
            "Kevin".to_string(),
            Temperament::Calm,
            Look::new_4(false, HairStyle::Sprouts),
            (*EXPRESSION_NEUTRAL),
            (*EXPRESSION_SLEEPY),
            1.0,
            0.30,
            0.0,
            25.0,
            45.0,
            5.0,
            11.0,
            0.7,
            0.6,
            1.0,
            0.0,
            1.0,
            1.0,
            false,
            Antic::Scratch,
            Arr::from_vec(vec![
                Personality::h(Antic::Look, 3.0),
                Personality::h(Antic::Rest, 2.0),
                Personality::h(Antic::Whistle, 2.0),
                Personality::h(Antic::Scratch, 2.0),
                Personality::h(Antic::Doze, 1.5),
                Personality::h(Antic::Yawn, 1.0),
                Personality::h(Antic::Hiccups, 0.8),
                Personality::h(Antic::Wave, 1.0),
                Personality::h(Antic::Sneeze, 0.5),
            ]),
        ),
        Personality::new(
            "Stuart".to_string(),
            Temperament::Aggressive,
            Look::new_4(true, HairStyle::Combed),
            (*EXPRESSION_GRUMPY),
            (*EXPRESSION_BORED),
            1.2,
            0.38,
            0.2,
            12.0,
            25.0,
            6.0,
            14.0,
            1.2,
            0.3,
            0.6,
            0.0,
            1.0,
            1.0,
            false,
            Antic::Taunt,
            Arr::from_vec(vec![
                Personality::h(Antic::Look, 2.5),
                Personality::h(Antic::Box, 2.0),
                Personality::h(Antic::Taunt, 2.0),
                Personality::h(Antic::Stomp, 1.0),
                Personality::h(Antic::Laugh, 0.8),
                Personality::h(Antic::Rest, 0.8),
            ]),
        ),
        Personality::new(
            "Bob".to_string(),
            Temperament::Friendly,
            Look::new(false, HairStyle::None, Look::GREEN, Look::BROWN),
            (*EXPRESSION_HAPPY),
            (*EXPRESSION_HAPPY),
            0.9,
            0.40,
            0.05,
            18.0,
            35.0,
            5.0,
            11.0,
            1.0,
            1.0,
            0.7,
            1.0,
            1.0,
            1.0,
            false,
            Antic::Point,
            Arr::from_vec(vec![
                Personality::h(Antic::Wave, 3.0),
                Personality::h(Antic::Dance, 2.5),
                Personality::h(Antic::Point, 1.5),
                Personality::h(Antic::Joy, 1.5),
                Personality::h(Antic::Laugh, 1.0),
                Personality::h(Antic::Rest, 1.0),
                Personality::h(Antic::Look, 1.0),
                Personality::h(Antic::Hiccups, 0.5),
                Personality::h(Antic::Sneeze, 0.3),
            ]),
        ),
        Personality::new(
            "Dave".to_string(),
            Temperament::Friendly,
            Look::new_4(false, HairStyle::Parted),
            (*EXPRESSION_CHEERFUL),
            (*EXPRESSION_HAPPY),
            0.95,
            0.42,
            0.0,
            12.0,
            25.0,
            5.0,
            11.0,
            1.4,
            0.8,
            0.8,
            0.0,
            1.0,
            1.0,
            false,
            Antic::Point,
            Arr::from_vec(vec![
                Personality::h(Antic::Wave, 2.5),
                Personality::h(Antic::Twirl, 2.5),
                Personality::h(Antic::Point, 2.0),
                Personality::h(Antic::Joy, 1.5),
                Personality::h(Antic::Dance, 1.0),
                Personality::h(Antic::Look, 1.0),
                Personality::h(Antic::Rest, 0.8),
            ]),
        ),
        Personality::new(
            "Mel".to_string(),
            Temperament::Aggressive,
            Look::new_4(false, HairStyle::Crew),
            (*EXPRESSION_STERN),
            (*EXPRESSION_BORED),
            1.15,
            0.34,
            0.25,
            15.0,
            30.0,
            6.0,
            14.0,
            1.3,
            0.3,
            0.5,
            0.0,
            1.0,
            1.0,
            false,
            Antic::Taunt,
            Arr::from_vec(vec![
                Personality::h(Antic::Look, 2.0),
                Personality::h(Antic::Flex, 2.5),
                Personality::h(Antic::Stomp, 1.5),
                Personality::h(Antic::Box, 1.5),
                Personality::h(Antic::Taunt, 0.7),
                Personality::h(Antic::Rest, 0.8),
            ]),
        ),
        Personality::new(
            "Jerry".to_string(),
            Temperament::Friendly,
            Look::new_4(false, HairStyle::Spiky),
            (*EXPRESSION_COOL),
            (*EXPRESSION_HAPPY),
            0.95,
            0.36,
            0.0,
            15.0,
            30.0,
            5.0,
            10.0,
            1.1,
            0.7,
            0.6,
            0.0,
            1.0,
            0.9,
            false,
            Antic::Strum,
            Arr::from_vec(vec![
                Personality::h(Antic::Strum, 3.0),
                Personality::h(Antic::Dance, 2.0),
                Personality::h(Antic::Whistle, 1.5),
                Personality::h(Antic::Wave, 1.5),
                Personality::h(Antic::Laugh, 1.0),
                Personality::h(Antic::Look, 1.0),
                Personality::h(Antic::Rest, 0.8),
                Personality::h(Antic::Joy, 0.7),
            ]),
        ),
        Personality::new(
            "Tim".to_string(),
            Temperament::Calm,
            Look::new_3(false, HairStyle::Quiff, Look::BLUE),
            (*EXPRESSION_SERIOUS),
            (*EXPRESSION_PROUD),
            1.05,
            0.26,
            0.05,
            20.0,
            40.0,
            5.0,
            11.0,
            1.6,
            0.4,
            0.5,
            0.0,
            0.94,
            1.12,
            false,
            Antic::Salute,
            Arr::from_vec(vec![
                Personality::h(Antic::Pushups, 2.5),
                Personality::h(Antic::Jog, 2.5),
                Personality::h(Antic::Salute, 2.5),
                Personality::h(Antic::Look, 2.0),
                Personality::h(Antic::Flex, 0.8),
                Personality::h(Antic::Rest, 0.8),
                Personality::h(Antic::Stomp, 0.4),
            ]),
        ),
        Personality::new(
            "Phil".to_string(),
            Temperament::Goofy,
            Look::new_4(true, HairStyle::Tuft),
            (*EXPRESSION_SILLY),
            (*EXPRESSION_HAPPY),
            1.0,
            0.44,
            0.0,
            10.0,
            22.0,
            4.0,
            9.0,
            1.2,
            0.8,
            0.9,
            0.6,
            1.0,
            1.0,
            true,
            Antic::Peekaboo,
            Arr::from_vec(vec![
                Personality::h(Antic::Moonwalk, 2.2),
                Personality::h(Antic::Toot, 2.0),
                Personality::h(Antic::Peekaboo, 2.0),
                Personality::h(Antic::Flap, 2.5),
                Personality::h(Antic::Laugh, 1.5),
                Personality::h(Antic::Dance, 1.0),
                Personality::h(Antic::Taunt, 0.8),
                Personality::h(Antic::Hiccups, 0.6),
                Personality::h(Antic::Look, 0.8),
            ]),
        ),
        Personality::new(
            "Carl".to_string(),
            Temperament::Timid,
            Look::new_3(false, HairStyle::Wild, Look::AMBER),
            (*EXPRESSION_NERVOUS),
            (*EXPRESSION_WORRIED),
            1.1,
            0.34,
            0.1,
            30.0,
            60.0,
            4.0,
            9.0,
            0.9,
            0.5,
            0.8,
            0.0,
            0.96,
            0.96,
            false,
            Antic::Hide,
            Arr::from_vec(vec![
                Personality::h(Antic::Look, 3.0),
                Personality::h(Antic::Fret, 2.5),
                Personality::h(Antic::Hide, 2.0),
                Personality::h(Antic::Rest, 1.5),
                Personality::h(Antic::Shrug, 0.8),
                Personality::h(Antic::Hiccups, 0.6),
                Personality::h(Antic::Sneeze, 0.4),
            ]),
        ),
        Personality::new(
            "Norbert".to_string(),
            Temperament::Calm,
            Look::new_4(true, HairStyle::None),
            (*EXPRESSION_DOPEY),
            (*EXPRESSION_SLEEPY),
            0.85,
            0.32,
            0.0,
            30.0,
            60.0,
            5.0,
            12.0,
            0.8,
            0.5,
            0.9,
            0.3,
            1.1,
            0.96,
            false,
            Antic::Kiss,
            Arr::from_vec(vec![
                Personality::h(Antic::Daydream, 3.0),
                Personality::h(Antic::Dig, 2.0),
                Personality::h(Antic::Sleepwalk, 1.5),
                Personality::h(Antic::Kiss, 1.5),
                Personality::h(Antic::Doze, 1.2),
                Personality::h(Antic::Scratch, 1.0),
                Personality::h(Antic::Look, 1.0),
                Personality::h(Antic::Hiccups, 0.8),
                Personality::h(Antic::Rest, 0.8),
            ]),
        ),
    ])
});

//
// The Angry Birds flock, in the order they join as the bird count
// goes up. Red is the first out. Round, hoppy and short-fused: they
// hop more than they walk, flutter down slowly, peck quiet sitters
// and your cursor, squabble, launch, lay eggs, inflate and explode -
// nothing like a Minion's habits.
//
pub static PERSONALITY_BIRD_ROSTER: rt::Global<Arr<Personality>> = rt::Global::new(|| {
    Arr::from_vec(vec![
        Personality::new(
            "Red".to_string(),
            Temperament::Feisty,
            Look::new_2(BirdKind::Red),
            (*EXPRESSION_GRUMPY),
            (*EXPRESSION_BORED),
            1.0,
            0.50,
            0.1,
            4.0,
            10.0,
            4.0,
            9.0,
            1.1,
            0.6,
            0.8,
            0.0,
            1.0,
            1.0,
            false,
            Antic::Peck,
            Arr::from_vec(vec![
                Personality::h(Antic::Rage, 2.5),
                Personality::h(Antic::Peck, 2.0),
                Personality::h(Antic::Launch, 2.0),
                Personality::h(Antic::Fly, 1.0),
                Personality::h(Antic::Look, 1.5),
                Personality::h(Antic::Rest, 1.0),
                Personality::h(Antic::Dance, 1.0),
                Personality::h(Antic::Yawn, 0.6),
                Personality::h(Antic::Sneeze, 0.4),
            ]),
        ),
        Personality::new(
            "Blues".to_string(),
            Temperament::Feisty,
            Look::new_2(BirdKind::Blues),
            (*EXPRESSION_HAPPY),
            (*EXPRESSION_HAPPY),
            1.15,
            0.55,
            0.05,
            3.0,
            8.0,
            4.0,
            8.0,
            1.3,
            0.9,
            0.9,
            0.3,
            0.8,
            0.8,
            false,
            Antic::Peck,
            Arr::from_vec(vec![
                Personality::h(Antic::Peck, 2.0),
                Personality::h(Antic::Fly, 2.0),
                Personality::h(Antic::Dance, 2.0),
                Personality::h(Antic::Joy, 1.5),
                Personality::h(Antic::Taunt, 1.0),
                Personality::h(Antic::Look, 1.5),
                Personality::h(Antic::Rest, 0.8),
                Personality::h(Antic::Hiccups, 0.5),
            ]),
        ),
        Personality::new(
            "Chuck".to_string(),
            Temperament::Feisty,
            Look::new_2(BirdKind::Chuck),
            (*EXPRESSION_DETERMINED),
            (*EXPRESSION_BORED),
            1.35,
            0.55,
            0.15,
            3.0,
            8.0,
            4.0,
            8.0,
            1.4,
            0.7,
            0.7,
            0.0,
            0.9,
            0.95,
            false,
            Antic::Peck,
            Arr::from_vec(vec![
                Personality::h(Antic::Fly, 3.0),
                Personality::h(Antic::Joy, 2.0),
                Personality::h(Antic::Peck, 1.5),
                Personality::h(Antic::Dance, 1.5),
                Personality::h(Antic::Taunt, 1.0),
                Personality::h(Antic::Look, 1.0),
                Personality::h(Antic::Rest, 0.7),
            ]),
        ),
        Personality::new(
            "Bomb".to_string(),
            Temperament::Feisty,
            Look::new_2(BirdKind::Bomb),
            (*EXPRESSION_STERN),
            (*EXPRESSION_SLEEPY),
            0.9,
            0.45,
            0.05,
            5.0,
            12.0,
            5.0,
            10.0,
            1.0,
            0.4,
            0.5,
            0.0,
            1.15,
            1.1,
            false,
            Antic::Peck,
            Arr::from_vec(vec![
                Personality::h(Antic::Blast, 2.5),
                Personality::h(Antic::Rage, 2.0),
                Personality::h(Antic::Doze, 1.5),
                Personality::h(Antic::Rest, 1.5),
                Personality::h(Antic::Look, 1.0),
            ]),
        ),
        Personality::new(
            "Matilda".to_string(),
            Temperament::Feisty,
            Look::new_2(BirdKind::Matilda),
            (*EXPRESSION_SERIOUS),
            (*EXPRESSION_HAPPY),
            0.9,
            0.45,
            0.05,
            5.0,
            12.0,
            5.0,
            10.0,
            1.0,
            0.8,
            0.6,
            0.0,
            1.05,
            1.0,
            false,
            Antic::Peck,
            Arr::from_vec(vec![
                Personality::h(Antic::Egg, 3.0),
                Personality::h(Antic::Rest, 2.0),
                Personality::h(Antic::Look, 1.5),
                Personality::h(Antic::Dance, 1.0),
                Personality::h(Antic::Rage, 1.0),
                Personality::h(Antic::Hiccups, 0.5),
            ]),
        ),
        Personality::new(
            "Hal".to_string(),
            Temperament::Feisty,
            Look::new_2(BirdKind::Hal),
            (*EXPRESSION_CHEERFUL),
            (*EXPRESSION_HAPPY),
            1.0,
            0.50,
            0.1,
            4.0,
            10.0,
            4.0,
            9.0,
            1.2,
            0.6,
            0.7,
            0.0,
            1.0,
            1.0,
            false,
            Antic::Peck,
            Arr::from_vec(vec![
                Personality::h(Antic::Fly, 2.5),
                Personality::h(Antic::Peck, 2.0),
                Personality::h(Antic::Whistle, 1.5),
                Personality::h(Antic::Look, 1.5),
                Personality::h(Antic::Rest, 1.0),
            ]),
        ),
        Personality::new(
            "Terence".to_string(),
            Temperament::Feisty,
            Look::new_2(BirdKind::Terence),
            (*EXPRESSION_STERN),
            (*EXPRESSION_BORED),
            0.85,
            0.40,
            0.1,
            6.0,
            14.0,
            5.0,
            11.0,
            1.5,
            0.3,
            0.4,
            0.0,
            1.3,
            1.25,
            false,
            Antic::Peck,
            Arr::from_vec(vec![
                Personality::h(Antic::Rage, 2.0),
                Personality::h(Antic::Stomp, 2.0),
                Personality::h(Antic::Rest, 2.0),
                Personality::h(Antic::Look, 1.5),
                Personality::h(Antic::Peck, 1.0),
            ]),
        ),
        Personality::new(
            "Bubbles".to_string(),
            Temperament::Feisty,
            Look::new_2(BirdKind::Bubbles),
            (*EXPRESSION_HAPPY),
            (*EXPRESSION_HAPPY),
            1.0,
            0.55,
            0.05,
            3.0,
            8.0,
            4.0,
            8.0,
            1.2,
            0.9,
            0.8,
            0.0,
            0.9,
            0.9,
            false,
            Antic::Peck,
            Arr::from_vec(vec![
                Personality::h(Antic::Inflate, 3.0),
                Personality::h(Antic::Joy, 2.0),
                Personality::h(Antic::Dance, 1.5),
                Personality::h(Antic::Peck, 1.5),
                Personality::h(Antic::Look, 1.0),
            ]),
        ),
        Personality::new(
            "Stella".to_string(),
            Temperament::Feisty,
            Look::new_2(BirdKind::Stella),
            (*EXPRESSION_CHEERFUL),
            (*EXPRESSION_HAPPY),
            1.05,
            0.55,
            0.05,
            3.0,
            8.0,
            4.0,
            8.0,
            1.2,
            0.9,
            0.8,
            0.0,
            0.95,
            0.95,
            false,
            Antic::Peck,
            Arr::from_vec(vec![
                Personality::h(Antic::Fly, 2.0),
                Personality::h(Antic::Dance, 2.0),
                Personality::h(Antic::Wave, 1.5),
                Personality::h(Antic::Peck, 1.5),
                Personality::h(Antic::Look, 1.5),
            ]),
        ),
        Personality::new(
            "Silver".to_string(),
            Temperament::Feisty,
            Look::new_2(BirdKind::Silver),
            (*EXPRESSION_CHEERFUL),
            (*EXPRESSION_HAPPY),
            1.1,
            0.55,
            0.1,
            3.0,
            9.0,
            4.0,
            8.0,
            1.2,
            0.7,
            1.0,
            0.2,
            1.0,
            1.0,
            false,
            Antic::Peck,
            Arr::from_vec(vec![
                Personality::h(Antic::Fly, 2.5),
                Personality::h(Antic::Launch, 2.0),
                Personality::h(Antic::Peck, 1.5),
                Personality::h(Antic::Dance, 1.5),
                Personality::h(Antic::Look, 1.5),
                Personality::h(Antic::Rest, 0.8),
                Personality::h(Antic::Whistle, 0.8),
            ]),
        ),
    ])
});

impl Personality {
    fn alloc__() -> Personality {
        let mut this = Personality::from_data(PersonalityData::default());
        this
    }
    pub fn new(
        mut name: String,
        mut temperament: Temperament,
        mut look: Look,
        mut base_face: Expression,
        mut rest_face: Expression,
        mut speed_factor: f64,
        mut walk_bob: f64,
        mut walk_lean: f64,
        mut hop_min: f64,
        mut hop_max: f64,
        mut idle_min: f64,
        mut idle_max: f64,
        mut stamina: f64,
        mut sociable: f64,
        mut curious: f64,
        mut clumsy: f64,
        mut build_x: f64,
        mut build_y: f64,
        mut mimic: bool,
        mut cursor_antic: Antic,
        mut habits: Arr<Habit>,
    ) -> Personality {
        let mut this = Self::alloc__();
        this.ctor(
            name,
            temperament,
            look,
            base_face,
            rest_face,
            speed_factor,
            walk_bob,
            walk_lean,
            hop_min,
            hop_max,
            idle_min,
            idle_max,
            stamina,
            sociable,
            curious,
            clumsy,
            build_x,
            build_y,
            mimic,
            cursor_antic,
            habits,
        );
        this
    }
    fn ctor(
        mut self,
        mut name: String,
        mut temperament: Temperament,
        mut look: Look,
        mut base_face: Expression,
        mut rest_face: Expression,
        mut speed_factor: f64,
        mut walk_bob: f64,
        mut walk_lean: f64,
        mut hop_min: f64,
        mut hop_max: f64,
        mut idle_min: f64,
        mut idle_max: f64,
        mut stamina: f64,
        mut sociable: f64,
        mut curious: f64,
        mut clumsy: f64,
        mut build_x: f64,
        mut build_y: f64,
        mut mimic: bool,
        mut cursor_antic: Antic,
        mut habits: Arr<Habit>,
    ) {
        self.name = name.clone();
        self.temperament = temperament;
        self.look = look;
        self.base_face = base_face;
        self.rest_face = rest_face;
        self.speed_factor = speed_factor;
        self.walk_bob = walk_bob;
        self.walk_lean = walk_lean;
        self.hop_min = hop_min;
        self.hop_max = hop_max;
        self.idle_min = idle_min;
        self.idle_max = idle_max;
        self.stamina = stamina;
        self.sociable = sociable;
        self.curious = curious;
        self.clumsy = clumsy;
        self.build_x = build_x;
        self.build_y = build_y;
        self.mimic = mimic;
        self.cursor_antic = cursor_antic;
        self.habits = habits.clone();
    }
    // Friendly, goofy, Mr. Bean, a bird or the Panther: waves, dances, plays tag, chats.
    pub fn warm(mut self) -> bool {
        return (((((self.temperament == Temperament::Friendly)
            || (self.temperament == Temperament::Goofy))
            || (self.temperament == Temperament::Eccentric))
            || (self.temperament == Temperament::Feisty))
            || (self.temperament == Temperament::Suave));
    }
    // Which painter draws it and which body it has.
    pub fn figure(mut self) -> Figure {
        return self.look.figure;
    }
    pub fn anatomy(mut self) -> Anatomy {
        return self.look.anatomy();
    }
    pub fn h(mut antic: Antic, mut weight: f64) -> Habit {
        return Habit::new(antic, weight);
    }
}
