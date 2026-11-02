// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

impl Walker {
    // ---- Wants ------------------------------------------------------------------
    // Energy drains with exertion and comes back with rest; the wish for
    // company and the curiosity about the cursor simply grow until they
    // are satisfied.
    pub fn step_mood(mut self, mut dt: f64) {
        // Exertion costs energy; everything else slowly gives it back,
        // a rest more so, a doze most of all.
        let mut drain: f64 = 0.0;
        let mut gain: f64 = 0.02;
        'sw1: {
            match self.mode {
                Mode::Walk => {
                    gain = 0.015;
                }
                Mode::Charge | Mode::Flee => {
                    drain = 0.08;
                    gain = 0.0;
                }
                Mode::Rest => {
                    gain = 0.08;
                }
                Mode::Greet => {
                    if self.greet_style == GreetStyle::Dance {
                        drain = 0.08;
                        gain = 0.0;
                    }
                }
                Mode::Chat => {
                    if self.chat_style == ChatStyle::Argue {
                        drain = 0.06;
                        gain = 0.0;
                    }
                }
                Mode::Antic => {
                    'sw2: {
                        match self.antic {
                            Antic::Dance
                            | Antic::Twirl
                            | Antic::Joy
                            | Antic::Box
                            | Antic::Flex
                            | Antic::Stomp
                            | Antic::Laugh
                            | Antic::Boo
                            | Antic::Trip
                            | Antic::Strum
                            | Antic::Pushups
                            | Antic::Jog
                            | Antic::Flap
                            | Antic::Dig
                            | Antic::Moonwalk
                            | Antic::Jive
                            | Antic::Sneak
                            | Antic::Drive
                            | Antic::Photo
                            | Antic::Peck
                            | Antic::Fly
                            | Antic::Launch
                            | Antic::Rage
                            | Antic::Inflate
                            | Antic::Blast => {
                                drain = 0.08;
                                gain = 0.0;
                            }
                            Antic::Doze => {
                                if self.mode_time > 2.1 {
                                    gain = 0.3;
                                }
                            }
                            Antic::Teddy => {
                                gain = 0.04; // a comfort
                            }
                            Antic::Sleepwalk => {
                                gain = 0.2;
                            }
                            Antic::Daydream => {
                                gain = 0.05;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        // Sleep restores better in the small hours; everything else restores
        // a little worse - the body keeps time even underground.
        let mut deep_rest: bool = ((self.mode == Mode::Antic)
            && ((self.antic == Antic::Doze) || (self.antic == Antic::Sleepwalk)));
        let mut night: f64 = Walker::night_tiredness();
        let mut rest: f64 = (if deep_rest {
            (gain * (1.0 + (0.3 * night)))
        } else {
            (gain * (1.0 - (0.25 * night)))
        });
        self.energy = rt::max_f64(
            0.0,
            rt::min_f64(
                1.0,
                (self.energy + ((rest - (drain / self.personality.stamina)) * dt)),
            ),
        );
        self.social = rt::min_f64(
            1.0,
            (self.social + ((0.015 * self.personality.sociable) * dt)),
        );
        self.curiosity = rt::min_f64(
            1.0,
            (self.curiosity + ((0.012 * self.personality.curious) * dt)),
        );
        self.step_emotion(dt);
    }
    // How much go it has left, 0..1.
    pub fn energy(mut self) -> f64 {
        return self.energy;
    }
    // An idle moment: survival first (the timid one), then play (tag),
    // then a choice of its own - each habit, or setting off to visit,
    // prank or inspect - picked by learned taste with an occasional
    // experiment. The previous choice is judged first, by how its
    // drives moved since and what happened meanwhile (see WalkerMind).
    pub fn find_habit(mut self, mut antic: Antic) -> i32 {
        let mut all: Arr<Habit> = self.personality.habits.clone();
        {
            let mut i: i32 = 0;
            'l1: while i < all.len() {
                if (*all.at(i)).antic == antic {
                    return i;
                }
                i += 1;
            }
        }
        return -1;
    }
    pub fn decide(mut self) {
        let mut other: Walker = self.company;
        let mut same: bool = ((((!other.is_null() && other.is_visible()) && other.is_grounded())
            && !(other.is_leaving()))
            && ((other.feet_y() - self.feet_y).abs() <= 2.0));
        let mut d: f64 = (if same {
            (other.centre_x() - self.cx).abs()
        } else {
            f64::MAX
        });
        let mut tm: Temperament = self.personality.temperament;
        // Always judge the last choice first: even a reflex interrupts
        // life, it should not erase the lesson from what came before.
        let mut state: i32 = self.mind_state();
        self.ensure_table();
        self.settle_previous();
        // The timid one keeps well away from the bullies - and the birds,
        // who are just as scary with a beak: hides if one is close,
        // otherwise turns and walks the other way, glancing back.
        // Hiding is remembered so safety learns its value; a bare turn
        // is a reflex and claims no credit.
        if ((same && (tm == Temperament::Timid))
            && ((other.temperament() == Temperament::Aggressive)
                || (other.temperament() == Temperament::Feisty)))
            && (d < (380.0 * self.size_factor))
        {
            if (d < (160.0 * self.size_factor)) || (self.rng.next_double() < 0.35) {
                let mut hide: i32 = self.find_habit(Antic::Hide);
                if hide >= 0 {
                    self.remember_choice(state, hide);
                }
                self.start_antic(Antic::Hide);
            } else if ((other.centre_x() - self.cx) * ((self.facing) as f64)) > 0.0 {
                self.after_turn = Mode::Walk;
                self.brake(Mode::Turn);
            } else {
                let mut look: i32 = self.find_habit(Antic::Look);
                if look >= 0 {
                    self.remember_choice(state, look);
                }
                self.start_antic(Antic::Look);
            }
            return;
        }
        // Anyone gives a wide berth to someone who hurt them very recently -
        // grudges walk, not just the timid. Bullies never back down.
        if ((same && (tm != Temperament::Aggressive))
            && (self.recent_feeling(other, Walker::EV_HURT, 90.0) > 0.5))
            && (d < (220.0 * self.size_factor))
        {
            if ((other.centre_x() - self.cx) * ((self.facing) as f64)) > 0.0 {
                self.after_turn = Mode::Walk;
                self.brake(Mode::Turn);
            } else {
                let mut look: i32 = self.find_habit(Antic::Look);
                if look >= 0 {
                    self.remember_choice(state, look);
                }
                self.start_antic(Antic::Look);
            }
            return;
        }
        // Two friendly ones with energy to burn play tag. Remembered as
        // a social outing so the fun that follows teaches its value.
        if ((((((same && self.personality.warm()) && other.personality().warm())
            && other.is_walking())
            && !(other.has_goal()))
            && (d < (500.0 * self.size_factor)))
            && (self.energy > 0.55))
            && (self.rng.next_double() < 0.4)
        {
            self.remember_choice(state, self.visit_choice());
            self.start_chase(other);
            return;
        }
        // Choose: every habit is always an option; the errands join in
        // only when they make sense (nearby company, a quiet victim,
        // a cursor worth a look). The last choice was already judged above.
        let mut visit_reach: f64 = (900.0 * self.size_factor);
        if (same && other.is_upset()) && self.personality.warm() {
            visit_reach = (1200.0 * self.size_factor); // hurry to comfort
        }
        if same && (other == self.best_fond()) {
            visit_reach = rt::max_f64(visit_reach, (1200.0 * self.size_factor));
            // best friends cross the desktop
        }
        let mut visit_valid: bool = (((same && (tm != Temperament::Aggressive))
            && (d > (1.6 * self.body_width())))
            && (d < visit_reach));
        let mut prank_valid: bool = (((((same
            && ((((tm == Temperament::Aggressive) || (tm == Temperament::Eccentric))
                || (tm == Temperament::Feisty))
                || (tm == Temperament::Suave)))
            && other.is_quiet())
            && (d > (1.6 * self.body_width())))
            && (d < (450.0 * self.size_factor)))
            && (self.energy > 0.4));
        let mut inspect_valid: bool = ((self.curiosity > 0.6) && self.cursor_on_this_ground());
        let mut inspect_near: bool = (inspect_valid
            && ((((self.cursor_x) as f64) - self.cx).abs() < (60.0 * self.size_factor)));
        let mut habits: Arr<Habit> = self.personality.habits.clone();
        let mut acts: Arr<i32> = Arr::new((habits.len() + 3));
        let mut n: i32 = 0;
        {
            let mut i: i32 = 0;
            'l1: while i < habits.len() {
                (*acts.at({
                    let t__ = n;
                    n += 1;
                    t__
                })) = i;
                i += 1;
            }
        }
        if visit_valid {
            (*acts.at({
                let t__ = n;
                n += 1;
                t__
            })) = self.visit_choice();
        }
        if prank_valid {
            (*acts.at({
                let t__ = n;
                n += 1;
                t__
            })) = self.prank_choice();
        }
        if inspect_valid {
            (*acts.at({
                let t__ = n;
                n += 1;
                t__
            })) = self.inspect_choice();
        }
        if n == 0 {
            return;
        }
        let mut choice: i32 = <i32>::default();
        if self.rng.next_double() < self.explore() {
            // Upgrade v3: motivated experiment, not dice - try what the
            // personality fancies + what is untried, minus what just repeated.
            // Still an experiment (not the best value), but never looks random.
            let mut total: f64 = 0.0;
            let mut w: Arr<f64> = Arr::new(n);
            {
                let mut i: i32 = 0;
                'l2: while i < n {
                    let mut pw: f64 = (((self.prior_for((*acts.at(i)))
                        + self.novelty_bonus((*acts.at(i))))
                        - self.habituation_penalty((*acts.at(i))))
                        + 0.05);
                    if pw < 0.05 {
                        pw = 0.05;
                    }
                    // Scared: scary show-offs are not even tried.
                    if (self.fear > 0.4) && ((*acts.at(i)) < habits.len()) {
                        let mut a: Antic = (*habits.at((*acts.at(i)))).antic;
                        if (((((((((a == Antic::Dance) || (a == Antic::Twirl))
                            || (a == Antic::Strum))
                            || (a == Antic::Jive))
                            || (a == Antic::Flex))
                            || (a == Antic::Box))
                            || (a == Antic::Taunt))
                            || (a == Antic::Launch))
                            || (a == Antic::Blast))
                            || (a == Antic::Rage)
                        {
                            pw *= (1.0 - (0.6 * self.fear));
                        }
                    }
                    (*w.at(i)) = pw;
                    total += pw;
                    i += 1;
                }
            }
            let mut r: f64 = (self.rng.next_double() * total);
            choice = (*acts.at((n - 1)));
            {
                let mut i: i32 = 0;
                'l3: while i < n {
                    r -= (*w.at(i));
                    if r <= 0.0 {
                        choice = (*acts.at(i));
                        break 'l3;
                    }
                    i += 1;
                }
            }
        } else {
            choice = (*acts.at(0));
            let mut best: f64 = self.option_value(state, choice, other);
            {
                let mut i: i32 = 1;
                'l4: while i < n {
                    let mut v: f64 = self.option_value(state, (*acts.at(i)), other);
                    if v > best {
                        best = v;
                        choice = (*acts.at(i));
                    }
                    i += 1;
                }
            }
        }
        self.remember_choice(state, choice);
        if choice < habits.len() {
            self.start_antic((*habits.at(choice)).antic);
        } else if choice == self.visit_choice() {
            self.want_at_visit = self.social;
            self.set_goal(Goal::Visit, other);
        } else if choice == self.prank_choice() {
            self.set_goal(Goal::Prank, other);
        } else if inspect_near {
            self.curiosity = 0.0;
            self.start_antic(self.inspect_antic());
        } else {
            self.set_goal(Goal::Inspect, <Walker>::default());
        }
    }
    // Learned value plus inborn taste, current drives, mood and who it would deal with.
    pub fn option_value(mut self, mut state: i32, mut choice: i32, mut other: Walker) -> f64 {
        let mut eccentric: bool = (self.personality.temperament == Temperament::Eccentric);
        // Upgrade v3: continuous generalizer. Same feat for all choices this Decide,
        // small at birth so newborns act from personality, elders from lived nuance.
        let mut feat_now: Arr<f64> = Arr::new(12);
        self.fill_features(feat_now.clone());
        let mut mlp: f64 = self.mlp_for(feat_now.clone(), choice);
        let mut v: f64 = <f64>::default();
        if choice < self.habit_count() {
            // Utility AI: the body's current needs scale the inborn taste.
            // A tired one means rest; a curious one means look and point.
            let mut h: Habit = (*self.personality.habits.at(choice));
            let mut drive: f64 = (if (h.weight > 0.0) {
                (self.habit_weight(h) / h.weight)
            } else {
                1.0
            });
            v = (((*self.values.at2(state, choice)) + mlp) + (self.prior_for(choice) * drive));
            if eccentric && (h.antic == Antic::Teddy) {
                v += (self.social * 0.4); // lonely: cling to the only friend
            }
            // Mood colours the habits: cheer invites show-offs, gloom invites rest,
            // fear invites hiding and kills showing off.
            if self.cheer > 0.0 {
                'sw1: {
                    match h.antic {
                        Antic::Dance
                        | Antic::Joy
                        | Antic::Strum
                        | Antic::Jive
                        | Antic::Wave
                        | Antic::Laugh
                        | Antic::Twirl
                        | Antic::Fly => {
                            v += (self.cheer * 0.20);
                        }
                        Antic::Rest | Antic::Doze | Antic::Hide | Antic::Fret => {
                            v -= (self.cheer * 0.10);
                        }
                        _ => {}
                    }
                }
            } else if self.cheer < 0.0 {
                'sw2: {
                    match h.antic {
                        Antic::Rest | Antic::Doze | Antic::Teddy | Antic::Daydream => {
                            v += (-self.cheer * 0.12);
                        }
                        Antic::Dance | Antic::Joy | Antic::Jive | Antic::Strum => {
                            v += (self.cheer * 0.15);
                        }
                        _ => {}
                    }
                }
            }
            if self.fear > 0.0 {
                'sw3: {
                    match h.antic {
                        Antic::Hide | Antic::Fret | Antic::Rest | Antic::Look => {
                            v += (self.fear * (if self.tm_is_timid() { 0.6 } else { 0.25 }));
                        }
                        Antic::Dance
                        | Antic::Twirl
                        | Antic::Strum
                        | Antic::Jive
                        | Antic::Flex
                        | Antic::Box
                        | Antic::Taunt
                        | Antic::Launch
                        | Antic::Blast
                        | Antic::Rage => {
                            v -= (self.fear * 0.4);
                        }
                        _ => {}
                    }
                }
            }
            // Curiosity for the rarely-tried, boredom with the repeated (habituation).
            v += (self.novelty_bonus(choice) - self.habituation_penalty(choice));
        } else {
            v = (((*self.values.at2(state, choice)) + mlp) + self.prior_for(choice));
            if (choice == self.visit_choice()) && !other.is_null() {
                v += (self.fond_of(other) * 0.5);
                if other.is_upset() && self.personality.warm() {
                    v += 0.3;
                }
                v += (self.recent_feeling(other, Walker::EV_KIND, 45.0) * 0.5); // fresh warmth pulls harder
                v -= (self.recent_feeling(other, Walker::EV_HURT, 90.0) * 0.8); // fresh hurt repels
                if other == self.best_fond() {
                    v += 0.4; // best friends first
                } else if other == self.rival() {
                    v -= 0.3; // enemies are avoided
                }
                if other.is_upset() && (other == self.best_fond()) {
                    v += 0.3; // a best friend's tears hurry
                }
                // Scared of who it would visit: no errand into dread.
                if (self.fear > 0.3)
                    && ((other.temperament() == Temperament::Aggressive)
                        || (other.temperament() == Temperament::Feisty))
                {
                    v -= (self.fear * 0.8);
                } else if (self.cheer < -0.3) && self.personality.warm() {
                    v += 0.15; // miserable and friendly: seek comfort
                }
                v += (self.novelty_bonus(choice) - self.habituation_penalty(choice));
            } else if (choice == self.prank_choice()) && !other.is_null() {
                // Bullies pick on the disliked; Bean pays back his tormentors.
                v += (if (self.personality.temperament == Temperament::Aggressive) {
                    (-(self.fond_of(other)) * 0.3)
                } else {
                    (if eccentric {
                        (if (self.fond_of(other) < -0.15) {
                            0.35
                        } else {
                            (self.fond_of(other) * 0.05)
                        })
                    } else {
                        (self.fond_of(other) * 0.1)
                    })
                });
                if (self.personality.temperament == Temperament::Aggressive)
                    && (other == self.rival())
                {
                    v += 0.4; // a vendetta: the old enemy gets it
                }
                if self.fear > 0.3 {
                    v -= (self.fear * 0.5);
                }
                if self.cheer > 0.4 {
                    v += (self.cheer * 0.15); // cheeky when happy
                }
                v += (self.novelty_bonus(choice) - self.habituation_penalty(choice));
            } else if choice == self.inspect_choice() {
                if self.fear > 0.4 {
                    v -= (self.fear * 0.4); // dread kills curiosity
                }
                v += (self.novelty_bonus(choice) - self.habituation_penalty(choice));
            }
        }
        return v;
    }
    pub fn tm_is_timid(mut self) -> bool {
        return (self.personality.temperament == Temperament::Timid);
    }
    // Tired walkers rest more and caper less; a curious one looks and points more.
    // And the small hours weigh on everyone: rest calls louder at night,
    // showing off quieter - the cast keeps a circadian rhythm.
    pub fn habit_weight(mut self, mut habit: Habit) -> f64 {
        let mut night: f64 = Walker::night_tiredness();
        'sw1: {
            match habit.antic {
                Antic::Rest | Antic::Yawn | Antic::Doze | Antic::Sleepwalk => {
                    return ((habit.weight * (0.4 + (2.5 * (1.0 - self.energy))))
                        * (1.0 + (0.8 * night)));
                }
                Antic::Dance
                | Antic::Twirl
                | Antic::Joy
                | Antic::Box
                | Antic::Flex
                | Antic::Stomp
                | Antic::Strum
                | Antic::Pushups
                | Antic::Jog
                | Antic::Flap
                | Antic::Moonwalk
                | Antic::Dig
                | Antic::Jive
                | Antic::Drive
                | Antic::Fly
                | Antic::Launch
                | Antic::Inflate
                | Antic::Blast => {
                    return ((habit.weight * (0.25 + (1.2 * self.energy))) * (1.0 - (0.4 * night)));
                }
                Antic::Wave
                | Antic::Point
                | Antic::Look
                | Antic::Peekaboo
                | Antic::Kiss
                | Antic::Hello
                | Antic::Waggle
                | Antic::Photo => {
                    return (habit.weight * (0.5 + self.curiosity));
                }
                _ => {
                    return habit.weight;
                }
            }
        }
        #[allow(unreachable_code)]
        unreachable!()
    }
    // What each one does on reaching the cursor.
    pub fn inspect_antic(mut self) -> Antic {
        return self.personality.cursor_antic;
    }
    pub fn cursor_on_this_ground(mut self) -> bool {
        return ((self.world.ground_at(((self.cursor_x) as f64)) - self.feet_y).abs() <= 2.0);
    }
    // ---- Errands -----------------------------------------------------------------
    pub fn set_goal(mut self, mut next: Goal, mut target: Walker) {
        self.goal = next;
        self.goal_target = target;
        self.goal_time = 0.0;
        if next == Goal::Inspect {
            let mut bounds: Rectangle = self
                .world
                .monitor_bounds_at(Point::new(self.cursor_x, self.cursor_y));
            self.goal_x = ((rt::max_i32(
                (bounds.left() + (12 * self.scale)),
                rt::min_i32((bounds.right() - (12 * self.scale)), self.cursor_x),
            )) as f64);
        }
    }
    // Walking speed while on an errand: creeping up, or hurrying over.
    pub fn goal_pace(mut self) -> f64 {
        'sw1: {
            match self.goal {
                Goal::Prank => {
                    return 0.65;
                }
                Goal::Visit => {
                    return 1.15;
                }
                _ => {
                    return 1.0;
                }
            }
        }
        #[allow(unreachable_code)]
        unreachable!()
    }
    // Each frame of walking while on an errand: give up if it has gone
    // stale, turn round if it is behind, stop on arrival.
    pub fn pursue_goal(mut self, mut dt: f64) {
        self.goal_time += dt;
        if !(self.goal_valid()) {
            self.abandon_goal();
            return;
        }
        let mut gx: f64 = self.goal_x();
        let mut d: f64 = (gx - self.cx);
        let mut close: bool = ((self.goal != Goal::Inspect)
            && ((self.goal_target.centre_x() - self.cx).abs() < (1.5 * self.body_width())));
        if close
            || ((d).abs()
                <= (((self.vx * self.vx) / ((2.0 * Settings::BRAKING) * self.size_factor)) + 2.0))
        {
            self.arrive();
            return;
        }
        if (d * ((self.facing) as f64)) < 0.0 {
            self.after_turn = Mode::Walk;
            self.brake(Mode::Turn);
        }
    }
    pub fn goal_valid(mut self) -> bool {
        if self.goal_time > 12.0 {
            return false;
        }
        if self.goal == Goal::Inspect {
            return self.cursor_on_this_ground();
        }
        let mut t: Walker = self.goal_target;
        return (((!t.is_null() && t.is_visible()) && !(t.is_leaving()))
            && (!(t.is_grounded()) || ((t.feet_y() - self.feet_y).abs() <= 2.0)));
    }
    // Where to walk to: a little short of the other walker, or under the cursor.
    pub fn goal_x(mut self) -> f64 {
        if self.goal == Goal::Inspect {
            return self.goal_x;
        }
        let mut tx: f64 = self.goal_target.centre_x();
        let mut dir: i32 = (if (tx >= self.cx) { 1 } else { -1 });
        return (tx - ((((dir) as f64) * 1.35) * self.body_width()));
    }
    // Nothing came of it: a foiled prankster stamps, a would-be visitor shrugs.
    pub fn abandon_goal(mut self) {
        let mut was: Goal = self.goal;
        self.goal = Goal::None;
        self.goal_target = <Walker>::default();
        if was == Goal::Prank {
            self.pending_bonus -= 0.15;
        } else if was == Goal::Visit {
            // Stood up: stings, and worse the lonelier the walk over was.
            self.pending_bonus -= (0.1
                + (if (self.personality.temperament == Temperament::Eccentric) {
                    (0.3 * (0.5 + self.social))
                } else {
                    0.0
                }));
            if self.pending_bonus < -(Walker::MAX_REWARD) {
                self.pending_bonus = -(Walker::MAX_REWARD);
            }
        }
        if was == Goal::Prank {
            self.start_antic(Antic::Stomp);
        } else if was == Goal::Visit {
            self.start_antic(Antic::Shrug);
        }
    }
    pub fn arrive(mut self) {
        let mut was: Goal = self.goal;
        self.goal = Goal::None;
        'sw1: {
            match was {
                Goal::Visit => {
                    let mut t: Walker = self.goal_target;
                    self.goal_target = <Walker>::default();
                    self.social = 0.1;
                    if t.is_upset() && t.can_greet() {
                        self.begin_chat(t, ChatStyle::Pat);
                    } else if t.can_greet() {
                        self.begin_chat(t, ChatStyle::Chat);
                    } else {
                        self.start_antic(Antic::Shrug); // busy, dozing, mid-twirl...
                    }
                }
                Goal::Prank => {
                    // The target is kept for the shout (or the poke, or the peck) itself.
                    self.antic = (if self.bird {
                        Antic::Peck
                    } else {
                        (if (self.bean || self.panther) {
                            Antic::Poke
                        } else {
                            Antic::Boo
                        })
                    });
                    self.antic_seconds = (if self.bird {
                        2.2
                    } else {
                        (if (self.bean || self.panther) {
                            2.8
                        } else {
                            1.6
                        })
                    });
                    self.antic_stage = 0;
                    self.stop_facing(self.goal_target.centre_x(), Mode::Antic);
                }
                Goal::Inspect => {
                    self.curiosity = 0.0;
                    self.start_antic(self.inspect_antic());
                }
                _ => {}
            }
        }
    }
    // The face while walking on an errand.
    pub fn goal_face(
        mut self,
        target: &mut Expression,
        gx: &mut f64,
        gy: &mut f64,
        at_cursor: &mut bool,
    ) {
        'sw1: {
            match self.goal {
                Goal::Prank => {
                    (*target) = (if self.bird {
                        (*EXPRESSION_DETERMINED)
                    } else {
                        (*EXPRESSION_MISCHIEVOUS)
                    });
                    if !self.goal_target.is_null() {
                        self.look_at(
                            self.goal_target.centre_x(),
                            self.goal_target.eye_y(),
                            &mut *gx,
                            &mut *gy,
                        );
                    }
                }
                Goal::Visit => {
                    (*target) = (if self.personality.warm() {
                        (*EXPRESSION_HAPPY)
                    } else {
                        (*EXPRESSION_CURIOUS)
                    });
                    if !self.goal_target.is_null() {
                        self.look_at(
                            self.goal_target.centre_x(),
                            self.goal_target.eye_y(),
                            &mut *gx,
                            &mut *gy,
                        );
                    }
                }
                _ => {
                    (*target) = (*EXPRESSION_CURIOUS);
                    (*at_cursor) = true;
                }
            }
        }
    }
    // ---- Chats, arguments, comforting ---------------------------------------------
    // Called by another walker that has come over: stop and face it.
    pub fn join_chat(
        mut self,
        mut partner: Walker,
        mut style: ChatStyle,
        mut facing_dir: i32,
        mut seconds: f64,
    ) -> bool {
        if !(self.can_greet()) {
            return false;
        }
        self.chat_partner = partner;
        self.chat_style = style;
        self.chat_lead = false;
        self.chat_facing = facing_dir;
        self.chat_seconds = seconds;
        self.last_stomp = -1;
        self.goal = Goal::None;
        self.goal_target = <Walker>::default();
        self.note_chat_joined(partner, style);
        self.social = 0.1;
        self.stop(Mode::Chat);
        return true;
    }
    // Square up to another aggressive walker for a shouting match.
    pub fn argue(mut self, mut other: Walker) {
        if !(self.can_greet()) {
            return;
        }
        self.begin_chat(other, ChatStyle::Argue);
    }
    pub fn begin_chat(mut self, mut partner: Walker, mut style: ChatStyle) {
        let mut dir: i32 = (if (partner.centre_x() >= self.cx) {
            1
        } else {
            -1
        });
        let mut seconds: f64 = (if (style == ChatStyle::Chat) {
            self.between(2.8, 4.6)
        } else {
            (if (style == ChatStyle::Argue) {
                3.2
            } else {
                2.2
            })
        });
        if !(partner.join_chat(self, style, -dir, seconds)) {
            self.start_antic(Antic::Shrug);
            return;
        }
        self.chat_partner = partner;
        self.chat_style = style;
        self.chat_lead = true;
        self.chat_facing = dir;
        self.chat_seconds = seconds;
        self.note_chat_begun(partner, style);
        self.last_stomp = -1;
        self.set_greet_cooldown(8.0);
        partner.set_greet_cooldown(8.0);
        if style == ChatStyle::Argue {
            self.set_charge_cooldown(8.0);
            partner.set_charge_cooldown(8.0);
        }
        self.stop(Mode::Chat);
    }
    // Whose turn it is: they swap every 0.9 s, the one who came over going first.
    pub fn talking(mut self) -> bool {
        return (((((self.mode_time / 0.9) as i32) % 2) == 0) == self.chat_lead);
    }
    pub fn step_chat(mut self) {
        // The one doing the shouting stamps a foot every so often.
        if (self.chat_style == ChatStyle::Argue) && self.talking() {
            let mut n: i32 = ((self.mode_time / 0.65) as i32);
            if n != self.last_stomp {
                self.last_stomp = n;
                self.slam(0.45);
            }
        }
    }
    pub fn end_chat(mut self) {
        let mut partner: Walker = self.chat_partner;
        self.chat_partner = <Walker>::default();
        let mut px: f64 = (if !partner.is_null() {
            partner.centre_x()
        } else {
            (self.cx + ((self.facing) as f64))
        });
        'sw1: {
            match self.chat_style {
                ChatStyle::Chat => {
                    if self.personality.warm() {
                        self.react_with(Reaction::Chuckle, px, 1.3, false);
                    } else {
                        self.set_mood((*EXPRESSION_HAPPY), 1.5);
                        self.enter(Mode::Walk);
                    }
                }
                ChatStyle::Argue => {
                    self.set_charge_cooldown(6.0);
                    self.react_with(Reaction::Huff, px, 1.1, true);
                }
                _ => {
                    // Comforted (or having comforted): cheered up.
                    self.set_mood((*EXPRESSION_HAPPY), 2.5);
                    self.enter(Mode::Walk);
                }
            }
        }
    }
    pub fn chat_face(
        mut self,
        target: &mut Expression,
        rate: &mut f64,
        gx: &mut f64,
        gy: &mut f64,
    ) {
        let mut t: f64 = self.mode_time;
        let mut friendly: bool = self.personality.warm();
        if !self.chat_partner.is_null() {
            self.look_at(
                self.chat_partner.centre_x(),
                self.chat_partner.eye_y(),
                &mut *gx,
                &mut *gy,
            );
        } else {
            (*gx) = 0.6;
            (*gy) = 0.0;
        }
        'sw1: {
            match self.chat_style {
                ChatStyle::Chat => {
                    if t > (self.chat_seconds - 0.5) {
                        (*target) = (if friendly {
                            (*EXPRESSION_LAUGH)
                        } else {
                            (*EXPRESSION_HAPPY)
                        });
                    } else if self.talking() {
                        (*target) = self.personality.base_face;
                    } else {
                        (*target) = (if friendly {
                            (*EXPRESSION_HAPPY)
                        } else {
                            (*EXPRESSION_CURIOUS)
                        });
                    }
                }
                ChatStyle::Argue => {
                    (*target) = (if self.talking() {
                        (*EXPRESSION_FURIOUS)
                    } else {
                        (*EXPRESSION_GRUMPY)
                    });
                    (*rate) = 25.0;
                }
                _ => {
                    (*target) = (if (t < (if self.chat_lead { 1.0 } else { 1.2 })) {
                        (*EXPRESSION_SAD)
                    } else {
                        (*EXPRESSION_HAPPY)
                    });
                    (*rate) = 10.0;
                }
            }
        }
    }
    pub fn chat_pose(mut self, pose: &mut Pose) {
        'sw1: {
            match self.chat_style {
                ChatStyle::Chat => {
                    if self.talking() {
                        // Gesturing with one hand.
                        (*pose).arm_near = Walker::q2((45.0 + (25.0 * (self.time * 5.5).sin())));
                        (*pose).fore_near = 40.0;
                        self.breathe(&mut *pose);
                    } else {
                        // Nodding along.
                        (*pose).body_dy = Walker::quantise(
                            ((0.3
                                * (1.0
                                    - (((2.0 * std::f64::consts::PI) * self.mode_time) / 0.6)
                                        .cos()))
                                / 2.0),
                        );
                    }
                }
                ChatStyle::Argue => {
                    if self.talking() {
                        // Both arms flailing, leaning in.
                        let mut s: f64 = (self.time * 9.0).sin();
                        (*pose).arm_near = Walker::q2((90.0 + (40.0 * s)));
                        (*pose).arm_far = -(Walker::q2((90.0 - (40.0 * s))));
                        (*pose).fore_near = 20.0;
                        (*pose).fore_far = -20.0;
                        (*pose).lean = 0.4;
                    } else {
                        Walker::hands_on_hips(&mut *pose);
                        (*pose).lean = -0.2;
                    }
                }
                _ => {
                    if self.chat_lead {
                        // A hand reaching for the other's shoulder.
                        (*pose).arm_near = (if !self.chat_partner.is_null() {
                            self.aim_at(
                                self.chat_partner.centre_x(),
                                self.chat_partner.shoulder_y(),
                            )
                        } else {
                            90.0
                        });
                        self.breathe(&mut *pose);
                    } else {
                        // Slumped, being consoled.
                        (*pose).body_dy = Walker::quantise(
                            (0.4 + ((0.15 * (1.0 + (self.time * 2.6).sin())) / 2.0)),
                        );
                        (*pose).lean = 0.15;
                    }
                }
            }
        }
    }
    pub fn chat_yaw(mut self) -> f64 {
        'sw1: {
            match self.chat_style {
                ChatStyle::Argue => {
                    return 25.0;
                }
                ChatStyle::Pat => {
                    return 40.0;
                }
                _ => {
                    return 35.0;
                }
            }
        }
        #[allow(unreachable_code)]
        unreachable!()
    }
    // ---- Pranks ------------------------------------------------------------------
    // Somebody at fromX just shouted: jump, and turn to face them.
    pub fn spook(mut self, mut from_x: f64) {
        if ((!(self.is_grounded()) || (self.mode == Mode::Charge)) || (self.mode == Mode::Flee))
            || (self.mode == Mode::Chat)
        {
            return;
        }
        self.goal = Goal::None;
        self.goal_target = <Walker>::default();
        self.has_pending = false;
        self.facing = (if (from_x >= self.cx) { 1 } else { -1 });
        self.woke_up = true;
        self.spooked = true;
        self.vx = 0.0;
        self.vy = ((-0.6 * Settings::STARTLE_SPEED) * self.size_factor);
        self.emit_dust(self.cx, self.feet_y, 3, 0.6);
        self.enter(Mode::Air);
    }
    // ---- Tag ---------------------------------------------------------------------
    pub fn start_chase(mut self, mut target: Walker) {
        if (self.mode != Mode::Walk) || self.braking {
            return;
        }
        self.charge_target = target;
        self.playful = true;
        self.set_charge_cooldown(8.0);
        self.enter(Mode::Charge);
    }
    // Somebody is coming to tag you: run, laughing.
    pub fn scamper(mut self, mut direction: i32) {
        if !(self.can_greet()) {
            return;
        }
        self.playful = true;
        self.facing = direction;
        self.flee_time = Walker::SCAMPER_SECONDS;
        self.goal = Goal::None;
        self.goal_target = <Walker>::default();
        self.enter(Mode::Flee);
    }
    // Caught them: "you're it!"
    pub fn tag(mut self, mut x: f64) {
        self.playful = false;
        self.charge_target = <Walker>::default();
        self.set_charge_cooldown(8.0);
        self.vx = 0.0;
        self.react_with(Reaction::Jeer, x, 1.6, false);
    }
    // Caught: laugh it off.
    pub fn tagged(mut self, mut x: f64) {
        if !(self.is_grounded()) {
            return;
        }
        self.playful = false;
        self.vx = 0.0;
        self.react_with(Reaction::Chuckle, x, 1.6, false);
    }
    // ---- Things that are catching ----------------------------------------------------
    // An antic that has just begun and that others nearby might catch; null if none.
    pub fn take_broadcast(mut self) -> Option<Antic> {
        let mut b: Option<Antic> = self.broadcast;
        self.broadcast = None;
        return b;
    }
    //
    // Saw somebody start to yawn or dance: maybe do the same, a moment
    // later. The mimic copies most anything it sees. Mr. Bean showing
    // off Teddy draws a wave, a look or a laugh, depending on who sees;
    // Matilda laying an egg draws the same crowd, and one bird's rage
    // sets the other birds off.
    //
    pub fn catch(mut self, mut what: Antic) {
        if (self.catch_cooldown > 0.0) || self.has_pending {
            return;
        }
        let mut chance: f64 = <f64>::default();
        if (what == Antic::Yawn) || (what == Antic::Doze) {
            chance = (if (self.personality.temperament == Temperament::Aggressive) {
                0.3
            } else {
                (if (self.personality.temperament == Temperament::Feisty) {
                    0.5
                } else {
                    0.7
                })
            });
            what = Antic::Yawn;
        } else if what == Antic::Dance {
            'sw1: {
                match self.personality.temperament {
                    Temperament::Friendly | Temperament::Goofy => {
                        chance = 0.85;
                    }
                    Temperament::Eccentric => {
                        chance = 0.8;
                        what = Antic::Jive;
                    }
                    Temperament::Feisty => {
                        chance = 0.6;
                    }
                    Temperament::Calm => {
                        chance = 0.45;
                    }
                    Temperament::Timid => {
                        chance = 0.25;
                    }
                    _ => {
                        chance = 0.0;
                    }
                }
            }
        } else if what == Antic::Teddy {
            'sw2: {
                match self.personality.temperament {
                    Temperament::Friendly | Temperament::Goofy => {
                        chance = 0.7;
                        what = Antic::Wave;
                    }
                    Temperament::Aggressive => {
                        chance = 0.5;
                        what = Antic::Laugh;
                    }
                    Temperament::Feisty => {
                        chance = 0.5;
                        what = Antic::Look;
                    }
                    Temperament::Calm | Temperament::Timid => {
                        chance = 0.5;
                        what = Antic::Look;
                    }
                    _ => {
                        return;
                    }
                }
            }
        } else if what == Antic::Egg {
            // Just like Teddy: everyone comes to look at the egg.
            'sw3: {
                match self.personality.temperament {
                    Temperament::Friendly | Temperament::Goofy => {
                        chance = 0.6;
                        what = Antic::Wave;
                    }
                    Temperament::Aggressive => {
                        chance = 0.4;
                        what = Antic::Laugh;
                    }
                    Temperament::Feisty => {
                        chance = 0.5;
                        what = Antic::Look;
                    }
                    Temperament::Calm | Temperament::Timid => {
                        chance = 0.5;
                        what = Antic::Look;
                    }
                    _ => {
                        return;
                    }
                }
            }
        } else if what == Antic::Rage {
            // Anger is catching, but only among birds.
            if self.personality.temperament != Temperament::Feisty {
                return;
            }
            chance = 0.6;
        } else if (((((((((self.personality.mimic && (what != Antic::Toot))
            && (what != Antic::Drive))
            && (what != Antic::Photo))
            && (what != Antic::Peck))
            && (what != Antic::Fly))
            && (what != Antic::Launch))
            && (what != Antic::Rage))
            && (what != Antic::Egg))
            && (what != Antic::Inflate))
            && (what != Antic::Blast)
        {
            chance = 0.65;
        } else {
            return;
        }
        if self.rng.next_double() >= chance {
            return;
        }
        self.has_pending = true;
        self.pending_antic = what;
        self.pending_in = (if (what == Antic::Yawn) {
            self.between(0.8, 1.6)
        } else {
            (if (what == Antic::Dance) {
                self.between(0.3, 0.7)
            } else {
                self.between(0.5, 1.1)
            })
        });
        self.catch_cooldown = ((if ((what == Antic::Yawn) || (what == Antic::Dance)) {
            12
        } else {
            9
        }) as f64);
    }
    pub fn step_pending(mut self, mut dt: f64) {
        if !(self.has_pending) {
            return;
        }
        self.pending_in -= dt;
        if self.pending_in > 0.0 {
            return;
        }
        self.has_pending = false;
        if self.can_greet() {
            self.start_antic(self.pending_antic);
        }
    }
    // ---- Reactions -------------------------------------------------------------------
    // A short reaction aimed at x (or, for a huff, away from it).
    pub fn react_with(mut self, mut what: Reaction, mut x: f64, mut seconds: f64, mut away: bool) {
        self.reaction = what;
        self.react_x = x;
        self.react_seconds = seconds;
        self.react_facing = ((if (x >= self.cx) { 1 } else { -1 }) * (if away { -1 } else { 1 }));
        self.enter(Mode::React);
    }
}
