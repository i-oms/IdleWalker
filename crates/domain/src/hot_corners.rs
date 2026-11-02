// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

// A corner of a monitor. The order is the order of the tray menu and the registry values.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(i32)]
pub enum Corner {
    #[default]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}
impl Corner {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => Corner::TopLeft,
            1 => Corner::TopRight,
            2 => Corner::BottomLeft,
            3 => Corner::BottomRight,
            _ => Self::default(),
        }
    }
}
impl std::fmt::Display for Corner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// What pushing the cursor into a hot corner does.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(i32)]
pub enum CornerAction {
    #[default]
    None,
    TaskView,      // Win+Tab
    ShowDesktop,   // Win+D
    StartMenu,     // Ctrl+Esc
    Notifications, // Win+N, the notification centre
    QuickSettings, // Win+A
    LockScreen,    // the lock screen
    ScreenSaver,   // start the screen saver
    MonitorOff,    // put the monitors to sleep
    CallMinions,   // bring the Minions out without waiting for the rest time
}
impl CornerAction {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => CornerAction::None,
            1 => CornerAction::TaskView,
            2 => CornerAction::ShowDesktop,
            3 => CornerAction::StartMenu,
            4 => CornerAction::Notifications,
            5 => CornerAction::QuickSettings,
            6 => CornerAction::LockScreen,
            7 => CornerAction::ScreenSaver,
            8 => CornerAction::MonitorOff,
            9 => CornerAction::CallMinions,
            _ => Self::default(),
        }
    }
}
impl std::fmt::Display for CornerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

//
// Watches for the cursor resting in a corner of any monitor. A corner
// fires once when the cursor has sat in it for the dwell time, and not
// again until the cursor has left it; a drag through a corner (a mouse
// button held) never fires. No Windows code in here: the caller feeds
// in the cursor, the monitor rectangles and (optionally) each monitor's
// sprite scale so the trigger zone stays the same physical size on
// high-DPI displays.
//
// Multi-monitor notes (why this looks the way it does):
//  - The armed spot keeps its monitor identity, not just the corner
//    type. Two monitors side by side share an edge: the left monitor's
//    TopRight and the right monitor's TopLeft are 1 px apart. Only
//    remembering TopRight vs TopLeft would reset the dwell on a 1 px
//    jitter across the seam, so inner corners could never fire. Here
//    co-located seam corners are treated as one physical hotspot.
//  - Arming uses a small box (Reach) but disarming uses a bigger one
//    (Reach + ReleaseExtra). Sensor jitter of a few px no longer resets
//    the dwell, on the main screen or anywhere else.
//  - The nearest corner across ALL monitors wins, not the first
//    monitor in the list, so overlapping / duplicate layouts behave.
//  - After firing (or being reset while armed, e.g. a full-screen app
//    takes over), the cursor must leave the spot's release zone before
//    anything can arm again, and the same hotspot stays quiet for a
//    short cooldown. Toggle actions (Task view, Show desktop) therefore
//    cannot bounce open-close-open under a held-still cursor.
// Zero allocations per call, integer math only: cheap enough to sample
// at 30 Hz without being felt.
//
#[derive(Clone, Default)]
pub struct HotCornersData {
    // How long the cursor must rest in a corner before it fires.
    pub dwell_seconds: f64,
    // Fallback trigger size when no per-monitor scale is given.
    pub reach: i32,
    pub armed_monitor: i32,
    pub armed_corner: i32,
    pub armed_x: i32,
    pub armed_y: i32,
    pub armed_release: i32,
    pub armed_since: f64,
    pub fired: bool,
    pub last_fire_time: f64,
    pub last_fire_monitor: i32,
    pub last_fire_corner: i32,
    pub need_leave: bool,
    pub leave_x: i32,
    pub leave_y: i32,
    pub leave_r: i32,
}
rt::handle!(HotCorners, HotCornersData);

impl HotCorners {
    fn alloc__() -> HotCorners {
        let mut this = HotCorners::from_data(HotCornersData::default());
        this.dwell_seconds = (((Settings::HOT_CORNER_DELAY_MS) as f64) / 1000.0);
        this.reach = Settings::HOT_CORNER_REACH;
        this.armed_monitor = -1;
        this.armed_corner = -1;
        this.last_fire_time = -1000.0;
        this.last_fire_monitor = -1;
        this.last_fire_corner = -1;
        this
    }
    pub fn new() -> HotCorners {
        Self::alloc__()
    }
    // Trigger size for one monitor, keeping the physical target constant across DPIs.
    pub fn reach_for(mut scale: i32) -> i32 {
        let mut reach: i32 = (scale * 2);
        if reach < Settings::HOT_CORNER_REACH {
            reach = Settings::HOT_CORNER_REACH;
        }
        return reach;
    }
    pub fn chebyshev(mut x1: i32, mut y1: i32, mut x2: i32, mut y2: i32) -> i32 {
        let mut dx: i32 = (x1 - x2);
        if dx < 0 {
            dx = -dx;
        }
        let mut dy: i32 = (y1 - y2);
        if dy < 0 {
            dy = -dy;
        }
        return (if (dx > dy) { dx } else { dy });
    }
    pub fn release_for(mut self, mut monitor: i32, mut scales: Arr<i32>) -> i32 {
        if ((!scales.is_null() && (monitor >= 0)) && (monitor < scales.len()))
            && ((*scales.at(monitor)) > 0)
        {
            return (HotCorners::reach_for((*scales.at(monitor)))
                + Settings::HOT_CORNER_RELEASE_EXTRA);
        }
        return (self.reach + Settings::HOT_CORNER_RELEASE_EXTRA);
    }
    //
    // Claims a completed dwell. Hands back true exactly when the action
    // should run; anything inside the refire cooldown is swallowed (but
    // still counts as fired, so it cannot retry on the next sample).
    //
    pub fn claim_fire(
        mut self,
        mut now: f64,
        mut mon: i32,
        mut c: i32,
        mut ax: i32,
        mut ay: i32,
        mut release: i32,
        monitor: &mut i32,
        corner: &mut Corner,
    ) -> bool {
        (*monitor) = -1;
        (*corner) = Corner::TopLeft;
        self.fired = true;
        if ((mon == self.last_fire_monitor) && (c == self.last_fire_corner))
            && ((now - self.last_fire_time)
                < (((Settings::HOT_CORNER_REFIRE_COOLDOWN_MS) as f64) / 1000.0))
        {
            return false;
        }
        self.last_fire_time = now;
        self.last_fire_monitor = mon;
        self.last_fire_corner = c;
        self.need_leave = true;
        self.leave_x = ax;
        self.leave_y = ay;
        self.leave_r = release;
        (*monitor) = mon;
        (*corner) = Corner::from_i32(c);
        return true;
    }
    //
    // Feeds one cursor sample. True when a corner has just fired, with
    // which monitor and which corner in `monitor` / `corner`.
    //
    pub fn step(
        mut self,
        mut now: f64,
        mut cursor: Point,
        mut monitors: Arr<Rectangle>,
        mut scales: Arr<i32>,
        mut button_down: bool,
        monitor: &mut i32,
        corner: &mut Corner,
    ) -> bool {
        (*monitor) = -1;
        (*corner) = Corner::TopLeft;
        // A fired (or reset) spot must be left before anything arms
        // again: holding the cursor still can never re-fire.
        if self.need_leave {
            if HotCorners::chebyshev(cursor.x, cursor.y, self.leave_x, self.leave_y) <= self.leave_r
            {
                return false;
            }
            self.need_leave = false;
        }
        let mut found_monitor: i32 = -1;
        let mut found_corner: i32 = -1;
        let mut found_x: i32 = 0;
        let mut found_y: i32 = 0;
        let mut found_dist: i32 = i32::MAX;
        {
            let mut i: i32 = 0;
            'l1: while i < monitors.len() {
                'c2: {
                    let mut b: Rectangle = (*monitors.at(i));
                    let mut reach: i32 = <i32>::default();
                    if (!scales.is_null() && (i < scales.len())) && ((*scales.at(i)) > 0) {
                        reach = HotCorners::reach_for((*scales.at(i)));
                    } else {
                        reach = self.reach;
                    }
                    let mut left: bool =
                        ((cursor.x >= b.left()) && (cursor.x < (b.left() + reach)));
                    let mut right: bool =
                        ((cursor.x < b.right()) && (cursor.x >= (b.right() - reach)));
                    let mut top: bool = ((cursor.y >= b.top()) && (cursor.y < (b.top() + reach)));
                    let mut bottom: bool =
                        ((cursor.y < b.bottom()) && (cursor.y >= (b.bottom() - reach)));
                    if !((((top && left) || (top && right)) || (bottom && left))
                        || (bottom && right))
                    {
                        break 'c2;
                    }
                    // One monitor matched: work out which corner and how central.
                    let mut c: i32 = <i32>::default();
                    let mut ax: i32 = <i32>::default();
                    let mut ay: i32 = <i32>::default();
                    if top && left {
                        c = (Corner::TopLeft as i32 as i32);
                        ax = b.left();
                        ay = b.top();
                    } else if top && right {
                        c = (Corner::TopRight as i32 as i32);
                        ax = (b.right() - 1);
                        ay = b.top();
                    } else if bottom && left {
                        c = (Corner::BottomLeft as i32 as i32);
                        ax = b.left();
                        ay = (b.bottom() - 1);
                    } else {
                        c = (Corner::BottomRight as i32 as i32);
                        ax = (b.right() - 1);
                        ay = (b.bottom() - 1);
                    }
                    let mut dist: i32 = HotCorners::chebyshev(cursor.x, cursor.y, ax, ay);
                    if dist < found_dist {
                        found_dist = dist;
                        found_monitor = i;
                        found_corner = c;
                        found_x = ax;
                        found_y = ay;
                    }
                }
                i += 1;
            }
        }
        if found_monitor >= 0 {
            if self.armed_monitor < 0 {
                self.armed_monitor = found_monitor;
                self.armed_corner = found_corner;
                self.armed_x = found_x;
                self.armed_y = found_y;
                self.armed_release = self.release_for(found_monitor, scales.clone());
                self.armed_since = now;
                self.fired = false;
            } else if (found_monitor != self.armed_monitor) || (found_corner != self.armed_corner) {
                // A different monitor corner: same physical hotspot
                // (seam neighbours are 1 px apart) keeps the dwell,
                // a truly different corner restarts it.
                let mut gap: i32 =
                    HotCorners::chebyshev(found_x, found_y, self.armed_x, self.armed_y);
                self.armed_monitor = found_monitor;
                self.armed_corner = found_corner;
                self.armed_x = found_x;
                self.armed_y = found_y;
                self.armed_release = self.release_for(found_monitor, scales.clone());
                if gap > Settings::HOT_CORNER_CO_LOCATE {
                    self.armed_since = now;
                    self.fired = false;
                }
            }
            // else: same spot, dwell continues.
            if button_down {
                self.armed_since = now; // dragging through: the dwell starts over once the button is up
                return false;
            }
            if self.fired || ((now - self.armed_since) < (self.dwell_seconds - 1e-6)) {
                return false; // a hair of slack for summed frame times
            }
            return self.claim_fire(
                now,
                found_monitor,
                found_corner,
                found_x,
                found_y,
                self.release_for(found_monitor, scales.clone()),
                &mut *monitor,
                &mut *corner,
            );
        }
        // Not inside any arm box: hysteresis keeps the dwell through a
        // few px of jitter (or a step across a seam) instead of
        // disarming at once.
        if self.armed_monitor >= 0 {
            if HotCorners::chebyshev(cursor.x, cursor.y, self.armed_x, self.armed_y)
                <= self.armed_release
            {
                if button_down {
                    self.armed_since = now;
                    return false;
                }
                if self.fired || ((now - self.armed_since) < (self.dwell_seconds - 1e-6)) {
                    return false;
                }
                return self.claim_fire(
                    now,
                    self.armed_monitor,
                    self.armed_corner,
                    self.armed_x,
                    self.armed_y,
                    self.armed_release,
                    &mut *monitor,
                    &mut *corner,
                );
            }
            self.armed_monitor = -1;
            self.armed_corner = -1;
        }
        return false;
    }
    //
    // Feeds one cursor sample. True when a corner has just fired, with
    // which one in `corner`. The monitor is not reported.
    //
    pub fn step_2(
        mut self,
        mut now: f64,
        mut cursor: Point,
        mut monitors: Arr<Rectangle>,
        mut button_down: bool,
        corner: &mut Corner,
    ) -> bool {
        let mut monitor: i32 = <i32>::default();
        return self.step(
            now,
            cursor,
            monitors.clone(),
            <Arr<i32>>::default(),
            button_down,
            rt::mref(&mut monitor),
            &mut *corner,
        );
    }
    //
    // Forget the corner the cursor is in. If one was armed, the cursor
    // must still leave its zone before anything arms again, so a reset
    // (full-screen app, switch toggled) can never cause a re-fire under
    // a held-still cursor.
    //
    pub fn reset(mut self) {
        if self.armed_monitor >= 0 {
            self.need_leave = true;
            self.leave_x = self.armed_x;
            self.leave_y = self.armed_y;
            self.leave_r = self.armed_release;
        }
        self.armed_monitor = -1;
        self.armed_corner = -1;
        self.fired = false;
    }
}
