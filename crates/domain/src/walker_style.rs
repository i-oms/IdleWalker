// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

impl Walker {
    pub const STYLE_DIMS: i32 = 6;
    pub const TRIAL_SECONDS: f64 = 18.0;
    // Bob amplitude for the stride, from its build plus its own touch.
    pub fn eff_bob(mut self) -> f64 {
        return (self.personality.walk_bob * (1.0 + (*self.style.at(0))));
    }
    // Forward lean while walking, from its build plus its own touch.
    pub fn eff_lean(mut self) -> f64 {
        return (self.personality.walk_lean + (*self.style.at(1)));
    }
    // Speed multiplier, from its build plus its own touch.
    pub fn eff_speed(mut self) -> f64 {
        return (self.personality.speed_factor * (1.0 + (*self.style.at(2))));
    }
    // Multiplier on the hop interval: above 1 hops lazier, below 1 livelier.
    pub fn eff_hop(mut self) -> f64 {
        let mut m: f64 = (1.0 + (*self.style.at(3)));
        return (if (m < 0.3) { 0.3 } else { m });
    }
    // Side-to-side swagger with the stride, in degrees.
    pub fn swagger_yaw(mut self) -> f64 {
        return ((*self.style.at(4))
            * (((self.walk_phase * 2.0) * std::f64::consts::PI) + self.swagger_seed).sin());
    }
    // Its fidget while standing about: a small personal sway.
    pub fn fidget_lean(mut self) -> f64 {
        return Walker::quantise(
            ((*self.style.at(5)) * ((self.time * self.fidget_freq) + self.fidget_seed).sin()),
        );
    }
    pub fn clamp_dim(mut dim: i32, mut v: f64) -> f64 {
        let mut lo: f64 = <f64>::default();
        let mut hi: f64 = <f64>::default();
        'sw1: {
            match dim {
                0 => {
                    lo = -0.45;
                    hi = 0.6;
                }
                1 => {
                    lo = -0.2;
                    hi = 0.25;
                }
                2 => {
                    lo = -0.15;
                    hi = 0.25;
                }
                3 => {
                    lo = -0.5;
                    hi = 1.5;
                }
                4 => {
                    lo = 0.0;
                    hi = 7.0;
                }
                _ => {
                    lo = 0.0;
                    hi = 0.12;
                }
            }
        }
        return (if (v < lo) {
            lo
        } else {
            (if (v > hi) { hi } else { v })
        });
    }
    pub fn step_size(mut dim: i32) -> f64 {
        'sw1: {
            match dim {
                0 => {
                    return 0.08;
                }
                1 => {
                    return 0.03;
                }
                2 => {
                    return 0.04;
                }
                3 => {
                    return 0.12;
                }
                4 => {
                    return 0.8;
                }
                _ => {
                    return 0.02;
                }
            }
        }
        #[allow(unreachable_code)]
        unreachable!()
    }
    pub fn seed_style(mut self) {
        self.style_ready = true;
        (*self.style.at(0)) = (((self.rng.next_double() + self.rng.next_double()) - 1.0) * 0.06);
        (*self.style.at(1)) = (((self.rng.next_double() + self.rng.next_double()) - 1.0) * 0.02);
        (*self.style.at(2)) = (((self.rng.next_double() + self.rng.next_double()) - 1.0) * 0.03);
        (*self.style.at(3)) = (((self.rng.next_double() + self.rng.next_double()) - 1.0) * 0.1);
        (*self.style.at(4)) = (self.rng.next_double() * 2.0);
        (*self.style.at(5)) = (self.rng.next_double() * 0.05);
        {
            let mut i: i32 = 0;
            'l1: while i < Walker::STYLE_DIMS {
                (*self.style.at(i)) = Walker::clamp_dim(i, (*self.style.at(i)));
                i += 1;
            }
        }
        self.swagger_seed = ((self.rng.next_double() * 2.0) * std::f64::consts::PI);
        self.fidget_seed = ((self.rng.next_double() * 2.0) * std::f64::consts::PI);
        self.fidget_freq = (2.0 + (self.rng.next_double() * 1.4));
        self.trial_energy0 = self.energy;
        self.trial_timer = (Walker::TRIAL_SECONDS * (0.8 + (self.rng.next_double() * 0.4)));
    }
    // One frame of the body learning: trial, judge, keep or revert, mutate.
    pub fn step_style(mut self, mut dt: f64) {
        if !(self.style_ready) {
            self.seed_style();
        }
        self.trial_timer -= dt;
        if self.trial_timer > 0.0 {
            return;
        }
        // Judge the trial: energy kept, good things banked, ground covered.
        // Upgrade v2: distance weight lowered (was rewarding pacing, not skill).
        let mut score: f64 = (((2.0 * self.trial_bonus) + (self.energy - self.trial_energy0))
            + ((0.00025 * self.trial_dist) / self.size_factor));
        if score > 2.0 {
            score = 2.0;
        } else if score < -2.0 {
            score = -2.0;
        }
        if score >= self.trial_best {
            self.trial_best = score;
        } else {
            (*self.style.at(self.trial_dim)) = self.trial_saved;
        }
        self.trial_best *= 0.995; // the world changes, so keep exploring
        self.style_trials += 1;
        // Next trial: change one thing a little.
        // Upgrade v2: anneal mutation with age so the walk settles but never freezes.
        self.trial_dim = self.rng.next(Walker::STYLE_DIMS);
        self.trial_saved = (*self.style.at(self.trial_dim));
        let mut anneal: f64 = (1.0 / (1.0 + (((self.style_trials) as f64) / 60.0)));
        if anneal < 0.35 {
            anneal = 0.35;
        }
        let mut push: f64 = (((((self.rng.next_double() + self.rng.next_double())
            + self.rng.next_double())
            - 1.5)
            * Walker::step_size(self.trial_dim))
            * anneal);
        (*self.style.at(self.trial_dim)) =
            Walker::clamp_dim(self.trial_dim, (self.trial_saved + push));
        self.trial_energy0 = self.energy;
        self.trial_bonus = 0.0;
        self.trial_dist = 0.0;
        self.trial_timer = (Walker::TRIAL_SECONDS * (0.8 + (self.rng.next_double() * 0.4)));
    }
}
