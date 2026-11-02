// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

//
// The whole cast: brings walkers out one at a time, sends them away,
// and lets them meet - greetings and dances between the sociable ones,
// shoving and shouting matches by the aggressive ones, games of tag,
// onlookers reacting, and yawns and dances that spread.
//
#[derive(Clone, Default)]
pub struct TroupeData {
    pub world: IWorld,
    pub rng: Random,
    pub walkers: Arr<Walker>,
    pub bean: Walker,
    pub panther: Walker,
    pub minions_enabled: bool,
    pub bean_enabled: bool,
    pub birds_enabled: bool,
    pub panther_enabled: bool,
    pub minion_mask: i32,
    pub bird_mask: i32,
    pub dress: i32,
    pub spawn_timer: f64,
}
rt::handle!(Troupe, TroupeData);

impl Troupe {
    fn alloc__() -> Troupe {
        let mut this = Troupe::from_data(TroupeData::default());
        this.walkers = Arr::new(0);
        this.minions_enabled = true;
        this.minion_mask = Troupe::first_n_mask(Settings::DEFAULT_WALKERS);
        this.bird_mask = Troupe::first_n_mask(Settings::DEFAULT_BIRDS);
        this.dress = (*(*SETTINGS_DRESS_COLORS).at(0));
        this
    }
    pub fn new(mut world: IWorld, mut rng: Random) -> Troupe {
        let mut this = Self::alloc__();
        this.ctor(world, rng);
        this
    }
    fn ctor(mut self, mut world: IWorld, mut rng: Random) {
        self.world = world;
        self.rng = rng;
        self.bean = Walker::new(world, rng, (*PERSONALITY_BEAN));
        self.walkers.add(self.bean);
        self.panther = Walker::new(world, rng, (*PERSONALITY_PANTHER));
        self.walkers.add(self.panther);
    }
    pub const GREET_DISTANCE_BODIES: f64 = 2.2;
    pub const CONTACT_DISTANCE_BODIES: f64 = 0.9;
    pub const GREET_COOLDOWN_SECONDS: f64 = 12.0;
    //
    // Every walker created so far: Mr. Bean first (hidden unless he is
    // switched on), the Pink Panther second (hidden unless he is
    // switched on), then the Minions in roster order, then the Angry
    // Birds in flock order. The list only ever grows, so each walker
    // keeps its index and its window.
    //
    pub fn walkers(mut self) -> Arr<Walker> {
        return self.walkers.clone();
    }
    pub fn full_cast() -> i32 {
        return ((2 + Settings::MAX_WALKERS) + Settings::MAX_BIRDS);
    }
    //
    // Which Minions may come out, as a bit mask over Personality.Roster:
    // bit i set means Roster[i] is enabled. Zero means none of them.
    //
    pub fn minion_mask(mut self) -> i32 {
        return self.minion_mask;
    }
    pub fn set_minion_mask(mut self, mut value: i32) {
        self.minion_mask = (value & Troupe::all_minions_mask());
    }
    //
    // Which Angry Birds may come out, as a bit mask over Personality.BirdRoster.
    //
    pub fn bird_mask(mut self) -> i32 {
        return self.bird_mask;
    }
    pub fn set_bird_mask(mut self, mut value: i32) {
        self.bird_mask = (value & Troupe::all_birds_mask());
    }
    pub fn is_minion_enabled(mut self, mut index: i32) -> bool {
        return (((index >= 0) && (index < Settings::MAX_WALKERS))
            && (((self.minion_mask >> (index)) & 1) != 0));
    }
    pub fn set_minion_enabled(mut self, mut index: i32, mut on: bool) {
        if (index < 0) || (index >= Settings::MAX_WALKERS) {
            return;
        }
        if on {
            self.minion_mask |= (1 << (index));
        } else {
            self.minion_mask &= !(1 << (index));
        }
    }
    pub fn set_all_minions(mut self, mut on: bool) {
        self.minion_mask = (if on { Troupe::all_minions_mask() } else { 0 });
    }
    pub fn is_bird_enabled(mut self, mut index: i32) -> bool {
        return (((index >= 0) && (index < Settings::MAX_BIRDS))
            && (((self.bird_mask >> (index)) & 1) != 0));
    }
    pub fn set_bird_enabled(mut self, mut index: i32, mut on: bool) {
        if (index < 0) || (index >= Settings::MAX_BIRDS) {
            return;
        }
        if on {
            self.bird_mask |= (1 << (index));
        } else {
            self.bird_mask &= !(1 << (index));
        }
    }
    pub fn set_all_birds(mut self, mut on: bool) {
        self.bird_mask = (if on { Troupe::all_birds_mask() } else { 0 });
    }
    pub fn all_minions_mask() -> i32 {
        return ((1 << (Settings::MAX_WALKERS)) - 1);
    }
    pub fn all_birds_mask() -> i32 {
        return ((1 << (Settings::MAX_BIRDS)) - 1);
    }
    pub fn first_n_mask(mut n: i32) -> i32 {
        let mut mask: i32 = 0;
        {
            let mut i: i32 = 0;
            'l1: while i < n {
                mask |= (1 << (i));
                i += 1;
            }
        }
        return mask;
    }
    pub fn pop_count(mut mask: i32) -> i32 {
        let mut n: i32 = 0;
        'l1: while mask != 0 {
            n += (mask & 1);
            mask = (mask >> (1));
        }
        return n;
    }
    //
    // Compatibility shim for the headless tools: how many Minions are
    // enabled. Setting it enables the first N in roster order.
    //
    pub fn count(mut self) -> i32 {
        return Troupe::pop_count(self.minion_mask);
    }
    pub fn set_count(mut self, mut value: i32) {
        self.minion_mask =
            Troupe::first_n_mask(rt::max_i32(1, rt::min_i32(Settings::MAX_WALKERS, value)));
    }
    //
    // Compatibility shim for the headless tools: how many birds are
    // enabled. Setting it enables the first N in flock order.
    //
    pub fn bird_count(mut self) -> i32 {
        return Troupe::pop_count(self.bird_mask);
    }
    pub fn set_bird_count(mut self, mut value: i32) {
        self.bird_mask =
            Troupe::first_n_mask(rt::max_i32(1, rt::min_i32(Settings::MAX_BIRDS, value)));
    }
    // Whether the Minions come out. Switching them off sends them away at once; Mr. Bean is not affected.
    pub fn minions_enabled(mut self) -> bool {
        return self.minions_enabled;
    }
    pub fn set_minions_enabled(mut self, mut value: bool) {
        self.minions_enabled = value;
    }
    // Whether Mr. Bean comes out. Switching him off sends him away at once; the Minions are not affected.
    pub fn bean_enabled(mut self) -> bool {
        return self.bean_enabled;
    }
    pub fn set_bean_enabled(mut self, mut value: bool) {
        self.bean_enabled = value;
    }
    // Whether the Angry Birds come out. Switching them off sends them away at once; the rest are not affected.
    pub fn birds_enabled(mut self) -> bool {
        return self.birds_enabled;
    }
    pub fn set_birds_enabled(mut self, mut value: bool) {
        self.birds_enabled = value;
    }
    // Whether the Pink Panther comes out. Switching him off sends him away at once; the rest are not affected.
    pub fn panther_enabled(mut self) -> bool {
        return self.panther_enabled;
    }
    pub fn set_panther_enabled(mut self, mut value: bool) {
        self.panther_enabled = value;
    }
    //
    // The overalls colour, 0xRRGGBB, or Settings.RandomDress for a fresh
    // random colour per Minion per appearance. A change shows at once.
    //
    pub fn dress(mut self) -> i32 {
        return self.dress;
    }
    pub fn set_dress(mut self, mut value: i32) {
        self.dress = value;
        'l1: for mut walker in self.walkers.items() {
            walker.dress = self.pick_dress();
        }
    }
    pub fn pick_dress(mut self) -> i32 {
        return (if (self.dress == Settings::RANDOM_DRESS) {
            Look::random_dress(self.rng)
        } else {
            self.dress
        });
    }
    pub fn any_visible(mut self) -> bool {
        'l1: for mut walker in self.walkers.items() {
            if walker.is_visible() {
                return true;
            }
        }
        return false;
    }
    // Seconds since the last step.
    // Where the cursor is resting.
    // The cursor has been idle long enough and we're not paused.
    // The cursor just moved, or we were paused.
    // Forgets everything every walker learned: taste, friendships, hangouts, walks.
    pub fn forget_brains(mut self) {
        'l1: for mut walker in self.walkers.items() {
            walker.forget_mind();
        }
    }
    // Makes sure every walker exists (without bringing anyone out), so memories have owners.
    pub fn ensure_cast(mut self) {
        'l1: while self.walkers.len() < Troupe::full_cast() {
            let mut i: i32 = self.walkers.len();
            if i <= (Settings::MAX_WALKERS + 1) {
                self.walkers.add(Walker::new(
                    self.world,
                    self.rng,
                    (*(*PERSONALITY_ROSTER).at((i - 2))),
                ));
            } else {
                self.walkers.add(Walker::new(
                    self.world,
                    self.rng,
                    (*(*PERSONALITY_BIRD_ROSTER).at(((i - Settings::MAX_WALKERS) - 2))),
                ));
            }
        }
    }
    pub fn step(mut self, mut dt: f64, mut cursor: Point, mut want_out: bool, mut disturbed: bool) {
        self.ensure_cast();
        if disturbed {
            'l1: for mut walker in self.walkers.items() {
                walker.startle();
            }
        }
        // Bring the cast out one at a time, a beat apart.
        if want_out {
            self.spawn_timer -= dt;
            if self.spawn_timer <= 0.0 {
                let mut next: Walker = self.next_to_appear();
                if !next.is_null() {
                    next.dress = self.pick_dress();
                    next.spawn(cursor, self.pick_spawn_x(cursor));
                    self.spawn_timer = (0.7 + (self.rng.next_double() * 0.7));
                }
            }
        } else {
            self.spawn_timer = 0.0;
        }
        // Disabled Minions go home - all of them if the group is switched
        // off, else just the unticked ones - as do disabled birds, and
        // Mr. Bean and the Panther if off.
        {
            let mut m: i32 = 0;
            'l2: while m < Settings::MAX_WALKERS {
                if !(self.minions_enabled) || (((self.minion_mask >> (m)) & 1) == 0) {
                    (*self.walkers.at((m + 2))).leave();
                }
                m += 1;
            }
        }
        {
            let mut b: i32 = 0;
            'l3: while b < Settings::MAX_BIRDS {
                if !(self.birds_enabled) || (((self.bird_mask >> (b)) & 1) == 0) {
                    (*self.walkers.at(((Settings::MAX_WALKERS + 2) + b))).leave();
                }
                b += 1;
            }
        }
        if !(self.bean_enabled) {
            self.bean.leave();
        }
        if !(self.panther_enabled) {
            self.panther.leave();
        }
        'l4: for mut walker in self.walkers.items() {
            if walker.is_visible() {
                walker.step(dt, cursor);
            }
        }
        self.interact();
        self.spread();
    }
    // The enabled Minions in roster order, then the enabled birds in
    // flock order, then Bean and Panther.
    pub fn next_to_appear(mut self) -> Walker {
        if self.minions_enabled {
            {
                let mut m: i32 = 0;
                'l1: while m < Settings::MAX_WALKERS {
                    if (((self.minion_mask >> (m)) & 1) != 0)
                        && (*self.walkers.at((m + 2))).is_hidden()
                    {
                        return (*self.walkers.at((m + 2)));
                    }
                    m += 1;
                }
            }
        }
        if self.birds_enabled {
            {
                let mut b: i32 = 0;
                'l2: while b < Settings::MAX_BIRDS {
                    if (((self.bird_mask >> (b)) & 1) != 0)
                        && (*self.walkers.at(((Settings::MAX_WALKERS + 2) + b))).is_hidden()
                    {
                        return (*self.walkers.at(((Settings::MAX_WALKERS + 2) + b)));
                    }
                    b += 1;
                }
            }
        }
        if self.bean_enabled && self.bean.is_hidden() {
            return self.bean;
        }
        if self.panther_enabled && self.panther.is_hidden() {
            return self.panther;
        }
        return <Walker>::default();
    }
    // A random spot on the cursor's monitor, preferably clear of the others.
    pub fn pick_spawn_x(mut self, mut cursor: Point) -> f64 {
        let mut bounds: Rectangle = self.world.monitor_bounds_at(cursor);
        let mut margin: f64 = rt::min_f64((((bounds.width) as f64) / 4.0), 120.0);
        let mut best: f64 = (((bounds.left()) as f64) + (((bounds.width) as f64) / 2.0));
        let mut best_gap: f64 = -1.0;
        {
            let mut attempt: i32 = 0;
            'l1: while attempt < 8 {
                let mut x: f64 = ((((bounds.left()) as f64) + margin)
                    + (self.rng.next_double() * (((bounds.width) as f64) - (2.0 * margin))));
                let mut gap: f64 = f64::MAX;
                'l2: for mut other in self.walkers.items() {
                    if other.is_visible() {
                        gap = rt::min_f64(gap, (other.centre_x() - x).abs());
                    }
                }
                if gap >= 200.0 {
                    return x;
                }
                if gap > best_gap {
                    best_gap = gap;
                    best = x;
                }
                attempt += 1;
            }
        }
        return best;
    }
    // ---- Social life ---------------------------------------------------------
    pub fn interact(mut self) {
        // Tell everyone who their nearest neighbour on the same ground is.
        'l1: for mut walker in self.walkers.items() {
            if !(walker.is_visible()) {
                continue 'l1;
            }
            let mut nearest: Walker = <Walker>::default();
            let mut nearest_distance: f64 = f64::MAX;
            'l2: for mut other in self.walkers.items() {
                if ((other == walker) || !(other.is_visible())) || other.is_leaving() {
                    continue 'l2;
                }
                if (other.feet_y() - walker.feet_y()).abs() > 2.0 {
                    continue 'l2;
                }
                let mut distance: f64 = (other.centre_x() - walker.centre_x()).abs();
                if distance < nearest_distance {
                    nearest = other;
                    nearest_distance = distance;
                }
            }
            walker.set_company(nearest);
        }
        {
            let mut i: i32 = 0;
            'l3: while i < self.walkers.len() {
                'c4: {
                    let mut a: Walker = (*self.walkers.at(i));
                    if !(a.is_grounded()) {
                        break 'c4;
                    }
                    // Somebody coming to tag you: run for it.
                    if a.is_charging() && a.is_playful() {
                        let mut quarry: Walker = a.charge_target();
                        if (!quarry.is_null() && quarry.is_grounded()) && !(quarry.is_fleeing()) {
                            quarry.scamper(
                                (if (quarry.centre_x() >= a.centre_x()) {
                                    1
                                } else {
                                    -1
                                }),
                            );
                        }
                    }
                    // A bully bearing down on the timid one: it sees him coming and bolts.
                    if a.is_charging() && !(a.is_playful()) {
                        let mut prey: Walker = a.charge_target();
                        if (((!prey.is_null() && (prey.temperament() == Temperament::Timid))
                            && prey.is_grounded())
                            && !(prey.is_fleeing()))
                            && ((prey.centre_x() - a.centre_x()).abs()
                                < (((300 * a.scale()) / Settings::BASE_PIXEL_SIZE) as f64))
                        {
                            prey.panic(
                                (if (prey.centre_x() >= a.centre_x()) {
                                    1
                                } else {
                                    -1
                                }),
                            );
                        }
                    }
                    {
                        let mut j: i32 = (i + 1);
                        'l5: while j < self.walkers.len() {
                            'c6: {
                                let mut b: Walker = (*self.walkers.at(j));
                                if !(b.is_grounded()) {
                                    break 'c6;
                                }
                                if (a.feet_y() - b.feet_y()).abs() > 2.0 {
                                    break 'c6; // different monitors' ground levels
                                }
                                let mut dx: f64 = (b.centre_x() - a.centre_x());
                                let mut distance: f64 = (dx).abs();
                                let mut towards_b: i32 = (if (dx >= 0.0) { 1 } else { -1 });
                                if self.resolve_contact(a, b, distance, towards_b) {
                                    break 'c6;
                                }
                                self.try_greet(a, b, distance, towards_b);
                                // Side by side, going the same way: walking together bonds.
                                if ((((a.is_walking() && b.is_walking())
                                    && (a.facing() == b.facing()))
                                    && (distance
                                        < (((300 * a.scale()) / Settings::BASE_PIXEL_SIZE)
                                            as f64)))
                                    && a.bond_ready())
                                    && b.bond_ready()
                                {
                                    a.note_walk_together(b);
                                    b.note_walk_together(a);
                                }
                            }
                            j += 1;
                        }
                    }
                    if a.temperament() == Temperament::Aggressive {
                        self.consider_charging(a);
                    }
                }
                i += 1;
            }
        }
    }
    // A charging walker that has reached its target shoves it - or, in a
    // game of tag, tags it. Anyone close enough to see a shove reacts.
    pub fn resolve_contact(
        mut self,
        mut a: Walker,
        mut b: Walker,
        mut distance: f64,
        mut towards_b: i32,
    ) -> bool {
        let mut a_reaches: bool = ((a.is_charging() && (a.charge_target() == b))
            && (distance < (Troupe::CONTACT_DISTANCE_BODIES * a.body_width())));
        let mut b_reaches: bool = ((b.is_charging() && (b.charge_target() == a))
            && (distance < (Troupe::CONTACT_DISTANCE_BODIES * b.body_width())));
        if !(a_reaches) && !(b_reaches) {
            return false;
        }
        if a_reaches && a.is_playful() {
            a.tag(b.centre_x());
            b.tagged(a.centre_x());
            a.note_tag_fun(b);
            b.note_tagged_fun(a);
            return true;
        }
        if b_reaches && b.is_playful() {
            b.tag(a.centre_x());
            a.tagged(b.centre_x());
            b.note_tag_fun(a);
            a.note_tagged_fun(b);
            return true;
        }
        if a_reaches && b_reaches {
            // Two bullies collide and both go flying.
            a.knock(-towards_b);
            b.knock(towards_b);
            self.witness(a, b, ((a.centre_x() + b.centre_x()) / 2.0));
        } else if a_reaches {
            b.knock(towards_b);
            b.note_shove(a);
            a.note_shove_landed(b);
            a.gloat();
            self.witness(a, b, b.centre_x());
        } else {
            a.knock(-towards_b);
            a.note_shove(b);
            b.note_shove_landed(a);
            b.gloat();
            self.witness(a, b, a.centre_x());
        }
        return true;
    }
    // Everyone else on the same ground within sight of x stops to gasp,
    // tut or jeer, as is their nature.
    pub fn witness(mut self, mut a: Walker, mut b: Walker, mut x: f64) {
        'l1: for mut other in self.walkers.items() {
            if ((other == a) || (other == b)) || !(other.is_grounded()) {
                continue 'l1;
            }
            if (other.feet_y() - a.feet_y()).abs() > 2.0 {
                continue 'l1;
            }
            if (other.centre_x() - x).abs()
                > (((620 * other.scale()) / Settings::BASE_PIXEL_SIZE) as f64)
            {
                continue 'l1;
            }
            other.react(x);
        }
    }
    // Two walkers about to pass each other stop to say hello: a friendly
    // one waves, a calm one nods, two friendly ones sometimes break into
    // a dance together, and two aggressive ones sometimes square up for
    // a shouting match instead of just barging past.
    pub fn try_greet(
        mut self,
        mut a: Walker,
        mut b: Walker,
        mut distance: f64,
        mut towards_b: i32,
    ) {
        if distance >= (Troupe::GREET_DISTANCE_BODIES * rt::max_f64(a.body_width(), b.body_width()))
        {
            return;
        }
        if !(a.can_greet()) || !(b.can_greet()) {
            return;
        }
        if (a.greet_cooldown() > 0.0) || (b.greet_cooldown() > 0.0) {
            return;
        }
        let mut approaching: bool = ((a.is_walking() && (a.facing() == towards_b))
            || (b.is_walking() && (b.facing() == -towards_b)));
        if !(approaching) {
            return;
        }
        a.set_greet_cooldown(Troupe::GREET_COOLDOWN_SECONDS);
        b.set_greet_cooldown(Troupe::GREET_COOLDOWN_SECONDS);
        let mut both_aggressive: bool = ((a.temperament() == Temperament::Aggressive)
            && (b.temperament() == Temperament::Aggressive));
        if both_aggressive {
            if self.rng.next_double() < 0.5 {
                a.argue(b);
            }
            return;
        }
        let mut both_friendly: bool = (a.personality().warm() && b.personality().warm());
        let mut dance: bool = (both_friendly && (self.rng.next_double() < 0.4));
        if a.temperament() != Temperament::Aggressive {
            a.greet(towards_b, Troupe::style_for(a, dance));
        }
        if b.temperament() != Temperament::Aggressive {
            b.greet(-towards_b, Troupe::style_for(b, dance));
        }
    }
    pub fn style_for(mut walker: Walker, mut dance: bool) -> GreetStyle {
        if !(walker.personality().warm()) {
            return GreetStyle::Nod;
        }
        return (if dance {
            GreetStyle::Dance
        } else {
            GreetStyle::Wave
        });
    }
    // An aggressive walker with somebody in range goes for them - unless
    // it is busy with an errand of its own, or too worn out.
    pub fn consider_charging(mut self, mut bully: Walker) {
        if ((!(bully.is_walking()) || bully.has_goal()) || (bully.charge_cooldown() > 0.0))
            || (bully.energy() < 0.35)
        {
            return;
        }
        let mut reach: f64 = (((380 * bully.scale()) / Settings::BASE_PIXEL_SIZE) as f64);
        let mut behind: f64 = (((120 * bully.scale()) / Settings::BASE_PIXEL_SIZE) as f64);
        let mut target: Walker = <Walker>::default();
        let mut target_distance: f64 = f64::MAX;
        'l1: for mut other in self.walkers.items() {
            if ((other == bully) || !(other.is_grounded())) || other.is_leaving() {
                continue 'l1;
            }
            if (other.feet_y() - bully.feet_y()).abs() > 2.0 {
                continue 'l1;
            }
            let mut dx: f64 = (other.centre_x() - bully.centre_x());
            let mut distance: f64 = (dx).abs();
            let mut in_front: bool = ((if (dx >= 0.0) { 1 } else { -1 }) == bully.facing());
            if (distance > (if in_front { reach } else { behind })) || (distance >= target_distance)
            {
                continue 'l1;
            }
            target = other;
            target_distance = distance;
        }
        if !target.is_null() {
            bully.start_charge(target);
        }
    }
    // A yawn or a dance that has just begun spreads to whoever is near
    // enough to see it, each according to their nature; so does Mr. Bean
    // bringing Teddy out, or Matilda laying an egg. A bird slingshotting
    // past - or Bomb going off - sends the neighbours diving for cover.
    pub fn spread(mut self) {
        'l1: for mut walker in self.walkers.items() {
            let mut started: Option<Antic> = walker.take_broadcast();
            if started.is_none() || !(walker.is_visible()) {
                continue 'l1;
            }
            let mut toot: bool = (started.unwrap() == Antic::Toot); // that one carries less far, and nobody copies it
            let mut scare: bool =
                ((started.unwrap() == Antic::Launch) || (started.unwrap() == Antic::Blast));
            let mut reach: f64 = ((((if toot {
                260
            } else {
                (if scare { 550 } else { 450 })
            }) * walker.scale())
                / Settings::BASE_PIXEL_SIZE) as f64);
            // A dance, a Teddy show or an egg before a crowd is applause for the performer.
            let mut show: bool = ((!(toot) && !(scare))
                && (((started.unwrap() == Antic::Dance) || (started.unwrap() == Antic::Teddy))
                    || (started.unwrap() == Antic::Egg)));
            let mut audience: i32 = 0;
            'l2: for mut other in self.walkers.items() {
                if ((other == walker) || !(other.is_visible())) || other.is_leaving() {
                    continue 'l2;
                }
                if (other.feet_y() - walker.feet_y()).abs() > 2.0 {
                    continue 'l2;
                }
                let mut gap: f64 = (other.centre_x() - walker.centre_x()).abs();
                if gap > reach {
                    continue 'l2;
                }
                if show {
                    audience += 1;
                }
                if toot {
                    other.smell(walker.centre_x());
                } else if scare {
                    // Duck! Close neighbours dive outright, the rest stop and bristle/gasp.
                    let mut close: f64 = ((((if (started.unwrap() == Antic::Blast) {
                        260
                    } else {
                        200
                    }) * walker.scale())
                        / Settings::BASE_PIXEL_SIZE)
                        as f64);
                    if (gap < close) && other.is_grounded() {
                        other.spook(walker.centre_x());
                    } else {
                        other.react(walker.centre_x());
                    }
                } else {
                    other.catch(started.unwrap());
                }
            }
            if show && (audience > 0) {
                walker.note_audience(rt::min_f64(0.3, (0.1 + (0.05 * ((audience) as f64)))));
            }
        }
    }
}
