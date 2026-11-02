//! The menu choices that survive between runs: their names, defaults and the checks
//! that keep a damaged or older value from doing harm. Where they are kept is the
//! `PreferenceStore` port's business.
use crate::ports::PreferenceStore;
use idlewalker_domain::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct Preferences(Arc<dyn PreferenceStore>);

fn first_n_mask(n: i32, max: i32) -> i32 {
    (0..n.clamp(0, max)).fold(0, |m, i| m | (1 << i))
}

const CORNER_KEYS: [&str; 4] = [
    "CornerTopLeft",
    "CornerTopRight",
    "CornerBottomLeft",
    "CornerBottomRight",
];

macro_rules! switch {
    ($load:ident, $save:ident, $key:expr, $default:expr) => {
        pub fn $load(&self) -> bool {
            self.load($key, $default as i32) != 0
        }
        pub fn $save(&self, on: bool) {
            self.0.save($key, on as i32)
        }
    };
}

impl Preferences {
    pub fn new(store: Arc<dyn PreferenceStore>) -> Self {
        Preferences(store)
    }

    fn load(&self, name: &str, fallback: i32) -> i32 {
        self.0.load(name).unwrap_or(fallback)
    }

    switch!(
        load_walkers_enabled,
        save_walkers_enabled,
        "WalkersEnabled",
        Settings::WALKERS_ENABLED_BY_DEFAULT
    );
    switch!(
        load_minions_enabled,
        save_minions_enabled,
        "MinionsEnabled",
        Settings::MINIONS_ENABLED_BY_DEFAULT
    );
    switch!(
        load_bean_enabled,
        save_bean_enabled,
        "MrBeanEnabled",
        Settings::MR_BEAN_BY_DEFAULT
    );
    switch!(
        load_birds_enabled,
        save_birds_enabled,
        "BirdsEnabled",
        Settings::BIRDS_ENABLED_BY_DEFAULT
    );
    switch!(
        load_panther_enabled,
        save_panther_enabled,
        "PantherEnabled",
        Settings::PANTHER_BY_DEFAULT
    );
    switch!(
        load_hide_on_move,
        save_hide_on_move,
        "HideOnMove",
        Settings::HIDE_ON_MOVE_BY_DEFAULT
    );
    switch!(
        load_hot_corners_enabled,
        save_hot_corners_enabled,
        "HotCornersEnabled",
        Settings::HOT_CORNERS_BY_DEFAULT
    );
    switch!(
        load_keep_awake,
        save_keep_awake,
        "KeepAwake",
        Settings::KEEP_AWAKE_BY_DEFAULT
    );

    /// True once "start at sign-in" has been set up, so the default applies only once.
    pub fn startup_decided(&self) -> bool {
        self.load("StartupDecided", 0) != 0
    }
    pub fn mark_startup_decided(&self) {
        self.0.save("StartupDecided", 1)
    }

    /// The newest version a background update check has already told the person about.
    pub fn load_update_announced(&self) -> i32 {
        self.load("UpdateAnnounced", 0)
    }
    pub fn save_update_announced(&self, stamp: i32) {
        self.0.save("UpdateAnnounced", stamp)
    }

    /// Which Minions may come out; upgrades from the old first-N count.
    pub fn load_minion_mask(&self) -> i32 {
        let all = (1 << Settings::MAX_WALKERS) - 1;
        self.load(
            "MinionMask",
            first_n_mask(
                self.load("WalkerCount", Settings::DEFAULT_WALKERS),
                Settings::MAX_WALKERS,
            ),
        ) & all
    }
    pub fn save_minion_mask(&self, mask: i32) {
        self.0
            .save("MinionMask", mask & ((1 << Settings::MAX_WALKERS) - 1))
    }

    pub fn load_bird_mask(&self) -> i32 {
        let all = (1 << Settings::MAX_BIRDS) - 1;
        self.load(
            "BirdMask",
            first_n_mask(
                self.load("BirdCount", Settings::DEFAULT_BIRDS),
                Settings::MAX_BIRDS,
            ),
        ) & all
    }
    pub fn save_bird_mask(&self, mask: i32) {
        self.0
            .save("BirdMask", mask & ((1 << Settings::MAX_BIRDS) - 1))
    }

    pub fn load_idle_seconds(&self) -> i32 {
        let s = self.load("IdleSeconds", Settings::IDLE_SECONDS);
        if SETTINGS_IDLE_CHOICES.contains(s) {
            s
        } else {
            Settings::IDLE_SECONDS
        }
    }
    pub fn save_idle_seconds(&self, s: i32) {
        self.0.save("IdleSeconds", s)
    }

    pub fn load_dress(&self) -> i32 {
        let d = self.load("Dress", SETTINGS_DRESS_COLORS[0]);
        if d == Settings::RANDOM_DRESS || (0..=0xFFFFFF).contains(&d) {
            d
        } else {
            SETTINGS_DRESS_COLORS[0]
        }
    }
    pub fn save_dress(&self, d: i32) {
        self.0.save("Dress", d)
    }

    pub fn load_corner_action(&self, c: usize) -> CornerAction {
        let fallback = SETTINGS_DEFAULT_CORNER_ACTIONS[c as i32];
        let v = self.load(CORNER_KEYS[c], fallback as i32);
        if (0..10).contains(&v) {
            CornerAction::from_i32(v)
        } else {
            fallback
        }
    }
    pub fn save_corner_action(&self, c: usize, a: CornerAction) {
        self.0.save(CORNER_KEYS[c], a as i32)
    }

    pub fn load_hot_corner_delay(&self) -> i32 {
        let ms = self.load("HotCornerDelayMs", Settings::HOT_CORNER_DELAY_MS);
        if SETTINGS_HOT_CORNER_DELAY_CHOICES.contains(ms) {
            ms
        } else {
            Settings::HOT_CORNER_DELAY_MS
        }
    }
    pub fn save_hot_corner_delay(&self, ms: i32) {
        self.0.save("HotCornerDelayMs", ms)
    }
}
