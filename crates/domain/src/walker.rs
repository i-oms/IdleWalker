// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

// How a walker says hello: a nod, a wave, or a little dance together.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(i32)]
pub enum GreetStyle {
    #[default]
    Nod,
    Wave,
    Dance,
}
impl GreetStyle {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => GreetStyle::Nod,
            1 => GreetStyle::Wave,
            2 => GreetStyle::Dance,
            _ => Self::default(),
        }
    }
}
impl std::fmt::Display for GreetStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// A short, in-character response to something that just happened.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(i32)]
pub enum Reaction {
    #[default]
    Gasp, // hands up by the face, then a worried look
    Headshake, // hands on hips, shaking its head
    Jeer,      // laughing and pointing
    Chuckle,   // laughing, hands on the belly
    Huff,      // turned away, hands on hips, grumpy
    Cower,     // crouched with the arms over the head, trembling
    Bristle,   // feathers puffed, wings half out, beak open: back off (Angry Birds)
}
impl Reaction {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => Reaction::Gasp,
            1 => Reaction::Headshake,
            2 => Reaction::Jeer,
            3 => Reaction::Chuckle,
            4 => Reaction::Huff,
            5 => Reaction::Cower,
            6 => Reaction::Bristle,
            _ => Self::default(),
        }
    }
}
impl std::fmt::Display for Reaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Something a walker has set out to do that means walking somewhere first.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(i32)]
pub enum Goal {
    #[default]
    None,
    Visit,   // walk up to another walker for a chat, or to comfort it
    Prank,   // sneak up on somebody standing quietly and shout boo
    Inspect, // walk over to underneath the cursor and have a look at it
}
impl Goal {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => Goal::None,
            1 => Goal::Visit,
            2 => Goal::Prank,
            3 => Goal::Inspect,
            _ => Self::default(),
        }
    }
}
impl std::fmt::Display for Goal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// What two walkers standing face to face are doing.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(i32)]
pub enum ChatStyle {
    #[default]
    Chat, // taking turns to talk, with gestures; ends in a laugh
    Argue, // a shouting match with stomping; ends in a huff
    Pat,   // one comforting the other
}
impl ChatStyle {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => ChatStyle::Chat,
            1 => ChatStyle::Argue,
            2 => ChatStyle::Pat,
            _ => Self::default(),
        }
    }
}
impl std::fmt::Display for ChatStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(i32)]
pub enum Mode {
    #[default]
    Hidden,
    PeekRise,
    PeekLook,
    PeekDuck,
    Air,
    Walk,
    Crouch,
    Turn,
    Look,
    Wave,
    Rest,
    Exit,
    Greet,  // stopped to wave, nod or dance at another walker
    Charge, // running at another walker to shove it (or, playfully, to tag it)
    Gloat,  // standing over a shoved walker
    Flee,   // running away after being shoved (or, playfully, from a chaser)
    Antic,  // one of the idle antics (dancing, boxing, dozing, ...)
    React,  // a short reaction: to a shove it saw, to being tagged, ...
    Chat,   // face to face with another walker: talking, arguing or comforting
}
impl Mode {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => Mode::Hidden,
            1 => Mode::PeekRise,
            2 => Mode::PeekLook,
            3 => Mode::PeekDuck,
            4 => Mode::Air,
            5 => Mode::Walk,
            6 => Mode::Crouch,
            7 => Mode::Turn,
            8 => Mode::Look,
            9 => Mode::Wave,
            10 => Mode::Rest,
            11 => Mode::Exit,
            12 => Mode::Greet,
            13 => Mode::Charge,
            14 => Mode::Gloat,
            15 => Mode::Flee,
            16 => Mode::Antic,
            17 => Mode::React,
            18 => Mode::Chat,
            _ => Self::default(),
        }
    }
}
impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

//
// One character: where it is, what it is doing and how it moves. Pure
// simulation - it knows nothing about windows or pixels on screen beyond
// what `IWorld` tells it. Walkers only meet each other
// through the `Troupe`. The idle antics and the reactions
// live in WalkerAntics.cs; what it wants and the errands and chats that
// come of it live in WalkerBrain.cs.
//
#[derive(Clone, Default)]
pub struct WalkerData {
    pub world: IWorld,
    pub rng: Random,
    pub personality: Personality,
    pub body: Anatomy,
    pub bird: bool,
    pub puffs: Arr<Puff>,
    pub mode: Mode,
    pub mode_time: f64,
    pub time: f64,
    pub scale: i32,
    pub size_factor: f64,
    pub world_left: i32,
    pub world_right: i32,
    pub cursor_x: i32,
    pub cursor_y: i32,
    pub company: Walker,
    pub company_nearby: bool,
    pub company_x: f64,
    pub company_eye_y: f64,
    pub wave_at_cursor: bool,
    pub face: Expression,
    pub gaze_x: f64,
    pub gaze_y: f64,
    pub yaw: f64,
    pub cx: f64,
    pub feet_y: f64,
    pub vx: f64,
    pub vy: f64,
    pub clip_y: f64,
    pub facing: i32,
    pub head_turned: bool,
    pub walk_phase: f64,
    pub squash: f64,
    pub braking: bool,
    pub after_brake: Mode,
    pub after_turn: Mode,
    pub pending_jump: f64,
    pub pending_air_speed: f64,
    pub land_pause: f64,
    pub rest_duration: f64,
    pub idle_timer: f64,
    pub hop_timer: f64,
    pub trip_timer: f64,
    pub blink_timer: f64,
    pub blink_left: f64,
    pub blink_again: bool,
    pub startled: bool,
    pub greet_facing: i32,
    pub greet_style: GreetStyle,
    pub knocked: bool,
    pub playful: bool,
    pub flee_time: f64,
    pub charge_target: Walker,
    // Antics and reactions (WalkerAntics.cs).
    pub antic: Antic,
    pub antic_seconds: f64,
    pub antic_stage: i32,
    pub reaction: Reaction,
    pub react_seconds: f64,
    pub react_facing: i32,
    pub react_x: f64,
    pub joy_hops: i32,
    pub celebrating: bool,
    pub woke_up: bool,
    pub spooked: bool,
    // What it wants, and the errands and chats that follow (WalkerBrain.cs).
    pub energy: f64,
    pub social: f64,
    pub curiosity: f64,
    pub goal: Goal,
    pub goal_target: Walker,
    pub goal_x: f64,
    pub goal_time: f64,
    pub broadcast: Option<Antic>,
    pub has_pending: bool,
    pub pending_antic: Antic,
    pub pending_in: f64,
    pub catch_cooldown: f64,
    pub chat_style: ChatStyle,
    pub chat_partner: Walker,
    pub chat_lead: bool,
    pub chat_facing: i32,
    pub chat_seconds: f64,
    pub last_stomp: i32,
    // Small touches laid over whatever else is going on.
    pub sweat: f64,
    pub mood_face: Expression,
    pub mood_left: f64,
    pub upset: bool,
    pub glance_timer: f64,
    pub glance_left: f64,
    pub glance_dx: f64,
    pub glance_dy: f64,
    // The overalls colour (0xRRGGBB); the troupe hands it out.
    pub dress: i32,
    pub eye_beat: EyeBeat,
    pub beat_time: f64,
    pub beat_timer: f64,
    pub beat_dir: f64,
    pub was_watching: bool,
    pub last_gx: f64,
    pub last_gy: f64,
    pub cross_eye: f64,
    pub pupil_interest: f64,
    pub values: Arr2<f64>,
    pub older_state: i32,
    pub older_choice: i32,
    pub older_feat: Arr<f64>,
    pub replay_feat: Arr<Arr<f64>>,
    pub replay_choice: Arr<i32>,
    pub replay_target: Arr<f64>,
    pub replay_count: i32,
    pub replay_pos: i32,
    pub fondness: Dict<Walker, f64>,
    pub place: Arr<f64>,
    pub steer_cooldown: f64,
    pub bond_cooldown: f64,
    pub has_prev: bool,
    pub prev_state: i32,
    pub prev_choice: i32,
    pub prev_social: f64,
    pub prev_curiosity: f64,
    pub prev_energy: f64,
    pub prev_cheer: f64,
    pub prev_fear: f64,
    pub pending_bonus: f64,
    pub want_at_visit: f64,
    // Lifelike layers: lingering cheer/dread, novelty counts, anti-routine.
    pub cheer: f64,
    pub fear: f64,
    pub use_count: Arr<i32>,
    pub last_choice: i32,
    pub repeat_count: i32,
    pub log: Arr<MemoryEvent>,
    pub log_pos: i32,
    pub optimism: f64,
    // Upgrade v2: lifetime stats for annealing + dashboard. Persisted.
    pub lifetime_decisions: i32,
    pub total_reward: f64,
    pub mlp_w1: Arr2<f64>,
    pub mlp_b1: Arr<f64>,
    pub mlp_w2: Arr2<f64>,
    pub mlp_b2: Arr<f64>,
    pub prev_feat: Arr<f64>,
    // What each dimension means: 0 bob, 1 lean, 2 speed, 3 hop, 4 swagger, 5 fidget.
    pub style: Arr<f64>,
    pub style_ready: bool,
    pub swagger_seed: f64,
    pub fidget_seed: f64,
    pub fidget_freq: f64,
    pub trial_timer: f64,
    pub trial_energy0: f64,
    pub trial_bonus: f64,
    pub trial_dist: f64,
    pub trial_best: f64,
    pub trial_dim: i32,
    pub trial_saved: f64,
    pub style_trials: i32,
    pub greet_cooldown_v: f64,
    pub charge_cooldown_v: f64,
}
rt::handle!(Walker, WalkerData);

impl Walker {
    fn alloc__() -> Walker {
        let mut this = Walker::from_data(WalkerData::default());
        this.puffs = Arr::new(0);
        this.mode = Mode::Hidden;
        this.scale = Settings::BASE_PIXEL_SIZE;
        this.size_factor = 1.0;
        this.face = (*EXPRESSION_NEUTRAL);
        this.gaze_x = 0.5;
        this.facing = 1;
        this.after_brake = Mode::Walk;
        this.after_turn = Mode::Walk;
        this.energy = 1.0;
        this.dress = (*(*SETTINGS_DRESS_COLORS).at(0));
        this.older_state = -1;
        this.older_choice = -1;
        this.fondness = Dict::new();
        this.place = Arr::new(8);
        this.last_choice = -1;
        this.log = Arr::new(6);
        this.style = Arr::new(Walker::STYLE_DIMS);
        this.trial_timer = Walker::TRIAL_SECONDS;
        this.trial_best = -10.0;
        this
    }
    pub fn new(mut world: IWorld, mut rng: Random, mut personality: Personality) -> Walker {
        let mut this = Self::alloc__();
        this.ctor(world, rng, personality);
        this
    }
    fn ctor(mut self, mut world: IWorld, mut rng: Random, mut personality: Personality) {
        self.world = world;
        self.rng = rng;
        self.personality = personality;
        self.body = personality.anatomy();
        self.bird = (personality.figure() == Figure::Bird);
    }
    pub const BLINK_SECONDS: f64 = 0.16;
    pub const GREET_SECONDS: f64 = 1.3;
    pub const DANCE_GREET_SECONDS: f64 = 2.6;
    pub const GLOAT_SECONDS: f64 = 0.7;
    pub const FLEE_SECONDS: f64 = 1.4;
    pub const CHARGE_SPEED_FACTOR: f64 = 2.0;
    pub const FLEE_SPEED_FACTOR: f64 = 1.6;
    pub const CHASE_SECONDS: f64 = 4.0;
    pub const SCAMPER_SECONDS: f64 = 3.5;
    pub const CHASE_SPEED_FACTOR: f64 = 1.5;
    pub const SCAMPER_SPEED_FACTOR: f64 = 1.3;
    // ---- What others need to know ----------------------------------------------
    pub fn personality(mut self) -> Personality {
        return self.personality;
    }
    pub fn temperament(mut self) -> Temperament {
        return self.personality.temperament;
    }
    pub fn is_hidden(mut self) -> bool {
        return (self.mode == Mode::Hidden);
    }
    pub fn is_visible(mut self) -> bool {
        return (self.mode != Mode::Hidden);
    }
    pub fn is_leaving(mut self) -> bool {
        return (self.mode == Mode::Exit);
    }
    pub fn scale(mut self) -> i32 {
        return self.scale;
    }
    pub fn centre_x(mut self) -> f64 {
        return self.cx;
    }
    pub fn feet_y(mut self) -> f64 {
        return self.feet_y;
    }
    pub fn facing(mut self) -> i32 {
        return self.facing;
    }
    pub fn faces_left(mut self) -> bool {
        return (self.facing < 0);
    }
    pub fn body_width(mut self) -> f64 {
        return (((2 * self.body.half_width) * self.scale) as f64);
    }
    pub fn puffs(mut self) -> Arr<Puff> {
        return self.puffs.clone();
    }
    // Where its eyes and shoulders are on screen, for others to look at or reach for.
    pub fn eye_y(mut self) -> f64 {
        return (self.feet_y - (self.body.eye_v * ((self.scale) as f64)));
    }
    pub fn shoulder_y(mut self) -> f64 {
        return (self.feet_y - (self.body.shoulder_v * ((self.scale) as f64)));
    }
    // The ground line under the character; nothing shows below it.
    pub fn ground_line(mut self) -> f64 {
        return self.clip_y;
    }
    // Feet on the ground (not underground, airborne or leaving).
    pub fn is_grounded(mut self) -> bool {
        'sw1: {
            match self.mode {
                Mode::Walk
                | Mode::Crouch
                | Mode::Turn
                | Mode::Look
                | Mode::Wave
                | Mode::Rest
                | Mode::Greet
                | Mode::Charge
                | Mode::Gloat
                | Mode::Flee
                | Mode::Antic
                | Mode::React
                | Mode::Chat => {
                    return true;
                }
                _ => {
                    return false;
                }
            }
        }
        #[allow(unreachable_code)]
        unreachable!()
    }
    // Walking along at speed (as opposed to stopping or standing).
    pub fn is_walking(mut self) -> bool {
        return (((self.mode == Mode::Walk) && !(self.braking))
            && ((self.vx).abs() > (6.0 * self.size_factor)));
    }
    // Free to stop and say hello, gawp at a shove, be drawn into a chat.
    pub fn can_greet(mut self) -> bool {
        return ((((self.mode == Mode::Walk) && !(self.braking)) || (self.mode == Mode::Rest))
            || (self.mode == Mode::Look));
    }
    // Standing about not paying attention: fair game for a prank.
    pub fn is_quiet(mut self) -> bool {
        if (self.mode == Mode::Rest) || (self.mode == Mode::Look) {
            return true;
        }
        if self.mode != Mode::Antic {
            return false;
        }
        return ((((((((((self.antic == Antic::Doze) || (self.antic == Antic::Whistle))
            || (self.antic == Antic::Scratch))
            || (self.antic == Antic::Yawn))
            || (self.antic == Antic::Hiccups))
            || (self.antic == Antic::Daydream))
            || (self.antic == Antic::Fret))
            || (self.antic == Antic::Dig))
            || (self.antic == Antic::Sleepwalk))
            || (self.antic == Antic::Egg));
    }
    // Still sad about being shoved.
    pub fn is_upset(mut self) -> bool {
        return (self.upset && (self.mood_left > 0.0));
    }
    // On an errand (so not to be interrupted by a charge).
    pub fn has_goal(mut self) -> bool {
        return (self.goal != Goal::None);
    }
    pub fn is_charging(mut self) -> bool {
        return (self.mode == Mode::Charge);
    }
    pub fn is_fleeing(mut self) -> bool {
        return (self.mode == Mode::Flee);
    }
    pub fn is_playful(mut self) -> bool {
        return self.playful;
    }
    pub fn charge_target(mut self) -> Walker {
        return self.charge_target;
    }
    // Seconds before this walker will greet or charge again.
    pub fn greet_cooldown(mut self) -> f64 {
        self.greet_cooldown_v
    }
    pub fn set_greet_cooldown(mut self, v: f64) {
        self.greet_cooldown_v = v;
    }
    pub fn charge_cooldown(mut self) -> f64 {
        self.charge_cooldown_v
    }
    pub fn set_charge_cooldown(mut self, v: f64) {
        self.charge_cooldown_v = v;
    }
    // What it is doing, in a word, for diagnostics.
    pub fn activity(mut self) -> String {
        'sw1: {
            match self.mode {
                Mode::Antic => {
                    return self.antic.to_string();
                }
                Mode::React => {
                    return self.reaction.to_string();
                }
                Mode::Chat => {
                    return self.chat_style.to_string();
                }
                Mode::Walk => {
                    return (if (self.goal != Goal::None) {
                        rt::cat(&"Walk:".to_string(), self.goal)
                    } else {
                        "Walk".to_string()
                    });
                }
                Mode::Charge => {
                    return (if self.playful {
                        "Chase".to_string()
                    } else {
                        "Charge".to_string()
                    });
                }
                Mode::Flee => {
                    return (if self.playful {
                        "Scamper".to_string()
                    } else {
                        "Flee".to_string()
                    });
                }
                Mode::Greet => {
                    return rt::cat(&"Greet:".to_string(), self.greet_style);
                }
                _ => {
                    return self.mode.to_string();
                }
            }
        }
        #[allow(unreachable_code)]
        unreachable!()
    }
    // ---- Commands ---------------------------------------------------------------
    // Come up out of the ground at x.
    pub fn spawn(mut self, mut cursor: Point, mut x: f64) {
        self.cursor_x = cursor.x;
        self.cursor_y = cursor.y;
        self.cx = x;
        self.update_world();
        self.facing = (if (self.rng.next(2) == 0) { -1 } else { 1 });
        self.face = self.personality.base_face;
        self.gaze_x = 0.5;
        self.gaze_y = 0.0;
        self.yaw = 0.0;
        self.clip_y = self.world.ground_at(self.cx);
        self.feet_y = (self.clip_y + ((self.body.hidden_depth * self.scale) as f64));
        self.vy = 0.0;
        self.vx = self.vy;
        self.squash = 0.0;
        self.walk_phase = 0.0;
        self.land_pause = 0.0;
        self.startled = false;
        self.knocked = false;
        self.charge_target = <Walker>::default();
        self.after_turn = Mode::Walk;
        self.idle_timer = self.between(4.0, 9.0);
        self.hop_timer =
            (self.between(self.personality.hop_min, self.personality.hop_max) * self.eff_hop());
        self.trip_timer = self.between(20.0, 60.0);
        self.blink_timer = self.between(1.0, 3.0);
        self.blink_again = false;
        self.glance_timer = self.between(0.5, 1.5);
        self.glance_left = 0.0;
        self.reset_eyes();
        self.energy = self.between(0.7, 1.0);
        self.social = self.between(0.0, 0.4);
        self.curiosity = self.between(0.0, 0.4);
        self.catch_cooldown = 0.0;
        self.clear_antics();
        self.reset_mind();
        self.set_greet_cooldown(2.0);
        self.set_charge_cooldown(3.0);
        for p__ in self.puffs.items() {
            p__.free();
        }
        self.puffs.clear();
        self.enter(Mode::PeekRise);
    }
    // The mouse moved: jump in fright, then drop out of sight.
    pub fn startle(mut self) {
        self.begin_exit(true);
    }
    // Asked to go (the count was lowered): hop and drop out of sight.
    pub fn leave(mut self) {
        self.begin_exit(false);
    }
    pub fn begin_exit(mut self, mut with_bubble: bool) {
        if (self.mode == Mode::Hidden) || (self.mode == Mode::Exit) {
            return;
        }
        let mut underground: bool = (((self.mode == Mode::PeekRise)
            || (self.mode == Mode::PeekLook))
            || (self.mode == Mode::PeekDuck));
        self.startled = (with_bubble && !(underground));
        if underground {
            self.vy = 0.0;
        } else if self.mode != Mode::Air {
            self.vy = (-(Settings::STARTLE_SPEED) * self.size_factor);
        }
        self.vx = 0.0;
        self.clear_antics();
        self.enter(Mode::Exit);
    }
    pub fn clear_antics(mut self) {
        self.joy_hops = 0;
        self.celebrating = false;
        self.woke_up = false;
        self.spooked = false;
        self.playful = false;
        self.sweat = 0.0;
        self.mood_left = 0.0;
        self.upset = false;
        self.goal = Goal::None;
        self.goal_target = <Walker>::default();
        self.has_pending = false;
        self.broadcast = None;
        self.chat_partner = <Walker>::default();
    }
    // The nearest other walker on the same ground, or null.
    pub fn set_company(mut self, mut nearest: Walker) {
        self.company = nearest;
        self.company_nearby = (!nearest.is_null()
            && ((nearest.centre_x() - self.cx).abs() < (350.0 * self.size_factor)));
        self.company_x = (if !nearest.is_null() {
            nearest.centre_x()
        } else {
            0.0
        });
        self.company_eye_y = (if !nearest.is_null() {
            nearest.eye_y()
        } else {
            0.0
        });
    }
    // Stop and face `direction` to greet someone there.
    pub fn greet(mut self, mut direction: i32, mut style: GreetStyle) {
        if !(self.can_greet()) {
            return;
        }
        self.greet_facing = direction;
        self.greet_style = style;
        self.social = 0.1;
        if style == GreetStyle::Dance {
            self.broadcast = Some(Antic::Dance);
        }
        self.note_greet(style);
        self.stop(Mode::Greet);
    }
    // Saw somebody shoved at x: stop and react in character.
    pub fn react(mut self, mut x: f64) {
        if !(self.can_greet()) {
            return;
        }
        'sw1: {
            match self.personality.temperament {
                Temperament::Friendly => {
                    self.react_with(Reaction::Gasp, x, 1.7, false);
                }
                Temperament::Calm => {
                    self.react_with(Reaction::Headshake, x, 1.6, false);
                }
                Temperament::Timid => {
                    self.react_with(Reaction::Cower, x, 2.0, false);
                }
                Temperament::Goofy => {
                    self.react_with(Reaction::Chuckle, x, 1.6, false);
                }
                Temperament::Feisty => {
                    self.react_with(Reaction::Bristle, x, 1.6, false);
                }
                _ => {
                    self.react_with(Reaction::Jeer, x, 1.8, false);
                }
            }
        }
    }
    // Somebody at x just let one off: stop and react in character.
    pub fn smell(mut self, mut x: f64) {
        if !(self.can_greet()) {
            return;
        }
        'sw1: {
            match self.personality.temperament {
                Temperament::Friendly | Temperament::Timid => {
                    self.react_with(Reaction::Gasp, x, 1.5, false);
                }
                Temperament::Calm => {
                    self.react_with(Reaction::Headshake, x, 1.4, false);
                }
                Temperament::Goofy => {
                    self.react_with(Reaction::Chuckle, x, 1.4, false);
                }
                Temperament::Feisty => {
                    self.react_with(Reaction::Bristle, x, 1.4, false);
                }
                _ => {
                    self.react_with(Reaction::Huff, x, 1.2, true);
                }
            }
        }
    }
    // A bully is coming: scream and run before it gets here.
    pub fn panic(mut self, mut direction: i32) {
        if ((!(self.is_grounded()) || (self.mode == Mode::Flee)) || (self.mode == Mode::Chat))
            || (self.mode == Mode::Charge)
        {
            return;
        }
        self.playful = false;
        self.facing = direction;
        self.flee_time = (Walker::FLEE_SECONDS * 1.3);
        self.goal = Goal::None;
        self.goal_target = <Walker>::default();
        self.has_pending = false;
        self.set_greet_cooldown(rt::max_f64(self.greet_cooldown(), (self.flee_time + 2.0)));
        self.start_sweat();
        self.emit_dust(self.cx, self.feet_y, 2, 0.5);
        self.enter(Mode::Flee);
    }
    // Run at another walker to shove it.
    pub fn start_charge(mut self, mut target: Walker) {
        if (self.mode != Mode::Walk) || self.braking {
            return;
        }
        self.charge_target = target;
        self.playful = false;
        self.enter(Mode::Charge);
    }
    // Shoved: fly off in `direction`, land, run away.
    pub fn knock(mut self, mut direction: i32) {
        if !(self.is_visible()) || (self.mode == Mode::Exit) {
            return;
        }
        if self.is_grounded() {
            self.emit_dust(self.cx, self.feet_y, 3, 0.8);
        }
        self.vx = (((direction * 240) as f64) * self.size_factor);
        self.vy = (-200.0 * self.size_factor);
        self.knocked = true;
        self.charge_target = <Walker>::default();
        self.clear_antics();
        self.enter(Mode::Air);
    }
    // Stand proud for a moment after a successful shove.
    pub fn gloat(mut self) {
        self.charge_target = <Walker>::default();
        self.set_charge_cooldown(self.between(20.0, 40.0)); // that will do for a while
        self.vx = 0.0;
        self.enter(Mode::Gloat);
    }
    // Advances the simulation by dt seconds.
    pub fn step(mut self, mut dt: f64, mut cursor: Point) {
        self.cursor_x = cursor.x;
        self.cursor_y = cursor.y;
        self.time += dt;
        self.mode_time += dt;
        self.squash = rt::max_f64(0.0, (self.squash - (dt / 0.18)));
        self.set_greet_cooldown((self.greet_cooldown() - dt));
        self.set_charge_cooldown((self.charge_cooldown() - dt));
        self.catch_cooldown -= dt;
        self.steer_cooldown -= dt;
        self.bond_cooldown -= dt;
        if self.mood_left > 0.0 {
            self.mood_left -= dt;
            if self.mood_left <= 0.0 {
                self.upset = false;
            }
        }
        if self.sweat > 0.0 {
            self.sweat += (dt / 1.6);
            if self.sweat >= 1.0 {
                self.sweat = 0.0;
            }
        }
        self.update_world();
        self.step_blink(dt);
        self.step_mood(dt);
        self.step_style(dt);
        self.step_pending(dt);
        'sw1: {
            match self.mode {
                Mode::PeekRise => {
                    // Ease the top of the head up over the edge.
                    let mut t: f64 = rt::min_f64(1.0, (self.mode_time / 0.4));
                    let mut eased: f64 = (1.0 - rt::pow((1.0 - t), 3.0));
                    self.feet_y = (self.clip_y
                        + ((((self.body.hidden_depth) as f64)
                            + (((self.body.peek_depth - self.body.hidden_depth) as f64) * eased))
                            * ((self.scale) as f64)));
                    if t >= 1.0 {
                        self.enter(Mode::PeekLook);
                    }
                }
                Mode::PeekLook => {
                    // Check that the coast is clear.
                    self.head_turned = ((self.mode_time > 0.3) && (self.mode_time < 0.8));
                    if self.mode_time >= 1.1 {
                        self.enter(Mode::PeekDuck);
                    }
                }
                Mode::PeekDuck => {
                    // Dip down, then spring out.
                    let mut t: f64 = rt::min_f64(1.0, (self.mode_time / 0.12));
                    self.feet_y = (self.clip_y
                        + ((((self.body.peek_depth) as f64) + (3.0 * t)) * ((self.scale) as f64)));
                    if t >= 1.0 {
                        let mut rise: f64 =
                            ((self.feet_y - self.clip_y) + ((10 * self.scale) as f64));
                        self.vy = -((((2.0 * Settings::GRAVITY) * self.size_factor) * rise).sqrt());
                        self.vx = (((((self.facing) as f64) * Settings::WALK_SPEED)
                            * self.size_factor)
                            * 0.6);
                        self.land_pause = 0.5;
                        self.emit_dust(self.cx, self.clip_y, 6, 1.0);
                        self.enter(Mode::Air);
                    }
                }
                Mode::Air => {
                    // Birds flutter: they fall markedly slower, wings out.
                    self.vy += (((Settings::GRAVITY * self.size_factor) * dt)
                        * (if (self.bird && (self.vy > 0.0)) {
                            0.55
                        } else {
                            1.0
                        }));
                    self.feet_y += (self.vy * dt);
                    self.move_x((self.vx * dt));
                    let mut ground: f64 = self.world.ground_at(self.cx);
                    self.clip_y = ground;
                    if (self.vy > 0.0) && (self.feet_y >= ground) {
                        self.land(ground);
                    }
                }
                Mode::Walk => {
                    self.step_walk(dt, 1.0, true);
                }
                Mode::Crouch => {
                    // A beat of anticipation before a jump.
                    if !(self.stay_on_ground()) {
                        break 'sw1;
                    }
                    self.move_x((self.vx * dt));
                    if self.mode_time >= 0.08 {
                        self.vy = -self.pending_jump;
                        self.vx = self.pending_air_speed;
                        self.enter(Mode::Air);
                    }
                }
                Mode::Turn => {
                    // The head turns first, the body follows.
                    if !(self.stay_on_ground()) {
                        break 'sw1;
                    }
                    if self.mode_time < 0.16 {
                        self.head_turned = true;
                    } else if self.head_turned {
                        // The body catches up with the head: what was "looking
                        // back" is now "looking ahead", so the eased values flip too.
                        self.head_turned = false;
                        self.facing = -self.facing;
                        self.yaw = -self.yaw;
                        self.gaze_x = -self.gaze_x;
                    }
                    if self.mode_time >= 0.28 {
                        let mut next: Mode = self.after_turn;
                        self.after_turn = Mode::Walk;
                        self.enter(next);
                    }
                }
                Mode::Look => {
                    if !(self.stay_on_ground()) {
                        break 'sw1;
                    }
                    self.head_turned = ((self.mode_time > 0.35) && (self.mode_time < 1.1));
                    if self.mode_time >= 1.5 {
                        self.enter(
                            (if (self.rng.next_double() < 0.3) {
                                Mode::Turn
                            } else {
                                Mode::Walk
                            }),
                        );
                    }
                }
                Mode::Wave => {
                    if !(self.stay_on_ground()) {
                        break 'sw1;
                    }
                    if self.mode_time >= 1.35 {
                        self.enter(Mode::Walk);
                    }
                }
                Mode::Rest => {
                    if !(self.stay_on_ground()) {
                        break 'sw1;
                    }
                    if self.mode_time >= self.rest_duration {
                        self.enter(Mode::Walk);
                    }
                }
                Mode::Greet => {
                    if !(self.stay_on_ground()) {
                        break 'sw1;
                    }
                    self.facing = self.greet_facing;
                    if self.mode_time
                        >= (if (self.greet_style == GreetStyle::Dance) {
                            Walker::DANCE_GREET_SECONDS
                        } else {
                            Walker::GREET_SECONDS
                        })
                    {
                        self.enter(Mode::Walk);
                    }
                }
                Mode::Charge => {
                    if !(self.stay_on_ground()) {
                        break 'sw1;
                    }
                    let mut target: Walker = self.charge_target;
                    if (((target.is_null() || !(target.is_visible())) || target.is_leaving())
                        || ((target.centre_x() - self.cx).abs() > (520.0 * self.size_factor)))
                        || (self.mode_time
                            > (if self.playful {
                                Walker::CHASE_SECONDS
                            } else {
                                3.0
                            }))
                    {
                        self.abort_charge();
                        break 'sw1;
                    }
                    if !(self.braking) {
                        self.facing = (if (target.centre_x() >= self.cx) {
                            1
                        } else {
                            -1
                        });
                    }
                    self.step_walk(
                        dt,
                        (if self.playful {
                            Walker::CHASE_SPEED_FACTOR
                        } else {
                            Walker::CHARGE_SPEED_FACTOR
                        }),
                        false,
                    );
                }
                Mode::Gloat => {
                    if !(self.stay_on_ground()) {
                        break 'sw1;
                    }
                    if self.mode_time >= Walker::GLOAT_SECONDS {
                        self.enter(Mode::Walk);
                    }
                }
                Mode::Flee => {
                    if !(self.stay_on_ground()) {
                        break 'sw1;
                    }
                    self.flee_time -= dt;
                    self.step_walk(
                        dt,
                        (if self.playful {
                            Walker::SCAMPER_SPEED_FACTOR
                        } else {
                            Walker::FLEE_SPEED_FACTOR
                        }),
                        false,
                    );
                    if (self.flee_time <= 0.0) && (self.mode == Mode::Flee) {
                        self.end_flee();
                    }
                }
                Mode::Antic => {
                    if !(self.stay_on_ground()) {
                        break 'sw1;
                    }
                    self.step_antic(dt);
                    if (self.mode == Mode::Antic) && (self.mode_time >= self.antic_seconds) {
                        self.end_antic();
                    }
                }
                Mode::React => {
                    if !(self.stay_on_ground()) {
                        break 'sw1;
                    }
                    self.facing = self.react_facing;
                    if self.mode_time >= self.react_seconds {
                        self.enter(Mode::Walk);
                    }
                }
                Mode::Chat => {
                    if !(self.stay_on_ground()) {
                        break 'sw1;
                    }
                    self.facing = self.chat_facing;
                    self.step_chat();
                    if (self.mode == Mode::Chat) && (self.mode_time >= self.chat_seconds) {
                        self.end_chat();
                    }
                }
                Mode::Exit => {
                    self.vy += ((Settings::EXIT_GRAVITY * self.size_factor) * dt);
                    self.feet_y += (self.vy * dt);
                    let mut tall: f64 = ((((self.body.hidden_depth
                        + (if self.startled { 8 } else { 0 }))
                        * self.scale) as f64)
                        * 1.2);
                    if (self.feet_y - self.clip_y) >= tall {
                        for p__ in self.puffs.items() {
                            p__.free();
                        }
                        self.puffs.clear();
                        self.enter(Mode::Hidden);
                    }
                }
                _ => {}
            }
        }
        // Reflection is not only for long walks: the idle timer keeps
        // running through standing pauses (rest, look, greetings,
        // reactions, gloating) so a crowd never starves anyone's
        // learning. It fires at the next moment free to act.
        if ((((self.mode == Mode::Rest) || (self.mode == Mode::Look))
            || (self.mode == Mode::Greet))
            || (self.mode == Mode::React))
            || (self.mode == Mode::Gloat)
        {
            self.idle_timer -= dt;
            if self.idle_timer <= 0.0 {
                self.idle_timer =
                    self.between(self.personality.idle_min, self.personality.idle_max);
                if self.can_greet() {
                    self.decide();
                }
            }
        }
        self.step_dust(dt);
        self.update_face(dt);
    }
    // Blinks come in the rhythm of the mood: quick and frequent when
    // frightened, slow and heavy when drowsy, sometimes two in a row -
    // and not at all while the eyes are already shut.
    pub fn step_blink(mut self, mut dt: f64) {
        self.blink_timer -= dt;
        self.blink_left -= dt;
        if (self.blink_left <= 0.0) && self.blink_again {
            self.blink_again = false;
            self.blink_left = (Walker::BLINK_SECONDS * 0.85);
            return;
        }
        if self.blink_timer > 0.0 {
            return;
        }
        let mut tense: bool = (self.face.pupil_size < 0.8);
        let mut heavy: bool = (self.face.lid > 0.5);
        if self.face.lid < 0.9 {
            self.blink_left = (Walker::BLINK_SECONDS
                * (if heavy {
                    1.9
                } else {
                    (if tense { 0.8 } else { 1.0 })
                }));
            self.blink_again = (!(heavy) && (self.rng.next_double() < 0.2));
        }
        self.blink_timer =
            ((self.between(1.8, 4.5) * (if tense { 0.4 } else { 1.0 })) * self.blink_pace());
    }
    // ---- The face --------------------------------------------------------------
    // Gaze towards a screen point, as -1..1 with +X ahead and +Y up.
    pub fn look_at(mut self, mut x: f64, mut y: f64, gx: &mut f64, gy: &mut f64) {
        let mut eye_y: f64 = (self.feet_y - (self.body.eye_v * ((self.scale) as f64)));
        let mut reach: f64 = (260.0 * self.size_factor);
        (*gx) =
            (rt::max_f64(-1.0, rt::min_f64(1.0, ((x - self.cx) / reach))) * ((self.facing) as f64));
        (*gy) = rt::max_f64(-1.0, rt::min_f64(1.0, ((eye_y - y) / reach)));
    }
    // Picks the expression and gaze for the current situation and eases
    // the face towards them, so moods change over a few frames - quickly
    // for a fright, slowly for drowsiness.
    pub fn update_face(mut self, mut dt: f64) {
        let mut target: Expression = self.personality.base_face;
        let mut rate: f64 = 14.0;
        let mut gx: f64 = 0.55;
        let mut gy: f64 = -0.05;
        let mut at_cursor: bool = false;
        let mut steady: bool = false; // standing about: the eyes may wander a little
        let mut watching: bool = false; // the eyes are on another walker
        'sw1: {
            match self.mode {
                Mode::PeekRise | Mode::PeekLook | Mode::PeekDuck => {
                    target = (*EXPRESSION_CURIOUS);
                    at_cursor = true;
                }
                Mode::Walk => {
                    if self.goal != Goal::None {
                        self.goal_face(
                            rt::mref(&mut target),
                            rt::mref(&mut gx),
                            rt::mref(&mut gy),
                            rt::mref(&mut at_cursor),
                        );
                    } else {
                        if self.mood_left > 0.0 {
                            target = self.mood_face;
                        }
                        // Passing someone: meet their eyes; otherwise the cursor if it is near.
                        watching = self.watch_company(false, rt::mref(&mut gx), rt::mref(&mut gy));
                        if !(watching) {
                            at_cursor = ((((self.cursor_x) as f64) - self.cx).abs()
                                < (260.0 * self.size_factor));
                        }
                    }
                }
                Mode::Charge => {
                    target = (if self.playful {
                        (*EXPRESSION_LAUGH)
                    } else {
                        (*EXPRESSION_DETERMINED)
                    });
                    if !self.charge_target.is_null() {
                        self.look_at(
                            self.charge_target.centre_x(),
                            self.charge_target.eye_y(),
                            rt::mref(&mut gx),
                            rt::mref(&mut gy),
                        );
                    }
                }
                Mode::Flee => {
                    if self.playful {
                        target = (*EXPRESSION_GIGGLE);
                    } else {
                        target = (if (self.flee_time > (Walker::FLEE_SECONDS * 0.45)) {
                            (*EXPRESSION_SCARED)
                        } else {
                            (*EXPRESSION_SAD)
                        });
                    }
                    rate = 24.0;
                    gx = -0.75;
                    gy = 0.1;
                }
                Mode::Crouch => {
                    target = (if self.celebrating {
                        (*EXPRESSION_EXCITED)
                    } else {
                        (*EXPRESSION_FOCUSED)
                    });
                    gy = 0.5;
                }
                Mode::Air => {
                    if self.knocked {
                        target = (*EXPRESSION_DIZZY);
                    } else if self.woke_up {
                        target = (*EXPRESSION_SHOCKED);
                        rate = 30.0;
                        at_cursor = !(self.spooked);
                        if self.spooked {
                            gx = 0.6;
                        }
                    } else if self.celebrating {
                        target = (*EXPRESSION_EXCITED);
                        at_cursor = true;
                    } else if (self.vy).abs() > ((1.2 * Settings::HOP_SPEED) * self.size_factor) {
                        target = (*EXPRESSION_EXCITED);
                    }
                    gy = (if (self.vy < 0.0) { 0.5 } else { -0.5 });
                }
                Mode::Exit => {
                    target = (if self.startled {
                        (*EXPRESSION_SURPRISED)
                    } else {
                        (*EXPRESSION_SAD)
                    });
                    rate = ((if self.startled { 30 } else { 14 }) as f64);
                    at_cursor = self.startled;
                }
                Mode::Wave => {
                    target = (if self.personality.warm() {
                        (if ((self.mode_time > 0.5) && (self.mode_time < 1.0)) {
                            (*EXPRESSION_WINKING)
                        } else {
                            (*EXPRESSION_GRIN)
                        })
                    } else {
                        (if (self.personality.temperament == Temperament::Timid) {
                            (*EXPRESSION_NERVOUS)
                        } else {
                            (*EXPRESSION_HAPPY)
                        })
                    });
                    if !(self.wave_at_cursor) && self.company_nearby {
                        self.look_at(
                            self.company_x,
                            self.company_eye_y,
                            rt::mref(&mut gx),
                            rt::mref(&mut gy),
                        );
                    } else {
                        at_cursor = true;
                    }
                    steady = true;
                }
                Mode::Greet => {
                    'sw2: {
                        match self.greet_style {
                            GreetStyle::Dance => {
                                target = (if (self.mode_time < (Walker::DANCE_GREET_SECONDS * 0.5))
                                {
                                    (*EXPRESSION_GRIN)
                                } else {
                                    (*EXPRESSION_LAUGH)
                                });
                                rate = 18.0;
                            }
                            GreetStyle::Wave => {
                                target = (*EXPRESSION_LAUGH);
                            }
                            _ => {
                                target = (*EXPRESSION_HAPPY);
                            }
                        }
                    }
                    self.look_at(
                        self.company_x,
                        self.company_eye_y,
                        rt::mref(&mut gx),
                        rt::mref(&mut gy),
                    );
                    steady = true;
                }
                Mode::Gloat => {
                    target = (*EXPRESSION_SMUG);
                    gx = 0.7;
                    gy = -0.45;
                }
                Mode::Look => {
                    target = (*EXPRESSION_CURIOUS);
                    at_cursor = true;
                    steady = true;
                }
                Mode::Rest => {
                    if (self.rest_duration > 1.2) && (self.mode_time > 0.4) {
                        target = self.personality.rest_face;
                        rate = 7.0;
                    }
                    at_cursor = true;
                    steady = true;
                }
                Mode::Turn => {
                    at_cursor = true;
                }
                Mode::Antic => {
                    self.antic_face(
                        rt::mref(&mut target),
                        rt::mref(&mut rate),
                        rt::mref(&mut gx),
                        rt::mref(&mut gy),
                        rt::mref(&mut at_cursor),
                    );
                    steady = true;
                }
                Mode::React => {
                    self.react_face(
                        rt::mref(&mut target),
                        rt::mref(&mut rate),
                        rt::mref(&mut gx),
                        rt::mref(&mut gy),
                    );
                    steady = true;
                }
                Mode::Chat => {
                    self.chat_face(
                        rt::mref(&mut target),
                        rt::mref(&mut rate),
                        rt::mref(&mut gx),
                        rt::mref(&mut gy),
                    );
                    steady = true;
                }
                _ => {}
            }
        }
        if at_cursor {
            self.look_at(
                ((self.cursor_x) as f64),
                ((self.cursor_y) as f64),
                rt::mref(&mut gx),
                rt::mref(&mut gy),
            );
        }
        if self.head_turned {
            gx = -0.8;
            gy = 0.0;
        }
        // Standing about: the eyes follow a passer-by, and now and then
        // act a little (WalkerEyes.cs). The darting only while they are free.
        let mut standing: bool =
            (((self.mode == Mode::Rest) || (self.mode == Mode::Look)) && !(self.head_turned));
        let mut strolling: bool =
            (((self.mode == Mode::Walk) && (self.goal == Goal::None)) && (self.mood_left <= 0.0));
        if standing {
            watching = self.watch_company(true, rt::mref(&mut gx), rt::mref(&mut gy));
        }
        self.notice_passer((standing && watching));
        let mut gaze_rate: f64 = Walker::GAZE_RATE;
        if standing || strolling {
            gaze_rate = self.eye_beats(
                dt,
                watching,
                strolling,
                rt::mref(&mut target),
                rt::mref(&mut rate),
                rt::mref(&mut gx),
                rt::mref(&mut gy),
            );
        } else {
            self.end_eye_beat();
        }
        self.glance(
            dt,
            (((steady && (target.lid < 0.9)) && (self.eye_beat == EyeBeat::None)) && !(watching)),
            rt::mref(&mut gx),
            rt::mref(&mut gy),
        );
        self.gaze_blink(gx, gy);
        self.step_eye_extras(dt, (at_cursor && !(watching)), watching);
        self.face = Expression::towards(self.face, target, (1.0 - (-dt * rate).exp()));
        let mut k: f64 = (1.0 - (-dt * gaze_rate).exp());
        self.gaze_x = Walker::settle(self.gaze_x, gx, k);
        self.gaze_y = Walker::settle(self.gaze_y, gy, k);
        let mut snap: bool = <bool>::default();
        let mut yaw_target: f64 = self.yaw_target(rt::mref(&mut snap));
        self.yaw = (if snap {
            yaw_target
        } else {
            Walker::settle(self.yaw, yaw_target, (1.0 - (-dt * 9.0).exp()))
        });
    }
    // Real eyes never hold perfectly still: every second or two, while
    // standing about, the pupils dart a little way off and come back.
    pub fn glance(mut self, mut dt: f64, mut allowed: bool, gx: &mut f64, gy: &mut f64) {
        self.glance_timer -= dt;
        if self.glance_left > 0.0 {
            self.glance_left -= dt;
        } else if (self.glance_timer <= 0.0) && allowed {
            self.glance_left = self.between(0.18, 0.32);
            self.glance_dx = self.between(-0.16, 0.16);
            self.glance_dy = self.between(-0.08, 0.08);
            self.glance_timer = self.between(0.8, 2.4);
        }
        if (self.glance_left > 0.0) && allowed {
            (*gx) += self.glance_dx;
            (*gy) += self.glance_dy;
        }
    }
    // How far round the body should be turned: towards where it is going
    // while on the move, towards whoever it is dealing with when stopped,
    // and towards you the rest of the time. `snap` means take the value
    // this frame instead of easing towards it (a spin, a head-shake).
    pub fn yaw_target(mut self, snap: &mut bool) -> f64 {
        (*snap) = false;
        'sw1: {
            match self.mode {
                Mode::PeekLook => {
                    return ((if (self.mode_time < 0.3) {
                        0
                    } else {
                        (if (self.mode_time < 0.8) { -45 } else { 45 })
                    }) as f64);
                }
                Mode::PeekDuck => {
                    return 45.0;
                }
                Mode::Walk => {
                    return (58.0 + self.swagger_yaw());
                }
                Mode::Crouch | Mode::Air => {
                    return 58.0;
                }
                Mode::Charge | Mode::Flee => {
                    return 64.0;
                }
                Mode::Turn => {
                    return ((if self.head_turned { -45 } else { 58 }) as f64);
                }
                Mode::Look => {
                    return ((if self.head_turned { -45 } else { 0 }) as f64);
                }
                Mode::Wave => {
                    return ((if (self.wave_at_cursor || !(self.company_nearby)) {
                        0
                    } else {
                        45
                    }) as f64);
                }
                Mode::Greet => {
                    return (if (self.greet_style == GreetStyle::Dance) {
                        Walker::dance_yaw(self.mode_time, 40.0)
                    } else {
                        40.0
                    });
                }
                Mode::Gloat => {
                    return 35.0;
                }
                Mode::Rest => {
                    return self.rest_yaw();
                }
                Mode::Antic => {
                    return self.antic_yaw(&mut *snap);
                }
                Mode::React => {
                    return self.react_yaw(&mut *snap);
                }
                Mode::Chat => {
                    return self.chat_yaw();
                }
                _ => {
                    return 0.0; // peeking up, leaving: face the viewer
                }
            }
        }
        #[allow(unreachable_code)]
        unreachable!()
    }
    // Standing still, each temperament holds itself differently: the
    // friendly ones rock gently, the calm one turns a little towards the
    // cursor, the aggressive ones stand half turned away.
    pub fn rest_yaw(mut self) -> f64 {
        if self.eye_beat == EyeBeat::Viewer {
            return 0.0; // the head comes round for a look at you
        }
        'sw1: {
            match self.personality.temperament {
                Temperament::Friendly | Temperament::Goofy => {
                    return (12.0 * (self.time * 1.4).sin());
                }
                Temperament::Aggressive => {
                    return 15.0;
                }
                Temperament::Timid => {
                    return -12.0; // half turned away, ready to run
                }
                Temperament::Feisty => {
                    // A bird turns its head in quick glances and holds each one, giving
                    // one eye a good look, rather than swaying smoothly.
                    return (((self.bird_glance() - 0.5) * 56.0) / 2.0).round() * 2.0;
                }
                _ => {
                    return ((if (((((self.cursor_x) as f64) - self.cx) * ((self.facing) as f64))
                        >= 0.0)
                    {
                        10
                    } else {
                        -10
                    }) as f64);
                }
            }
        }
        #[allow(unreachable_code)]
        unreachable!()
    }
    pub fn settle(mut value: f64, mut target: f64, mut k: f64) -> f64 {
        let mut next: f64 = (value + ((target - value) * k));
        return (if ((target - next).abs() < 0.004) {
            target
        } else {
            next
        });
    }
    // How the body is arranged right now.
    pub fn current_pose(mut self) -> Pose {
        let mut pose: Pose = Pose::standing();
        pose.face = self.face;
        // Fine motion is stepped so a figure that is only easing into
        // place is not repainted for changes too small to see.
        pose.gaze_x = (rt::round_even((self.gaze_x * 32.0)) / 32.0);
        pose.gaze_y = (rt::round_even((self.gaze_y * 32.0)) / 32.0);
        pose.yaw = rt::round_even(self.yaw);
        let mut breathe: bool = false;
        'sw1: {
            match self.mode {
                Mode::Walk => {
                    self.walk(rt::mref(&mut pose));
                    pose.lean = self.eff_lean();
                    if self.goal == Goal::Prank {
                        // Creeping up: leaning in, fists up by the chest.
                        pose.lean = 0.35;
                        pose.arm_near = 60.0;
                        pose.fore_near = 90.0;
                        pose.arm_far = 60.0;
                        pose.fore_far = 90.0;
                    }
                }
                Mode::Charge => {
                    self.walk(rt::mref(&mut pose));
                    pose.lean = (if self.playful { 0.4 } else { 0.6 });
                    if self.playful {
                        // Reaching out to tag.
                        pose.arm_near = 80.0;
                        pose.fore_near = 10.0;
                    }
                }
                Mode::Flee => {
                    // Running with the arms flailing overhead.
                    self.walk(rt::mref(&mut pose));
                    pose.lean = 0.4;
                    pose.arm_near = (115.0 + (25.0 * (self.time * 22.0).sin()));
                    pose.arm_far = (-115.0 - (25.0 * (self.time * 22.0).sin()));
                    pose.fore_near = 30.0;
                    pose.fore_far = -30.0;
                }
                Mode::Crouch => {
                    // Wound up for the jump: squashed, arms out for balance.
                    pose.stretch_x = 1.12;
                    pose.stretch_y = 0.84;
                    pose.arm_near = ((if self.celebrating { 120 } else { 30 }) as f64);
                    pose.arm_far = ((if self.celebrating { -120 } else { -30 }) as f64);
                    pose.fore_near = 15.0;
                    pose.fore_far = -15.0;
                }
                Mode::Air | Mode::Exit => {
                    let mut rising: bool = (self.vy < 0.0);
                    pose.leg_tuck = (if rising { 1.0 } else { 0.3 });
                    // Mirrored (one ahead, one behind) so both hands don't end
                    // up on the same side when jumping past sideways.
                    pose.arm_near = ((if rising { 60 } else { 55 }) as f64);
                    pose.arm_far = ((if rising { -60 } else { -55 }) as f64);
                    pose.fore_near = ((if rising { 25 } else { 30 }) as f64);
                    pose.fore_far = ((if rising { -25 } else { -30 }) as f64);
                    let mut stretch: f64 =
                        (rt::min_f64(1.0, ((self.vy).abs() / (600.0 * self.size_factor))) * 0.1);
                    pose.stretch_y = (1.0 + stretch);
                    pose.stretch_x = (1.0 - (stretch * 0.7));
                    if self.knocked {
                        // Flung into the air, arms thrown up.
                        pose.arm_near = 140.0;
                        pose.arm_far = -140.0;
                        pose.fore_near = 15.0;
                        pose.fore_far = -15.0;
                        pose.leg_tuck = 0.6;
                        pose.bubble = true;
                    } else if self.celebrating {
                        // Jumping for joy: arms straight up, knees tucked.
                        pose.arm_near = 160.0;
                        pose.arm_far = -160.0;
                        pose.fore_near = -10.0;
                        pose.fore_far = 10.0;
                        pose.leg_tuck = (if rising { 1.0 } else { 0.5 });
                    } else if self.woke_up {
                        // Jerked awake or made to jump: arms flung out, a "!" overhead.
                        pose.arm_near = 70.0;
                        pose.arm_far = -70.0;
                        pose.fore_near = 10.0;
                        pose.fore_far = -10.0;
                        pose.bubble = true;
                    } else if self.mode == Mode::Exit {
                        pose.bubble = self.startled;
                    }
                }
                Mode::Wave => {
                    self.wave_arm(rt::mref(&mut pose));
                }
                Mode::Greet => {
                    'sw2: {
                        match self.greet_style {
                            GreetStyle::Dance => {
                                self.dance_pose(rt::mref(&mut pose), self.mode_time);
                            }
                            GreetStyle::Wave => {
                                self.wave_arm(rt::mref(&mut pose));
                            }
                            _ => {
                                pose.body_dy = Walker::quantise(
                                    ((0.45
                                        * (1.0
                                            - (((self.mode_time * 2.0) * std::f64::consts::PI)
                                                / 0.5)
                                                .cos()))
                                        / 2.0),
                                ); // a nod
                            }
                        }
                    }
                }
                Mode::Gloat => {
                    // Chest out, hands on hips.
                    Walker::hands_on_hips(rt::mref(&mut pose));
                    pose.stretch_y = 1.04;
                    pose.stretch_x = 1.03;
                    pose.lean = -0.3;
                }
                Mode::Rest => {
                    // The aggressive ones wait with their hands on their hips.
                    if self.personality.temperament == Temperament::Aggressive {
                        Walker::hands_on_hips(rt::mref(&mut pose));
                        pose.lean = -0.15;
                    } else if self.personality.temperament == Temperament::Timid {
                        // The nervous one wrings its hands.
                        pose.arm_near = 35.0;
                        pose.arm_far = 35.0;
                        let mut wring: f64 = ((if ((self.time * 7.0).sin() > 0.0) {
                            6
                        } else {
                            0
                        }) as f64); // two positions, a few times a second
                        pose.fore_near = (70.0 + wring);
                        pose.fore_far = (76.0 - wring);
                        pose.lean = 0.1;
                    }
                    breathe = true;
                }
                Mode::Antic => {
                    self.antic_pose(rt::mref(&mut pose));
                }
                Mode::React => {
                    self.react_pose(rt::mref(&mut pose));
                }
                Mode::Chat => {
                    self.chat_pose(rt::mref(&mut pose));
                }
                Mode::Turn | Mode::Look => {
                    breathe = true;
                }
                _ => {}
            }
        }
        if breathe {
            pose.body_dy = Walker::quantise(((0.3 * (1.0 + (self.time * 2.6).sin())) / 2.0));
            pose.lean = Walker::quantise((pose.lean + self.fidget_lean()));
        }
        if self.bird {
            self.bird_touch(rt::mref(&mut pose));
        }
        pose.stretch_y *= ((1.0 - (0.22 * self.squash)) * self.personality.build_y);
        pose.stretch_x *= ((1.0 + (0.16 * self.squash)) * self.personality.build_x);
        if self.sweat > 0.0 {
            pose.sweat = ((self.sweat * 12.0).ceil() / 12.0);
        }
        // A blink is a quick close-and-open laid over whatever the face is doing.
        if self.blink_left > 0.0 {
            let mut blink: f64 =
                (std::f64::consts::PI * (1.0 - (self.blink_left / Walker::BLINK_SECONDS))).sin();
            if blink > pose.face.lid {
                pose.face.lid = blink;
            }
        }
        // Eyes crossing on a cursor under the nose (or for fun), and
        // pupils that grow for something liked and shrink for a stare.
        if self.cross_eye > 0.01 {
            pose.face.cross_eye = rt::max_f64(
                pose.face.cross_eye,
                (rt::round_even((self.cross_eye * 16.0)) / 16.0),
            );
        }
        if self.pupil_interest != 0.0 {
            pose.face.pupil_size *=
                (1.0 + ((0.2 * rt::round_even((self.pupil_interest * 8.0))) / 8.0));
        }
        // Talking: the mouth flaps over whatever the face is doing.
        if ((self.mode == Mode::Chat) && (self.chat_style != ChatStyle::Pat)) && self.talking() {
            let mut amp: f64 = (if (self.chat_style == ChatStyle::Argue) {
                0.8
            } else {
                0.45
            });
            let mut flap: f64 =
                (rt::round_even((((0.5 + (0.5 * (self.time * 16.0).sin())) * amp) * 8.0)) / 8.0);
            if flap > pose.face.mouth_open {
                pose.face.mouth_open = flap;
            }
        }
        return pose;
    }
    pub fn walk(mut self, pose: &mut Pose) {
        if (self.vx).abs() <= (6.0 * self.size_factor) {
            return;
        }
        (*pose).walking = true;
        (*pose).walk_phase = self.walk_phase;
        if self.bird {
            self.hop_gait(&mut *pose);
            return;
        }
        (*pose).body_dy = ((self.eff_bob()
            * (1.0 - ((self.walk_phase * 4.0) * std::f64::consts::PI).cos()))
            / 2.0); // two bobs per stride
    }
    // Small birds hop rather than stride: one bound per stride, stretched
    // out in the air and squashed into the landing, with the shadow left
    // on the ground beneath.
    pub fn hop_gait(mut self, pose: &mut Pose) {
        let mut p: f64 = (self.walk_phase - self.walk_phase.floor());
        let mut height: f64 = (1.0 + self.eff_bob());
        if p < 0.72 {
            let mut s: f64 = (p / 0.72);
            (*pose).body_dy = -(Walker::quantise((((4.0 * s) * (1.0 - s)) * height)));
            (*pose).stretch_y = (1.0 + (0.05 * (1.0 - s)));
            (*pose).stretch_x = (1.0 - (0.04 * (1.0 - s)));
        } else {
            let mut c: f64 = (((p - 0.72) / 0.28) * std::f64::consts::PI).sin();
            (*pose).body_dy = Walker::quantise((0.25 * c));
            (*pose).stretch_y = (1.0 - (0.10 * c));
            (*pose).stretch_x = (1.0 + (0.08 * c));
        }
    }
    // A fixed offset for each bird, so the flock never fidgets in step. Worked
    // out from the name: this must not draw on the simulation's random numbers.
    pub fn bird_offset(mut self) -> f64 {
        let mut sum: u32 = self.personality.name.bytes().map(|b| b as u32).sum();
        return (((sum as f64) * 0.37) % 1.0);
    }
    // 0..1, held for a second or two and then jumping to another value.
    pub fn bird_glance(mut self) -> f64 {
        let mut slot: f64 = ((self.time / 1.7) + (self.bird_offset() * 7.0)).floor();
        let mut n: f64 = ((slot * 12.9898).sin() * 43758.5453);
        return (n - n.floor());
    }
    // How birds differ from the rest when standing about and in the air.
    pub fn bird_touch(mut self, pose: &mut Pose) {
        if (self.mode == Mode::Rest) || (self.mode == Mode::Look) {
            // A small body breathes quickly and shallowly.
            (*pose).body_dy = Walker::quantise(((0.18 * (1.0 + (self.time * 6.5).sin())) / 2.0));
            // No neck to cock: the whole body tips to go with each glance.
            (*pose).lean = Walker::quantise(((*pose).lean + ((self.bird_glance() - 0.5) * 0.24)));
            // Every so often, a rouse: every feather raised, a sharp side-to-side
            // shake, then the plumage settles back into place.
            let mut cycle: f64 = (self.time / Walker::ROUSE_EVERY) + self.bird_offset();
            let mut t: f64 = ((cycle - cycle.floor()) * Walker::ROUSE_EVERY);
            if t < 0.8 {
                let mut up: f64 = (if (t < 0.15) {
                    Walker::ease((t / 0.15))
                } else {
                    (if (t < 0.6) {
                        1.0
                    } else {
                        (1.0 - Walker::ease(((t - 0.6) / 0.2)))
                    })
                });
                (*pose).puff_up = (rt::round_even(((up * 0.3) * 16.0)) / 16.0);
                if (t > 0.12) && (t < 0.6) {
                    let mut shake: f64 = (t * 38.0).sin();
                    (*pose).yaw = rt::round_even(((*pose).yaw + (14.0 * shake)));
                    (*pose).lean = (rt::round_even(((0.22 * shake) * 8.0)) / 8.0);
                }
            }
        } else if self.mode == Mode::Air {
            // Drawn out along the way it is travelling, more the faster it goes.
            let mut k: f64 = rt::min_f64(1.0, ((self.vy).abs() / (500.0 * self.size_factor)));
            k = (rt::round_even((k * 4.0)) / 4.0);
            (*pose).stretch_y *= (1.0 + (0.08 * k));
            (*pose).stretch_x *= (1.0 - (0.06 * k));
        }
    }
    pub const ROUSE_EVERY: f64 = 13.0;
    pub fn wave_arm(mut self, pose: &mut Pose) {
        if self.bird {
            // No wings to wave with: a happy bouncing bow instead.
            (*pose).body_dy = Walker::quantise(
                ((0.4 * (1.0 - (((self.mode_time * 2.0) * std::f64::consts::PI) / 0.45).cos()))
                    / 2.0),
            );
            (*pose).lean = (rt::round_even(((0.2 * (self.mode_time * 7.0).sin()) * 8.0)) / 8.0);
            (*pose).stretch_y = 1.03;
            return;
        }
        // Arm up, forearm out to the side so the hand clears the goggles.
        // Turned sideways the goggles swing out to the edge, so open the
        // arm up (more horizontal) to stay past them instead of waving
        // over the face.
        let mut widen: f64 = (1.0 - ((self.yaw * std::f64::consts::PI) / 180.0).cos());
        (*pose).arm_near = ((150.0 + (8.0 * (self.mode_time * 16.0).sin())) - (130.0 * widen));
        (*pose).fore_near = (-25.0 + (25.0 * (self.mode_time * 16.0).sin()));
    }
    pub fn hands_on_hips(pose: &mut Pose) {
        (*pose).arm_near = 40.0;
        (*pose).arm_far = -40.0;
        (*pose).fore_near = -85.0;
        (*pose).fore_far = 85.0;
    }
    // Slow motions are stepped in eighths of a unit, so a walker that is
    // only breathing is not repainted every single frame.
    pub fn quantise(mut units: f64) -> f64 {
        return (((units * 8.0) + 0.5).floor() / 8.0);
    }
    // ---- Terrain ------------------------------------------------------------
    pub fn update_world(mut self) {
        self.world_left = self.world.left();
        self.world_right = self.world.right();
        let mut new_scale: i32 = self.world.scale_at(self.cx);
        if new_scale != self.scale {
            let mut ratio: f64 = (((new_scale) as f64) / ((self.scale) as f64));
            self.vx *= ratio;
            self.vy *= ratio;
            self.scale = new_scale;
        }
        self.size_factor = (((self.scale) as f64) / ((Settings::BASE_PIXEL_SIZE) as f64));
    }
    // Moves horizontally unless that would mean walking into the side of
    // higher ground (a neighbouring monitor whose bottom edge is higher).
    pub fn move_x(mut self, mut dx: f64) -> bool {
        let mut half: f64 = ((self.body.half_width * self.scale) as f64);
        let mut next: f64 = rt::max_f64(
            (((self.world_left) as f64) + half),
            rt::min_f64((((self.world_right) as f64) - half), (self.cx + dx)),
        );
        if self.world.ground_at(next) < (self.feet_y - ((self.scale) as f64)) {
            return false;
        }
        self.cx = next;
        return true;
    }
    // Looks for a step up ahead, e.g. the edge of a monitor that sits higher.
    pub fn wall_ahead(mut self, height: &mut f64, distance: &mut f64) -> bool {
        (*height) = 0.0;
        (*distance) = f64::MAX;
        let mut reach: f64 = (400.0 * self.size_factor);
        let mut edges: Arr<i32> = self.world.monitor_edges();
        {
            let mut i: i32 = 0;
            'l1: while i < edges.len() {
                'c2: {
                    let mut edge: i32 = (*edges.at(i));
                    let mut d: f64 = ((((edge) as f64) - self.cx) * ((self.facing) as f64));
                    if ((d <= 0.0) || (d > reach)) || (d >= (*distance)) {
                        break 'c2;
                    }
                    let mut beyond: f64 = self
                        .world
                        .ground_at(((if (self.facing > 0) { edge } else { (edge - 1) }) as f64));
                    if beyond < (self.feet_y - ((self.scale) as f64)) {
                        (*distance) = d;
                        (*height) = (self.feet_y - beyond);
                    }
                }
                i += 1;
            }
        }
        return ((*distance) != f64::MAX);
    }
    // Keeps the feet on the ground; returns false if there is none (it
    // has walked off the edge of a higher monitor) and a fall has begun.
    pub fn stay_on_ground(mut self) -> bool {
        let mut ground: f64 = self.world.ground_at(self.cx);
        self.clip_y = ground;
        if ground > (self.feet_y + 1.5) {
            self.vy = 0.0;
            self.enter(Mode::Air);
            return false;
        }
        self.feet_y = ground;
        return true;
    }
    // ---- Behaviour ----------------------------------------------------------
    pub fn between(mut self, mut low: f64, mut high: f64) -> f64 {
        return (low + (self.rng.next_double() * (high - low)));
    }
    pub fn approach(mut value: f64, mut target: f64, mut step: f64) -> f64 {
        return (if (value < target) {
            rt::min_f64(target, (value + step))
        } else {
            rt::max_f64(target, (value - step))
        });
    }
    pub fn enter(mut self, mut next: Mode) {
        self.mode = next;
        self.mode_time = 0.0;
        self.head_turned = false;
        self.braking = false;
    }
    pub fn brake(mut self, mut next: Mode) {
        self.braking = true;
        self.after_brake = next;
    }
    // Goes into `next`: braking to a halt first if on the move, at once if standing.
    pub fn stop(mut self, mut next: Mode) {
        if self.mode == Mode::Walk {
            self.brake(next);
        } else {
            self.enter(next);
        }
    }
    pub fn abort_charge(mut self) {
        self.charge_target = <Walker>::default();
        self.set_charge_cooldown(8.0);
        if self.playful {
            // Couldn't catch them: give up, laughing and out of breath.
            self.playful = false;
            self.set_mood((*EXPRESSION_PANTING), 1.4);
        }
        self.enter(Mode::Walk);
    }
    pub fn end_flee(mut self) {
        if !(self.playful) {
            // Still shaken for a while: friends may come to help.
            self.set_mood((*EXPRESSION_SAD), 5.0);
            self.upset = true;
        }
        self.playful = false;
        self.enter(Mode::Walk);
    }
    // A face to keep for a while once back on the move.
    pub fn set_mood(mut self, mut mood: Expression, mut seconds: f64) {
        self.mood_face = mood;
        self.mood_left = seconds;
        self.upset = false;
    }
    pub fn start_sweat(mut self) {
        if self.sweat <= 0.0 {
            self.sweat = 0.001;
        }
    }
    // Walking, charging and fleeing all share this: ease towards the
    // target speed, turn at the ends of the desktop, jump up onto higher
    // monitors. `antics` allows the errands, random hops and idle stops.
    pub fn step_walk(mut self, mut dt: f64, mut speed_factor: f64, mut antics: bool) {
        if !(self.stay_on_ground()) {
            return;
        }
        let mut full: f64 =
            (((Settings::WALK_SPEED * self.size_factor) * self.eff_speed()) * speed_factor);
        if antics {
            full *= self.goal_pace();
        }
        let mut target: f64 = (if self.braking {
            0.0
        } else {
            (((self.facing) as f64) * full)
        });
        self.vx = Walker::approach(
            self.vx,
            target,
            (((if self.braking {
                Settings::BRAKING
            } else {
                Settings::ACCELERATION
            }) * self.size_factor)
                * dt),
        );
        let mut blocked: bool = !(self.move_x((self.vx * dt)));
        self.walk_phase +=
            (((self.vx).abs() * dt) / (self.body.stride_cells * ((self.scale) as f64)));
        self.trial_dist += ((self.vx).abs() * dt);
        if self.braking {
            if (self.vx).abs() < 1.0 {
                self.vx = 0.0;
                self.enter(self.after_brake);
            }
            return;
        }
        // Turn around at the outer edge of the first and last monitor,
        // slowing down in time to stop right at it.
        let mut half: f64 = ((self.body.half_width * self.scale) as f64);
        let mut room: f64 = (if (self.facing > 0) {
            ((((self.world_right) as f64) - half) - self.cx)
        } else {
            (self.cx - (((self.world_left) as f64) + half))
        });
        if room <= (((self.vx * self.vx) / ((2.0 * Settings::BRAKING) * self.size_factor)) + 1.0) {
            if (self.vx).abs() > (0.7 * full) {
                self.emit_dust(
                    (self.cx + (((self.facing * 4) * self.scale) as f64)),
                    self.feet_y,
                    2,
                    0.6,
                );
            }
            self.after_turn = Mode::Walk;
            self.brake(Mode::Turn);
            return;
        }
        // A monitor with a higher bottom edge ahead: jump up onto it,
        // timing the take-off so the top of the arc is over the edge.
        let mut wall_height: f64 = <f64>::default();
        let mut wall_distance: f64 = <f64>::default();
        if self.wall_ahead(rt::mref(&mut wall_height), rt::mref(&mut wall_distance)) {
            let mut gravity: f64 = (Settings::GRAVITY * self.size_factor);
            let mut launch: f64 =
                ((2.0 * gravity) * (wall_height + ((6 * self.scale) as f64))).sqrt();
            if blocked || (wall_distance <= ((full * launch) / gravity)) {
                self.start_jump(launch, (((self.facing) as f64) * full));
                return;
            }
        } else if blocked {
            self.after_turn = Mode::Walk;
            self.brake(Mode::Turn);
            return;
        }
        if !(antics) {
            self.goal = Goal::None; // being chased or doing the chasing cancels any errand
            return;
        }
        // On an errand: head for it, and nothing else until it is done.
        if self.goal != Goal::None {
            self.pursue_goal(dt);
            return;
        }
        // The clumsy one trips now and then.
        if self.personality.clumsy > 0.0 {
            self.trip_timer -= dt;
            if self.trip_timer <= 0.0 {
                self.trip_timer = (self.between(40.0, 100.0) / self.personality.clumsy);
                if (self.vx).abs() > (0.7 * full) {
                    self.start_trip();
                    return;
                }
            }
        }
        // Walking with a purpose, not just pacing: lonely with a friend
        // behind, turn back instead of marching away; now and then head
        // for the stretch of desktop where life has been good.
        if !(self.braking) && (self.steer_cooldown <= 0.0) {
            let mut together: bool = (((!self.company.is_null() && self.company.is_visible())
                && !(self.company.is_leaving()))
                && ((self.company.feet_y() - self.feet_y).abs() <= 2.0));
            let mut cdx: f64 = (if together {
                (self.company.centre_x() - self.cx)
            } else {
                0.0
            });
            let mut adx: f64 = (cdx).abs();
            if (((together && (self.social > 0.6)) && (adx > (2.5 * self.body_width())))
                && (adx < (800.0 * self.size_factor)))
                && ((cdx * ((self.facing) as f64)) < 0.0)
            {
                self.steer_cooldown = self.between(6.0, 12.0);
                self.after_turn = Mode::Walk;
                self.brake(Mode::Turn);
                return;
            }
            if self.rng.next_double() < (dt * 0.05) {
                let mut hx: f64 = self.bin_center(self.best_place());
                if ((hx - self.cx) * ((self.facing) as f64)) < -(self.body_width()) {
                    self.steer_cooldown = self.between(8.0, 15.0);
                    self.after_turn = Mode::Walk;
                    self.brake(Mode::Turn);
                    return;
                }
                self.steer_cooldown = self.between(4.0, 8.0);
            }
        }
        self.idle_timer -= dt;
        self.hop_timer -= dt;
        if self.idle_timer <= 0.0 {
            self.idle_timer = self.between(self.personality.idle_min, self.personality.idle_max);
            self.decide();
        } else if self.hop_timer <= 0.0 {
            // A hop only while it has the energy to spare.
            self.hop_timer =
                (self.between(self.personality.hop_min, self.personality.hop_max) * self.eff_hop());
            if (self.energy > 0.6) && ((self.vx).abs() > (0.8 * full)) {
                self.start_jump((Settings::HOP_SPEED * self.size_factor), self.vx);
            }
        }
    }
    pub fn start_jump(mut self, mut speed: f64, mut air_speed: f64) {
        self.pending_jump = speed;
        self.pending_air_speed = air_speed;
        self.enter(Mode::Crouch);
    }
    pub fn land(mut self, mut ground: f64) {
        let mut impact: f64 = self.vy;
        self.feet_y = ground;
        self.vy = 0.0;
        self.squash = rt::max_f64(
            0.35,
            rt::min_f64(1.0, (impact / (Settings::HOP_SPEED * self.size_factor))),
        );
        if impact > ((0.5 * Settings::HOP_SPEED) * self.size_factor) {
            self.emit_dust(self.cx, ground, 4, self.squash);
        }
        if self.knocked {
            // Run away from whoever did that.
            self.knocked = false;
            if self.vx != 0.0 {
                self.facing = (if (self.vx > 0.0) { 1 } else { -1 });
            }
            self.flee_time = Walker::FLEE_SECONDS;
            self.set_greet_cooldown(rt::max_f64(
                self.greet_cooldown(),
                (Walker::FLEE_SECONDS + 2.0),
            ));
            self.start_sweat();
            self.enter(Mode::Flee);
        } else if self.celebrating && (self.joy_hops > 0) {
            // Straight back up for the next hop.
            self.joy_hops -= 1;
            self.start_jump(((0.85 * Settings::HOP_SPEED) * self.size_factor), 0.0);
        } else if self.celebrating {
            self.celebrating = false;
            self.set_mood((*EXPRESSION_GIGGLE), 0.9);
            self.enter(Mode::Walk);
        } else if self.woke_up {
            self.woke_up = false;
            if self.spooked {
                // Each takes a fright its own way.
                self.spooked = false;
                'sw1: {
                    match self.personality.temperament {
                        Temperament::Friendly | Temperament::Goofy => {
                            self.set_mood((*EXPRESSION_GIGGLE), 1.6);
                        }
                        Temperament::Calm => {
                            self.set_mood((*EXPRESSION_GRUMPY), 2.5);
                        }
                        Temperament::Timid => {
                            // Too much: out cold.
                            self.vx = 0.0;
                            self.start_antic(Antic::Faint);
                            return;
                        }
                        Temperament::Feisty => {
                            // Bristling fury: pecking order to sort out.
                            self.set_mood((*EXPRESSION_FURIOUS), 2.5);
                        }
                        _ => {
                            self.set_mood((*EXPRESSION_FURIOUS), 2.0);
                            self.set_charge_cooldown(0.0); // and may well go for whoever did it
                        }
                    }
                }
            } else {
                // Where am I?
                self.set_mood((*EXPRESSION_CURIOUS), 1.0);
            }
            self.enter(Mode::Walk);
        } else if self.land_pause > 0.0 {
            self.vx = 0.0;
            self.rest_duration = self.land_pause;
            self.land_pause = 0.0;
            self.enter(Mode::Rest);
        } else {
            self.enter(Mode::Walk);
        }
    }
    // ---- Dust ---------------------------------------------------------------
    pub fn emit_dust(mut self, mut x: f64, mut y: f64, mut count: i32, mut strength: f64) {
        {
            let mut i: i32 = 0;
            'l1: while i < count {
                let mut side: f64 = ((if ((i % 2) == 0) { -1 } else { 1 }) as f64);
                let mut puff: Puff = Puff::new();
                puff.x = (x + ((side * self.between(2.0, 6.0)) * ((self.scale) as f64)));
                puff.y = (y - (self.between(0.0, 1.0) * ((self.scale) as f64)));
                puff.vx = (((side * self.between(25.0, 70.0)) * self.size_factor) * strength);
                puff.vy = (-(self.between(8.0, 35.0)) * self.size_factor);
                puff.max_life = self.between(0.28, 0.5);
                puff.life = puff.max_life;
                self.puffs.add(puff);
                i += 1;
            }
        }
    }
    pub fn step_dust(mut self, mut dt: f64) {
        {
            let mut i: i32 = (self.puffs.len() - 1);
            'l1: while i >= 0 {
                'c2: {
                    let mut puff: Puff = (*self.puffs.at(i));
                    puff.life -= dt;
                    if puff.life <= 0.0 {
                        puff.free();
                        self.puffs.remove_at(i);
                        break 'c2;
                    }
                    puff.x += (puff.vx * dt);
                    puff.y += (puff.vy * dt);
                    let mut drag: f64 = rt::max_f64(0.0, (1.0 - (3.0 * dt)));
                    puff.vx *= drag;
                    puff.vy *= drag;
                }
                i -= 1;
            }
        }
    }
}

// The hiccups come at 0.4, 1.0, 1.7 and 2.3 s; each is a sharp jolt
// that dies away over 0.14 s.
pub static WALKER_HICCUP_TIMES: rt::Global<Arr<f64>> =
    rt::Global::new(|| Arr::from_vec(vec![0.4, 1.0, 1.7, 2.3]));

// The local hour, asked for by every walker every tick: converting
// the clock to local time each time is wasted work, so it is worked
// out once and kept until the hour ends (or the clock is set back).
pub static WALKER_CACHED_HOUR: rt::Global<i32> = rt::Global::new(|| -1);

pub static WALKER_HOUR_ENDS_UTC: rt::Global<i64> = rt::Global::new(|| <i64>::default());
