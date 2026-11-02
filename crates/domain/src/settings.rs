// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

//
// The knobs worth turning. Speeds are in pixels per second at 100%
// display scaling; they are scaled up automatically on high-DPI monitors.
//
#[derive(Clone, Default)]
pub struct SettingsData {}
rt::handle!(Settings, SettingsData);

// The rest times the tray menu offers, in seconds: one to five seconds, one to five minutes.
pub static SETTINGS_IDLE_CHOICES: rt::Global<Arr<i32>> =
    rt::Global::new(|| Arr::from_vec(vec![1, 2, 3, 4, 5, 60, 120, 180, 240, 300]));

//
// The overalls colours the Dress menu offers, by name, as 0xRRGGBB. The
// first is the default - the classic blue. The menu also offers Random
// and a colour picker for anything else.
//
pub static SETTINGS_DRESS_NAMES: rt::Global<Arr<String>> = rt::Global::new(|| {
    Arr::from_vec(vec![
        "Classic blue".to_string(),
        "Red".to_string(),
        "Green".to_string(),
        "Purple".to_string(),
        "Orange".to_string(),
        "Pink".to_string(),
        "Teal".to_string(),
        "Black".to_string(),
    ])
});

pub static SETTINGS_DRESS_COLORS: rt::Global<Arr<i32>> = rt::Global::new(|| {
    Arr::from_vec(vec![
        0x2b59b8, 0xc0392b, 0x2e8b57, 0x6e3fb0, 0xe07a1f, 0xe2559a, 0x1f8a8a, 0x2a2a30,
    ])
});

// What each corner does until the tray menu says otherwise, in Corner order.
pub static SETTINGS_DEFAULT_CORNER_ACTIONS: rt::Global<Arr<CornerAction>> = rt::Global::new(|| {
    Arr::from_vec(vec![
        CornerAction::TaskView,
        CornerAction::Notifications,
        CornerAction::StartMenu,
        CornerAction::ShowDesktop,
    ])
});

pub static SETTINGS_HOT_CORNER_DELAY_CHOICES: rt::Global<Arr<i32>> =
    rt::Global::new(|| Arr::from_vec(vec![100, 200, 300, 500, 1000]));

impl Settings {
    fn alloc__() -> Settings {
        let mut this = Settings::from_data(SettingsData::default());
        this
    }
    pub fn new() -> Settings {
        Self::alloc__()
    }
    // Cursor rest time before the walker appears, until the user picks another from the tray menu.
    pub const IDLE_SECONDS: i32 = 2;
    // Pixels of sensor jitter that don't count as movement.
    pub const MOVE_TOLERANCE: i32 = 2;
    // Size of one sprite cell at 100% display scaling.
    pub const BASE_PIXEL_SIZE: i32 = 3;
    // How many walkers can be ticked on at once (one menu entry per Minion).
    pub const MAX_WALKERS: i32 = 10;
    // How many come out on a fresh install (the first ones ticked).
    pub const DEFAULT_WALKERS: i32 = 5;
    //
    // The share of one CPU core each painter thread may use (0..1); with two
    // painters the cast may use twice this. When the cast
    // needs more than this - many walkers, a slow machine - they are drawn
    // every 2nd (3rd, 4th) refresh instead of every one, while still moving
    // at full rate. The frame rate degrades; the computer does not.
    //
    pub const PAINT_BUDGET: f64 = 0.5;
    // The furthest the budget may thin the drawing: every n-th refresh at most.
    pub const MAX_PAINT_EVERY: i32 = 4;
    // How many Angry Birds can be ticked on at once (one menu entry per bird).
    pub const MAX_BIRDS: i32 = 10;
    // How many birds come out on a fresh install (the first ones ticked).
    pub const DEFAULT_BIRDS: i32 = 3;
    // The Dress menu's "Random" entry: every Minion picks its own colour each time it comes out.
    pub const RANDOM_DRESS: i32 = -1;
    // The master switch: whether anyone comes out at all until the tray menu says otherwise.
    pub const WALKERS_ENABLED_BY_DEFAULT: bool = true;
    // Whether the Minions are on until the tray menu says otherwise.
    pub const MINIONS_ENABLED_BY_DEFAULT: bool = true;
    // Whether Mr. Bean is on until the tray menu says otherwise. He and the Minions are independent of each other.
    pub const MR_BEAN_BY_DEFAULT: bool = false;
    // Whether the Angry Birds are on until the tray menu says otherwise. Independent of the Minions and Mr. Bean.
    pub const BIRDS_ENABLED_BY_DEFAULT: bool = false;
    // Whether the Pink Panther is on until the tray menu says otherwise. Independent of the rest.
    pub const PANTHER_BY_DEFAULT: bool = false;
    // Whether a mouse move sends everyone back underground until the tray menu says otherwise.
    pub const HIDE_ON_MOVE_BY_DEFAULT: bool = true;
    // Whether the hot corners are on until the tray menu says otherwise.
    pub const HOT_CORNERS_BY_DEFAULT: bool = true;
    //
    // Whether "Keep system awake" is on until the tray menu says otherwise.
    // Off by default: blocking sleep is the user's choice, never silent.
    //
    pub const KEEP_AWAKE_BY_DEFAULT: bool = false;
    //
    // While "Keep system awake" is on and the real mouse has sat still
    // this many seconds, a harmless F15 tap is sent this often to reset
    // the idle timer. Kept well under a one-minute screen-saver timeout,
    // so the session never reads as idle. Only fires once the user is
    // away, so it never disturbs active work.
    //
    pub const ANTI_IDLE_SECONDS: f64 = 30.0;
    // How far in from a corner's two edges (pixels at 100% scaling, grown with DPI) still counts as being in it.
    pub const HOT_CORNER_REACH: i32 = 6;
    // How many extra pixels of wobble past the trigger box still count as holding the corner (hysteresis).
    pub const HOT_CORNER_RELEASE_EXTRA: i32 = 8;
    // Two corner points this close together are one physical hotspot (neighbouring monitors on a shared seam).
    pub const HOT_CORNER_CO_LOCATE: i32 = 6;
    // How long the cursor must rest in a corner before it fires (ms), and the choices the menu offers.
    pub const HOT_CORNER_DELAY_MS: i32 = 300;
    // After a corner fires, the same corner cannot fire again for this long (ms): toggle actions can't bounce.
    pub const HOT_CORNER_REFIRE_COOLDOWN_MS: i32 = 1500;
    // Whether "Start with Windows" is switched on the first time the program runs.
    pub const START_WITH_WINDOWS_BY_DEFAULT: bool = true;
    // How fast each walker updates its learned preferences (0..1). Higher = changes its mind faster.
    pub const LEARN_RATE: f64 = 0.12;
    // How often a walker tries something unexpected instead of its favourite (0..1).
    pub const EXPLORE_RATE: f64 = 0.15;
    pub const WALK_SPEED: f64 = 90.0;
    pub const ACCELERATION: f64 = 420.0;
    pub const BRAKING: f64 = 520.0;
    pub const GRAVITY: f64 = 1100.0;
    pub const HOP_SPEED: f64 = 320.0;
    pub const STARTLE_SPEED: f64 = 210.0;
    pub const EXIT_GRAVITY: f64 = 2200.0;
}
