// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

// A short piece of eye acting laid over the resting face.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(i32)]
pub enum EyeBeat {
    #[default]
    None,
    SideEye,    // eyes slide to one side without the head, hold, come back
    Shifty,     // a quick look one way, then the other: is the coast clear?
    EyeRoll,    // up and over to the other side, ending half-lidded and bored
    Viewer,     // straight out at the viewer, with a look to match
    CrossEyed,  // eyes cross for a moment, then a shake and a blink
    DoubleTake, // a passer-by: look, look away, snap back wide-eyed
}
impl EyeBeat {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => EyeBeat::None,
            1 => EyeBeat::SideEye,
            2 => EyeBeat::Shifty,
            3 => EyeBeat::EyeRoll,
            4 => EyeBeat::Viewer,
            5 => EyeBeat::CrossEyed,
            6 => EyeBeat::DoubleTake,
            _ => Self::default(),
        }
    }
}
impl std::fmt::Display for EyeBeat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Walker {
    pub const GAZE_RATE: f64 = 22.0;
    pub const BEAT_GAZE_RATE: f64 = 34.0;
    // How long this one leaves between blinks compared with the average Minion.
    pub fn blink_pace(mut self) -> f64 {
        'sw1: {
            match self.personality.temperament {
                Temperament::Timid => {
                    return 0.6; // nervous: blinks a lot
                }
                Temperament::Calm => {
                    return 1.25; // unhurried
                }
                Temperament::Aggressive => {
                    return 1.5; // stares people down
                }
                Temperament::Feisty => {
                    return 1.3; // a bird's hard stare
                }
                Temperament::Goofy => {
                    return 0.85;
                }
                _ => {
                    return 1.0;
                }
            }
        }
        #[allow(unreachable_code)]
        unreachable!()
    }
    pub fn reset_eyes(mut self) {
        self.eye_beat = EyeBeat::None;
        self.beat_timer = self.between(1.5, 4.0);
        self.was_watching = false;
        self.cross_eye = 0.0;
        self.pupil_interest = 0.0;
        self.last_gx = 0.55;
        self.last_gy = 0.0;
    }
    //
    // Eyes on the company: a standing Minion follows a walker going by
    // (movingOnly), a walking one meets the eyes of the one it passes.
    // True when the gaze has been taken.
    //
    pub fn watch_company(mut self, mut moving_only: bool, gx: &mut f64, gy: &mut f64) -> bool {
        if self.company.is_null() || !(self.company.is_grounded()) {
            return false;
        }
        let mut range: f64 = (((if moving_only { 420 } else { 200 }) as f64) * self.size_factor);
        if (self.company.centre_x() - self.cx).abs() > range {
            return false;
        }
        if moving_only && ((self.company.vx).abs() < (20.0 * self.size_factor)) {
            return false;
        }
        self.look_at(
            self.company.centre_x(),
            (self.company.feet_y() - ((12 * self.scale) as f64)),
            &mut *gx,
            &mut *gy,
        );
        return true;
    }
    //
    // Standing about (Rest or Look, head not turned) or strolling with
    // nothing particular in mind: keeps the current beat going, or starts
    // one when it is time and nothing else has the eyes. Returns the rate
    // the gaze should move at.
    //
    pub fn eye_beats(
        mut self,
        mut dt: f64,
        mut watching: bool,
        mut strolling: bool,
        target: &mut Expression,
        rate: &mut f64,
        gx: &mut f64,
        gy: &mut f64,
    ) -> f64 {
        if self.eye_beat == EyeBeat::None {
            self.beat_timer -= dt;
            if ((self.beat_timer > 0.0) || watching) || (self.mood_left > 0.0) {
                return Walker::GAZE_RATE;
            }
            self.start_eye_beat(strolling);
        }
        self.beat_time += dt;
        let mut t: f64 = self.beat_time;
        let mut side: f64 = self.beat_dir;
        'sw1: {
            match self.eye_beat {
                EyeBeat::SideEye => {
                    // Slide over, hold, and come back; the head stays put.
                    let mut hold: f64 =
                        (if (self.personality.temperament == Temperament::Aggressive) {
                            1.6
                        } else {
                            1.1
                        });
                    if t > hold {
                        self.end_eye_beat();
                        break 'sw1;
                    }
                    (*gx) = (side * 0.95);
                    (*gy) = 0.05;
                    'sw2: {
                        match self.personality.temperament {
                            Temperament::Aggressive => {
                                (*target) = (*EXPRESSION_SUSPICIOUS);
                                (*rate) = 10.0;
                            }
                            Temperament::Feisty => {
                                (*target) = (*EXPRESSION_SUSPICIOUS);
                                (*rate) = 10.0;
                            }
                            Temperament::Goofy => {
                                (*target) = (*EXPRESSION_SMUG);
                                (*rate) = 10.0;
                            }
                            Temperament::Friendly => {
                                (*target) = (*EXPRESSION_HAPPY);
                                (*rate) = 10.0;
                            }
                            _ => {}
                        }
                    }
                }
                EyeBeat::Shifty => {
                    // Behind, then ahead, quickly, then back to business.
                    if t > 0.9 {
                        self.end_eye_beat();
                        break 'sw1;
                    }
                    (*gx) = (if (t < 0.45) { -0.95 } else { 0.95 });
                    (*gy) = 0.05;
                    (*target) = (*EXPRESSION_SHIFTY);
                    (*rate) = 16.0;
                }
                EyeBeat::EyeRoll => {
                    // Up from one side and over the top to the other, then a
                    // bored half-lidded look held off to that side.
                    let mut sweep: f64 = 0.7;
                    if t > (sweep + 0.9) {
                        self.end_eye_beat();
                        break 'sw1;
                    }
                    if t < sweep {
                        let mut a: f64 = ((std::f64::consts::PI * t) / sweep);
                        (*gx) = ((-side * 0.85) * (a).cos());
                        (*gy) = (0.15 + (0.8 * (a).sin()));
                        (*target) = (*EXPRESSION_DISAPPROVING);
                        (*rate) = 14.0;
                    } else {
                        (*gx) = (side * 0.7);
                        (*gy) = 0.1;
                        (*target) = (*EXPRESSION_BORED);
                        (*rate) = 10.0;
                    }
                }
                EyeBeat::Viewer => {
                    // A look straight out at you (the head comes round too).
                    let mut hold: f64 = (if self.personality.warm() { 1.5 } else { 1.1 });
                    if t > hold {
                        self.end_eye_beat();
                        break 'sw1;
                    }
                    (*gx) = 0.0;
                    (*gy) = 0.0;
                    'sw3: {
                        match self.personality.temperament {
                            Temperament::Friendly => {
                                (*target) = (*EXPRESSION_HAPPY);
                            }
                            Temperament::Goofy => {
                                (*target) = (*EXPRESSION_SILLY);
                            }
                            Temperament::Aggressive => {
                                (*target) = (*EXPRESSION_STERN);
                            }
                            Temperament::Feisty => {
                                (*target) = (*EXPRESSION_STERN);
                            }
                            _ => {
                                (*target) = (*EXPRESSION_KNOWING);
                            }
                        }
                    }
                    (*rate) = 10.0;
                }
                EyeBeat::CrossEyed => {
                    // Eyes cross and go a little vacant, then a double blink shakes it off.
                    if t > 1.0 {
                        self.end_eye_beat();
                        break 'sw1;
                    }
                    if t < 0.7 {
                        (*target) = (*EXPRESSION_DOPEY);
                        (*rate) = 12.0;
                    } else if ((self.blink_left <= 0.0) && !(self.blink_again)) && (t < 0.75) {
                        self.blink_left = Walker::BLINK_SECONDS;
                        self.blink_again = true;
                    }
                }
                EyeBeat::DoubleTake => {
                    // Look at the passer-by, back to the cursor, then snap
                    // back wide-eyed; after that the ordinary following goes on.
                    if (t > 1.3) || !(watching) {
                        self.end_eye_beat();
                        break 'sw1;
                    }
                    if (t >= 0.35) && (t < 0.6) {
                        self.look_at(
                            ((self.cursor_x) as f64),
                            ((self.cursor_y) as f64),
                            &mut *gx,
                            &mut *gy,
                        );
                    }
                    if t >= 0.6 {
                        (*target) = (*EXPRESSION_SURPRISED);
                        (*rate) = 30.0;
                    }
                }
                _ => {}
            }
        }
        return (if (self.eye_beat == EyeBeat::None) {
            Walker::GAZE_RATE
        } else {
            Walker::BEAT_GAZE_RATE
        });
    }
    // Each temperament has its own repertoire: the aggressive ones
    // side-eye and roll their eyes, the timid one checks over both
    // shoulders, the goofy one goes cross-eyed, the friendly ones look
    // at you, the calm ones mostly side-eye. On the move there is only
    // time for a glance at you - or, for the timid one, over the shoulder.
    pub fn start_eye_beat(mut self, mut strolling: bool) {
        let mut r: f64 = self.rng.next_double();
        self.beat_time = 0.0;
        if strolling {
            self.eye_beat = (if ((self.personality.temperament == Temperament::Timid) && (r < 0.6))
            {
                EyeBeat::Shifty
            } else {
                EyeBeat::Viewer
            });
            self.beat_dir = 1.0;
            return;
        }
        'sw1: {
            match self.personality.temperament {
                Temperament::Aggressive => {
                    self.eye_beat = (if (r < 0.55) {
                        EyeBeat::SideEye
                    } else {
                        (if (r < 0.85) {
                            EyeBeat::EyeRoll
                        } else {
                            EyeBeat::Viewer
                        })
                    });
                }
                Temperament::Feisty => {
                    self.eye_beat = (if (r < 0.5) {
                        EyeBeat::SideEye
                    } else {
                        (if (r < 0.8) {
                            EyeBeat::EyeRoll
                        } else {
                            EyeBeat::Viewer
                        })
                    });
                }
                Temperament::Timid => {
                    self.eye_beat = (if (r < 0.75) {
                        EyeBeat::Shifty
                    } else {
                        EyeBeat::SideEye
                    });
                }
                Temperament::Goofy => {
                    self.eye_beat = (if (r < 0.3) {
                        EyeBeat::CrossEyed
                    } else {
                        (if (r < 0.65) {
                            EyeBeat::Viewer
                        } else {
                            EyeBeat::SideEye
                        })
                    });
                }
                Temperament::Friendly => {
                    self.eye_beat = (if (r < 0.55) {
                        EyeBeat::Viewer
                    } else {
                        EyeBeat::SideEye
                    });
                }
                _ => {
                    self.eye_beat = (if (r < 0.6) {
                        EyeBeat::SideEye
                    } else {
                        (if (r < 0.9) {
                            EyeBeat::Viewer
                        } else {
                            EyeBeat::EyeRoll
                        })
                    });
                }
            }
        }
        // A side-eye goes towards the company if there is any, else away from the cursor.
        self.beat_dir = ((if !self.company.is_null() {
            rt::sign_f64(((self.company.centre_x() - self.cx) * ((self.facing) as f64)))
        } else {
            -(rt::sign_f64(((((self.cursor_x) as f64) - self.cx) * ((self.facing) as f64))))
        }) as f64);
        if self.beat_dir == 0.0 {
            self.beat_dir = 1.0;
        }
    }
    pub fn end_eye_beat(mut self) {
        if self.eye_beat == EyeBeat::None {
            return;
        }
        let mut on_the_move: bool = (self.mode == Mode::Walk);
        self.eye_beat = EyeBeat::None;
        self.beat_timer = ((if on_the_move {
            self.between(5.0, 11.0)
        } else {
            self.between(2.5, 6.0)
        }) * (if (self.personality.temperament == Temperament::Calm) {
            1.3
        } else {
            1.0
        }));
    }
    // A passer-by has just come into view: the warm ones sometimes do a double take.
    pub fn notice_passer(mut self, mut watching: bool) {
        if ((((watching && !(self.was_watching)) && (self.eye_beat == EyeBeat::None))
            && self.personality.warm())
            && (self.beat_timer < 2.5))
            && (self.rng.next_double() < 0.6)
        {
            self.eye_beat = EyeBeat::DoubleTake;
            self.beat_time = 0.0;
        }
        self.was_watching = watching;
    }
    // Real eyes blink when they jump a long way; a glance dart is too small to count.
    pub fn gaze_blink(mut self, mut gx: f64, mut gy: f64) {
        if (((self.eye_beat == EyeBeat::None) && (self.blink_left <= 0.0)) && (self.face.lid < 0.9))
            && (((gx - self.last_gx).abs() + (gy - self.last_gy).abs()) > 0.75)
        {
            self.blink_left = Walker::BLINK_SECONDS;
            if self.blink_timer < 1.2 {
                self.blink_timer = 1.2;
            }
        }
        self.last_gx = gx;
        self.last_gy = gy;
    }
    // Eases the eyes crossing (only ever on purpose: the goofy one's
    // beat) and the pupils' interest in what they are looking at. Both
    // eyes always point the same way otherwise.
    pub fn step_eye_extras(mut self, mut dt: f64, mut at_cursor: bool, mut watching: bool) {
        let mut cross: f64 = ((if ((self.eye_beat == EyeBeat::CrossEyed) && (self.beat_time < 0.7))
        {
            1
        } else {
            0
        }) as f64);
        self.cross_eye = Walker::settle(self.cross_eye, cross, (1.0 - (-dt * 10.0).exp()));
        let mut interest: f64 = 0.0;
        if (self.personality.temperament == Temperament::Aggressive)
            || (self.personality.temperament == Temperament::Feisty)
        {
            if watching || (self.eye_beat == EyeBeat::SideEye) {
                interest = -1.0;
            }
        } else if ((watching && !self.company.is_null()) && self.company.personality().warm())
            && self.personality.warm()
        {
            interest = 1.0;
        } else if at_cursor
            && ((((self.cursor_x) as f64) - self.cx).abs() < (160.0 * self.size_factor))
        {
            interest = 0.7;
        }
        self.pupil_interest =
            Walker::settle(self.pupil_interest, interest, (1.0 - (-dt * 4.0).exp()));
    }
}
