// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

impl Walker {
    pub const GAMMA: f64 = 0.9;
    pub const MAX_REWARD: f64 = 1.5;
    // Situations: three levels each of energy, the wish for company and
    // curiosity, times who is near (nobody, somebody harmless, a bully):
    // 81 in all. A finer table learns each cell more slowly, so one
    // lesson is shared with the neighbouring situations (same tired /
    // lonely / curious reading, other levels) at a fraction, the
    // choice before the judged one gets a decaying share (an eligibility
    // trace), and the net replays recent lessons. Together they learn
    // faster than the old eight-way table did, not slower.
    pub const STATE_COUNT: i32 = 81;
    pub const SIBLING_SHARE: f64 = 0.35;
    pub const TRACE_DECAY: f64 = 0.5;
    pub const REPLAY_SIZE: i32 = 24;
    pub const REPLAY_PER_LESSON: i32 = 3;
    // Episodic memory (the session's diary) and life attitude (persisted):
    // a few recent hurts/kindnesses with who and when, and a slow
    // optimism that a good or hard life builds up over days.
    pub const EV_HURT: i32 = 0;
    pub const EV_KIND: i32 = 1;
    // Upgrade v3: tiny MLP-Q generalizer (12 feats -> 12 tanh -> N actions).
    // Hybrid with the table: total = table + mlp. Table learns fast discrete,
    // MLP learns slow continuous so new combos still feel motivated, never random.
    pub const MLP_FEATS: i32 = 12;
    pub const MLP_HIDDEN: i32 = 12;
    pub fn habit_count(mut self) -> i32 {
        return self.personality.habits.len();
    }
    pub fn visit_choice(mut self) -> i32 {
        return self.habit_count();
    }
    pub fn prank_choice(mut self) -> i32 {
        return (self.habit_count() + 1);
    }
    pub fn inspect_choice(mut self) -> i32 {
        return (self.habit_count() + 2);
    }
    pub fn ensure_table(mut self) {
        if self.values.is_null() {
            self.values = Arr2::new(Walker::STATE_COUNT, (self.habit_count() + 3));
        }
        if self.use_count.is_null() || (self.use_count.len() != (self.habit_count() + 3)) {
            let mut n: i32 = (self.habit_count() + 3);
            let mut grown: Arr<i32> = Arr::new(n);
            if !self.use_count.is_null() {
                let mut m: i32 = (if (self.use_count.len() < n) {
                    self.use_count.len()
                } else {
                    n
                });
                {
                    let mut i: i32 = 0;
                    'l1: while i < m {
                        (*grown.at(i)) = (*self.use_count.at(i));
                        i += 1;
                    }
                }
            }
            self.use_count = grown.clone();
        }
        self.ensure_mlp();
    }
    pub fn ensure_mlp(mut self) {
        let mut n: i32 = (self.habit_count() + 3);
        if ((!self.mlp_w1.is_null() && !self.mlp_b1.is_null()) && !self.mlp_w2.is_null())
            && !self.mlp_b2.is_null()
        {
            if (((((self.mlp_w1.get_length(0) == Walker::MLP_FEATS)
                && (self.mlp_w1.get_length(1) == Walker::MLP_HIDDEN))
                && (self.mlp_b1.len() == Walker::MLP_HIDDEN))
                && (self.mlp_w2.get_length(0) == Walker::MLP_HIDDEN))
                && (self.mlp_w2.get_length(1) == n))
                && (self.mlp_b2.len() == n)
            {
                return;
            }
        }
        self.mlp_w1 = Arr2::new(Walker::MLP_FEATS, Walker::MLP_HIDDEN);
        self.mlp_b1 = Arr::new(Walker::MLP_HIDDEN);
        self.mlp_w2 = Arr2::new(Walker::MLP_HIDDEN, n);
        self.mlp_b2 = Arr::new(n);
        {
            let mut f: i32 = 0;
            'l1: while f < Walker::MLP_FEATS {
                {
                    let mut h: i32 = 0;
                    'l2: while h < Walker::MLP_HIDDEN {
                        (*self.mlp_w1.at2(f, h)) =
                            (((self.rng.next_double() + self.rng.next_double()) - 1.0) * 0.05);
                        h += 1;
                    }
                }
                f += 1;
            }
        }
        {
            let mut h: i32 = 0;
            'l3: while h < Walker::MLP_HIDDEN {
                {
                    let mut a: i32 = 0;
                    'l4: while a < n {
                        (*self.mlp_w2.at2(h, a)) =
                            (((self.rng.next_double() + self.rng.next_double()) - 1.0) * 0.05);
                        a += 1;
                    }
                }
                h += 1;
            }
        }
        if self.prev_feat.is_null() || (self.prev_feat.len() != Walker::MLP_FEATS) {
            self.prev_feat = Arr::new(Walker::MLP_FEATS);
        }
    }
    pub fn fill_features(mut self, mut f: Arr<f64>) {
        (*f.at(0)) = self.energy;
        (*f.at(1)) = self.social;
        (*f.at(2)) = self.curiosity;
        (*f.at(3)) = self.cheer;
        (*f.at(4)) = self.fear;
        (*f.at(5)) = Walker::night_tiredness();
        (*f.at(6)) = self.optimism;
        (*f.at(7)) = self.boredom();
        let mut has_company: bool = ((!self.company.is_null() && self.company.is_visible())
            && !(self.company.is_leaving()));
        (*f.at(8)) = ((if has_company { 1 } else { 0 }) as f64);
        let mut bully_near: bool = ((has_company
            && ((self.company.temperament() == Temperament::Aggressive)
                || (self.company.temperament() == Temperament::Feisty)))
            && (self.fear > 0.15));
        (*f.at(9)) = ((if bully_near { 1 } else { 0 }) as f64);
        let mut upset_near: bool = (has_company && self.company.is_upset());
        (*f.at(10)) = ((if upset_near { 1 } else { 0 }) as f64);
        let mut q: f64 = (*self.place.at(self.best_place()));
        if q > 1.0 {
            q = 1.0;
        } else if q < -1.0 {
            q = -1.0;
        }
        (*f.at(11)) = q;
    }
    pub fn mlp_for(mut self, mut feat: Arr<f64>, mut choice: i32) -> f64 {
        self.ensure_mlp();
        let mut n: i32 = (self.habit_count() + 3);
        if (choice < 0) || (choice >= n) {
            return 0.0;
        }
        let mut hid: Arr<f64> = Arr::new(Walker::MLP_HIDDEN);
        {
            let mut h: i32 = 0;
            'l1: while h < Walker::MLP_HIDDEN {
                let mut s: f64 = (*self.mlp_b1.at(h));
                {
                    let mut f: i32 = 0;
                    'l2: while f < Walker::MLP_FEATS {
                        s += ((*feat.at(f)) * (*self.mlp_w1.at2(f, h)));
                        f += 1;
                    }
                }
                // tanh, clamped input for safety
                if s > 8.0 {
                    s = 8.0;
                } else if s < -8.0 {
                    s = -8.0;
                }
                let mut e2: f64 = (2.0 * s).exp();
                (*hid.at(h)) = ((e2 - 1.0) / (e2 + 1.0));
                h += 1;
            }
        }
        let mut q: f64 = (*self.mlp_b2.at(choice));
        {
            let mut h: i32 = 0;
            'l3: while h < Walker::MLP_HIDDEN {
                q += ((*hid.at(h)) * (*self.mlp_w2.at2(h, choice)));
                h += 1;
            }
        }
        if q > Walker::MAX_REWARD {
            q = Walker::MAX_REWARD;
        } else if q < -(Walker::MAX_REWARD) {
            q = -(Walker::MAX_REWARD);
        }
        return q;
    }
    pub fn max_learned(mut self, mut state: i32, mut feat_now: Arr<f64>) -> f64 {
        self.ensure_table();
        let mut n: i32 = (self.habit_count() + 3);
        let mut best: f64 = f64::MIN;
        let mut first: bool = true;
        {
            let mut a: i32 = 0;
            'l1: while a < n {
                let mut q: f64 = ((*self.values.at2(state, a)) + self.mlp_for(feat_now.clone(), a));
                if first || (q > best) {
                    best = q;
                    first = false;
                }
                a += 1;
            }
        }
        return best;
    }
    pub fn mlp_learn(mut self, mut feat: Arr<f64>, mut choice: i32, mut error: f64, mut rate: f64) {
        self.ensure_mlp();
        let mut n: i32 = (self.habit_count() + 3);
        if (choice < 0) || (choice >= n) {
            return;
        }
        if error > Walker::MAX_REWARD {
            error = Walker::MAX_REWARD;
        } else if error < -(Walker::MAX_REWARD) {
            error = -(Walker::MAX_REWARD);
        }
        let mut hid: Arr<f64> = Arr::new(Walker::MLP_HIDDEN);
        {
            let mut h: i32 = 0;
            'l1: while h < Walker::MLP_HIDDEN {
                let mut s: f64 = (*self.mlp_b1.at(h));
                {
                    let mut f: i32 = 0;
                    'l2: while f < Walker::MLP_FEATS {
                        s += ((*feat.at(f)) * (*self.mlp_w1.at2(f, h)));
                        f += 1;
                    }
                }
                if s > 8.0 {
                    s = 8.0;
                } else if s < -8.0 {
                    s = -8.0;
                }
                let mut e2: f64 = (2.0 * s).exp();
                (*hid.at(h)) = ((e2 - 1.0) / (e2 + 1.0));
                h += 1;
            }
        }
        // Output layer: only the taken action moves (Q-learning).
        {
            let mut h: i32 = 0;
            'l3: while h < Walker::MLP_HIDDEN {
                (*self.mlp_w2.at2(h, choice)) += ((rate * error) * (*hid.at(h)));
                h += 1;
            }
        }
        (*self.mlp_b2.at(choice)) += (rate * error);
        // Hidden layer backprop through tanh.
        {
            let mut h: i32 = 0;
            'l4: while h < Walker::MLP_HIDDEN {
                let mut dh: f64 = ((error * (*self.mlp_w2.at2(h, choice)))
                    * (1.0 - ((*hid.at(h)) * (*hid.at(h)))));
                // Clamp the backprop signal so one thrill cannot rewire the net.
                if dh > 1.0 {
                    dh = 1.0;
                } else if dh < -1.0 {
                    dh = -1.0;
                }
                {
                    let mut f: i32 = 0;
                    'l5: while f < Walker::MLP_FEATS {
                        (*self.mlp_w1.at2(f, h)) += ((rate * dh) * (*feat.at(f)));
                        f += 1;
                    }
                }
                (*self.mlp_b1.at(h)) += ((rate * dh) * 0.5);
                h += 1;
            }
        }
        // Gentle decay + clamps: the net generalises, the table remembers.
        {
            let mut f: i32 = 0;
            'l6: while f < Walker::MLP_FEATS {
                {
                    let mut h: i32 = 0;
                    'l7: while h < Walker::MLP_HIDDEN {
                        (*self.mlp_w1.at2(f, h)) *= 0.9995;
                        if (*self.mlp_w1.at2(f, h)) > 2.0 {
                            (*self.mlp_w1.at2(f, h)) = 2.0;
                        } else if (*self.mlp_w1.at2(f, h)) < -2.0 {
                            (*self.mlp_w1.at2(f, h)) = -2.0;
                        }
                        h += 1;
                    }
                }
                f += 1;
            }
        }
        {
            let mut h: i32 = 0;
            'l8: while h < Walker::MLP_HIDDEN {
                if (*self.mlp_b1.at(h)) > 2.0 {
                    (*self.mlp_b1.at(h)) = 2.0;
                } else if (*self.mlp_b1.at(h)) < -2.0 {
                    (*self.mlp_b1.at(h)) = -2.0;
                }
                {
                    let mut a: i32 = 0;
                    'l9: while a < n {
                        (*self.mlp_w2.at2(h, a)) *= 0.9995;
                        if (*self.mlp_w2.at2(h, a)) > 2.0 {
                            (*self.mlp_w2.at2(h, a)) = 2.0;
                        } else if (*self.mlp_w2.at2(h, a)) < -2.0 {
                            (*self.mlp_w2.at2(h, a)) = -2.0;
                        }
                        a += 1;
                    }
                }
                h += 1;
            }
        }
        {
            let mut a: i32 = 0;
            'l10: while a < n {
                if (*self.mlp_b2.at(a)) > 2.0 {
                    (*self.mlp_b2.at(a)) = 2.0;
                } else if (*self.mlp_b2.at(a)) < -2.0 {
                    (*self.mlp_b2.at(a)) = -2.0;
                }
                a += 1;
            }
        }
    }
    //
    // Banks a lesson for the net (features, choice, what the net should
    // have said) and replays a few earlier ones at a gentle rate.
    //
    pub fn replay(
        mut self,
        mut feat: Arr<f64>,
        mut choice: i32,
        mut net_target: f64,
        mut rate: f64,
    ) {
        if self.replay_feat.is_null() || (self.replay_feat.len() != Walker::REPLAY_SIZE) {
            self.replay_feat = Arr::new(Walker::REPLAY_SIZE);
            self.replay_choice = Arr::new(Walker::REPLAY_SIZE);
            self.replay_target = Arr::new(Walker::REPLAY_SIZE);
            self.replay_count = 0;
            self.replay_pos = 0;
        }
        if net_target > Walker::MAX_REWARD {
            net_target = Walker::MAX_REWARD;
        } else if net_target < -(Walker::MAX_REWARD) {
            net_target = -(Walker::MAX_REWARD);
        }
        if (*self.replay_feat.at(self.replay_pos)).is_null() {
            (*self.replay_feat.at(self.replay_pos)) = Arr::new(Walker::MLP_FEATS);
        }
        rt::array_copy(
            &feat,
            0,
            &(*self.replay_feat.at(self.replay_pos)),
            0,
            Walker::MLP_FEATS,
        );
        (*self.replay_choice.at(self.replay_pos)) = choice;
        (*self.replay_target.at(self.replay_pos)) = net_target;
        self.replay_pos = ((self.replay_pos + 1) % Walker::REPLAY_SIZE);
        if self.replay_count < Walker::REPLAY_SIZE {
            self.replay_count += 1;
        }
        let mut n: i32 = (self.habit_count() + 3);
        {
            let mut k: i32 = 0;
            'l1: while (k < Walker::REPLAY_PER_LESSON) && (k < self.replay_count) {
                'c2: {
                    let mut i: i32 = self.rng.next(self.replay_count);
                    let mut c: i32 = (*self.replay_choice.at(i));
                    if (c < 0) || (c >= n) {
                        break 'c2;
                    }
                    let mut error: f64 = ((*self.replay_target.at(i))
                        - self.mlp_for((*self.replay_feat.at(i)).clone(), c));
                    self.mlp_learn((*self.replay_feat.at(i)).clone(), c, error, rate);
                }
                k += 1;
            }
        }
    }
    pub fn mlp_weight_norm(mut self) -> f64 {
        if self.mlp_w1.is_null() || self.mlp_w2.is_null() {
            return 0.0;
        }
        let mut s: f64 = 0.0;
        'l1: for mut v in self.mlp_w1.items() {
            s += (v * v);
        }
        'l2: for mut v in self.mlp_w2.items() {
            s += (v * v);
        }
        return (s).sqrt();
    }
    //
    // The situation in one number (0..80): how tired, how lonely, how
    // curious, each in three steps, and who is near - nobody, somebody
    // harmless, or a bully within sight on the same ground.
    //
    pub fn mind_state(mut self) -> i32 {
        let mut e: i32 = (if (self.energy < 0.45) {
            0
        } else {
            (if (self.energy < 0.75) { 1 } else { 2 })
        });
        let mut s: i32 = (if (self.social < 0.25) {
            0
        } else {
            (if (self.social <= 0.5) { 1 } else { 2 })
        });
        let mut c: i32 = (if (self.curiosity < 0.3) {
            0
        } else {
            (if (self.curiosity <= 0.55) { 1 } else { 2 })
        });
        let mut k: i32 = 0;
        let mut other: Walker = self.company;
        if (((!other.is_null() && other.is_visible()) && !(other.is_leaving()))
            && ((other.feet_y() - self.feet_y).abs() <= 2.0))
            && ((other.centre_x() - self.cx).abs() < (500.0 * self.size_factor))
        {
            k = (if ((other.temperament() == Temperament::Aggressive)
                || (other.temperament() == Temperament::Feisty))
            {
                2
            } else {
                1
            });
        }
        return (((e + (3 * s)) + (9 * c)) + (27 * k));
    }
    //
    // The old eight-way reading of a situation (tired, lonely, curious
    // bits): the family a fine situation belongs to, for sharing
    // lessons and for reading v3 files. The level boundaries were
    // chosen so the old thresholds fall exactly on them.
    //
    pub fn coarse_of(mut state: i32) -> i32 {
        let mut e: i32 = (state % 3);
        let mut s: i32 = ((state / 3) % 3);
        let mut c: i32 = ((state / 9) % 3);
        return (((if (e == 0) { 1 } else { 0 }) | (if (s == 2) { 2 } else { 0 }))
            | (if (c == 2) { 4 } else { 0 }));
    }
    //
    // One lesson for a cell. The other situations in the same family
    // learn a share of it too, so the finer table fills in quickly and
    // a walker that has only ever been tired-and-alone still has a
    // view on tired-with-a-friend-near.
    //
    pub fn teach(mut self, mut state: i32, mut choice: i32, mut delta: f64) {
        (*self.values.at2(state, choice)) += delta;
        let mut family: i32 = Walker::coarse_of(state);
        let mut share: f64 = (delta * Walker::SIBLING_SHARE);
        {
            let mut s: i32 = 0;
            'l1: while s < Walker::STATE_COUNT {
                if (s != state) && (Walker::coarse_of(s) == family) {
                    (*self.values.at2(s, choice)) += share;
                }
                s += 1;
            }
        }
    }
    pub fn max_value(mut self, mut state: i32) -> f64 {
        self.ensure_table();
        let mut best: f64 = (*self.values.at2(state, 0));
        let mut n: i32 = (self.habit_count() + 3);
        {
            let mut a: i32 = 1;
            'l1: while a < n {
                if (*self.values.at2(state, a)) > best {
                    best = (*self.values.at2(state, a));
                }
                a += 1;
            }
        }
        return best;
    }
    // What the personality thinks of a choice before any experience: the inborn taste.
    pub fn prior_for(mut self, mut choice: i32) -> f64 {
        // Errands must compete with the best habits from the start, or
        // the eager ones (bullies prowling, friends visiting, the
        // curious snooping) would starve before learning anything.
        if choice < self.habit_count() {
            return ((*self.personality.habits.at(choice)).weight * 0.15);
        }
        if choice == self.visit_choice() {
            return (0.15 + (0.45 * self.personality.sociable));
        }
        if choice == self.prank_choice() {
            return (if (self.personality.temperament == Temperament::Aggressive) {
                0.8
            } else {
                0.55
            });
        }
        return (0.15 + (0.45 * self.personality.curious));
    }
    pub fn fond_of(mut self, mut other: Walker) -> f64 {
        let mut v: f64 = <f64>::default();
        return (if (!other.is_null() && self.fondness.try_get_value(&other, &mut v)) {
            v
        } else {
            0.0
        });
    }
    pub fn like(mut self, mut other: Walker, mut amount: f64) {
        if other.is_null() || (other == self) {
            return;
        }
        let mut v: f64 = (self.fond_of(other) + amount);
        if v > 1.0 {
            v = 1.0;
        } else if v < -1.0 {
            v = -1.0;
        }
        self.fondness.set(other, v);
        if self.fondness.len() > 40 {
            self.prune_memory();
        }
    }
    pub fn prune_memory(mut self) {
        let mut keys: Arr<Walker> = Arr::new(self.fondness.len());
        rt::array_copy(
            &self.fondness.keys(),
            0,
            &keys,
            0,
            self.fondness.keys().len(),
        );
        'l1: for mut key in keys.items() {
            let mut v: f64 = <f64>::default();
            if self.fondness.try_get_value(&key, &mut v) && ((v).abs() < 0.05) {
                self.fondness.remove(&key);
            }
        }
    }
    pub fn decay_memory(mut self) {
        if self.fondness.len() == 0 {
            return;
        }
        let mut keys: Arr<Walker> = Arr::new(self.fondness.len());
        rt::array_copy(
            &self.fondness.keys(),
            0,
            &keys,
            0,
            self.fondness.keys().len(),
        );
        'l1: for mut key in keys.items() {
            let mut v: f64 = <f64>::default();
            if !(self.fondness.try_get_value(&key, &mut v)) {
                continue 'l1;
            }
            v *= 0.999;
            if (v).abs() < 0.02 {
                self.fondness.remove(&key);
            } else {
                self.fondness.set(key, v);
            }
        }
    }
    // Coming out again: a fresh situation to learn from, but the taste and memories stay.
    pub fn reset_mind(mut self) {
        self.has_prev = false;
        self.older_state = -1;
        self.pending_bonus = 0.0;
        self.prev_social = self.social;
        self.prev_curiosity = self.curiosity;
        self.prev_energy = self.energy;
        self.prev_cheer = self.cheer;
        self.prev_fear = self.fear;
        self.ensure_mlp();
        self.fill_features(self.prev_feat.clone());
    }
    //
    // Settles the previous choice before picking the next: the reward
    // is how the drives moved since (company kept, curiosity met,
    // energy well spent, fear relieved, cheer gained) plus whatever
    // events banked meanwhile. Weighted per temperament so the same
    // events teach a friendly one and a bully different lessons.
    //
    pub fn settle_previous(mut self) {
        if !(self.has_prev) {
            self.pending_bonus = 0.0;
            return;
        }
        self.has_prev = false;
        self.ensure_table();
        let mut w_social: f64 = (0.4 + (0.6 * self.personality.sociable));
        let mut w_curious: f64 = (0.4 + (0.6 * self.personality.curious));
        let mut w_energy: f64 = (if (self.personality.temperament == Temperament::Aggressive) {
            0.3
        } else {
            0.5
        });
        if self.personality.temperament == Temperament::Calm {
            w_energy = 0.65;
        }
        let mut reward: f64 = ((((((w_social * (self.prev_social - self.social))
            + (w_curious * (self.prev_curiosity - self.curiosity)))
            + (w_energy * (self.energy - self.prev_energy)))
            + (self.fear_weight() * (self.prev_fear - self.fear)))
            + (0.35 * (self.cheer - self.prev_cheer)))
            + self.pending_bonus);
        self.pending_bonus = 0.0;
        if reward > Walker::MAX_REWARD {
            reward = Walker::MAX_REWARD;
        } else if reward < -(Walker::MAX_REWARD) {
            reward = -(Walker::MAX_REWARD);
        }
        let mut rate: f64 = self.learn_rate_for();
        // Upgrade v3: hybrid TD - total = table + MLP. Bootstrap from total,
        // update table fast + MLP slow so behaviour stays smooth while generalising.
        let mut next_state: i32 = self.mind_state();
        let mut feat_now: Arr<f64> = Arr::new(Walker::MLP_FEATS);
        self.fill_features(feat_now.clone());
        let mut prev_mlp: f64 = self.mlp_for(self.prev_feat.clone(), self.prev_choice);
        let mut prev_total: f64 =
            ((*self.values.at2(self.prev_state, self.prev_choice)) + prev_mlp);
        let mut target: f64 =
            (reward + (self.mind_gamma() * self.max_learned(next_state, feat_now.clone())));
        let mut error: f64 = (target - prev_total);
        if error > (Walker::MAX_REWARD * 2.0) {
            error = (Walker::MAX_REWARD * 2.0);
        } else if error < (-(Walker::MAX_REWARD) * 2.0) {
            error = (-(Walker::MAX_REWARD) * 2.0);
        }
        self.teach(self.prev_state, self.prev_choice, (rate * error));
        self.mlp_learn(
            self.prev_feat.clone(),
            self.prev_choice,
            error,
            (rate * 0.4),
        );
        // The trace: the choice before this one set the scene for it, so
        // it earns a decaying share of the same lesson.
        if (self.older_state >= 0) && !self.older_feat.is_null() {
            self.teach(
                self.older_state,
                self.older_choice,
                ((rate * error) * Walker::TRACE_DECAY),
            );
            self.mlp_learn(
                self.older_feat.clone(),
                self.older_choice,
                (error * Walker::TRACE_DECAY),
                (rate * 0.4),
            );
        }
        // Replay: the net goes over a few recent lessons again, which is
        // what lets a dozen weights settle on a trend after a handful of
        // real events instead of hundreds.
        self.replay(
            self.prev_feat.clone(),
            self.prev_choice,
            (target - (*self.values.at2(self.prev_state, self.prev_choice))),
            (rate * 0.2),
        );
        self.older_state = self.prev_state;
        self.older_choice = self.prev_choice;
        if self.older_feat.is_null() || (self.older_feat.len() != Walker::MLP_FEATS) {
            self.older_feat = Arr::new(Walker::MLP_FEATS);
        }
        rt::array_copy(&self.prev_feat, 0, &self.older_feat, 0, Walker::MLP_FEATS);
        // Upgrade v2: track lifetime stats for annealing + dashboard.
        self.lifetime_decisions += 1;
        self.total_reward += reward;
        if self.lifetime_decisions < 0 {
            self.lifetime_decisions = i32::MAX;
        }
        if self.total_reward > 1000000.0 {
            self.total_reward = 1000000.0;
        } else if self.total_reward < -1000000.0 {
            self.total_reward = -1000000.0;
        }
        // Upgrade v2: slow forgetting so stale tastes fade and values cannot saturate.
        // Global decay is tiny per lesson; the touched cell was just reinforced above.
        {
            let mut r: i32 = 0;
            'l1: while r < self.values.get_length(0) {
                {
                    let mut c: i32 = 0;
                    'l2: while c < self.values.get_length(1) {
                        (*self.values.at2(r, c)) *= 0.9995;
                        c += 1;
                    }
                }
                r += 1;
            }
        }
        // The same lesson stains the map where it happened: hangouts form.
        let mut bin: i32 = self.place_index();
        (*self.place.at(bin)) += (rate * (reward - (*self.place.at(bin))));
        {
            let mut i: i32 = 0;
            'l3: while i < self.place.len() {
                (*self.place.at(i)) *= 0.999;
                i += 1;
            }
        }
        self.decay_memory();
    }
    // Remembers a choice just made, so the next think can judge it.
    pub fn remember_choice(mut self, mut state: i32, mut choice: i32) {
        self.ensure_table();
        self.has_prev = true;
        self.prev_state = state;
        self.prev_choice = choice;
        self.prev_social = self.social;
        self.prev_curiosity = self.curiosity;
        self.prev_energy = self.energy;
        self.prev_cheer = self.cheer;
        self.prev_fear = self.fear;
        self.fill_features(self.prev_feat.clone());
        if (choice >= 0) && (choice < self.use_count.len()) {
            (*self.use_count.at(choice)) += 1;
        }
        if choice == self.last_choice {
            self.repeat_count += 1;
        } else {
            self.last_choice = choice;
            self.repeat_count = 0;
        }
    }
    pub fn add_bonus(mut self, mut amount: f64) {
        self.pending_bonus += amount;
        if self.pending_bonus > Walker::MAX_REWARD {
            self.pending_bonus = Walker::MAX_REWARD;
        } else if self.pending_bonus < -(Walker::MAX_REWARD) {
            self.pending_bonus = -(Walker::MAX_REWARD);
        }
        self.trial_bonus += amount; // the body trial judges by the same good news
        self.cheer += (amount * 0.25);
        if self.cheer > 1.0 {
            self.cheer = 1.0;
        } else if self.cheer < -1.0 {
            self.cheer = -1.0;
        }
        if amount < -0.2 {
            self.fear += (-amount * 0.2);
        }
        if self.fear > 1.0 {
            self.fear = 1.0;
        }
        self.optimism += (amount * 0.02); // a good or hard life shifts the attitude, very slowly
        if self.optimism > 1.0 {
            self.optimism = 1.0;
        } else if self.optimism < -1.0 {
            self.optimism = -1.0;
        }
    }
    // How far ahead it connects acts to consequences: a child lives in the moment.
    pub fn mind_gamma(mut self) -> f64 {
        'sw1: {
            match self.personality.temperament {
                Temperament::Calm => {
                    return 0.92;
                }
                Temperament::Friendly => {
                    return 0.88;
                }
                Temperament::Aggressive => {
                    return 0.82;
                }
                Temperament::Timid => {
                    return 0.75;
                }
                Temperament::Goofy => {
                    return 0.85;
                }
                Temperament::Eccentric => {
                    return 0.7;
                }
                Temperament::Feisty => {
                    return 0.8;
                }
                Temperament::Suave => {
                    return 0.87;
                }
                _ => {
                    return Walker::GAMMA;
                }
            }
        }
        #[allow(unreachable_code)]
        unreachable!()
    }
    // How fast this individual rewires after one lesson: the nervous learn fast, the calm slow.
    pub fn learn_rate_for(mut self) -> f64 {
        let mut base_rate: f64 = <f64>::default();
        'sw1: {
            match self.personality.temperament {
                Temperament::Calm => {
                    base_rate = 0.08;
                }
                Temperament::Friendly => {
                    base_rate = 0.12;
                }
                Temperament::Aggressive => {
                    base_rate = 0.10;
                }
                Temperament::Timid => {
                    base_rate = 0.18;
                }
                Temperament::Goofy => {
                    base_rate = 0.14;
                }
                Temperament::Eccentric => {
                    base_rate = 0.16;
                }
                Temperament::Feisty => {
                    base_rate = 0.11;
                }
                Temperament::Suave => {
                    base_rate = 0.13;
                }
                _ => {
                    base_rate = Settings::LEARN_RATE;
                }
            }
        }
        if self.personality.name == "Phil".to_string() {
            base_rate += 0.02;
        } else if self.personality.name == "Carl".to_string() {
            base_rate += 0.03;
        } else if self.personality.name == "Kevin".to_string() {
            base_rate -= 0.02;
        } else if self.personality.name == "Blues".to_string() {
            base_rate += 0.02;
        } else if self.personality.name == "Terence".to_string() {
            base_rate -= 0.03;
        }
        // Quicker on the uptake since the finer table (v4): a quarter
        // more per lesson, and the ageing is slower, so an elder still
        // moves at a third of a newborn's pace after a thousand choices.
        base_rate *= 1.25;
        if self.lifetime_decisions > 0 {
            base_rate *= (1.0 / (1.0 + (((self.lifetime_decisions) as f64) / 1500.0)));
        }
        if base_rate < 0.04 {
            base_rate = 0.04;
        } else if base_rate > 0.3 {
            base_rate = 0.3;
        }
        return base_rate;
    }
    // How often it tries something unexpected: clowns and children experiment, the timid do not.
    pub fn explore(mut self) -> f64 {
        let mut base_rate: f64 = <f64>::default();
        'sw1: {
            match self.personality.temperament {
                Temperament::Calm => {
                    base_rate = 0.08;
                }
                Temperament::Friendly => {
                    base_rate = 0.14;
                }
                Temperament::Aggressive => {
                    base_rate = 0.10;
                }
                Temperament::Timid => {
                    base_rate = 0.06;
                }
                Temperament::Goofy => {
                    base_rate = 0.24;
                }
                Temperament::Eccentric => {
                    base_rate = 0.22;
                }
                Temperament::Feisty => {
                    base_rate = 0.16;
                }
                Temperament::Suave => {
                    base_rate = 0.18;
                }
                _ => {
                    base_rate = Settings::EXPLORE_RATE;
                }
            }
        }
        if self.personality.name == "Blues".to_string() {
            base_rate += 0.04;
        } else if self.personality.name == "Chuck".to_string() {
            base_rate += 0.02;
        } else if self.personality.name == "Terence".to_string() {
            base_rate -= 0.05;
        } else if self.personality.name == "Tim".to_string() {
            base_rate -= 0.03;
        }
        // Boredom and curiosity drive novelty-seeking (curiosity-driven exploration).
        base_rate += (((0.10 * self.curiosity) + (0.05 * self.boredom())) + (0.03 * self.optimism));
        // Upgrade v2: anneal exploration with age so elders mostly exploit.
        if self.lifetime_decisions > 0 {
            base_rate *= (1.0 / (1.0 + (((self.lifetime_decisions) as f64) / 400.0)));
        }
        if self.fear > 0.5 {
            base_rate *= (1.0 - (0.6 * self.fear)); // scared: no experiments
        }
        if base_rate < 0.02 {
            base_rate = 0.02;
        } else if base_rate > 0.35 {
            base_rate = 0.35;
        }
        return base_rate;
    }
    pub fn fear_weight(mut self) -> f64 {
        'sw1: {
            match self.personality.temperament {
                Temperament::Timid => {
                    return 1.2;
                }
                Temperament::Friendly => {
                    return 0.5;
                }
                Temperament::Calm => {
                    return 0.4;
                }
                Temperament::Goofy => {
                    return 0.3;
                }
                Temperament::Eccentric => {
                    return 0.4;
                }
                Temperament::Feisty => {
                    return 0.2;
                }
                Temperament::Suave => {
                    return 0.3;
                }
                _ => {
                    return 0.1;
                }
            }
        }
        #[allow(unreachable_code)]
        unreachable!()
    }
    // 0..1: how stale life feels - repeats of the same act in a row (hedonic treadmill).
    pub fn boredom(mut self) -> f64 {
        if self.repeat_count <= 0 {
            return 0.0;
        }
        let mut b: f64 = (((self.repeat_count) as f64) * 0.25);
        return (if (b > 1.0) { 1.0 } else { b });
    }
    // Small intrinsic bonus for rarely-tried acts: curiosity for the unknown.
    pub fn novelty_bonus(mut self, mut choice: i32) -> f64 {
        if (self.use_count.is_null() || (choice < 0)) || (choice >= self.use_count.len()) {
            return 0.0;
        }
        let mut n: i32 = (*self.use_count.at(choice));
        return ((0.20 * self.personality.curious) / (1.0 + (((n) as f64) * 0.3)));
    }
    // Penalty for doing the same thing over and over: habituation.
    pub fn habituation_penalty(mut self, mut choice: i32) -> f64 {
        if (choice != self.last_choice) || (self.repeat_count <= 0) {
            return 0.0;
        }
        let mut p: f64 = (((self.repeat_count) as f64) * 0.18);
        return (if (p > 0.6) { 0.6 } else { p });
    }
    // One frame of feeling: cheer settles towards the life-attitude, fear tracks nearby threats.
    pub fn step_emotion(mut self, mut dt: f64) {
        let mut base_cheer: f64 = (self.optimism * 0.3);
        if base_cheer < -0.3 {
            base_cheer = -0.3;
        } else if base_cheer > 0.3 {
            base_cheer = 0.3;
        }
        if self.cheer > base_cheer {
            self.cheer = rt::max_f64(base_cheer, (self.cheer - (dt * 0.05)));
        } else if self.cheer < base_cheer {
            self.cheer = rt::min_f64(base_cheer, (self.cheer + (dt * 0.05)));
        }
        let mut threat: f64 = 0.0;
        let mut other: Walker = self.company;
        if ((!other.is_null() && other.is_visible()) && !(other.is_leaving()))
            && ((other.temperament() == Temperament::Aggressive)
                || (other.temperament() == Temperament::Feisty))
        {
            let mut d: f64 = (other.centre_x() - self.cx).abs();
            let mut reach: f64 = (380.0 * self.size_factor);
            if (d < reach) && ((other.feet_y() - self.feet_y).abs() <= 2.0) {
                threat = (1.0 - (d / reach));
            }
        }
        let mut feel: f64 = 1.0;
        if self.personality.temperament == Temperament::Timid {
            feel = 2.2;
        } else if self.personality.temperament == Temperament::Friendly {
            feel = 1.0;
        } else if self.personality.temperament == Temperament::Calm {
            feel = 0.7;
        } else if self.personality.temperament == Temperament::Aggressive {
            feel = 0.25;
        } else if self.personality.temperament == Temperament::Goofy {
            feel = 0.6;
        } else if self.personality.temperament == Temperament::Feisty {
            feel = 0.4;
        } else if self.personality.temperament == Temperament::Suave {
            feel = 0.5;
        } else if self.personality.temperament == Temperament::Eccentric {
            feel = 0.8;
        }
        let mut rate: f64 = (if (threat > self.fear) {
            (2.5 * feel)
        } else {
            0.8
        });
        self.fear += ((threat - self.fear) * rt::min_f64(1.0, (dt * rate)));
        if self.fear < 0.0 {
            self.fear = 0.0;
        } else if self.fear > 1.0 {
            self.fear = 1.0;
        }
    }
    // ---- Episodic memory, reflection, rhythm ----------------------------------
    //
    // The diary (who hurt / was kind to me lately) fades within a minute
    // or two; the reflections distilled from it - best friend, rival,
    // optimism - live on. Same idea as the memory-stream architectures,
    // but a diary of six entries instead of a language model.
    // Writes one line in the diary.
    pub fn remember_event(mut self, mut kind: i32, mut who: Walker) {
        if who.is_null() || (who == self) {
            return;
        }
        let mut e: MemoryEvent = MemoryEvent::default();
        e.kind = kind;
        e.who = who;
        e.at = self.time;
        (*self.log.at((self.log_pos % self.log.len()))) = e;
        self.log_pos += 1;
    }
    // How strongly kind/hurt events with someone still sting or warm (0..1).
    pub fn recent_feeling(mut self, mut who: Walker, mut kind: i32, mut window: f64) -> f64 {
        if who.is_null() {
            return 0.0;
        }
        let mut sum: f64 = 0.0;
        {
            let mut i: i32 = 0;
            'l1: while i < self.log.len() {
                'c2: {
                    if ((*self.log.at(i)).who != who) || ((*self.log.at(i)).kind != kind) {
                        break 'c2;
                    }
                    let mut age: f64 = (self.time - (*self.log.at(i)).at);
                    if age < 0.0 {
                        age = 0.0;
                    }
                    if age < window {
                        sum += (1.0 - (age / window));
                    }
                }
                i += 1;
            }
        }
        if sum > 1.0 {
            sum = 1.0;
        }
        return sum;
    }
    // The fondest remembered walker past affection, if any: the best friend.
    pub fn best_fond(mut self) -> Walker {
        let mut best: Walker = <Walker>::default();
        let mut bv: f64 = 0.35;
        'l1: for mut kv in self.fondness.items() {
            if kv.key.is_null() || (kv.key == self) {
                continue 'l1;
            }
            if kv.value > bv {
                bv = kv.value;
                best = kv.key;
            }
        }
        return best;
    }
    // The most resented remembered walker past dislike, if any: the rival.
    pub fn rival(mut self) -> Walker {
        let mut worst: Walker = <Walker>::default();
        let mut wv: f64 = -0.35;
        'l1: for mut kv in self.fondness.items() {
            if kv.key.is_null() || (kv.key == self) {
                continue 'l1;
            }
            if kv.value < wv {
                wv = kv.value;
                worst = kv.key;
            }
        }
        return worst;
    }
    pub fn best_friend_name(mut self) -> String {
        let mut best: Walker = self.best_fond();
        return (if best.is_null() {
            "-".to_string()
        } else {
            best.personality.name.clone()
        });
    }
    pub fn rival_name(mut self) -> String {
        let mut worst: Walker = self.rival();
        return (if worst.is_null() {
            "-".to_string()
        } else {
            worst.personality.name.clone()
        });
    }
    pub fn local_hour() -> i32 {
        let mut now: i64 = rt::DateTime::utc_now().ticks();
        if (((*WALKER_CACHED_HOUR.at()) < 0) || (now >= (*WALKER_HOUR_ENDS_UTC.at())))
            || (now < ((*WALKER_HOUR_ENDS_UTC.at()) - 36_000_000_000i64))
        {
            let mut local: DateTime = rt::DateTime::now();
            (*WALKER_CACHED_HOUR.at()) = local.hour();
            (*WALKER_HOUR_ENDS_UTC.at()) =
                (now + (36_000_000_000i64 - (local.ticks() % 36_000_000_000i64)));
        }
        return (*WALKER_CACHED_HOUR.at());
    }
    // 0 by day, up to 1 in the deep night: the world has a rhythm too.
    pub fn night_tiredness() -> f64 {
        let mut hour: i32 = <i32>::default();
        if !rt::attempt(|| {
            hour = Walker::local_hour();
        }) {
            return 0.0;
        }
        if (hour >= 7) && (hour < 21) {
            return 0.0;
        }
        if hour == 21 {
            return 0.25;
        }
        if (hour == 22) || (hour == 6) {
            return 0.5;
        }
        return (if (hour == 23) { 0.75 } else { 1.0 });
    }
    // ---- What the world teaches it ------------------------------------------
    // A chat began with partner (as the one who came over).
    pub fn note_chat_begun(mut self, mut partner: Walker, mut style: ChatStyle) {
        if style == ChatStyle::Argue {
            self.like(partner, -0.1);
            if self.personality.temperament == Temperament::Aggressive {
                self.add_bonus(0.15);
            }
            return;
        }
        // Hunger: contact after a long lonely trek teaches more.
        let mut bonus: f64 = (if (style == ChatStyle::Pat) { 0.6 } else { 0.5 });
        if (style == ChatStyle::Pat) && (self.personality.temperament == Temperament::Eccentric) {
            bonus += 0.2;
        }
        self.like(
            partner,
            (if (style == ChatStyle::Pat) { 0.2 } else { 0.15 }),
        );
        self.remember_event(Walker::EV_KIND, partner);
        self.add_bonus((bonus * (0.5 + self.want_at_visit)));
    }
    // Someone came over and a chat with them began (called before the relief).
    pub fn note_chat_joined(mut self, mut partner: Walker, mut style: ChatStyle) {
        if style == ChatStyle::Argue {
            self.like(partner, -0.1);
            if self.personality.temperament == Temperament::Aggressive {
                self.add_bonus(0.15);
            }
            return;
        }
        let mut bonus: f64 = (if (style == ChatStyle::Pat) { 0.6 } else { 0.5 });
        if (style == ChatStyle::Pat) && (self.personality.temperament == Temperament::Eccentric) {
            bonus += 0.2;
        }
        self.like(
            partner,
            (if (style == ChatStyle::Pat) { 0.2 } else { 0.15 }),
        );
        self.remember_event(Walker::EV_KIND, partner);
        self.add_bonus((bonus * (0.5 + self.social)));
    }
    // Stopped to greet: a dance together delights, a nod pleases.
    pub fn note_greet(mut self, mut style: GreetStyle) {
        self.add_bonus(
            (if (style == GreetStyle::Dance) {
                0.3
            } else {
                0.15
            }),
        );
    }
    // The prank landed: the victim jumped. Bean revels in his schemes.
    pub fn note_prank_landed(mut self, mut target: Walker) {
        self.like(target, -0.1);
        self.add_bonus(
            (if (self.personality.temperament == Temperament::Eccentric) {
                0.65
            } else {
                0.5
            }),
        );
    }
    // Pranked by someone: remembers who.
    pub fn note_pranked(mut self, mut by: Walker) {
        self.like(by, -0.25);
        self.remember_event(Walker::EV_HURT, by);
        self.add_bonus(-0.25);
    }
    // Shoved by someone: remembers who, and it stings.
    pub fn note_shove(mut self, mut by: Walker) {
        self.like(by, -0.35);
        self.remember_event(Walker::EV_HURT, by);
        self.add_bonus(-0.4);
    }
    // Shoved someone: bullies relish it.
    pub fn note_shove_landed(mut self, mut victim: Walker) {
        self.like(victim, -0.1);
        self.add_bonus(0.3);
    }
    // Tagged someone in play: good fun, and friends grow fonder.
    pub fn note_tag_fun(mut self, mut other: Walker) {
        self.like(other, 0.1);
        self.remember_event(Walker::EV_KIND, other);
        self.add_bonus(0.25);
    }
    // Tagged in play: no hard feelings, rather the opposite.
    pub fn note_tagged_fun(mut self, mut other: Walker) {
        self.like(other, 0.1);
        self.remember_event(Walker::EV_KIND, other);
        self.add_bonus(0.2);
    }
    // Danced, showed off Teddy, laid an egg before a crowd: applause.
    pub fn note_audience(mut self, mut amount: f64) {
        self.add_bonus(amount);
    }
    // Covering ground side by side with someone: walking together bonds.
    pub fn note_walk_together(mut self, mut other: Walker) {
        self.like(other, 0.03);
        self.add_bonus(0.03);
        self.bond_cooldown = 2.0;
    }
    // True when it is ready to bond over another shared stretch.
    pub fn bond_ready(mut self) -> bool {
        return (self.bond_cooldown <= 0.0);
    }
    // Which stretch of the desktop has treated it best.
    pub fn best_place(mut self) -> i32 {
        let mut best: i32 = 0;
        {
            let mut i: i32 = 1;
            'l1: while i < self.place.len() {
                if (*self.place.at(i)) > (*self.place.at(best)) {
                    best = i;
                }
                i += 1;
            }
        }
        return best;
    }
    pub fn place_index(mut self) -> i32 {
        let mut span: i32 = (self.world_right - self.world_left);
        if span <= 0 {
            return 0;
        }
        let mut bin: i32 = ((((self.cx - ((self.world_left) as f64)) / ((span) as f64))
            * ((self.place.len()) as f64)) as i32);
        if bin < 0 {
            bin = 0;
        } else if bin >= self.place.len() {
            bin = (self.place.len() - 1);
        }
        return bin;
    }
    pub fn bin_center(mut self, mut bin: i32) -> f64 {
        let mut span: i32 = (self.world_right - self.world_left);
        if span <= 0 {
            return self.cx;
        }
        return (((self.world_left) as f64)
            + (((((bin) as f64) + 0.5) / ((self.place.len()) as f64)) * ((span) as f64)));
    }
    // ---- Remembering between runs -------------------------------------------
    //
    // Everything worth keeping (learned taste, friendships, hangouts, the
    // body's walk, novelty counts) packs into plain key=value lines so the
    // program can save them to a file and reload them next launch.
    // Moods (cheer, fear, energy) are the moment, not the character, and
    // start fresh. Corrupt or mismatched data is ignored, never fatal.
    pub fn dbl(mut v: f64) -> String {
        return rt::format_num(v, &"R".to_string());
    }
    pub fn try_dbl(mut s: String, v: &mut f64) -> bool {
        if !rt::attempt(|| {
            (*v) = rt::parse_f64(&s);
        }) {
            (*v) = 0.0;
            return false;
        }
        return true;
        #[allow(unreachable_code)]
        unreachable!()
    }
    pub fn clamp(mut v: f64, mut lo: f64, mut hi: f64) -> f64 {
        return (if (v < lo) {
            lo
        } else {
            (if (v > hi) { hi } else { v })
        });
    }
    // Packs this walker's lasting memory into key=value lines (no header).
    pub fn pack_mind(mut self) -> String {
        return self.copy_mind().pack();
    }
    //
    // A copy of everything PackMind writes, taken on the thread that
    // owns the walker; it can then be turned into text anywhere. The
    // copying is cheap; the text (tens of thousands of numbers) is not,
    // so the background save does that part off the UI thread.
    //
    pub fn copy_mind(mut self) -> MindCopy {
        let mut m: MindCopy = MindCopy::new();
        m.values = (if self.values.is_null() {
            <Arr2<f64>>::default()
        } else {
            self.values.deep_copy()
        });
        m.fondness = Arr::new(0);
        'l1: for mut kv in self.fondness.items() {
            if kv.key.is_null() || (kv.key == self) {
                continue 'l1;
            }
            m.fondness.add(Kv {
                key: kv.key.personality.name.clone(),
                value: kv.value,
            });
        }
        m.place = self.place.deep_copy();
        m.style_ready = self.style_ready;
        m.style = self.style.deep_copy();
        m.swagger_seed = self.swagger_seed;
        m.fidget_seed = self.fidget_seed;
        m.fidget_freq = self.fidget_freq;
        m.trial_best = self.trial_best;
        m.style_trials = self.style_trials;
        m.use_count = (if self.use_count.is_null() {
            <Arr<i32>>::default()
        } else {
            self.use_count.deep_copy()
        });
        m.last_choice = self.last_choice;
        m.repeat_count = self.repeat_count;
        m.optimism = self.optimism;
        m.lifetime_decisions = self.lifetime_decisions;
        m.total_reward = self.total_reward;
        m.mlp_w1 = (if self.mlp_w1.is_null() {
            <Arr2<f64>>::default()
        } else {
            self.mlp_w1.deep_copy()
        });
        m.mlp_b1 = (if self.mlp_b1.is_null() {
            <Arr<f64>>::default()
        } else {
            self.mlp_b1.deep_copy()
        });
        m.mlp_w2 = (if self.mlp_w2.is_null() {
            <Arr2<f64>>::default()
        } else {
            self.mlp_w2.deep_copy()
        });
        m.mlp_b2 = (if self.mlp_b2.is_null() {
            <Arr<f64>>::default()
        } else {
            self.mlp_b2.deep_copy()
        });
        return m;
    }
    // How many learned tastes have moved from zero: a rough "how much it knows".
    pub fn learned_entries(mut self) -> i32 {
        if self.values.is_null() {
            return 0;
        }
        let mut n: i32 = 0;
        'l1: for mut v in self.values.items() {
            if (v > 0.000000001) || (v < -0.000000001) {
                n += 1;
            }
        }
        return n;
    }
    // How many friendships and grudges it holds.
    pub fn friendships(mut self) -> i32 {
        return self.fondness.len();
    }
    // Whether its walk has been personalised beyond the cast default.
    pub fn has_own_walk(mut self) -> bool {
        return self.style_ready;
    }
    // How many judged choices this walker has lived through.
    pub fn experience(mut self) -> i32 {
        return self.lifetime_decisions;
    }
    // Mean clamped reward per judged choice: is life going well?
    pub fn avg_reward(mut self) -> f64 {
        return (if (self.lifetime_decisions > 0) {
            (self.total_reward / ((self.lifetime_decisions) as f64))
        } else {
            0.0
        });
    }
    // Forgets everything learned: taste, friendships, hangouts, walk, novelty.
    pub fn forget_mind(mut self) {
        self.values = <Arr2<f64>>::default();
        self.fondness.clear();
        {
            let mut i: i32 = 0;
            'l1: while i < self.place.len() {
                (*self.place.at(i)) = 0.0;
                i += 1;
            }
        }
        self.style_ready = false;
        {
            let mut i: i32 = 0;
            'l2: while i < Walker::STYLE_DIMS {
                (*self.style.at(i)) = 0.0;
                i += 1;
            }
        }
        self.style_trials = 0;
        self.trial_best = -10.0;
        self.use_count = <Arr<i32>>::default();
        self.last_choice = -1;
        self.repeat_count = 0;
        self.cheer = 0.0;
        self.fear = 0.0;
        self.optimism = 0.0;
        self.lifetime_decisions = 0;
        self.total_reward = 0.0;
        self.mlp_w1 = <Arr2<f64>>::default();
        self.mlp_b1 = <Arr<f64>>::default();
        self.mlp_w2 = <Arr2<f64>>::default();
        self.mlp_b2 = <Arr<f64>>::default();
        if !self.prev_feat.is_null() {
            {
                let mut i: i32 = 0;
                'l3: while i < self.prev_feat.len() {
                    (*self.prev_feat.at(i)) = 0.0;
                    i += 1;
                }
            }
        }
        self.log_pos = 0;
        {
            let mut i: i32 = 0;
            'l4: while i < self.log.len() {
                (*self.log.at(i)).who = <Walker>::default();
                i += 1;
            }
        }
        self.pending_bonus = 0.0;
        self.has_prev = false;
    }
    // Restores lasting memory from key=value fields; bad entries are skipped.
    pub fn unpack_mind(
        mut self,
        mut fields: Dict<String, String>,
        mut by_name: Dict<String, Walker>,
    ) {
        self.ensure_table();
        let mut rows: String = String::new();
        let mut cols: String = String::new();
        let mut flat: String = String::new();
        let mut r: i32 = 0;
        let mut c: i32 = 0;
        let mut sparse: bool = fields.try_get_value(&"vs".to_string(), &mut flat);
        if (fields.try_get_value(&"vrows".to_string(), &mut rows)
            && fields.try_get_value(&"vcols".to_string(), &mut cols))
            && (sparse || fields.try_get_value(&"v".to_string(), &mut flat))
        {
            if !rt::attempt(|| {
                r = rt::parse_i32(&rows);
                c = rt::parse_i32(&cols);
            }) {
                r = 0;
                c = 0;
            }
            // The file's table: v4's 81 situations, or v3's 8 (dense or
            // sparse either way). A v3 table is spread over the fine
            // situations in its family, so nothing learned is lost.
            if ((r == Walker::STATE_COUNT) || (r == 8)) && (c == (self.habit_count() + 3)) {
                let mut ok: bool = true;
                let mut loaded: Arr2<f64> = Arr2::new(r, c);
                let mut nums: Arr<String> = rt::split(&flat, ',');
                if sparse {
                    {
                        let mut i: i32 = 0;
                        'l1: while (i < nums.len()) && ok {
                            'c2: {
                                let mut item: String = (*nums.at(i)).trim().to_string();
                                if item.chars().count() as i32 == 0 {
                                    break 'c2;
                                }
                                let mut colon: i32 = rt::index_of(&item, ':');
                                let mut index: i32 = <i32>::default();
                                let mut v: f64 = <f64>::default();
                                if ((((((colon <= 0)
                                    || !(rt::try_parse_i32(
                                        &rt::substr(&item, 0, colon),
                                        &mut index,
                                    )))
                                    || (index < 0))
                                    || (index >= (r * c)))
                                    || !(Walker::try_dbl(
                                        rt::substr(&item, (colon + 1), -1),
                                        rt::mref(&mut v),
                                    )))
                                    || (v).is_nan())
                                    || (v).is_infinite()
                                {
                                    ok = false;
                                    break 'l1;
                                }
                                (*loaded.at2((index / c), (index % c))) = Walker::clamp(
                                    v,
                                    (-(Walker::MAX_REWARD) * 4.0),
                                    (Walker::MAX_REWARD * 4.0),
                                );
                            }
                            i += 1;
                        }
                    }
                } else if nums.len() == (r * c) {
                    {
                        let mut i: i32 = 0;
                        'l3: while i < nums.len() {
                            let mut v: f64 = <f64>::default();
                            if (!(Walker::try_dbl((*nums.at(i)).clone(), rt::mref(&mut v)))
                                || (v).is_nan())
                                || (v).is_infinite()
                            {
                                ok = false;
                                break 'l3;
                            }
                            (*loaded.at2((i / c), (i % c))) = Walker::clamp(
                                v,
                                (-(Walker::MAX_REWARD) * 4.0),
                                (Walker::MAX_REWARD * 4.0),
                            );
                            i += 1;
                        }
                    }
                } else {
                    ok = false;
                }
                if ok && (r == 8) {
                    let mut spread: Arr2<f64> = Arr2::new(Walker::STATE_COUNT, c);
                    {
                        let mut s: i32 = 0;
                        'l4: while s < Walker::STATE_COUNT {
                            let mut family: i32 = Walker::coarse_of(s);
                            {
                                let mut a: i32 = 0;
                                'l5: while a < c {
                                    (*spread.at2(s, a)) = (*loaded.at2(family, a));
                                    a += 1;
                                }
                            }
                            s += 1;
                        }
                    }
                    loaded = spread.clone();
                }
                if ok {
                    self.values = loaded.clone();
                }
            }
        }
        let mut fond: String = String::new();
        if fields.try_get_value(&"fond".to_string(), &mut fond) {
            let mut pairs: Arr<String> = rt::split(&fond, ';');
            {
                let mut i: i32 = 0;
                'l6: while i < pairs.len() {
                    'c7: {
                        let mut kv: Arr<String> = rt::split(&(*pairs.at(i)), ',');
                        if kv.len() != 2 {
                            break 'c7;
                        }
                        let mut other: Walker = <Walker>::default();
                        if !(by_name.try_get_value(&(*kv.at(0)).clone(), &mut other))
                            || (other == self)
                        {
                            break 'c7;
                        }
                        let mut v: f64 = <f64>::default();
                        if !(Walker::try_dbl((*kv.at(1)).clone(), rt::mref(&mut v))) {
                            break 'c7;
                        }
                        self.fondness.set(other, Walker::clamp(v, -1.0, 1.0));
                    }
                    i += 1;
                }
            }
        }
        let mut pl: String = String::new();
        if fields.try_get_value(&"place".to_string(), &mut pl) {
            let mut nums: Arr<String> = rt::split(&pl, ',');
            if nums.len() == self.place.len() {
                let mut ok: bool = true;
                let mut loaded: Arr<f64> = Arr::new(self.place.len());
                {
                    let mut i: i32 = 0;
                    'l8: while i < nums.len() {
                        if (!(Walker::try_dbl(
                            (*nums.at(i)).clone(),
                            rt::mref(&mut (*loaded.at(i))),
                        )) || (*loaded.at(i)).is_nan())
                            || (*loaded.at(i)).is_infinite()
                        {
                            ok = false;
                            break 'l8;
                        }
                        (*loaded.at(i)) = Walker::clamp(
                            (*loaded.at(i)),
                            (-(Walker::MAX_REWARD) * 4.0),
                            (Walker::MAX_REWARD * 4.0),
                        );
                        i += 1;
                    }
                }
                if ok {
                    {
                        let mut i: i32 = 0;
                        'l9: while i < self.place.len() {
                            (*self.place.at(i)) = (*loaded.at(i));
                            i += 1;
                        }
                    }
                }
            }
        }
        let mut st: String = String::new();
        if fields.try_get_value(&"style".to_string(), &mut st) {
            let mut nums: Arr<String> = rt::split(&st, ',');
            if nums.len() == Walker::STYLE_DIMS {
                let mut ok: bool = true;
                let mut loaded: Arr<f64> = Arr::new(Walker::STYLE_DIMS);
                {
                    let mut i: i32 = 0;
                    'l10: while i < nums.len() {
                        if (!(Walker::try_dbl(
                            (*nums.at(i)).clone(),
                            rt::mref(&mut (*loaded.at(i))),
                        )) || (*loaded.at(i)).is_nan())
                            || (*loaded.at(i)).is_infinite()
                        {
                            ok = false;
                            break 'l10;
                        }
                        i += 1;
                    }
                }
                if ok {
                    {
                        let mut i: i32 = 0;
                        'l11: while i < Walker::STYLE_DIMS {
                            (*self.style.at(i)) = Walker::clamp_dim(i, (*loaded.at(i)));
                            i += 1;
                        }
                    }
                    self.style_ready = true;
                    self.trial_timer = Walker::TRIAL_SECONDS;
                    self.trial_bonus = 0.0;
                    self.trial_dist = 0.0;
                    self.trial_energy0 = self.energy;
                }
            }
        }
        let mut sd: String = String::new();
        if self.style_ready && fields.try_get_value(&"seeds".to_string(), &mut sd) {
            let mut nums: Arr<String> = rt::split(&sd, ',');
            let mut a: f64 = <f64>::default();
            let mut b: f64 = <f64>::default();
            let mut fr: f64 = <f64>::default();
            let mut tb: f64 = <f64>::default();
            if (((((nums.len() == 4) || (nums.len() == 5))
                && Walker::try_dbl((*nums.at(0)).clone(), rt::mref(&mut a)))
                && Walker::try_dbl((*nums.at(1)).clone(), rt::mref(&mut b)))
                && Walker::try_dbl((*nums.at(2)).clone(), rt::mref(&mut fr)))
                && Walker::try_dbl((*nums.at(3)).clone(), rt::mref(&mut tb))
            {
                self.swagger_seed = a;
                self.fidget_seed = b;
                if (fr >= 1.0) && (fr <= 5.0) {
                    self.fidget_freq = fr;
                }
                if (tb >= -2.0) && (tb <= 2.0) {
                    self.trial_best = tb;
                }
                if nums.len() == 5 {
                    if !rt::attempt(|| {
                        let mut tr: i32 = rt::parse_i32(&(*nums.at(4)));
                        if (tr >= 0) && (tr <= 1000000) {
                            self.style_trials = tr;
                        }
                    }) {}
                }
            }
        }
        let mut us: String = String::new();
        if fields.try_get_value(&"use".to_string(), &mut us) {
            let mut nums: Arr<String> = rt::split(&us, ',');
            if nums.len() == (self.habit_count() + 3) {
                let mut ok: bool = true;
                let mut loaded: Arr<i32> = Arr::new(nums.len());
                {
                    let mut i: i32 = 0;
                    'l12: while i < nums.len() {
                        match rt::attempt_val(|| {
                            (*loaded.at(i)) = rt::parse_i32(&(*nums.at(i)));
                            if ((*loaded.at(i)) < 0) || ((*loaded.at(i)) > 1000000) {
                                ok = false;
                                return true;
                            }
                            false
                        }) {
                            Some(true) => break 'l12,
                            Some(false) => {}
                            None => {
                                ok = false;
                                break 'l12;
                            }
                        }
                        i += 1;
                    }
                }
                if ok {
                    self.use_count = loaded.clone();
                }
            }
        }
        let mut last: String = String::new();
        if fields.try_get_value(&"last".to_string(), &mut last) {
            let mut nums: Arr<String> = rt::split(&last, ',');
            if nums.len() == 2 {
                if !rt::attempt(|| {
                    let mut lc: i32 = rt::parse_i32(&(*nums.at(0)));
                    let mut rp: i32 = rt::parse_i32(&(*nums.at(1)));
                    if (((lc >= -1) && (lc < (self.habit_count() + 3))) && (rp >= 0)) && (rp <= 40)
                    {
                        self.last_choice = lc;
                        self.repeat_count = rp;
                    }
                }) {}
            }
        }
        let mut opt: String = String::new();
        if fields.try_get_value(&"opt".to_string(), &mut opt) {
            let mut o: f64 = <f64>::default();
            if (Walker::try_dbl(opt.clone(), rt::mref(&mut o)) && !((o).is_nan()))
                && !((o).is_infinite())
            {
                self.optimism = Walker::clamp(o, -1.0, 1.0);
            }
        }
        let mut exp: String = String::new();
        if fields.try_get_value(&"exp".to_string(), &mut exp) {
            if !rt::attempt(|| {
                let mut e: i32 = rt::parse_i32(&exp);
                if (e >= 0) && (e <= 1000000) {
                    self.lifetime_decisions = e;
                }
            }) {}
        }
        let mut rw: String = String::new();
        if fields.try_get_value(&"rew".to_string(), &mut rw) {
            let mut t: f64 = <f64>::default();
            if (Walker::try_dbl(rw.clone(), rt::mref(&mut t)) && !((t).is_nan()))
                && !((t).is_infinite())
            {
                self.total_reward = Walker::clamp(t, -1000000.0, 1000000.0);
            }
        }
        // Upgrade v3: MLP weights. Missing (v1/v2 files) means fresh small net -
        // the table still carries the old taste, so migration stays smooth.
        let mut mshape: String = String::new();
        let mut mw1: String = String::new();
        let mut mb1: String = String::new();
        let mut mw2: String = String::new();
        let mut mb2: String = String::new();
        if (((fields.try_get_value(&"mshape".to_string(), &mut mshape)
            && fields.try_get_value(&"mw1".to_string(), &mut mw1))
            && fields.try_get_value(&"mb1".to_string(), &mut mb1))
            && fields.try_get_value(&"mw2".to_string(), &mut mw2))
            && fields.try_get_value(&"mb2".to_string(), &mut mb2)
        {
            if !rt::attempt(|| {
                let mut dims: Arr<String> = rt::split(&mshape, ',');
                let mut mf: i32 = rt::parse_i32(&(*dims.at(0)));
                let mut mh: i32 = rt::parse_i32(&(*dims.at(1)));
                let mut mn: i32 = rt::parse_i32(&(*dims.at(2)));
                if ((mf == Walker::MLP_FEATS) && (mh == Walker::MLP_HIDDEN))
                    && (mn == (self.habit_count() + 3))
                {
                    let mut s_w1: Arr<String> = rt::split(&mw1, ',');
                    let mut s_b1: Arr<String> = rt::split(&mb1, ',');
                    let mut s_w2: Arr<String> = rt::split(&mw2, ',');
                    let mut s_b2: Arr<String> = rt::split(&mb2, ',');
                    if (((s_w1.len() == (mf * mh)) && (s_b1.len() == mh))
                        && (s_w2.len() == (mh * mn)))
                        && (s_b2.len() == mn)
                    {
                        let mut nw1: Arr2<f64> = Arr2::new(mf, mh);
                        let mut nb1: Arr<f64> = Arr::new(mh);
                        let mut nw2: Arr2<f64> = Arr2::new(mh, mn);
                        let mut nb2: Arr<f64> = Arr::new(mn);
                        let mut ok: bool = true;
                        {
                            let mut i: i32 = 0;
                            'l13: while (i < s_w1.len()) && ok {
                                let mut v: f64 = <f64>::default();
                                if (!(Walker::try_dbl((*s_w1.at(i)).clone(), rt::mref(&mut v)))
                                    || (v).is_nan())
                                    || (v).is_infinite()
                                {
                                    ok = false;
                                } else {
                                    (*nw1.at2((i / mh), (i % mh))) = Walker::clamp(v, -2.0, 2.0);
                                }
                                i += 1;
                            }
                        }
                        {
                            let mut i: i32 = 0;
                            'l14: while (i < s_b1.len()) && ok {
                                let mut v: f64 = <f64>::default();
                                if (!(Walker::try_dbl((*s_b1.at(i)).clone(), rt::mref(&mut v)))
                                    || (v).is_nan())
                                    || (v).is_infinite()
                                {
                                    ok = false;
                                } else {
                                    (*nb1.at(i)) = Walker::clamp(v, -2.0, 2.0);
                                }
                                i += 1;
                            }
                        }
                        {
                            let mut i: i32 = 0;
                            'l15: while (i < s_w2.len()) && ok {
                                let mut v: f64 = <f64>::default();
                                if (!(Walker::try_dbl((*s_w2.at(i)).clone(), rt::mref(&mut v)))
                                    || (v).is_nan())
                                    || (v).is_infinite()
                                {
                                    ok = false;
                                } else {
                                    (*nw2.at2((i / mn), (i % mn))) = Walker::clamp(v, -2.0, 2.0);
                                }
                                i += 1;
                            }
                        }
                        {
                            let mut i: i32 = 0;
                            'l16: while (i < s_b2.len()) && ok {
                                let mut v: f64 = <f64>::default();
                                if (!(Walker::try_dbl((*s_b2.at(i)).clone(), rt::mref(&mut v)))
                                    || (v).is_nan())
                                    || (v).is_infinite()
                                {
                                    ok = false;
                                } else {
                                    (*nb2.at(i)) = Walker::clamp(v, -2.0, 2.0);
                                }
                                i += 1;
                            }
                        }
                        if ok {
                            self.mlp_w1 = nw1.clone();
                            self.mlp_b1 = nb1.clone();
                            self.mlp_w2 = nw2.clone();
                            self.mlp_b2 = nb2.clone();
                        }
                    }
                }
            }) {}
        }
    }
}

#[derive(Clone, Default, Copy)]
pub struct MemoryEvent {
    pub kind: i32,
    pub who: Walker,
    pub at: f64,
}

impl MemoryEvent {}

// A walker's lasting memory, copied out (see `CopyMind`).
#[derive(Clone, Default)]
pub struct MindCopyData {
    pub values: Arr2<f64>,
    pub fondness: Arr<Kv<String, f64>>,
    pub place: Arr<f64>,
    pub style_ready: bool,
    pub style: Arr<f64>,
    pub swagger_seed: f64,
    pub fidget_seed: f64,
    pub fidget_freq: f64,
    pub trial_best: f64,
    pub style_trials: i32,
    pub use_count: Arr<i32>,
    pub last_choice: i32,
    pub repeat_count: i32,
    pub optimism: f64,
    pub lifetime_decisions: i32,
    pub total_reward: f64,
    pub mlp_w1: Arr2<f64>,
    pub mlp_b1: Arr<f64>,
    pub mlp_w2: Arr2<f64>,
    pub mlp_b2: Arr<f64>,
}
rt::handle!(MindCopy, MindCopyData);

impl MindCopy {
    fn alloc__() -> MindCopy {
        let mut this = MindCopy::from_data(MindCopyData::default());
        this
    }
    pub fn new() -> MindCopy {
        Self::alloc__()
    }
    // The key=value lines (no header).
    pub fn pack(mut self) -> String {
        let mut sb: StringBuilder = StringBuilder::new();
        if !self.values.is_null() {
            // Sparse since v4: the fine table is mostly zero for a long
            // time, so only the cells that have learned something are
            // written, as index:value pairs.
            sb.append_str(&"vrows=".to_string())
                .append_disp(self.values.get_length(0))
                .append_str(&"\n".to_string());
            sb.append_str(&"vcols=".to_string())
                .append_disp(self.values.get_length(1))
                .append_str(&"\n".to_string());
            let mut v: StringBuilder = StringBuilder::new();
            let mut cols: i32 = self.values.get_length(1);
            {
                let mut r: i32 = 0;
                'l1: while r < self.values.get_length(0) {
                    {
                        let mut c: i32 = 0;
                        'l2: while c < cols {
                            'c3: {
                                let mut x: f64 = (*self.values.at2(r, c));
                                if x == 0.0 {
                                    break 'c3;
                                }
                                if v.length() > 0 {
                                    v.append_str(&",".to_string());
                                }
                                v.append_str(&((r * cols) + c).to_string())
                                    .append_str(&":".to_string())
                                    .append_str(&Walker::dbl(x));
                            }
                            c += 1;
                        }
                    }
                    r += 1;
                }
            }
            sb.append_str(&"vs=".to_string())
                .append_str(&v.to_string())
                .append_str(&"\n".to_string());
        }
        if self.fondness.len() > 0 {
            let mut f: StringBuilder = StringBuilder::new();
            {
                let mut i: i32 = 0;
                'l4: while i < self.fondness.len() {
                    if i != 0 {
                        f.append_str(&";".to_string());
                    }
                    f.append_str(&(*self.fondness.at(i)).key)
                        .append_str(&",".to_string())
                        .append_str(&Walker::dbl((*self.fondness.at(i)).value));
                    i += 1;
                }
            }
            sb.append_str(&"fond=".to_string())
                .append_str(&f.to_string())
                .append_str(&"\n".to_string());
        }
        let mut p: StringBuilder = StringBuilder::new();
        {
            let mut i: i32 = 0;
            'l5: while i < self.place.len() {
                if i != 0 {
                    p.append_str(&",".to_string());
                }
                p.append_str(&Walker::dbl((*self.place.at(i))));
                i += 1;
            }
        }
        sb.append_str(&"place=".to_string())
            .append_str(&p.to_string())
            .append_str(&"\n".to_string());
        if self.style_ready {
            let mut s: StringBuilder = StringBuilder::new();
            {
                let mut i: i32 = 0;
                'l6: while i < Walker::STYLE_DIMS {
                    if i != 0 {
                        s.append_str(&",".to_string());
                    }
                    s.append_str(&Walker::dbl((*self.style.at(i))));
                    i += 1;
                }
            }
            sb.append_str(&"style=".to_string())
                .append_str(&s.to_string())
                .append_str(&"\n".to_string());
            sb.append_str(&"seeds=".to_string())
                .append_str(&Walker::dbl(self.swagger_seed))
                .append_str(&",".to_string())
                .append_str(&Walker::dbl(self.fidget_seed))
                .append_str(&",".to_string())
                .append_str(&Walker::dbl(self.fidget_freq))
                .append_str(&",".to_string())
                .append_str(&Walker::dbl(self.trial_best))
                .append_str(&",".to_string())
                .append_str(&(self.style_trials).to_string())
                .append_str(&"\n".to_string());
        }
        if !self.use_count.is_null() {
            let mut u: StringBuilder = StringBuilder::new();
            {
                let mut i: i32 = 0;
                'l7: while i < self.use_count.len() {
                    if i != 0 {
                        u.append_str(&",".to_string());
                    }
                    u.append_str(&(*self.use_count.at(i)).to_string());
                    i += 1;
                }
            }
            sb.append_str(&"use=".to_string())
                .append_str(&u.to_string())
                .append_str(&"\n".to_string());
        }
        sb.append_str(&"last=".to_string())
            .append_str(&(self.last_choice).to_string())
            .append_str(&",".to_string())
            .append_str(&(self.repeat_count).to_string())
            .append_str(&"\n".to_string());
        sb.append_str(&"opt=".to_string())
            .append_str(&Walker::dbl(self.optimism))
            .append_str(&"\n".to_string());
        sb.append_str(&"exp=".to_string())
            .append_str(&(self.lifetime_decisions).to_string())
            .append_str(&"\n".to_string());
        sb.append_str(&"rew=".to_string())
            .append_str(&Walker::dbl(self.total_reward))
            .append_str(&"\n".to_string());
        if ((!self.mlp_w1.is_null() && !self.mlp_b1.is_null()) && !self.mlp_w2.is_null())
            && !self.mlp_b2.is_null()
        {
            let mut mn: i32 = self.mlp_b2.len();
            sb.append_str(&"mshape=".to_string())
                .append_str(&(Walker::MLP_FEATS).to_string())
                .append_str(&",".to_string())
                .append_str(&(Walker::MLP_HIDDEN).to_string())
                .append_str(&",".to_string())
                .append_str(&(mn).to_string())
                .append_str(&"\n".to_string());
            let mut w1: StringBuilder = StringBuilder::new();
            {
                let mut f: i32 = 0;
                'l8: while f < Walker::MLP_FEATS {
                    {
                        let mut h: i32 = 0;
                        'l9: while h < Walker::MLP_HIDDEN {
                            if (f != 0) || (h != 0) {
                                w1.append_str(&",".to_string());
                            }
                            w1.append_str(&Walker::dbl((*self.mlp_w1.at2(f, h))));
                            h += 1;
                        }
                    }
                    f += 1;
                }
            }
            sb.append_str(&"mw1=".to_string())
                .append_str(&w1.to_string())
                .append_str(&"\n".to_string());
            let mut b1: StringBuilder = StringBuilder::new();
            {
                let mut h: i32 = 0;
                'l10: while h < Walker::MLP_HIDDEN {
                    if h != 0 {
                        b1.append_str(&",".to_string());
                    }
                    b1.append_str(&Walker::dbl((*self.mlp_b1.at(h))));
                    h += 1;
                }
            }
            sb.append_str(&"mb1=".to_string())
                .append_str(&b1.to_string())
                .append_str(&"\n".to_string());
            let mut w2: StringBuilder = StringBuilder::new();
            {
                let mut h: i32 = 0;
                'l11: while h < Walker::MLP_HIDDEN {
                    {
                        let mut a: i32 = 0;
                        'l12: while a < mn {
                            if (h != 0) || (a != 0) {
                                w2.append_str(&",".to_string());
                            }
                            w2.append_str(&Walker::dbl((*self.mlp_w2.at2(h, a))));
                            a += 1;
                        }
                    }
                    h += 1;
                }
            }
            sb.append_str(&"mw2=".to_string())
                .append_str(&w2.to_string())
                .append_str(&"\n".to_string());
            let mut b2: StringBuilder = StringBuilder::new();
            {
                let mut a: i32 = 0;
                'l13: while a < mn {
                    if a != 0 {
                        b2.append_str(&",".to_string());
                    }
                    b2.append_str(&Walker::dbl((*self.mlp_b2.at(a))));
                    a += 1;
                }
            }
            sb.append_str(&"mb2=".to_string())
                .append_str(&b2.to_string())
                .append_str(&"\n".to_string());
        }
        return sb.to_string();
    }
}
