//! Mouse macros: what the mouse did for a while, kept under a name and done again when
//! asked, from the menu or from the Telegram remote. Only the mouse is noted (where it
//! went, its buttons and its wheel), never the keyboard: what is typed stays unwritten.
//!
//! A macro is a small text file in the program's folder, a step per line, so it can be
//! read, renamed, mended or thrown away by hand. Playing it back is by position: the
//! same places on the screen are clicked, whatever is there by then.
use crate::ports::{Desktop, MouseKey, Os, PointerAct, PointerEvent, Workstation};
use idlewalker_domain::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const HEADER: &str = "\
# An IdleWalker mouse macro. Each line is a step: the milliseconds to wait first, what
# the mouse does (move, down, up or wheel), and where on the screen (x y).
";

/// The longest a step may wait: a mistyped number must not hold the mouse for days.
const LONGEST_WAIT_MS: u32 = 10 * 60 * 1000;

/// Idle this long when a macro starts, the display may be off or asleep: it is woken,
/// and given a moment, before the first step.
const IDLE_SECONDS: f64 = 20.0;
/// Waking, monitors are given at least this long, and at most this, to come back.
const SETTLE_AT_LEAST: Duration = Duration::from_millis(1500);
const SETTLE_AT_MOST: Duration = Duration::from_secs(8);
/// The longest nap while waiting for a step: how soon a stop is heard.
const NAP: Duration = Duration::from_millis(20);
/// A nap that runs this far over means the machine slept or the program was held back.
const OVERSLEPT: Duration = Duration::from_secs(1);
/// How often, while playing, the screen is asked whether it has locked.
const LOCK_CHECK_EVERY: Duration = Duration::from_secs(1);

/// Whether every place the steps go to is on one of these screens.
fn all_on_screen(steps: &[PointerEvent], screens: &[Rectangle]) -> bool {
    !screens.is_empty()
        && steps
            .iter()
            .all(|s| screens.iter().any(|r| r.contains(Point::new(s.x, s.y))))
}

/// A macro's name is its file's: letters, digits, "-" and "_", so that it is safe in a
/// path and fits a Telegram button.
pub fn valid_name(name: &str) -> bool {
    let word = |b: u8| b.is_ascii_alphanumeric() || b == b'-' || b == b'_';
    (1..=32).contains(&name.len()) && name.bytes().all(word)
}

fn key_name(key: MouseKey) -> &'static str {
    match key {
        MouseKey::Left => "left",
        MouseKey::Right => "right",
        MouseKey::Middle => "middle",
    }
}

fn key_named(name: &str) -> Option<MouseKey> {
    [MouseKey::Left, MouseKey::Right, MouseKey::Middle]
        .into_iter()
        .find(|k| key_name(*k) == name)
}

/// The steps as the file's text.
fn write(steps: &[PointerEvent]) -> String {
    let mut out = String::from(HEADER);
    let mut at = 0;
    for s in steps {
        let wait = s.at_ms.saturating_sub(at);
        at = s.at_ms;
        let what = match s.act {
            PointerAct::Move => "move".to_string(),
            PointerAct::Down(k) => format!("down {}", key_name(k)),
            PointerAct::Up(k) => format!("up {}", key_name(k)),
            PointerAct::Wheel(amount) => format!("wheel {amount}"),
        };
        out.push_str(&format!("{wait} {what} {} {}\n", s.x, s.y));
    }
    out
}

fn read_step(line: &str, at: &mut u32) -> Option<PointerEvent> {
    let mut words = line.split_whitespace();
    let wait: u32 = words.next()?.parse().ok()?;
    let act = match words.next()? {
        "move" => PointerAct::Move,
        "down" => PointerAct::Down(key_named(words.next()?)?),
        "up" => PointerAct::Up(key_named(words.next()?)?),
        "wheel" => PointerAct::Wheel(words.next()?.parse().ok()?),
        _ => return None,
    };
    let x = words.next()?.parse().ok()?;
    let y = words.next()?.parse().ok()?;
    if words.next().is_some() || wait > LONGEST_WAIT_MS {
        return None;
    }
    *at = at.checked_add(wait)?;
    Some(PointerEvent {
        at_ms: *at,
        x,
        y,
        act,
    })
}

/// The steps in a file's text; `None` when a line is not a step, since half a macro is
/// worse than none.
fn read(text: &str) -> Option<Vec<PointerEvent>> {
    let mut at = 0;
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| read_step(l, &mut at))
        .collect()
}

/// What was watched, made into a macro. The clicks that stopped the recording (one on
/// the tray icon, and where that only opens a menu, another in it) are not part of it,
/// nor the way there; a button let go that was never pressed (the click that started
/// it) is dropped, one still held at the end is let go, and the clock starts at the
/// first step.
fn tidy(mut events: Vec<PointerEvent>, stopping_clicks: usize) -> Vec<PointerEvent> {
    let down = |e: &PointerEvent| matches!(e.act, PointerAct::Down(_));
    for _ in 0..stopping_clicks {
        if let Some(last) = events.iter().rposition(down) {
            events.truncate(last);
        }
    }
    while events.last().is_some_and(|e| e.act == PointerAct::Move) {
        events.pop();
    }
    let mut held: Vec<MouseKey> = Vec::new();
    events.retain(|e| match e.act {
        PointerAct::Down(k) => {
            held.push(k);
            true
        }
        PointerAct::Up(k) => match held.iter().position(|h| *h == k) {
            Some(i) => {
                held.remove(i);
                true
            }
            None => false,
        },
        _ => true,
    });
    if let Some(&last) = events.last() {
        for k in held {
            events.push(PointerEvent {
                act: PointerAct::Up(k),
                ..last
            });
        }
    }
    let start = events.first().map_or(0, |e| e.at_ms);
    for e in &mut events {
        e.at_ms -= start;
    }
    events
}

/// How a request to play ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Played {
    Done,
    /// Asked to stop part-way.
    Stopped,
    /// No such macro, or its file is not one.
    Missing,
    /// Another is being played, or one is being recorded.
    Busy,
    /// The screen is locked, before or part-way: what was played went nowhere.
    Locked,
}

/// How a request to rename ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Renamed {
    Done,
    Missing,
    /// The new name is not one a macro can have.
    BadName,
    /// The new name is another macro's.
    Taken,
    Failed,
}

#[derive(Default)]
struct State {
    recording: AtomicBool,
    playing: AtomicBool,
    stop: AtomicBool,
}

#[derive(Clone)]
pub struct Macros {
    ws: Arc<dyn Workstation>,
    desktop: Arc<dyn Desktop>,
    state: Arc<State>,
}

impl Macros {
    pub fn new(ws: Arc<dyn Workstation>, desktop: Arc<dyn Desktop>) -> Self {
        Macros {
            ws,
            desktop,
            state: Arc::new(State::default()),
        }
    }

    /// Whether this system lets the mouse be watched and worked. Wayland does not: a
    /// program there is shown only what happens over its own windows.
    pub fn supported(&self) -> bool {
        let set = |name: &str| self.ws.env(name).filter(|v| !v.is_empty());
        let wayland = set("WAYLAND_DISPLAY").is_some()
            || set("XDG_SESSION_TYPE").is_some_and(|v| v.eq_ignore_ascii_case("wayland"));
        self.ws.os() != Os::Linux || !wayland
    }

    /// Where the macros are kept.
    pub fn dir(&self) -> PathBuf {
        self.ws.app_dir().join("macros")
    }

    /// The folder, made if it was not there, to show it in the file manager.
    pub fn folder(&self) -> PathBuf {
        let dir = self.dir();
        let _ = self.ws.make_dir(&dir);
        dir
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir().join(format!("{name}.txt"))
    }

    /// Every macro's name, in order.
    pub fn list(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .ws
            .list_files(&self.dir(), 1)
            .iter()
            .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("txt")))
            .filter_map(|p| p.file_stem()?.to_str().map(str::to_string))
            .filter(|n| valid_name(n))
            .collect();
        names.sort_by_key(|n| n.to_ascii_lowercase());
        names
    }

    /// A macro's name as it is kept, however its letters were typed.
    pub fn find(&self, name: &str) -> Option<String> {
        self.list()
            .into_iter()
            .find(|n| n.eq_ignore_ascii_case(name))
    }

    pub fn recording(&self) -> bool {
        self.state.recording.load(Ordering::Relaxed)
    }

    pub fn playing(&self) -> bool {
        self.state.playing.load(Ordering::Relaxed)
    }

    /// Starts noting what the mouse does. False when the system will not allow it, or a
    /// macro is being played: that is not the person's own doing.
    pub fn start_recording(&self) -> bool {
        if self.playing() {
            return false;
        }
        if self.state.recording.swap(true, Ordering::SeqCst) {
            return true;
        }
        let watching = self.desktop.pointer_watch(true);
        self.state.recording.store(watching, Ordering::SeqCst);
        watching
    }

    /// Stops recording, on as many clicks as it took to ask, which are themselves left
    /// out, and keeps what was done under the next free name: that name, or `None`
    /// when nothing was done or nothing was being recorded.
    pub fn stop_recording(&self, stopping_clicks: usize) -> Option<String> {
        if !self.state.recording.swap(false, Ordering::SeqCst) {
            return None;
        }
        self.desktop.pointer_watch(false);
        let steps = tidy(self.desktop.pointer_watched(), stopping_clicks);
        if steps.is_empty() {
            return None;
        }
        let taken = self.list();
        let name = (1..)
            .map(|n| format!("macro-{n}"))
            .find(|n| !taken.iter().any(|t| t.eq_ignore_ascii_case(n)))?;
        self.ws
            .write_text(&self.path(&name), &write(&steps))
            .ok()
            .map(|_| name)
    }

    /// Drops a recording under way, keeping nothing.
    pub fn abandon_recording(&self) {
        if self.state.recording.swap(false, Ordering::SeqCst) {
            self.desktop.pointer_watch(false);
            let _ = self.desktop.pointer_watched();
        }
    }

    /// Does again what a macro holds, at the pace it was done. Slow, as slow as the
    /// macro is long: off the UI thread.
    pub fn play(&self, name: &str) -> Played {
        let steps = self
            .find(name)
            .and_then(|n| self.ws.read_text(&self.path(&n)))
            .and_then(|text| read(&text));
        let Some(steps) = steps else {
            return Played::Missing;
        };
        if self.recording() || self.state.playing.swap(true, Ordering::SeqCst) {
            return Played::Busy;
        }
        self.state.stop.store(false, Ordering::SeqCst);
        // Asked before the screen is woken, which is itself input and resets the count.
        let was_idle = self.desktop.system_idle_seconds() >= IDLE_SECONDS;
        // The display may be off, a screen saver up, or idle sleep near: woken now, and
        // kept so to the end, or the first clicks only wake the screen, and a long pause
        // in the macro lets it go dark (or the machine sleep) again part-way.
        self.desktop.hold_screen_for_input(true);
        let played = self.play_steps(&steps, was_idle);
        self.desktop.hold_screen_for_input(false);
        self.state.playing.store(false, Ordering::SeqCst);
        played
    }

    fn stopping(&self) -> bool {
        self.state.stop.load(Ordering::Relaxed)
    }

    /// Naps until `until` while asking to stop is heard; false when it was.
    fn nap_until(&self, until: Instant) -> bool {
        while !self.stopping() {
            let left = until.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return true;
            }
            std::thread::sleep(left.min(NAP));
        }
        false
    }

    /// After a long idle, monitors that slept come back a moment after the screen
    /// wakes, and some (DisplayPort ones above all) are missing from the layout until
    /// then: clicked now, a step meant for one would land on another. So the steps wait,
    /// a little, for every place they go to be on a screen again.
    fn settle_after_waking(&self, steps: &[PointerEvent]) -> bool {
        let woke = Instant::now();
        if !self.nap_until(woke + SETTLE_AT_LEAST) {
            return false;
        }
        while woke.elapsed() < SETTLE_AT_MOST {
            if all_on_screen(steps, &self.desktop.screen_bounds()) {
                return true;
            }
            if !self.nap_until(Instant::now() + Duration::from_millis(250)) {
                return false;
            }
        }
        // A monitor gone for good: played as it lies, as it would be without waiting.
        true
    }

    fn play_steps(&self, steps: &[PointerEvent], was_idle: bool) -> Played {
        if was_idle && !self.settle_after_waking(steps) {
            return Played::Stopped;
        }
        // Input played to a locked screen goes to its password box, not the desktop:
        // said so, rather than "done" with nothing done.
        if self.desktop.screen_locked() {
            return Played::Locked;
        }
        let mut checked = Instant::now();
        let mut started = Instant::now();
        let mut held: Vec<MouseKey> = Vec::new();
        let mut last = None;
        let mut locked = false;
        'steps: for step in steps {
            let due = Duration::from_millis(step.at_ms as u64);
            loop {
                if self.stopping() {
                    break 'steps;
                }
                let left = due.saturating_sub(started.elapsed());
                if left.is_zero() {
                    break;
                }
                // In short naps, so that a stop is heard soon.
                let nap = left.min(NAP);
                let before = Instant::now();
                std::thread::sleep(nap);
                // A nap that ran far over is the machine having slept, or this program
                // having been held back: the clock moves on by as much, so the steps
                // left keep their pace instead of all being done at once on waking.
                let over = before.elapsed().saturating_sub(nap);
                if over >= OVERSLEPT {
                    started += over;
                }
            }
            if checked.elapsed() >= LOCK_CHECK_EVERY {
                checked = Instant::now();
                if self.desktop.screen_locked() {
                    locked = true;
                    break;
                }
            }
            self.desktop.pointer_send(step);
            match step.act {
                PointerAct::Down(k) => held.push(k),
                PointerAct::Up(k) => held.retain(|h| *h != k),
                _ => {}
            }
            last = Some(*step);
        }
        // Stopped mid-drag, or a file mended by hand: no button is left held.
        if let Some(last) = last {
            for k in held {
                self.desktop.pointer_send(&PointerEvent {
                    act: PointerAct::Up(k),
                    ..last
                });
            }
        }
        if locked {
            Played::Locked
        } else if self.stopping() {
            Played::Stopped
        } else {
            Played::Done
        }
    }

    /// Asks the macro being played to stop; false when none is.
    pub fn stop_playing(&self) -> bool {
        let playing = self.playing();
        if playing {
            self.state.stop.store(true, Ordering::SeqCst);
        }
        playing
    }

    pub fn rename(&self, old: &str, new: &str) -> Renamed {
        let Some(old) = self.find(old) else {
            return Renamed::Missing;
        };
        if !valid_name(new) {
            return Renamed::BadName;
        }
        // The same name in other letters is its own to take.
        if self.find(new).is_some_and(|n| n != old) {
            return Renamed::Taken;
        }
        if old == new {
            return Renamed::Done;
        }
        let from = self.path(&old);
        let Some(text) = self.ws.read_text(&from) else {
            return Renamed::Failed;
        };
        // Out of the way first: where names differ only by their letters' case, the
        // old file and the new are one.
        self.ws.remove_file(&from);
        match self.ws.write_text(&self.path(new), &text) {
            Ok(()) => Renamed::Done,
            Err(_) => {
                let _ = self.ws.write_text(&from, &text);
                Renamed::Failed
            }
        }
    }

    /// Throws a macro away; false when there was none by that name.
    pub fn delete(&self, name: &str) -> bool {
        match self.find(name) {
            Some(n) => {
                self.ws.remove_file(&self.path(&n));
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(at_ms: u32, x: i32, y: i32, act: PointerAct) -> PointerEvent {
        PointerEvent { at_ms, x, y, act }
    }

    use MouseKey::{Left, Right};
    use PointerAct::{Down, Move, Up, Wheel};

    #[test]
    fn a_name_is_safe_in_a_path_and_on_a_button() {
        assert!(valid_name("macro-1"));
        assert!(valid_name("Open_mail"));
        assert!(!valid_name(""));
        assert!(!valid_name("open mail"));
        assert!(!valid_name("../x"));
        assert!(!valid_name("a:b"));
        assert!(!valid_name(&"x".repeat(33)));
    }

    #[test]
    fn a_step_on_a_monitor_still_asleep_waits_for_it() {
        let steps = vec![at(0, 100, 100, Move), at(50, 2500, 300, Down(Left))];
        let one = Rectangle::new(0, 0, 1920, 1080);
        let two = Rectangle::new(1920, 0, 1920, 1080);
        assert!(!all_on_screen(&steps, &[one]));
        assert!(all_on_screen(&steps, &[one, two]));
        // No layout to be had yet: not ready either.
        assert!(!all_on_screen(&steps, &[]));
    }

    #[test]
    fn a_macro_is_written_as_it_is_read() {
        let steps = vec![
            at(0, 812, 440, Move),
            at(120, 812, 440, Down(Right)),
            at(200, 812, 440, Up(Right)),
            at(500, -850, 520, Wheel(-120)),
        ];
        let text = write(&steps);
        assert!(text.ends_with(
            "0 move 812 440\n120 down right 812 440\n80 up right 812 440\n\
             300 wheel -120 -850 520\n"
        ));
        assert_eq!(read(&text), Some(steps));
        assert_eq!(read(HEADER), Some(vec![]));
    }

    #[test]
    fn a_line_that_is_not_a_step_spoils_the_macro() {
        assert_eq!(read("0 move 1"), None);
        assert_eq!(read("0 jump 1 2"), None);
        assert_eq!(read("0 down thumb 1 2"), None);
        assert_eq!(read("0 move 1 2 3"), None);
        assert_eq!(read("soon move 1 2"), None);
        // A wait too long to be meant.
        assert_eq!(read("999999999 move 1 2"), None);
    }

    #[test]
    fn the_click_that_stops_the_recording_is_left_out() {
        let watched = vec![
            // The click on "Record" being let go.
            at(40, 5, 5, Up(Left)),
            at(900, 100, 100, Move),
            at(1000, 100, 100, Down(Right)),
            at(1080, 100, 100, Up(Right)),
            at(1500, 120, 160, Down(Left)),
            at(1560, 120, 160, Up(Left)),
            // The way to the tray icon, and the click on it.
            at(2000, 900, 1000, Move),
            at(2100, 1900, 1060, Move),
            at(2200, 1900, 1060, Down(Left)),
            at(2260, 1900, 1060, Up(Left)),
        ];
        // Stopped from a menu the tray icon opened: both of those clicks go.
        assert_eq!(tidy(watched.clone(), 2).len(), 3);
        assert_eq!(
            tidy(watched, 1),
            vec![
                at(0, 100, 100, Move),
                at(100, 100, 100, Down(Right)),
                at(180, 100, 100, Up(Right)),
                at(600, 120, 160, Down(Left)),
                at(660, 120, 160, Up(Left)),
            ]
        );
        // Nothing but the stopping click: nothing to keep.
        assert_eq!(
            tidy(vec![at(10, 1, 1, Move), at(20, 1, 1, Down(Left))], 1),
            vec![]
        );
        assert_eq!(tidy(vec![], 1), vec![]);
    }
}
