//! The running program, minus its windows: watches the cursor, keeps the session
//! awake, holds back for full-screen apps, fires the hot corners, steps the cast once
//! per frame, saves what it learned, and applies the menu's choices. The presentation
//! layer calls `tick` once per frame and shows the result.
use crate::brains::Brains;
use crate::ports::{Desktop, Displays, Journal, Modifier};
use crate::preferences::{Indicator, Preferences};
use idlewalker_domain::prelude::*;
use std::sync::Arc;

/// Every choice the menu shows.
#[derive(Clone)]
pub struct Choices {
    pub walkers: bool,
    pub minions: bool,
    pub birds: bool,
    pub hide_on_move: bool,
    pub corners: bool,
    pub keep_awake: bool,
    /// A coloured spot under the cursor at every click.
    pub click_highlight: bool,
    /// Pressing `find_key` twice shows where the cursor is.
    pub find_cursor: bool,
    pub find_key: Modifier,
    /// The colour of each cursor indicator, in `Indicator::ALL` order, as 0xRRGGBB.
    pub indicator_colours: [i32; 3],
    pub startup: bool,
    pub minion_mask: i32,
    pub bird_mask: i32,
    pub idle_seconds: i32,
    pub dress: i32,
    pub actions: [CornerAction; 4],
    pub delay_ms: i32,
}

/// Tracks the cursor and how long it has been still. Movement below the tolerance is
/// ignored, so a jittery sensor can't keep resetting the clock.
struct CursorWatcher {
    anchor: Point,
    latest: Point,
    last_move: f64,
}

impl CursorWatcher {
    fn new(at: Option<Point>) -> Self {
        let p = at.unwrap_or_default();
        CursorWatcher {
            anchor: p,
            latest: p,
            last_move: 0.0,
        }
    }
    fn poll(&mut self, at: Option<Point>, now: f64) -> bool {
        let Some(c) = at else {
            return false;
        };
        self.latest = c;
        let moved = (c.x - self.anchor.x).abs() > Settings::MOVE_TOLERANCE
            || (c.y - self.anchor.y).abs() > Settings::MOVE_TOLERANCE;
        if moved {
            self.anchor = c;
            self.last_move = now;
        }
        moved
    }
    fn idle_for(&self, now: f64, seconds: f64) -> bool {
        now - self.last_move >= seconds
    }
}

/// A hot corner that fired this frame.
#[derive(Clone, Copy, Debug)]
pub struct CornerFire {
    /// Index into `Displays::screens`.
    pub monitor: i32,
    pub corner: Corner,
    pub action: CornerAction,
}

/// What a frame did, for the presentation layer.
#[derive(Clone, Copy, Debug, Default)]
pub struct TickReport {
    /// The cursor moved past the tolerance.
    pub moved: bool,
    /// The power source, when it was read this frame.
    pub on_battery: Option<bool>,
    pub fired: Option<CornerFire>,
    /// Any walker is on screen.
    pub any_visible: bool,
}

/// What the session needs from outside, handed over by the composition root.
pub struct SessionPorts {
    pub desktop: Arc<dyn Desktop>,
    pub displays: Box<dyn Displays>,
    pub journal: Arc<dyn Journal>,
    pub preferences: Preferences,
    pub brains: Brains,
}

pub struct Session {
    desktop: Arc<dyn Desktop>,
    displays: Box<dyn Displays>,
    journal: Arc<dyn Journal>,
    preferences: Preferences,
    brains: Brains,
    troupe: Troupe,
    corners: HotCorners,
    cursor: CursorWatcher,
    choices: Choices,
    next_check: f64,
    next_brain_save: f64,
    next_nudge: f64,
    nudged_at: f64,
    nudge_blocked: bool,
    brains_dirty: bool,
    full_screen: bool,
    summoned: bool,
}

impl Session {
    /// Restores the cast and the choices. `manage_startup` lets the session apply the
    /// "start at sign-in" default and follow a moved program; a test run leaves it alone,
    /// so a development build cannot re-point it at a build folder.
    pub fn new(ports: SessionPorts, manage_startup: bool) -> Self {
        let SessionPorts {
            desktop,
            displays,
            journal,
            preferences,
            brains,
        } = ports;
        let prefs = &preferences;
        let troupe = Troupe::new(displays.world(), Random::new());
        troupe.set_minion_mask(prefs.load_minion_mask());
        troupe.set_bird_mask(prefs.load_bird_mask());
        troupe.set_dress(prefs.load_dress());
        troupe.set_minions_enabled(prefs.load_minions_enabled());
        troupe.set_birds_enabled(prefs.load_birds_enabled());
        brains.load(troupe);

        let delay_ms = prefs.load_hot_corner_delay();
        let mut corners = HotCorners::new();
        corners.dwell_seconds = delay_ms as f64 / 1000.0;

        if manage_startup {
            // On the very first run, apply the default; after that the user's choice stands.
            if !prefs.startup_decided() {
                let wanted = Settings::START_WITH_WINDOWS_BY_DEFAULT;
                if !wanted || desktop.startup_set(true).is_ok() {
                    prefs.mark_startup_decided();
                }
            }
            // Follow the program if it was moved or renamed.
            if desktop.startup_is_enabled() {
                let _ = desktop.startup_set(true);
            }
        }

        let choices = Choices {
            walkers: prefs.load_walkers_enabled(),
            minions: troupe.minions_enabled(),
            birds: troupe.birds_enabled(),
            hide_on_move: prefs.load_hide_on_move(),
            corners: prefs.load_hot_corners_enabled(),
            keep_awake: prefs.load_keep_awake(),
            click_highlight: prefs.load_click_highlight(),
            find_cursor: prefs.load_find_cursor(),
            find_key: prefs.load_find_key(),
            indicator_colours: Indicator::ALL.map(|i| prefs.load_indicator_colour(i)),
            startup: desktop.startup_is_enabled(),
            minion_mask: troupe.minion_mask(),
            bird_mask: troupe.bird_mask(),
            idle_seconds: prefs.load_idle_seconds(),
            dress: troupe.dress(),
            actions: [0, 1, 2, 3].map(|c| prefs.load_corner_action(c)),
            delay_ms,
        };
        Session {
            cursor: CursorWatcher::new(desktop.cursor_pos()),
            desktop,
            displays,
            journal,
            preferences,
            brains,
            troupe,
            corners,
            choices,
            next_check: 0.0,
            next_brain_save: 0.0,
            next_nudge: 0.0,
            nudged_at: 0.0,
            nudge_blocked: false,
            brains_dirty: false,
            full_screen: false,
            summoned: false,
        }
    }

    pub fn troupe(&self) -> Troupe {
        self.troupe
    }
    pub fn choices(&self) -> &Choices {
        &self.choices
    }
    pub fn displays(&self) -> &dyn Displays {
        &*self.displays
    }
    pub fn desktop(&self) -> &Arc<dyn Desktop> {
        &self.desktop
    }
    pub fn journal(&self) -> &Arc<dyn Journal> {
        &self.journal
    }
    pub fn brains(&self) -> &Brains {
        &self.brains
    }
    /// The cursor as last read.
    pub fn cursor(&self) -> Point {
        self.cursor.latest
    }
    /// Where the cursor last came to rest.
    pub fn cursor_anchor(&self) -> Point {
        self.cursor.anchor
    }
    pub fn full_screen(&self) -> bool {
        self.full_screen
    }

    /// Arms "keep awake" as chosen; call from a thread that lives as long as the program.
    pub fn arm_keep_awake(&self) {
        self.desktop.keep_awake(self.choices.keep_awake);
    }

    /// One frame. `now` is seconds since start, `dt` the step for the cast.
    pub fn tick(&mut self, now: f64, dt: f64) -> TickReport {
        let mut report = TickReport::default();
        self.displays.refresh();
        let moved = self.cursor.poll(self.desktop.cursor_pos(), now);
        report.moved = moved;
        if moved {
            self.summoned = false;
        }

        // Keep the session counted as active while the user is away. Once the system's
        // own idle clock has run past the limit, a harmless nudge resets it. If a nudge
        // does not register, it is retried every few seconds and logged once.
        if self.choices.keep_awake
            && now >= self.next_nudge
            && self.desktop.system_idle_seconds() >= Settings::ANTI_IDLE_SECONDS
        {
            if self.nudged_at > 0.0 && !self.nudge_blocked {
                self.nudge_blocked = true;
                self.journal.note(&format!(
                    "anti-idle nudge did not register; foreground {}",
                    self.desktop.foreground_description()
                ));
            }
            self.next_nudge = now + 5.0;
            self.nudged_at = now;
            self.desktop.nudge_activity();
        } else if self.nudged_at > 0.0
            && now - self.nudged_at < 5.0
            && self.desktop.system_idle_seconds() < now - self.nudged_at + 1.0
        {
            self.nudged_at = 0.0;
            if self.nudge_blocked {
                self.journal.note("anti-idle nudge registers again");
            }
            self.nudge_blocked = false;
        }

        // Twice a second: a full-screen app in front keeps the cast away, and on battery
        // the animation runs at half rate. Before the corners, so they can honour it too.
        if now >= self.next_check {
            self.next_check = now + 0.5;
            let was_full = self.full_screen;
            self.full_screen = self.desktop.is_fullscreen();
            if self.full_screen && !was_full {
                self.journal.note(&format!(
                    "foreground fullscreen: corners held ({})",
                    self.desktop.foreground_description()
                ));
            } else if !self.full_screen && was_full {
                self.journal.note("foreground windowed: corners live");
            }
            report.on_battery = Some(self.desktop.on_battery());
        }

        // The hot corners: the cursor resting in a corner of any monitor.
        if self.choices.corners && !self.full_screen {
            let (mut fired, mut corner) = (0, Corner::default());
            if self.corners.step(
                now,
                self.cursor.latest,
                self.displays.monitor_bounds(),
                self.displays.monitor_scales(),
                self.desktop.mouse_button_down(),
                &mut fired,
                &mut corner,
            ) {
                let action = self.choices.actions[corner as usize];
                self.journal.note(&format!(
                    "fire monitor={} corner={} action={} cursor={},{} foreground={}",
                    fired,
                    corner,
                    action,
                    self.cursor.latest.x,
                    self.cursor.latest.y,
                    self.desktop.foreground_description()
                ));
                report.fired = Some(CornerFire {
                    monitor: fired,
                    corner,
                    action,
                });
                if action == CornerAction::CallMinions {
                    self.summoned = self.choices.walkers;
                } else {
                    self.desktop.perform_corner(action);
                }
            }
        } else if self.full_screen {
            self.corners.reset();
        }

        // Every minute: write what they learned, only if somebody has been out.
        if now >= self.next_brain_save {
            self.next_brain_save = now + 60.0;
            if self.brains_dirty {
                self.brains_dirty = false;
                self.brains.save_in_background(self.troupe);
            }
        }

        // The master switch; a mouse move only sends them away while "Hide on mouse move" is ticked.
        let enabled = self.choices.walkers && !self.full_screen;
        let want_out = enabled
            && (self.summoned || self.cursor.idle_for(now, self.choices.idle_seconds as f64));
        let disturbed = (moved && self.choices.hide_on_move) || !enabled;
        self.troupe
            .step(dt, self.cursor.anchor, want_out, disturbed);

        report.any_visible = self.troupe.any_visible();
        if report.any_visible {
            self.brains_dirty = true;
        }
        report
    }

    /// Applies a menu choice and remembers it. Returns false for an id that is not a
    /// choice (opening a window, say), which is the presentation layer's to handle.
    pub fn apply(&mut self, id: &str) -> bool {
        let prefs = self.preferences.clone();
        let (name, arg) = id.split_once(':').unwrap_or((id, ""));
        let value: i32 = arg.parse().unwrap_or(0);
        if let Some(which) = Indicator::ALL.into_iter().find(|i| i.id() == name) {
            if arg.parse::<i32>().is_ok() {
                self.choose_indicator_colour(which, value);
            }
            return true;
        }
        match name {
            "walkers" => {
                self.choices.walkers = !self.choices.walkers;
                prefs.save_walkers_enabled(self.choices.walkers);
            }
            "hide-on-move" => {
                self.choices.hide_on_move = !self.choices.hide_on_move;
                prefs.save_hide_on_move(self.choices.hide_on_move);
            }
            "minions" => {
                self.choices.minions = !self.choices.minions;
                self.troupe.set_minions_enabled(self.choices.minions);
                prefs.save_minions_enabled(self.choices.minions);
            }
            "minions-all" => {
                let all = (1 << Settings::MAX_WALKERS) - 1;
                self.set_minion_mask(if self.choices.minion_mask == all {
                    0
                } else {
                    all
                });
            }
            "minion" => self.set_minion_mask(self.choices.minion_mask ^ (1 << value)),
            "birds" => {
                self.choices.birds = !self.choices.birds;
                self.troupe.set_birds_enabled(self.choices.birds);
                prefs.save_birds_enabled(self.choices.birds);
            }
            "birds-all" => {
                let all = (1 << Settings::MAX_BIRDS) - 1;
                self.set_bird_mask(if self.choices.bird_mask == all {
                    0
                } else {
                    all
                });
            }
            "bird" => self.set_bird_mask(self.choices.bird_mask ^ (1 << value)),
            "idle" => {
                self.choices.idle_seconds = value;
                prefs.save_idle_seconds(value);
            }
            "dress" => self.choose_dress(value),
            "corners" => {
                self.choices.corners = !self.choices.corners;
                self.corners.reset();
                prefs.save_hot_corners_enabled(self.choices.corners);
            }
            "corner" => {
                if let Some((c, a)) = arg.split_once(':') {
                    if let (Ok(c), Ok(a)) = (c.parse::<usize>(), a.parse::<i32>()) {
                        let action = CornerAction::from_i32(a);
                        self.choices.actions[c] = action;
                        prefs.save_corner_action(c, action);
                    }
                }
            }
            "delay" => {
                self.choices.delay_ms = value;
                self.corners.dwell_seconds = value as f64 / 1000.0;
                prefs.save_hot_corner_delay(value);
            }
            "keep-awake" => {
                self.choices.keep_awake = !self.choices.keep_awake;
                prefs.save_keep_awake(self.choices.keep_awake);
                self.desktop.keep_awake(self.choices.keep_awake);
            }
            "click-highlight" => {
                self.choices.click_highlight = !self.choices.click_highlight;
                prefs.save_click_highlight(self.choices.click_highlight);
            }
            "find-cursor" => {
                self.choices.find_cursor = !self.choices.find_cursor;
                prefs.save_find_cursor(self.choices.find_cursor);
            }
            "find-key" => {
                if let Some(key) = Modifier::from_i32(value) {
                    self.choices.find_key = key;
                    prefs.save_find_key(key);
                }
            }
            "startup" => {
                let enable = !self.choices.startup;
                match self.desktop.startup_set(enable) {
                    Ok(()) => {
                        self.choices.startup = enable;
                        prefs.mark_startup_decided();
                    }
                    Err(e) => self
                        .journal
                        .note(&format!("could not change the start-up setting: {}", e)),
                }
            }
            _ => return false,
        }
        true
    }

    fn set_minion_mask(&mut self, m: i32) {
        self.choices.minion_mask = m;
        self.troupe.set_minion_mask(m);
        self.preferences.save_minion_mask(m);
    }

    fn set_bird_mask(&mut self, m: i32) {
        self.choices.bird_mask = m;
        self.troupe.set_bird_mask(m);
        self.preferences.save_bird_mask(m);
    }

    /// Dresses the Minions; picking Random again re-rolls everyone.
    pub fn choose_dress(&mut self, chosen: i32) {
        if chosen != self.choices.dress || chosen == Settings::RANDOM_DRESS {
            self.choices.dress = chosen;
            self.troupe.set_dress(chosen);
            self.preferences.save_dress(chosen);
        }
    }

    /// Colours a cursor indicator, as 0xRRGGBB.
    pub fn choose_indicator_colour(&mut self, which: Indicator, rgb: i32) {
        let rgb = rgb & 0xFFFFFF;
        self.choices.indicator_colours[which as usize] = rgb;
        self.preferences.save_indicator_colour(which, rgb);
    }

    /// Forgets everything the cast learned, in memory and on disk.
    pub fn forget_brains(&self) {
        self.brains.forget(self.troupe);
    }

    /// At exit: lets the machine sleep again and saves what was learned.
    pub fn shut_down(&self) {
        self.desktop.keep_awake(false);
        self.brains.save(self.troupe);
    }
}
