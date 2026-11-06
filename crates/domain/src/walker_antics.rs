// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

impl Walker {
    pub const DANCE_BEAT: f64 = 0.32;
    // ---- Starting, stepping, ending -----------------------------------------------
    pub fn start_antic(mut self, mut choice: Antic) {
        self.antic = choice;
        self.antic_stage = 0;
        // Having a look at the cursor takes the edge off the curiosity.
        if ((choice == Antic::Look) || (choice == Antic::Wave)) || (choice == Antic::Point) {
            self.curiosity *= 0.4;
        }
        'sw1: {
            match choice {
                Antic::Look => {
                    self.stop(Mode::Look);
                    return;
                }
                Antic::Rest => {
                    self.rest_duration = self.between(0.8, 2.0);
                    self.stop(Mode::Rest);
                    return;
                }
                Antic::Wave => {
                    // Wave at a nearby walker if there is one, otherwise at the cursor.
                    self.wave_at_cursor = !(self.company_nearby);
                    self.stop_facing(
                        (if self.company_nearby {
                            self.company_x
                        } else {
                            ((self.cursor_x) as f64)
                        }),
                        Mode::Wave,
                    );
                    return;
                }
                Antic::Point => {
                    self.antic_seconds = 2.0;
                    self.stop_facing(((self.cursor_x) as f64), Mode::Antic);
                    return;
                }
                Antic::Taunt => {
                    self.antic_seconds = 2.0;
                    self.stop_facing(((self.cursor_x) as f64), Mode::Antic);
                    return;
                }
                Antic::Laugh => {
                    self.antic_seconds = 1.6;
                    self.stop_facing(((self.cursor_x) as f64), Mode::Antic);
                    return;
                }
                Antic::Salute => {
                    self.antic_seconds = 2.0;
                    self.broadcast = Some(Antic::Salute);
                    self.stop_facing(((self.cursor_x) as f64), Mode::Antic);
                    return;
                }
                Antic::Peekaboo => {
                    self.antic_seconds = 2.2;
                    self.stop_facing(((self.cursor_x) as f64), Mode::Antic);
                    return;
                }
                Antic::Kiss => {
                    self.antic_seconds = 2.2;
                    self.stop_facing(((self.cursor_x) as f64), Mode::Antic);
                    return;
                }
                Antic::Peck => {
                    self.antic_seconds = 2.2;
                    self.stop_facing(
                        (if !self.goal_target.is_null() {
                            self.goal_target.centre_x()
                        } else {
                            ((self.cursor_x) as f64)
                        }),
                        Mode::Antic,
                    );
                    return;
                }
                Antic::Fly => {
                    self.antic_seconds = 2.6;
                }
                Antic::Launch => {
                    self.antic_seconds = 2.6;
                    self.broadcast = Some(Antic::Launch);
                }
                Antic::Rage => {
                    self.antic_seconds = 2.2;
                    self.broadcast = Some(Antic::Rage);
                }
                Antic::Egg => {
                    self.antic_seconds = 2.8;
                    self.broadcast = Some(Antic::Egg);
                }
                Antic::Inflate => {
                    self.antic_seconds = 2.8;
                    self.broadcast = Some(Antic::Inflate);
                }
                Antic::Blast => {
                    self.antic_seconds = 2.4;
                    self.broadcast = Some(Antic::Blast); // the neighbours dive for cover
                    self.stop_facing(((self.cursor_x) as f64), Mode::Antic);
                    return;
                }
                Antic::Strum => {
                    self.antic_seconds = 3.4;
                    self.broadcast = Some(Antic::Dance);
                }
                Antic::Pushups => {
                    self.antic_seconds = 3.2;
                    self.broadcast = Some(Antic::Pushups);
                }
                Antic::Jog => {
                    self.antic_seconds = 2.8;
                    self.broadcast = Some(Antic::Jog);
                }
                Antic::Moonwalk => {
                    self.antic_seconds = 2.4;
                    self.broadcast = Some(Antic::Moonwalk);
                }
                Antic::Toot => {
                    self.antic_seconds = 2.2;
                    self.broadcast = Some(Antic::Toot);
                }
                Antic::Flap => {
                    self.antic_seconds = 2.4;
                    self.broadcast = Some(Antic::Flap);
                }
                Antic::Hide => {
                    self.antic_seconds = 2.2;
                }
                Antic::Fret => {
                    self.antic_seconds = 2.4;
                }
                Antic::Faint => {
                    self.antic_seconds = 3.0;
                }
                Antic::Dig => {
                    self.antic_seconds = 3.0;
                }
                Antic::Sleepwalk => {
                    self.antic_seconds = self.between(3.0, 5.0);
                    self.broadcast = Some(Antic::Yawn);
                }
                Antic::Daydream => {
                    self.antic_seconds = 2.8;
                }
                Antic::Dance => {
                    self.antic_seconds = 2.6;
                    self.broadcast = Some(Antic::Dance);
                }
                Antic::Twirl => {
                    self.antic_seconds = 1.7;
                    self.broadcast = Some(choice);
                }
                Antic::Yawn => {
                    self.antic_seconds = 2.1;
                    self.broadcast = Some(Antic::Yawn);
                }
                Antic::Doze => {
                    self.antic_seconds = (2.1 + self.between(2.5, 4.5));
                    self.broadcast = Some(Antic::Yawn);
                }
                Antic::Scratch => {
                    self.antic_seconds = 2.0;
                    self.broadcast = Some(choice);
                }
                Antic::Whistle => {
                    self.antic_seconds = 2.7;
                    self.broadcast = Some(choice);
                }
                Antic::Sneeze => {
                    self.antic_seconds = 1.6;
                }
                Antic::Box => {
                    self.antic_seconds = 2.2;
                    self.broadcast = Some(choice);
                }
                Antic::Stomp => {
                    self.antic_seconds = 1.1;
                    self.broadcast = Some(choice);
                }
                Antic::Flex => {
                    self.antic_seconds = 2.4;
                    self.broadcast = Some(choice);
                }
                Antic::Joy => {
                    self.antic_seconds = 1.0;
                    self.broadcast = Some(choice);
                }
                Antic::Hiccups => {
                    self.antic_seconds = 2.6;
                }
                Antic::Shrug => {
                    self.antic_seconds = 1.3;
                    self.broadcast = Some(choice);
                }
                Antic::Trip => {
                    self.antic_seconds = 1.8;
                }
                Antic::Boo => {
                    self.antic_seconds = 1.6;
                }
                _ => {}
            }
        }
        self.stop(Mode::Antic);
    }
    // Stops, turning round first if the target is behind.
    pub fn stop_facing(mut self, mut target_x: f64, mut then: Mode) {
        let mut behind: bool =
            (((target_x - self.cx) * ((self.facing) as f64)) < -(self.body_width()));
        if behind && (self.mode == Mode::Walk) {
            self.after_turn = then;
            self.brake(Mode::Turn);
        } else if behind {
            self.after_turn = then;
            self.enter(Mode::Turn);
        } else {
            self.stop(then);
        }
    }
    // A trip happens mid-stride, without stopping first.
    pub fn start_trip(mut self) {
        self.antic = Antic::Trip;
        self.antic_seconds = 1.8;
        self.antic_stage = 0;
        self.enter(Mode::Antic);
    }
    // The one-off effects along the way: a stomp lands, a sneeze sprays,
    // a shout makes somebody jump.
    pub fn step_antic(mut self, mut dt: f64) {
        let mut t: f64 = self.mode_time;
        'sw1: {
            match self.antic {
                Antic::Joy => {
                    if self.antic_stage == 0 {
                        self.antic_stage = 1;
                        self.celebrating = true;
                        self.joy_hops = 1;
                        self.start_jump(((0.85 * Settings::HOP_SPEED) * self.size_factor), 0.0);
                    }
                }
                Antic::Stomp => {
                    if (self.antic_stage == 0) && (t >= 0.25) {
                        self.antic_stage = 1;
                        self.slam(0.6);
                    } else if (self.antic_stage == 1) && (t >= 0.65) {
                        self.antic_stage = 2;
                        self.slam(0.8);
                    }
                }
                Antic::Sneeze => {
                    if (self.antic_stage == 0) && (t >= 0.75) {
                        self.antic_stage = 1;
                        self.emit_dust(
                            (self.cx + (((self.facing * 7) * self.scale) as f64)),
                            (self.feet_y - (self.body.mouth_v * ((self.scale) as f64))),
                            3,
                            0.9,
                        );
                        self.start_sweat();
                    }
                }
                Antic::Box => {
                    if (self.antic_stage == 0) && (t >= 1.5) {
                        self.antic_stage = 1;
                        self.start_sweat();
                    }
                }
                Antic::Twirl => {
                    if (self.antic_stage == 0) && (t >= 0.35) {
                        self.antic_stage = 1;
                        self.emit_dust(self.cx, self.feet_y, 3, 0.5);
                    }
                }
                Antic::Trip => {
                    // Still travelling as it goes over, then skidding to a halt.
                    self.vx = Walker::approach(
                        self.vx,
                        0.0,
                        (((Settings::BRAKING * 1.5) * self.size_factor) * dt),
                    );
                    self.move_x((self.vx * dt));
                    self.walk_phase +=
                        (((self.vx).abs() * dt) / (self.body.stride_cells * ((self.scale) as f64)));
                    if (self.antic_stage == 0) && (t >= 0.12) {
                        self.antic_stage = 1;
                        self.slam(0.9);
                        self.vx *= 0.3;
                    }
                }
                Antic::Boo => {
                    if self.antic_stage == 0 {
                        self.antic_stage = 1;
                        if !self.goal_target.is_null() {
                            self.goal_target.spook(self.cx);
                            self.note_prank_landed(self.goal_target);
                            self.goal_target.note_pranked(self);
                        }
                    }
                }
                Antic::Pushups => {
                    if (self.antic_stage == 0) && (t >= 2.0) {
                        self.antic_stage = 1;
                        self.start_sweat();
                    }
                }
                Antic::Moonwalk => {
                    // Sliding backwards; the feet step forwards as if walking.
                    if t > 0.3 {
                        let mut slide: f64 =
                            (((0.5 * Settings::WALK_SPEED) * self.size_factor) * dt);
                        if !(self.move_x((((-self.facing) as f64) * slide))) {
                            self.mode_time = self.antic_seconds; // ran out of floor
                        }
                        self.walk_phase +=
                            (slide / (self.body.stride_cells * ((self.scale) as f64)));
                    }
                }
                Antic::Sleepwalk => {
                    // Shuffling forward with its eyes shut until it bumps into something.
                    if t > 1.0 {
                        let mut shuffle: f64 =
                            (((0.35 * Settings::WALK_SPEED) * self.size_factor) * dt);
                        if !(self.move_x((((self.facing) as f64) * shuffle))) {
                            self.mode_time = self.antic_seconds;
                        }
                        self.walk_phase +=
                            (shuffle / (self.body.stride_cells * ((self.scale) as f64)));
                    }
                }
                Antic::Toot => {
                    if (self.antic_stage == 0) && (t >= 0.55) {
                        self.antic_stage = 1;
                        self.squash = 0.25;
                        self.emit_dust(
                            (self.cx - (((self.facing * 5) * self.scale) as f64)),
                            (self.feet_y - (self.body.seat_v * ((self.scale) as f64))),
                            3,
                            0.5,
                        );
                    }
                }
                Antic::Dig => {
                    // A puff of dirt flies up on each scoop.
                    let mut scoop: i32 = ((t / 0.5) as i32);
                    if (scoop > self.antic_stage) && (scoop <= 5) {
                        self.antic_stage = scoop;
                        self.emit_dust(
                            (self.cx + (((self.facing * 5) * self.scale) as f64)),
                            self.feet_y,
                            2,
                            0.7,
                        );
                    }
                }
                Antic::Faint => {
                    if (self.antic_stage == 0) && (t >= 0.45) {
                        self.antic_stage = 1;
                        self.slam(0.8);
                    }
                }
                Antic::Peck => {
                    // Two hard jabs: whoever it was jumps.
                    if (self.antic_stage == 0) && (t >= 0.5) {
                        self.antic_stage = 1;
                        if !self.goal_target.is_null() {
                            self.goal_target.spook(self.cx);
                            self.note_prank_landed(self.goal_target);
                            self.goal_target.note_pranked(self);
                        }
                    } else if (self.antic_stage == 1) && (t >= 1.1) {
                        self.antic_stage = 2;
                        if !self.goal_target.is_null() && self.goal_target.is_grounded() {
                            self.goal_target.spook(self.cx);
                            self.note_prank_landed(self.goal_target);
                            self.goal_target.note_pranked(self);
                        }
                    }
                }
                Antic::Fly => {
                    // Flapping along the ground, feet trailing, until the floor runs out.
                    if t > 0.3 {
                        let mut flap: f64 =
                            (((0.9 * Settings::WALK_SPEED) * self.size_factor) * dt);
                        if !(self.move_x((((self.facing) as f64) * flap))) {
                            self.mode_time = self.antic_seconds;
                        }
                        self.walk_phase +=
                            ((2.0 * flap) / (self.body.stride_cells * ((self.scale) as f64)));
                    }
                }
                Antic::Launch => {
                    // Slingshot: hold the crouch, then spring high and far.
                    if (self.antic_stage == 0) && (t >= 0.35) {
                        self.antic_stage = 1;
                        self.set_mood((*EXPRESSION_DETERMINED), 2.0);
                        self.emit_dust(self.cx, self.feet_y, 5, 1.0);
                        self.start_jump(
                            ((1.7 * Settings::HOP_SPEED) * self.size_factor),
                            (((((self.facing) as f64) * 1.9) * Settings::WALK_SPEED)
                                * self.size_factor),
                        );
                    }
                }
                Antic::Rage => {
                    // Stamping and shaking: dust on every stamp.
                    let mut stamp: i32 = ((t / 0.45) as i32);
                    if (stamp > self.antic_stage) && (stamp <= 4) {
                        self.antic_stage = stamp;
                        self.slam(0.5);
                    }
                }
                Antic::Egg => {
                    if (self.antic_stage == 0) && (t >= 2.2) {
                        // The egg pops out with a soft thump.
                        self.antic_stage = 1;
                        self.squash = 0.3;
                        self.emit_dust(
                            (self.cx + (((self.facing * 5) * self.scale) as f64)),
                            self.feet_y,
                            2,
                            0.4,
                        );
                    }
                }
                Antic::Blast => {
                    if (self.antic_stage == 0) && (t >= 0.6) {
                        self.antic_stage = 1;
                        self.squash = 0.6;
                        self.emit_dust(self.cx, self.feet_y, 8, 1.0);
                        self.start_sweat();
                    } else if (self.antic_stage == 1) && (t >= 1.0) {
                        self.antic_stage = 2;
                        self.emit_dust(self.cx, self.feet_y, 4, 0.6);
                    }
                }
                _ => {}
            }
        }
    }
    pub fn slam(mut self, mut strength: f64) {
        self.squash = strength;
        self.emit_dust(self.cx, self.feet_y, 4, strength);
    }
    pub fn end_antic(mut self) {
        'sw1: {
            match self.antic {
                Antic::Doze => {
                    // Jerks awake with a little hop.
                    self.woke_up = true;
                    self.vx = 0.0;
                    self.vy = ((-0.5 * Settings::STARTLE_SPEED) * self.size_factor);
                    self.emit_dust(self.cx, self.feet_y, 2, 0.5);
                    self.enter(Mode::Air);
                    return;
                }
                Antic::Box => {
                    self.set_mood((*EXPRESSION_PANTING), 1.3);
                }
                Antic::Twirl => {
                    self.set_mood((*EXPRESSION_DIZZY), 0.9);
                }
                Antic::Sneeze => {
                    self.set_mood((*EXPRESSION_DIZZY), 0.5);
                }
                Antic::Taunt => {
                    self.set_mood((*EXPRESSION_MISCHIEVOUS), 1.4);
                }
                Antic::Scratch => {
                    self.set_mood((*EXPRESSION_HAPPY), 1.0);
                }
                Antic::Stomp => {
                    self.set_mood((*EXPRESSION_GRUMPY), 1.2);
                }
                Antic::Laugh => {
                    self.set_mood((*EXPRESSION_GIGGLE), 0.8);
                }
                Antic::Trip => {
                    self.set_mood((*EXPRESSION_GIGGLE), 1.0);
                }
                Antic::Boo => {
                    self.set_mood((*EXPRESSION_MISCHIEVOUS), 1.5);
                    self.goal_target = <Walker>::default();
                }
                Antic::Sleepwalk => {
                    // Bumped into something, or just came to: jerks awake.
                    self.woke_up = true;
                    self.vx = 0.0;
                    self.vy = ((-0.5 * Settings::STARTLE_SPEED) * self.size_factor);
                    self.emit_dust(self.cx, self.feet_y, 2, 0.5);
                    self.enter(Mode::Air);
                    return;
                }
                Antic::Dig => {
                    // Nothing there after all.
                    self.start_antic(Antic::Shrug);
                    return;
                }
                Antic::Strum => {
                    self.set_mood((*EXPRESSION_COOL), 1.5);
                }
                Antic::Pushups => {
                    self.set_mood((*EXPRESSION_PROUD), 1.5);
                }
                Antic::Jog => {
                    self.set_mood((*EXPRESSION_PANTING), 1.2);
                }
                Antic::Moonwalk => {
                    self.set_mood((*EXPRESSION_SMUG), 1.4);
                }
                Antic::Toot => {
                    self.set_mood((*EXPRESSION_GIGGLE), 1.0);
                }
                Antic::Peekaboo => {
                    self.set_mood((*EXPRESSION_GIGGLE), 1.0);
                }
                Antic::Flap => {
                    self.set_mood((*EXPRESSION_SILLY), 1.0);
                }
                Antic::Hide => {
                    self.set_mood((*EXPRESSION_WORRIED), 1.5);
                }
                Antic::Fret => {
                    self.set_mood((*EXPRESSION_NERVOUS), 1.5);
                }
                Antic::Faint => {
                    self.set_mood((*EXPRESSION_DIZZY), 1.2);
                }
                Antic::Daydream => {
                    self.set_mood((*EXPRESSION_CURIOUS), 1.0);
                }
                Antic::Kiss => {
                    self.set_mood((*EXPRESSION_HAPPY), 1.5);
                }
                Antic::Peck => {
                    self.set_mood((*EXPRESSION_DETERMINED), 1.2);
                    self.goal_target = <Walker>::default();
                }
                Antic::Fly => {
                    self.set_mood((*EXPRESSION_PROUD), 1.2);
                }
                Antic::Launch => {
                    self.set_mood((*EXPRESSION_DETERMINED), 1.5);
                }
                Antic::Rage => {
                    self.set_mood((*EXPRESSION_FURIOUS), 1.5);
                }
                Antic::Egg => {
                    self.set_mood((*EXPRESSION_HAPPY), 1.5);
                }
                Antic::Inflate => {
                    self.set_mood((*EXPRESSION_SMUG), 1.2);
                }
                Antic::Blast => {
                    self.set_mood((*EXPRESSION_DIZZY), 0.8);
                }
                _ => {}
            }
        }
        self.enter(Mode::Walk);
    }
    // ---- The face during an antic -----------------------------------------------------
    pub fn antic_face(
        mut self,
        target: &mut Expression,
        rate: &mut f64,
        gx: &mut f64,
        gy: &mut f64,
        at_cursor: &mut bool,
    ) {
        let mut t: f64 = self.mode_time;
        'sw1: {
            match self.antic {
                Antic::Dance => {
                    (*target) = (if ((((t / Walker::DANCE_BEAT) as i32) % 4) == 3) {
                        (*EXPRESSION_WINKING)
                    } else {
                        (*EXPRESSION_GRIN)
                    });
                    (*rate) = 18.0;
                    (*at_cursor) = true;
                }
                Antic::Twirl => {
                    (*target) = (if (t < 0.35) {
                        (*EXPRESSION_FOCUSED)
                    } else {
                        (if (t < 1.05) {
                            (*EXPRESSION_EXCITED)
                        } else {
                            (*EXPRESSION_PROUD)
                        })
                    });
                    (*rate) = 20.0;
                    if t >= 1.05 {
                        (*at_cursor) = true;
                    } else {
                        (*gx) = 0.6;
                        (*gy) = 0.0;
                    }
                }
                Antic::Point => {
                    (*target) = (if (t < 0.9) {
                        (*EXPRESSION_CURIOUS)
                    } else {
                        (*EXPRESSION_GIGGLE)
                    });
                    (*at_cursor) = true;
                }
                Antic::Yawn => {
                    (*target) = (if (t < 1.45) {
                        (*EXPRESSION_YAWN)
                    } else {
                        (*EXPRESSION_SLEEPY)
                    });
                    (*rate) = ((if (t < 1.45) { 9 } else { 6 }) as f64);
                    (*gy) = 0.3;
                }
                Antic::Doze => {
                    (*target) = (if (t < 1.45) {
                        (*EXPRESSION_YAWN)
                    } else {
                        (*EXPRESSION_ASLEEP)
                    });
                    (*rate) = 6.0;
                    (*gy) = 0.3;
                }
                Antic::Scratch => {
                    // Puzzling it over, then: an idea!
                    (*target) = (if (t < 1.55) {
                        (*EXPRESSION_THINKING)
                    } else {
                        (*EXPRESSION_SURPRISED)
                    });
                    (*rate) = ((if (t < 1.55) { 12 } else { 30 }) as f64);
                    (*gx) = (if (t < 1.55) { 0.25 } else { 0.0 });
                    (*gy) = (if (t < 1.55) { 0.35 } else { 0.1 });
                }
                Antic::Whistle => {
                    (*target) = (*EXPRESSION_WHISTLING);
                    (*rate) = 10.0;
                    (*gx) = 0.3;
                    (*gy) = 0.55;
                }
                Antic::Sneeze => {
                    (*target) = (if (t < 0.75) {
                        (*EXPRESSION_INHALE)
                    } else {
                        (if (t < 1.05) {
                            (*EXPRESSION_SNEEZE)
                        } else {
                            (*EXPRESSION_DIZZY)
                        })
                    });
                    (*rate) = ((if (t < 0.75) { 8 } else { 40 }) as f64);
                    (*gy) = (if (t < 0.75) { 0.5 } else { -0.4 });
                }
                Antic::Box => {
                    (*target) = (*EXPRESSION_DETERMINED);
                    (*gx) = 0.8;
                    (*gy) = 0.0;
                }
                Antic::Taunt => {
                    (*target) = (if (t < 1.3) {
                        (*EXPRESSION_RASPBERRY)
                    } else {
                        (*EXPRESSION_MISCHIEVOUS)
                    });
                    (*rate) = 18.0;
                    (*at_cursor) = true;
                }
                Antic::Stomp => {
                    (*target) = (*EXPRESSION_FURIOUS);
                    (*rate) = 30.0;
                    (*gx) = 0.4;
                    (*gy) = -0.3;
                }
                Antic::Flex => {
                    if t < 1.4 {
                        (*target) = (*EXPRESSION_PROUD);
                        (*at_cursor) = true;
                    } else {
                        // Admiring the near arm.
                        (*target) = (*EXPRESSION_WINKING);
                        (*gx) = 0.9;
                        (*gy) = 0.2;
                    }
                }
                Antic::Joy => {
                    (*target) = (*EXPRESSION_EXCITED);
                    (*at_cursor) = true;
                }
                Antic::Laugh => {
                    (*target) = (*EXPRESSION_LAUGH);
                    (*rate) = 20.0;
                    (*at_cursor) = true;
                }
                Antic::Hiccups => {
                    // Holding its breath between the jolts.
                    let mut jolted: bool = (Walker::hiccup_jolt(t) > 0.0);
                    (*target) = (if jolted {
                        (*EXPRESSION_SURPRISED)
                    } else {
                        (*EXPRESSION_FOCUSED)
                    });
                    (*rate) = ((if jolted { 40 } else { 12 }) as f64);
                    (*at_cursor) = true;
                }
                Antic::Shrug => {
                    (*target) = (*EXPRESSION_CURIOUS);
                    (*at_cursor) = true;
                }
                Antic::Trip => {
                    (*target) = (if (t < 0.12) {
                        (*EXPRESSION_SHOCKED)
                    } else {
                        (if (t < 1.0) {
                            (*EXPRESSION_DIZZY)
                        } else {
                            (*EXPRESSION_GIGGLE)
                        })
                    });
                    (*rate) = ((if (t < 0.12) { 40 } else { 14 }) as f64);
                    (*gx) = 0.3;
                    (*gy) = (if (t < 1.0) { -0.5 } else { 0.0 });
                }
                Antic::Boo => {
                    (*target) = (if (t < 0.6) {
                        (*EXPRESSION_EXCITED)
                    } else {
                        (*EXPRESSION_LAUGH)
                    });
                    (*rate) = 40.0;
                    if !self.goal_target.is_null() {
                        self.look_at(
                            self.goal_target.centre_x(),
                            self.goal_target.eye_y(),
                            &mut *gx,
                            &mut *gy,
                        );
                    } else {
                        (*gx) = 0.7;
                    }
                }
                Antic::Strum => {
                    // Grinning, then laughing, head down over the strings.
                    (*target) = (if ((((t / 0.8) as i32) % 2) == 0) {
                        (*EXPRESSION_GRIN)
                    } else {
                        (*EXPRESSION_LAUGH)
                    });
                    (*rate) = 18.0;
                    (*gx) = 0.3;
                    (*gy) = -0.3;
                }
                Antic::Pushups => {
                    (*target) = (*EXPRESSION_DETERMINED);
                    (*gx) = 0.2;
                    (*gy) = -0.4;
                }
                Antic::Jog => {
                    (*target) = (*EXPRESSION_FOCUSED);
                    (*gx) = 0.5;
                    (*gy) = 0.0;
                }
                Antic::Salute => {
                    (*target) = (if (t < 1.7) {
                        (*EXPRESSION_SERIOUS)
                    } else {
                        (*EXPRESSION_WINKING)
                    });
                    (*rate) = 20.0;
                    (*at_cursor) = true;
                }
                Antic::Moonwalk => {
                    (*target) = (*EXPRESSION_COOL);
                    (*rate) = 12.0;
                    (*at_cursor) = true;
                }
                Antic::Toot => {
                    // Concentrating, a little start, then the picture of innocence.
                    (*target) = (if (t < 0.55) {
                        (*EXPRESSION_FOCUSED)
                    } else {
                        (if (t < 0.8) {
                            (*EXPRESSION_SURPRISED)
                        } else {
                            (*EXPRESSION_WHISTLING)
                        })
                    });
                    (*rate) = ((if (t < 0.8) { 30 } else { 10 }) as f64);
                    (*gx) = (if (t < 0.8) {
                        0.3
                    } else {
                        (if ((t * 3.0).sin() > 0.0) { 0.7 } else { -0.7 })
                    });
                    (*gy) = (if (t < 0.8) { -0.2 } else { 0.5 });
                }
                Antic::Peekaboo => {
                    (*target) = (if (t < 1.1) {
                        (*EXPRESSION_ASLEEP)
                    } else {
                        (if (t < 1.5) {
                            (*EXPRESSION_EXCITED)
                        } else {
                            (*EXPRESSION_LAUGH)
                        })
                    });
                    (*rate) = ((if (t < 1.1) { 12 } else { 40 }) as f64);
                    (*at_cursor) = true;
                }
                Antic::Flap => {
                    (*target) = (*EXPRESSION_SILLY);
                    (*rate) = 14.0;
                    (*gx) = 0.6;
                    (*gy) = 0.0;
                }
                Antic::Hide => {
                    (*target) = (*EXPRESSION_SCARED);
                    (*rate) = 20.0;
                    if (t > 1.0) && (t < 1.5) {
                        (*at_cursor) = true;
                    } else {
                        (*gx) = 0.0;
                        (*gy) = -0.2;
                    }
                }
                Antic::Fret => {
                    // Eyes darting from side to side.
                    (*target) = (*EXPRESSION_WORRIED);
                    (*rate) = 14.0;
                    (*gx) = (if ((t * 6.0).sin() > 0.0) { 0.7 } else { -0.7 });
                    (*gy) = 0.1;
                }
                Antic::Faint => {
                    (*target) = (if (t < 0.45) {
                        (*EXPRESSION_DIZZY)
                    } else {
                        (if (t < 2.2) {
                            (*EXPRESSION_ASLEEP)
                        } else {
                            (*EXPRESSION_DIZZY)
                        })
                    });
                    (*rate) = ((if (t < 0.45) { 30 } else { 8 }) as f64);
                    (*gy) = 0.3;
                }
                Antic::Dig => {
                    (*target) = (*EXPRESSION_FOCUSED);
                    (*gx) = 0.4;
                    (*gy) = -0.6;
                }
                Antic::Sleepwalk => {
                    (*target) = (*EXPRESSION_ASLEEP);
                    (*rate) = 6.0;
                    (*gy) = -0.2;
                }
                Antic::Daydream => {
                    // Miles away, then a start.
                    (*target) = (if (t < 2.4) {
                        (*EXPRESSION_DREAMY)
                    } else {
                        (*EXPRESSION_SURPRISED)
                    });
                    (*rate) = ((if (t < 2.4) { 8 } else { 40 }) as f64);
                    (*gx) = 0.3;
                    (*gy) = (if (t < 2.4) { 0.7 } else { 0.0 });
                }
                Antic::Kiss => {
                    (*target) = (if (t < 0.9) {
                        (*EXPRESSION_SMOOCH)
                    } else {
                        (*EXPRESSION_WINKING)
                    });
                    (*rate) = 16.0;
                    (*at_cursor) = true;
                }
                Antic::Peck => {
                    (*target) = (*EXPRESSION_DETERMINED);
                    (*rate) = 24.0;
                    if !self.goal_target.is_null() {
                        self.look_at(
                            self.goal_target.centre_x(),
                            self.goal_target.eye_y(),
                            &mut *gx,
                            &mut *gy,
                        );
                    } else {
                        (*at_cursor) = true;
                    }
                }
                Antic::Fly => {
                    (*target) = (*EXPRESSION_EXCITED);
                    (*rate) = 18.0;
                    (*gx) = 0.7;
                    (*gy) = 0.1;
                }
                Antic::Launch => {
                    (*target) = (if (t < 0.35) {
                        (*EXPRESSION_FOCUSED)
                    } else {
                        (*EXPRESSION_EXCITED)
                    });
                    (*rate) = ((if (t < 0.35) { 12 } else { 30 }) as f64);
                    (*gx) = 0.6;
                    (*gy) = (if (t < 0.35) { -0.3 } else { 0.4 });
                }
                Antic::Rage => {
                    (*target) = (*EXPRESSION_FURIOUS);
                    (*rate) = 30.0;
                    (*gx) = 0.4;
                    (*gy) = -0.2;
                }
                Antic::Egg => {
                    if t < 1.4 {
                        // Straining.
                        (*target) = (*EXPRESSION_FOCUSED);
                        (*rate) = 12.0;
                        (*gx) = 0.2;
                        (*gy) = -0.5;
                    } else if t < 2.2 {
                        // Gazing at the egg.
                        (*target) = (*EXPRESSION_HAPPY);
                        (*rate) = 10.0;
                        (*gx) = 0.5;
                        (*gy) = -0.5;
                    } else {
                        (*target) = (*EXPRESSION_GRIN);
                        (*rate) = 18.0;
                        (*at_cursor) = true;
                    }
                }
                Antic::Inflate => {
                    (*target) = (if (t < 2.0) {
                        (*EXPRESSION_FOCUSED)
                    } else {
                        (*EXPRESSION_SURPRISED)
                    });
                    (*rate) = ((if (t < 2.0) { 10 } else { 30 }) as f64);
                    (*gx) = 0.3;
                    (*gy) = (if (t < 2.0) { -0.2 } else { 0.2 });
                }
                Antic::Blast => {
                    (*target) = (if (t < 0.6) {
                        (*EXPRESSION_DETERMINED)
                    } else {
                        (if (t < 1.0) {
                            (*EXPRESSION_SHOCKED)
                        } else {
                            (*EXPRESSION_DIZZY)
                        })
                    });
                    (*rate) = ((if (t < 0.6) { 14 } else { 40 }) as f64);
                    (*gx) = 0.3;
                    (*gy) = (if (t < 1.0) { 0.0 } else { -0.4 });
                }
                _ => {}
            }
        }
    }
    // ---- Which way it turns ----------------------------------------------------------
    pub fn antic_yaw(mut self, snap: &mut bool) -> f64 {
        (*snap) = false;
        let mut t: f64 = self.mode_time;
        'sw1: {
            match self.antic {
                Antic::Dance => {
                    return Walker::dance_yaw(t, 0.0);
                }
                Antic::Twirl => {
                    if t < 0.35 {
                        return 58.0;
                    }
                    if t >= 1.05 {
                        return 10.0;
                    }
                    // One full turn, taken directly rather than eased; it
                    // comes back round to where it started.
                    (*snap) = true;
                    let mut y: f64 = (58.0 + (360.0 * Walker::ease(((t - 0.35) / 0.7))));
                    return (if (y >= 417.5) { 58.0 } else { y });
                }
                Antic::Point => {
                    return 15.0;
                }
                Antic::Scratch => {
                    return 12.0;
                }
                Antic::Sneeze => {
                    return 30.0;
                }
                Antic::Box => {
                    return 45.0;
                }
                Antic::Stomp => {
                    return 15.0;
                }
                Antic::Flex => {
                    return ((if (t < 1.4) { 20 } else { 38 }) as f64);
                }
                Antic::Shrug => {
                    return 10.0;
                }
                Antic::Trip => {
                    return ((if (t < 1.0) { 58 } else { 30 }) as f64);
                }
                Antic::Boo => {
                    return 30.0;
                }
                Antic::Strum => {
                    return 25.0;
                }
                Antic::Pushups => {
                    return 70.0; // side on, so the press-ups read
                }
                Antic::Jog => {
                    return 35.0;
                }
                Antic::Salute => {
                    return 5.0;
                }
                Antic::Moonwalk => {
                    return 58.0;
                }
                Antic::Toot => {
                    return ((if (t < 0.8) {
                        40
                    } else {
                        (if (t < 1.5) { -30 } else { 30 })
                    }) as f64); // glancing about afterwards
                }
                Antic::Flap => {
                    return 35.0;
                }
                Antic::Fret => {
                    return 10.0;
                }
                Antic::Faint => {
                    return ((if (t < 0.45) { 20 } else { 0 }) as f64);
                }
                Antic::Dig => {
                    return 55.0;
                }
                Antic::Sleepwalk => {
                    return 58.0;
                }
                Antic::Daydream => {
                    return 20.0;
                }
                Antic::Kiss => {
                    return 10.0;
                }
                Antic::Peck => {
                    return ((if (t < 1.4) { 45 } else { 20 }) as f64);
                }
                Antic::Fly => {
                    return 64.0;
                }
                Antic::Launch => {
                    return 58.0;
                }
                Antic::Rage => {
                    return 15.0;
                }
                Antic::Egg => {
                    return 30.0;
                }
                Antic::Inflate => {
                    return 10.0;
                }
                Antic::Blast => {
                    return 5.0;
                }
                _ => {
                    return 0.0; // yawning, dozing, whistling, taunting, laughing, hiccuping, peekaboo, hiding, eyebrows: facing you
                }
            }
        }
        #[allow(unreachable_code)]
        unreachable!()
    }
    // A dancer sways from side to side every two beats.
    pub fn dance_yaw(mut t: f64, mut centre: f64) -> f64 {
        return (centre
            + ((if ((((t / Walker::DANCE_BEAT) * std::f64::consts::PI) / 2.0).sin() >= 0.0) {
                25
            } else {
                -25
            }) as f64));
    }
    // ---- The body during an antic -----------------------------------------------------
    pub fn antic_pose(mut self, pose: &mut Pose) {
        let mut t: f64 = self.mode_time;
        'sw1: {
            match self.antic {
                Antic::Dance => {
                    self.dance_pose(&mut *pose, t);
                }
                Antic::Twirl => {
                    if t < 0.35 {
                        // Arms out, a little crouch to wind up.
                        let mut e: f64 = Walker::ease((t / 0.35));
                        (*pose).arm_near = Walker::q2((70.0 * e));
                        (*pose).arm_far = -(Walker::q2((70.0 * e)));
                        (*pose).stretch_y = (1.0 - (0.05 * e));
                        (*pose).stretch_x = (1.0 + (0.03 * e));
                    } else if t < 1.05 {
                        (*pose).arm_near = 95.0;
                        (*pose).arm_far = -95.0;
                        (*pose).stretch_y = 1.03;
                    } else {
                        // Ta-da.
                        let mut e: f64 = Walker::ease(rt::min_f64(1.0, ((t - 1.05) / 0.25)));
                        (*pose).arm_near = Walker::q2((95.0 + (60.0 * e)));
                        (*pose).arm_far = -(Walker::q2((95.0 + (60.0 * e))));
                        (*pose).fore_near = -(Walker::q2((20.0 * e)));
                        (*pose).fore_far = Walker::q2((20.0 * e));
                        (*pose).stretch_y = (1.0 + (0.04 * e));
                        (*pose).body_dy = -(Walker::quantise((0.25 * e)));
                    }
                }
                Antic::Point => {
                    (*pose).arm_near =
                        self.aim_at(((self.cursor_x) as f64), ((self.cursor_y) as f64));
                    (*pose).point_near = true;
                    if t > 1.0 {
                        (*pose).body_dy = Walker::quantise(
                            ((0.25
                                * (1.0
                                    - (((2.0 * std::f64::consts::PI) * (t - 1.0)) / 0.22).cos()))
                                / 2.0),
                        ); // giggling
                    }
                }
                Antic::Yawn => {
                    Walker::stretch_pose(&mut *pose, Walker::yawn_lift(t));
                }
                Antic::Doze => {
                    if t < 2.1 {
                        Walker::stretch_pose(&mut *pose, Walker::yawn_lift(t));
                    } else {
                        // Slumped, nodding further off every so often and
                        // jerking back up, with letters floating away.
                        let mut n: f64 = (((t - 2.1) % 1.3) / 1.3);
                        let mut droop: f64 = (if (n < 0.35) {
                            (n / 0.35)
                        } else {
                            (if (n < 0.42) {
                                (1.0 - ((n - 0.35) / 0.07))
                            } else {
                                0.0
                            })
                        });
                        (*pose).body_dy = Walker::quantise((0.45 + (0.6 * droop)));
                        (*pose).lean = (0.2 + ((droop * 4.0).floor() / 16.0));
                        (*pose).zzz = true;
                        (*pose).decor_phase = Walker::q16((((t - 2.1) / 1.6) % 1.0));
                    }
                }
                Antic::Scratch => {
                    (*pose).arm_near = 170.0;
                    (*pose).fore_near = (-50.0
                        + (if ((t > 0.3) && (t < 1.55)) {
                            Walker::q2((7.0 * (((t * 2.0) * std::f64::consts::PI) * 3.5).sin()))
                        } else {
                            0.0
                        }));
                    (*pose).bubble = (t >= 1.55);
                    self.breathe(&mut *pose);
                }
                Antic::Whistle => {
                    (*pose).note = true;
                    (*pose).decor_phase = Walker::q16(((t / 0.9) % 1.0));
                    (*pose).lean = (rt::round_even(((0.15 * (t * 2.2).sin()) * 8.0)) / 8.0); // rocking on its heels
                    self.breathe(&mut *pose);
                }
                Antic::Sneeze => {
                    if t < 0.75 {
                        // The breath in: up on tiptoe, head back.
                        let mut e: f64 = Walker::ease((t / 0.75));
                        (*pose).stretch_y = (1.0 + (0.08 * e));
                        (*pose).body_dy = -(Walker::quantise((0.4 * e)));
                        (*pose).lean = (-(rt::round_even(((0.3 * e) * 8.0))) / 8.0);
                        (*pose).arm_near = Walker::q2((25.0 * e));
                        (*pose).arm_far = -(Walker::q2((25.0 * e)));
                    } else if t < 0.95 {
                        // The sneeze: folded forward, squashed.
                        (*pose).lean = 0.8;
                        (*pose).stretch_y = 0.9;
                        (*pose).stretch_x = 1.1;
                        (*pose).body_dy = 0.5;
                        (*pose).arm_near = 50.0;
                        (*pose).arm_far = -50.0;
                        (*pose).fore_near = 40.0;
                        (*pose).fore_far = -40.0;
                    } else {
                        let mut e: f64 = (1.0 - Walker::ease(rt::min_f64(1.0, ((t - 0.95) / 0.4))));
                        (*pose).lean = (rt::round_even(((0.8 * e) * 8.0)) / 8.0);
                        (*pose).body_dy = Walker::quantise((0.5 * e));
                        (*pose).stretch_y = (1.0 - (0.1 * e));
                        (*pose).stretch_x = (1.0 + (0.1 * e));
                        (*pose).arm_near = Walker::q2((50.0 * e));
                        (*pose).arm_far = -(Walker::q2((50.0 * e)));
                        (*pose).fore_near = Walker::q2((40.0 * e));
                        (*pose).fore_far = -(Walker::q2((40.0 * e)));
                    }
                }
                Antic::Box => {
                    // Fists up by the face, bouncing on the toes; the arms
                    // take turns shooting out straight and coming back.
                    let mut beat: f64 = 0.3;
                    let mut b: f64 = (t / beat);
                    let mut n: i32 = ((b) as i32);
                    let mut f: f64 = (b - ((n) as f64));
                    let mut punch: f64 = (if (t < 1.9) {
                        (if (f < 0.5) {
                            (f / 0.5)
                        } else {
                            (1.0 - ((f - 0.5) / 0.5))
                        })
                    } else {
                        0.0
                    });
                    let mut near_punch: bool = ((n % 2) == 0);
                    (*pose).arm_near =
                        Walker::q2((60.0 + (if near_punch { (35.0 * punch) } else { 0.0 })));
                    (*pose).fore_near =
                        Walker::q2((100.0 - (if near_punch { (100.0 * punch) } else { 0.0 })));
                    (*pose).arm_far =
                        Walker::q2((60.0 + (if near_punch { 0.0 } else { (35.0 * punch) })));
                    (*pose).fore_far =
                        Walker::q2((100.0 - (if near_punch { 0.0 } else { (100.0 * punch) })));
                    (*pose).lean = 0.3;
                    (*pose).stretch_x = 1.02;
                    (*pose).body_dy = Walker::quantise(
                        ((0.25 * (1.0 - ((b * 2.0) * std::f64::consts::PI).cos())) / 2.0),
                    );
                }
                Antic::Taunt => {
                    // One hand beckoning, the other on the hip, hips wiggling.
                    (*pose).arm_near = 75.0;
                    (*pose).fore_near = Walker::q2(
                        (30.0 + (35.0 * (((t * 2.0) * std::f64::consts::PI) * 2.5).sin())),
                    );
                    (*pose).arm_far = -40.0;
                    (*pose).fore_far = 85.0;
                    (*pose).stretch_x = (1.0
                        + ((0.03
                            * rt::round_even(
                                ((((t * 2.0) * std::f64::consts::PI) * 2.5).sin() * 2.0),
                            ))
                            / 2.0));
                }
                Antic::Stomp => {
                    // Knees up, then down hard - twice. Fists clenched at the sides.
                    let mut lift: f64 = (if (t < 0.25) {
                        (t / 0.25)
                    } else {
                        (if ((t >= 0.4) && (t < 0.65)) {
                            ((t - 0.4) / 0.25)
                        } else {
                            0.0
                        })
                    });
                    (*pose).leg_tuck = (rt::round_even(((0.5 * lift) * 8.0)) / 8.0);
                    (*pose).body_dy = -(Walker::quantise((0.4 * lift)));
                    (*pose).stretch_y = (1.0 + (0.05 * lift));
                    (*pose).arm_near = 25.0;
                    (*pose).arm_far = -25.0;
                    (*pose).fore_near = 10.0;
                    (*pose).fore_far = -10.0;
                    (*pose).lean = 0.15;
                }
                Antic::Flex => {
                    // Both arms curl up, then pump while it admires them.
                    let mut e: f64 = Walker::ease(rt::min_f64(1.0, (t / 0.5)));
                    let mut pump: f64 = (if (t > 0.5) {
                        Walker::q2((8.0 * (((t * 2.0) * std::f64::consts::PI) * 2.0).sin()))
                    } else {
                        0.0
                    });
                    (*pose).arm_near = Walker::q2((95.0 * e));
                    (*pose).arm_far = -(Walker::q2((95.0 * e)));
                    (*pose).fore_near = (Walker::q2((110.0 * e)) + pump);
                    (*pose).fore_far = -(Walker::q2((110.0 * e)) + pump);
                    (*pose).lean = -0.3;
                    (*pose).stretch_x = 1.06;
                }
                Antic::Joy => {}
                Antic::Laugh => {
                    self.laugh_pose(&mut *pose, t, false);
                }
                Antic::Hiccups => {
                    // Each hiccup is a jolt: up on the toes, arms twitching out.
                    let mut jolt: f64 = Walker::hiccup_jolt(t);
                    (*pose).stretch_y = (1.0 + (0.09 * jolt));
                    (*pose).stretch_x = (1.0 - (0.05 * jolt));
                    (*pose).body_dy = -(Walker::quantise((0.5 * jolt)));
                    (*pose).arm_near = Walker::q2((25.0 * jolt));
                    (*pose).arm_far = -(Walker::q2((25.0 * jolt)));
                    if jolt <= 0.0 {
                        self.breathe(&mut *pose);
                    }
                }
                Antic::Shrug => {
                    // Arms out, shoulders up: oh well. Then the arms drop.
                    let mut e: f64 = (if (t < 0.7) {
                        1.0
                    } else {
                        (1.0 - Walker::ease(rt::min_f64(1.0, ((t - 0.7) / 0.4))))
                    });
                    (*pose).arm_near = Walker::q2((60.0 * e));
                    (*pose).arm_far = -(Walker::q2((60.0 * e)));
                    (*pose).fore_near = Walker::q2((50.0 * e));
                    (*pose).fore_far = -(Walker::q2((50.0 * e)));
                    (*pose).stretch_y = (1.0 + (0.03 * e));
                }
                Antic::Trip => {
                    if t < 0.12 {
                        // Pitching forward mid-stride, arms out to catch itself.
                        self.walk(&mut *pose);
                        (*pose).lean = 0.6;
                        (*pose).arm_near = 70.0;
                        (*pose).arm_far = 70.0;
                        (*pose).fore_near = 20.0;
                        (*pose).fore_far = 20.0;
                    } else if t < 1.0 {
                        // Flat out.
                        (*pose).lean = 1.0;
                        (*pose).stretch_y = 0.72;
                        (*pose).stretch_x = 1.18;
                        (*pose).body_dy = 0.9;
                        (*pose).arm_near = 80.0;
                        (*pose).arm_far = 80.0;
                        (*pose).fore_near = 10.0;
                        (*pose).fore_far = 10.0;
                    } else {
                        // Picking itself up.
                        let mut e: f64 = (1.0 - Walker::ease(rt::min_f64(1.0, ((t - 1.0) / 0.6))));
                        (*pose).lean = (rt::round_even((e * 8.0)) / 8.0);
                        (*pose).stretch_y = (1.0 - (0.28 * e));
                        (*pose).stretch_x = (1.0 + (0.18 * e));
                        (*pose).body_dy = Walker::quantise((0.9 * e));
                        (*pose).arm_near = Walker::q2((80.0 * e));
                        (*pose).arm_far = Walker::q2((80.0 * e));
                        (*pose).fore_near = Walker::q2((10.0 * e));
                        (*pose).fore_far = Walker::q2((10.0 * e));
                    }
                }
                Antic::Boo => {
                    if t < 0.6 {
                        // The shout: lunging, arms thrown up, a "!" overhead.
                        (*pose).arm_near = 150.0;
                        (*pose).arm_far = -150.0;
                        (*pose).fore_near = -20.0;
                        (*pose).fore_far = 20.0;
                        (*pose).lean = 0.3;
                        (*pose).stretch_y = 1.06;
                        (*pose).bubble = true;
                    } else {
                        self.laugh_pose(&mut *pose, (t - 0.6), false);
                    }
                }
                Antic::Strum => {
                    // Far hand up on the neck, near hand strumming over the
                    // belly, head banging on the beat, notes flying.
                    let mut b: f64 = (t / 0.4);
                    let mut bang: f64 = ((1.0 - ((b * 2.0) * std::f64::consts::PI).cos()) / 2.0);
                    (*pose).arm_far = 45.0;
                    (*pose).fore_far = 50.0;
                    (*pose).arm_near = 30.0;
                    (*pose).fore_near =
                        Walker::q2((70.0 + (30.0 * ((b * 4.0) * std::f64::consts::PI).sin())));
                    (*pose).lean = (rt::round_even(((0.35 * bang) * 8.0)) / 8.0);
                    (*pose).body_dy = Walker::quantise((0.35 * bang));
                    (*pose).stretch_x = 1.02;
                    (*pose).note = true;
                    (*pose).decor_phase = Walker::q16(((b / 2.0) % 1.0));
                }
                Antic::Pushups => {
                    // Down flat, then pressing up and sinking down, twice a second.
                    let mut n: f64 = Walker::ease(rt::min_f64(1.0, (t / 0.3)));
                    let mut up: f64 = (if (t < 0.3) {
                        0.0
                    } else {
                        (rt::round_even(
                            (((1.0 - ((((t - 0.3) / 0.6) * 2.0) * std::f64::consts::PI).cos())
                                / 2.0)
                                * 8.0),
                        ) / 8.0)
                    });
                    (*pose).lean = (rt::round_even((n * 8.0)) / 8.0);
                    (*pose).stretch_y = ((1.0 - (0.28 * n)) + ((0.1 * up) * n));
                    (*pose).stretch_x = (1.0 + (0.18 * n));
                    (*pose).body_dy = Walker::quantise(((0.9 * n) - ((0.4 * up) * n)));
                    (*pose).arm_near = Walker::q2((80.0 * n));
                    (*pose).arm_far = Walker::q2((80.0 * n));
                    (*pose).fore_near = Walker::q2(((10.0 + (50.0 * (1.0 - up))) * n));
                    (*pose).fore_far = (*pose).fore_near;
                }
                Antic::Jog => {
                    // Running on the spot: feet stepping, arms pumping, bobbing.
                    let mut b: f64 = (t / 0.45);
                    let mut s: f64 = ((b * 2.0) * std::f64::consts::PI).sin();
                    (*pose).walking = true;
                    (*pose).walk_phase = ((b * 12.0).floor() / 12.0);
                    (*pose).body_dy = Walker::quantise(
                        ((0.3 * (1.0 - ((b * 4.0) * std::f64::consts::PI).cos())) / 2.0),
                    );
                    (*pose).arm_near = Walker::q2((40.0 + (25.0 * s)));
                    (*pose).fore_near = 95.0;
                    (*pose).arm_far = -(Walker::q2((40.0 - (25.0 * s))));
                    (*pose).fore_far = -95.0;
                    (*pose).lean = 0.15;
                }
                Antic::Salute => {
                    // Snaps to attention, chest out, the hand up at the brow.
                    let mut e: f64 = Walker::ease(rt::min_f64(1.0, (t / 0.25)));
                    (*pose).stretch_y = (1.0 + (0.04 * e));
                    (*pose).lean = (-(rt::round_even(((0.15 * e) * 8.0))) / 8.0);
                    (*pose).arm_near = Walker::q2((120.0 * e));
                    (*pose).fore_near = Walker::q2((100.0 * e));
                }
                Antic::Moonwalk => {
                    // Leaning back, feet stepping forward while it slides back.
                    (*pose).walking = (t > 0.3);
                    (*pose).walk_phase = self.walk_phase;
                    (*pose).lean = -0.25;
                    (*pose).arm_near = 20.0;
                    (*pose).arm_far = -20.0;
                    (*pose).fore_near = -30.0;
                    (*pose).fore_far = 30.0;
                    (*pose).body_dy = Walker::quantise(
                        ((0.2 * (1.0 - ((self.walk_phase * 4.0) * std::f64::consts::PI).cos()))
                            / 2.0),
                    );
                }
                Antic::Toot => {
                    if t < 0.55 {
                        // A little squat and a look of concentration.
                        let mut e: f64 = Walker::ease((t / 0.55));
                        (*pose).stretch_y = (1.0 - (0.06 * e));
                        (*pose).stretch_x = (1.0 + (0.04 * e));
                        (*pose).body_dy = Walker::quantise((0.3 * e));
                        (*pose).lean = (rt::round_even(((0.2 * e) * 8.0)) / 8.0);
                    } else {
                        // Hands behind the back, rocking on the heels: wasn't me.
                        (*pose).arm_near = -25.0;
                        (*pose).arm_far = 25.0;
                        (*pose).fore_near = -60.0;
                        (*pose).fore_far = 60.0;
                        (*pose).lean = (-(rt::round_even(((0.15 * (t * 2.2).sin()) * 8.0))) / 8.0);
                        self.breathe(&mut *pose);
                    }
                }
                Antic::Peekaboo => {
                    if t < 1.1 {
                        Walker::cover_eyes(&mut *pose, 0.0);
                    } else {
                        // Peekaboo! Arms flung wide, up on the toes.
                        let mut e: f64 = Walker::ease(rt::min_f64(1.0, ((t - 1.1) / 0.15)));
                        (*pose).arm_near = Walker::q2((178.0 - (58.0 * e)));
                        (*pose).arm_far = -((*pose).arm_near);
                        (*pose).fore_near = Walker::q2((92.0 - (92.0 * e)));
                        (*pose).fore_far = -((*pose).fore_near);
                        (*pose).stretch_y = (1.0 + (0.05 * e));
                        (*pose).body_dy = -(Walker::quantise((0.2 * e)));
                        (*pose).bubble = (t < 1.6);
                    }
                }
                Antic::Flap => {
                    // Elbows out and flapping, head jutting on each beat, strutting.
                    let mut b: f64 = (t / 0.3);
                    let mut n: i32 = ((b) as i32);
                    let mut f: f64 = (b - ((n) as f64));
                    let mut flap: f64 = (if (f < 0.5) {
                        (f / 0.5)
                    } else {
                        (1.0 - ((f - 0.5) / 0.5))
                    });
                    (*pose).arm_near = Walker::q2((50.0 + (30.0 * flap)));
                    (*pose).fore_near = Walker::q2((150.0 - (20.0 * flap)));
                    (*pose).arm_far = -((*pose).arm_near);
                    (*pose).fore_far = -((*pose).fore_near);
                    (*pose).lean = (if ((n % 2) == 0) { 0.3 } else { 0.1 });
                    (*pose).walking = true;
                    (*pose).walk_phase = ((b * 6.0).floor() / 24.0);
                    (*pose).body_dy = Walker::quantise((0.2 * flap));
                }
                Antic::Hide => {
                    // Hands clapped over the eyes, shaking; a peek through the fingers.
                    Walker::cover_eyes(
                        &mut *pose,
                        ((if ((t * 28.0).sin() > 0.0) { 3 } else { -3 }) as f64),
                    ); // shaking: two positions, nine times a second
                    if (t > 1.0) && (t < 1.5) {
                        (*pose).arm_near = 140.0;
                        (*pose).fore_near = 60.0;
                    }
                    (*pose).stretch_y = 0.96;
                    (*pose).body_dy = 0.35;
                    (*pose).lean = 0.1;
                }
                Antic::Fret => {
                    // Nibbling the fingers of one hand, the other arm wrapped round the belly.
                    (*pose).arm_near = 25.0;
                    (*pose).fore_near =
                        ((170 + (if ((t * 20.0).sin() > 0.0) { 4 } else { 0 })) as f64);
                    (*pose).arm_far = 40.0;
                    (*pose).fore_far = 100.0;
                    (*pose).lean = 0.1;
                    self.breathe(&mut *pose);
                }
                Antic::Faint => {
                    if t < 0.45 {
                        // Swaying, arms out for balance, then over it goes.
                        (*pose).lean = (rt::round_even(((0.3 * (t * 14.0).sin()) * 8.0)) / 8.0);
                        (*pose).arm_near = 50.0;
                        (*pose).arm_far = -50.0;
                        (*pose).fore_near = 20.0;
                        (*pose).fore_far = -20.0;
                    } else if t < 2.2 {
                        // Flat on its back, arms spread.
                        (*pose).lean = -1.0;
                        (*pose).stretch_y = 0.72;
                        (*pose).stretch_x = 1.18;
                        (*pose).body_dy = 0.9;
                        (*pose).arm_near = 85.0;
                        (*pose).arm_far = -85.0;
                    } else {
                        // Coming round and sitting up.
                        let mut e: f64 = (1.0 - Walker::ease(rt::min_f64(1.0, ((t - 2.2) / 0.6))));
                        (*pose).lean = (-(rt::round_even((e * 8.0))) / 8.0);
                        (*pose).stretch_y = (1.0 - (0.28 * e));
                        (*pose).stretch_x = (1.0 + (0.18 * e));
                        (*pose).body_dy = Walker::quantise((0.9 * e));
                        (*pose).arm_near = Walker::q2((85.0 * e));
                        (*pose).arm_far = -(Walker::q2((85.0 * e)));
                    }
                }
                Antic::Dig => {
                    // Bent double, both arms scooping at the ground in front.
                    let mut f: f64 = ((t / 0.5) % 1.0);
                    let mut scoop: f64 = (if (f < 0.4) {
                        (f / 0.4)
                    } else {
                        (1.0 - ((f - 0.4) / 0.6))
                    });
                    (*pose).lean = 0.6;
                    (*pose).stretch_y = 0.9;
                    (*pose).stretch_x = 1.05;
                    (*pose).body_dy = Walker::quantise((0.5 + (0.2 * scoop)));
                    (*pose).arm_near = Walker::q2((70.0 - (30.0 * scoop)));
                    (*pose).arm_far = (*pose).arm_near;
                    (*pose).fore_near = Walker::q2((20.0 + (50.0 * scoop)));
                    (*pose).fore_far = (*pose).fore_near;
                }
                Antic::Sleepwalk => {
                    // Arms straight out ahead, feet shuffling, eyes shut, Zzz.
                    (*pose).arm_near = 90.0;
                    (*pose).arm_far = 90.0;
                    (*pose).fore_near = -5.0;
                    (*pose).fore_far = 5.0;
                    (*pose).lean = 0.2;
                    (*pose).walking = (t > 1.0);
                    (*pose).walk_phase = self.walk_phase;
                    (*pose).body_dy =
                        Walker::quantise((0.4 + ((0.15 * (1.0 + (self.time * 2.6).sin())) / 2.0)));
                    (*pose).zzz = true;
                    (*pose).decor_phase = Walker::q16(((t / 1.6) % 1.0));
                }
                Antic::Daydream => {
                    if t < 2.4 {
                        // Hands clasped behind, head back, swaying.
                        (*pose).arm_near = -20.0;
                        (*pose).arm_far = 20.0;
                        (*pose).fore_near = -70.0;
                        (*pose).fore_far = 70.0;
                        (*pose).lean =
                            -(Walker::quantise((0.1 + ((0.15 * (1.0 + (t * 1.3).sin())) / 2.0))));
                        self.breathe(&mut *pose);
                    } else {
                        // Snapping out of it with a little jolt.
                        let mut e: f64 = (1.0 - Walker::ease(rt::min_f64(1.0, ((t - 2.4) / 0.3))));
                        (*pose).stretch_y = (1.0 + (0.05 * e));
                        (*pose).body_dy = -(Walker::quantise((0.3 * e)));
                        (*pose).arm_near = Walker::q2((30.0 * e));
                        (*pose).arm_far = -(Walker::q2((30.0 * e)));
                        (*pose).bubble = (t < 2.7);
                    }
                }
                Antic::Kiss => {
                    if t < 0.9 {
                        // Hand to the lips.
                        let mut e: f64 = Walker::ease(rt::min_f64(1.0, (t / 0.3)));
                        (*pose).arm_near = Walker::q2((25.0 * e));
                        (*pose).fore_near = Walker::q2((165.0 * e));
                        (*pose).lean = (rt::round_even(((0.1 * e) * 8.0)) / 8.0);
                    } else {
                        // ...and blown towards the cursor, a heart floating off.
                        let mut e: f64 = Walker::ease(rt::min_f64(1.0, ((t - 0.9) / 0.25)));
                        let mut aim: f64 =
                            self.aim_at(((self.cursor_x) as f64), ((self.cursor_y) as f64));
                        (*pose).arm_near = Walker::q2((25.0 + ((aim - 25.0) * e)));
                        (*pose).fore_near = Walker::q2((165.0 * (1.0 - e)));
                        (*pose).heart = true;
                        (*pose).decor_phase = Walker::q16(rt::min_f64(0.999, ((t - 0.9) / 1.3)));
                        (*pose).lean = -0.1;
                    }
                }
                Antic::Peck => {
                    self.peck_pose(&mut *pose, t);
                }
                Antic::Fly => {
                    self.fly_pose(&mut *pose, t);
                }
                Antic::Launch => {
                    self.launch_pose(&mut *pose, t);
                }
                Antic::Rage => {
                    self.rage_pose(&mut *pose, t);
                }
                Antic::Egg => {
                    self.egg_pose(&mut *pose, t);
                }
                Antic::Inflate => {
                    self.inflate_pose(&mut *pose, t);
                }
                Antic::Blast => {
                    self.blast_pose(&mut *pose, t);
                }
                _ => {}
            }
        }
    }
    // The birds'. Wings are the arms: raised wings read on every figure,
    // and the Bird painter turns them into feathers.
    // Two hard beak jabs at whoever it hopped up to (or your cursor):
    // all lunge and squash, beak first. No limbs to pose with.
    pub fn peck_pose(mut self, pose: &mut Pose, mut t: f64) {
        if t < 0.5 {
            // Rocking back, beak up for the first jab.
            let mut e: f64 = Walker::ease((t / 0.5));
            (*pose).lean = (rt::round_even(((0.35 * e) * 8.0)) / 8.0);
            (*pose).body_dy = -(Walker::quantise((0.3 * e)));
        } else if t < 1.4 {
            let mut f: f64 = (((t - 0.5) / 0.45) % 1.0);
            let mut jab: f64 = (if (f < 0.5) {
                (f / 0.5)
            } else {
                (1.0 - ((f - 0.5) / 0.5))
            });
            (*pose).lean = (rt::round_even(((0.35 + (0.3 * jab)) * 8.0)) / 8.0);
            (*pose).body_dy = Walker::quantise((0.3 * jab));
        } else {
            // Settling back down, pleased.
            let mut e: f64 = (1.0 - Walker::ease(rt::min_f64(1.0, ((t - 1.4) / 0.4))));
            (*pose).lean = (rt::round_even(((0.35 * e) * 8.0)) / 8.0);
            (*pose).body_dy = Walker::quantise((0.3 * e));
        }
    }
    // Dashing off along the ground: leaning forward, bouncing hard,
    // stretched long like a rolled ball.
    pub fn fly_pose(mut self, pose: &mut Pose, mut t: f64) {
        (*pose).lean = 0.45;
        (*pose).walking = true;
        (*pose).walk_phase = self.walk_phase;
        (*pose).body_dy = Walker::quantise(
            (0.2 + ((0.25 * (1.0 - (((t * 2.0) * std::f64::consts::PI) * 3.5).cos())) / 2.0)),
        );
        (*pose).stretch_y = 1.03;
        (*pose).stretch_x = 0.96;
    }
    // Slingshot launch: pulled back into a trembling squash.
    // The actual spring leaves in StepAntic (a real jump across the sky).
    pub fn launch_pose(mut self, pose: &mut Pose, mut t: f64) {
        if t < 0.35 {
            let mut e: f64 = Walker::ease((t / 0.35));
            (*pose).stretch_y = (1.0 - (0.22 * e));
            (*pose).stretch_x = (1.0 + (0.16 * e));
            (*pose).body_dy = Walker::quantise((0.8 * e));
            (*pose).lean = (-(rt::round_even(((0.3 * e) * 8.0))) / 8.0);
        }
    }
    // Bristling rage: shaking and stamping in place, squash wobbling.
    pub fn rage_pose(mut self, pose: &mut Pose, mut t: f64) {
        (*pose).lean = (rt::round_even(((0.15 * (t * 14.0).sin()) * 8.0)) / 8.0);
        (*pose).body_dy = Walker::quantise(
            ((0.25 * (1.0 - (((t * 2.0) * std::f64::consts::PI) / 0.45).cos())) / 2.0),
        );
        (*pose).stretch_x = 1.08;
        (*pose).stretch_y = 0.94;
    }
    // Laying an egg: hunched and straining, then gazing down at it.
    pub fn egg_pose(mut self, pose: &mut Pose, mut t: f64) {
        if t < 1.4 {
            (*pose).lean = 0.35;
            (*pose).stretch_y = 0.92;
            (*pose).stretch_x = 1.08;
            (*pose).body_dy = Walker::quantise((0.6 + ((0.15 * (1.0 - (t * 9.0).cos())) / 2.0)));
        } else {
            (*pose).egg = true;
            (*pose).lean = 0.25;
            (*pose).body_dy = Walker::quantise(0.4);
        }
    }
    // Blowing up huge like Bubbles: swelling, wobbling, then letting go.
    pub fn inflate_pose(mut self, pose: &mut Pose, mut t: f64) {
        let mut up: f64 = (if (t < 2.0) {
            Walker::ease((t / 2.0))
        } else {
            (1.0 - Walker::ease(rt::min_f64(1.0, ((t - 2.0) / 0.8))))
        });
        (*pose).puff_up = (rt::round_even((up * 8.0)) / 8.0);
        (*pose).lean = (rt::round_even((((0.1 * (t * 6.0).sin()) * up) * 8.0)) / 8.0);
        (*pose).body_dy = Walker::quantise((0.3 * up));
        (*pose).stretch_x = (1.0 + (0.1 * up));
    }
    // Bomb goes off: squashed down while the flash fires, then wobbling
    // upright, dizzy, ears ringing.
    pub fn blast_pose(mut self, pose: &mut Pose, mut t: f64) {
        if t < 0.6 {
            // The fuse is lit: hunkering down, trembling.
            let mut e: f64 = Walker::ease((t / 0.6));
            (*pose).stretch_y = (1.0 - (0.18 * e));
            (*pose).stretch_x = (1.0 + (0.12 * e));
            (*pose).body_dy = Walker::quantise((0.7 * e));
        } else if t < 1.0 {
            (*pose).blast_flash = true;
            (*pose).stretch_y = 1.12;
            (*pose).stretch_x = 0.9;
            (*pose).body_dy = -(Walker::quantise(0.4));
            (*pose).bubble = true;
        } else {
            let mut e: f64 = (1.0 - Walker::ease(rt::min_f64(1.0, ((t - 1.0) / 0.6))));
            (*pose).lean = (rt::round_even((((0.25 * (t * 12.0).sin()) * e) * 8.0)) / 8.0);
            (*pose).body_dy = Walker::quantise((0.5 * e));
        }
    }
    // Both hands clapped over the goggles, shaking by `tremble` degrees.
    pub fn cover_eyes(pose: &mut Pose, mut tremble: f64) {
        (*pose).arm_near = (178.0 + tremble);
        (*pose).arm_far = (-178.0 - tremble);
        (*pose).fore_near = 92.0;
        (*pose).fore_far = -92.0;
    }
    // A jig on the spot: bouncing on the beat, feet stepping, arms
    // pumping up and down in turn, a note floating off every two beats.
    pub fn dance_pose(mut self, pose: &mut Pose, mut t: f64) {
        let mut b: f64 = (t / Walker::DANCE_BEAT);
        let mut bounce: f64 = ((1.0 - ((b * 2.0) * std::f64::consts::PI).cos()) / 2.0); // down on the beat, up between
        let mut side: f64 = (b * std::f64::consts::PI).sin(); // which arm is up, swapping each beat
        (*pose).body_dy = Walker::quantise((0.5 * bounce));
        (*pose).arm_near = Walker::q2((60.0 + (60.0 * side)));
        (*pose).arm_far = -(Walker::q2((60.0 - (60.0 * side))));
        (*pose).fore_near = 35.0;
        (*pose).fore_far = -35.0;
        (*pose).walking = true;
        (*pose).walk_phase = ((b * 12.0).floor() / 24.0);
        (*pose).stretch_y = (1.0 + ((0.03 * rt::round_even((side * 2.0))) / 2.0));
        (*pose).note = true;
        (*pose).decor_phase = Walker::q16(((b / 2.0) % 1.0));
    }
    // Shaking with laughter, hands on the belly - or one of them pointing.
    pub fn laugh_pose(mut self, pose: &mut Pose, mut t: f64, mut pointing: bool) {
        (*pose).body_dy = Walker::quantise(
            ((0.4 * (1.0 - (((2.0 * std::f64::consts::PI) * t) / 0.26).cos())) / 2.0),
        );
        (*pose).lean = (if (t < 1.1) { -0.25 } else { 0.3 });
        (*pose).arm_far = -20.0;
        (*pose).fore_far = 80.0;
        if pointing {
            (*pose).arm_near = self.aim_at(self.react_x, (self.feet_y - ((9 * self.scale) as f64)));
            (*pose).point_near = true;
        } else {
            (*pose).arm_near = 20.0;
            (*pose).fore_near = -80.0;
        }
    }
    // Stretching up and yawning: the arms rise over the first 0.8 s,
    // hold, then drop.
    pub fn yawn_lift(mut t: f64) -> f64 {
        return (if (t < 0.8) {
            Walker::ease((t / 0.8))
        } else {
            (if (t < 1.4) {
                1.0
            } else {
                (1.0 - Walker::ease(rt::min_f64(1.0, ((t - 1.4) / 0.7))))
            })
        });
    }
    pub fn stretch_pose(pose: &mut Pose, mut up: f64) {
        (*pose).arm_near = Walker::q2((165.0 * up));
        (*pose).arm_far = -(Walker::q2((165.0 * up)));
        (*pose).fore_near = -(Walker::q2((15.0 * up)));
        (*pose).fore_far = Walker::q2((15.0 * up));
        (*pose).stretch_y = (1.0 + (0.06 * up));
        (*pose).body_dy = -(Walker::quantise((0.3 * up)));
        (*pose).lean = (-(rt::round_even(((0.25 * up) * 8.0))) / 8.0);
    }
    pub fn hiccup_jolt(mut t: f64) -> f64 {
        let mut at: Arr<f64> = (*WALKER_HICCUP_TIMES).clone();
        {
            let mut i: i32 = 0;
            'l1: while i < at.len() {
                if (t >= (*at.at(i))) && (t < ((*at.at(i)) + 0.14)) {
                    return (1.0 - ((t - (*at.at(i))) / 0.14));
                }
                i += 1;
            }
        }
        return 0.0;
    }
    pub fn breathe(mut self, pose: &mut Pose) {
        (*pose).body_dy = Walker::quantise(((0.3 * (1.0 + (self.time * 2.6).sin())) / 2.0));
    }
    // The upper-arm angle that points the near arm at a screen point.
    pub fn aim_at(mut self, mut x: f64, mut y: f64) -> f64 {
        let mut dx: f64 = (((x - self.cx) * ((self.facing) as f64)) / ((self.scale) as f64));
        let mut dy: f64 = (((self.feet_y - y) / ((self.scale) as f64)) - self.body.shoulder_v);
        let mut raise: f64 = (((dx).atan2(-dy) * 180.0) / std::f64::consts::PI);
        if raise < 15.0 {
            raise = 15.0;
        } else if raise > 165.0 {
            raise = 165.0;
        }
        return Walker::q2(raise);
    }
    // ---- Reactions ---------------------------------------------------------------------
    pub fn react_face(
        mut self,
        target: &mut Expression,
        rate: &mut f64,
        gx: &mut f64,
        gy: &mut f64,
    ) {
        let mut t: f64 = self.mode_time;
        'sw1: {
            match self.reaction {
                Reaction::Gasp => {
                    (*target) = (if (t < 0.9) {
                        (*EXPRESSION_SHOCKED)
                    } else {
                        (*EXPRESSION_SAD)
                    });
                    (*rate) = ((if (t < 0.9) { 30 } else { 10 }) as f64);
                }
                Reaction::Headshake => {
                    (*target) = (*EXPRESSION_DISAPPROVING);
                    (*rate) = 12.0;
                }
                Reaction::Huff => {
                    (*target) = (*EXPRESSION_GRUMPY);
                    (*rate) = 16.0;
                    (*gx) = 0.5; // pointedly looking the other way
                    (*gy) = 0.1;
                    return;
                }
                Reaction::Cower => {
                    (*target) = (if (t < 1.3) {
                        (*EXPRESSION_SCARED)
                    } else {
                        (*EXPRESSION_WORRIED)
                    });
                    (*rate) = ((if (t < 1.3) { 30 } else { 10 }) as f64);
                }
                Reaction::Bristle => {
                    (*target) = (if (t < 0.9) {
                        (*EXPRESSION_FURIOUS)
                    } else {
                        (*EXPRESSION_GRUMPY)
                    });
                    (*rate) = ((if (t < 0.9) { 30 } else { 12 }) as f64);
                }
                _ => {
                    (*target) = (*EXPRESSION_LAUGH);
                    (*rate) = 20.0;
                }
            }
        }
        self.look_at(
            self.react_x,
            (self.feet_y - ((12 * self.scale) as f64)),
            &mut *gx,
            &mut *gy,
        );
    }
    pub fn react_pose(mut self, pose: &mut Pose) {
        let mut t: f64 = self.mode_time;
        'sw1: {
            match self.reaction {
                Reaction::Gasp => {
                    // Hands fly up beside the face, then sink as the worry sets in.
                    let mut up: f64 = (if (t < 0.15) {
                        Walker::ease((t / 0.15))
                    } else {
                        (if (t < 0.9) {
                            1.0
                        } else {
                            (1.0 - Walker::ease(rt::min_f64(1.0, ((t - 0.9) / 0.4))))
                        })
                    });
                    (*pose).arm_near = Walker::q2((30.0 + (125.0 * up)));
                    (*pose).arm_far = -(Walker::q2((30.0 + (125.0 * up))));
                    (*pose).fore_near = Walker::q2((58.0 * up));
                    (*pose).fore_far = -(Walker::q2((58.0 * up)));
                    (*pose).lean = (-(rt::round_even(((0.3 * up) * 8.0))) / 8.0);
                    (*pose).stretch_y = (1.0 + (0.05 * up));
                }
                Reaction::Headshake => {
                    Walker::hands_on_hips(&mut *pose);
                    (*pose).lean = -0.15;
                    self.breathe(&mut *pose);
                }
                Reaction::Huff => {
                    Walker::hands_on_hips(&mut *pose);
                    (*pose).lean = -0.25;
                    (*pose).stretch_y = 1.03;
                    self.breathe(&mut *pose);
                }
                Reaction::Chuckle => {
                    self.laugh_pose(&mut *pose, t, false);
                }
                Reaction::Cower => {
                    // Drops into a crouch with the hands up beside the face, shaking.
                    let mut down: f64 = Walker::ease(rt::min_f64(1.0, (t / 0.15)));
                    let mut tremble: f64 = ((if (t < 1.6) {
                        (if ((t * 26.0).sin() > 0.0) { 3 } else { -3 })
                    } else {
                        0
                    }) as f64);
                    (*pose).stretch_y = (1.0 - (0.15 * down));
                    (*pose).stretch_x = (1.0 + (0.1 * down));
                    (*pose).body_dy = Walker::quantise((0.6 * down));
                    (*pose).arm_near = (Walker::q2((160.0 * down)) + tremble);
                    (*pose).arm_far = (-(Walker::q2((160.0 * down))) - tremble);
                    (*pose).fore_near = Walker::q2((80.0 * down));
                    (*pose).fore_far = -(Walker::q2((80.0 * down)));
                    (*pose).lean = 0.2;
                }
                Reaction::Bristle => {
                    // Puffed up and shaking, beak snapping: back off.
                    let mut up: f64 = Walker::ease(rt::min_f64(1.0, (t / 0.2)));
                    (*pose).lean = (rt::round_even(((0.2 * up) * 8.0)) / 8.0);
                    (*pose).body_dy =
                        Walker::quantise((((0.2 * up) * (1.0 - (t * 20.0).cos())) / 2.0));
                    (*pose).stretch_x = (1.0 + (0.06 * up));
                    (*pose).stretch_y = (1.0 + (0.04 * up));
                }
                _ => {
                    self.laugh_pose(&mut *pose, t, true);
                }
            }
        }
    }
    pub fn react_yaw(mut self, snap: &mut bool) -> f64 {
        (*snap) = false;
        'sw1: {
            match self.reaction {
                Reaction::Gasp => {
                    return 30.0;
                }
                Reaction::Headshake => {
                    // Two shakes of the head, taken directly.
                    let mut t: f64 = self.mode_time;
                    if (t < 0.2) || (t >= 1.2) {
                        return 0.0;
                    }
                    (*snap) = true;
                    return (20.0 * (((2.0 * std::f64::consts::PI) * (t - 0.2)) / 0.5).sin());
                }
                Reaction::Huff => {
                    return 45.0;
                }
                Reaction::Chuckle => {
                    return 20.0;
                }
                Reaction::Cower => {
                    return 10.0;
                }
                Reaction::Bristle => {
                    return 15.0;
                }
                _ => {
                    return 35.0;
                }
            }
        }
        #[allow(unreachable_code)]
        unreachable!()
    }
    // ---- Small helpers --------------------------------------------------------------
    // Smoothstep: 0..1 in, 0..1 out with soft ends.
    pub fn ease(mut s: f64) -> f64 {
        if s <= 0.0 {
            return 0.0;
        }
        if s >= 1.0 {
            return 1.0;
        }
        return ((s * s) * (3.0 - (2.0 * s)));
    }
    // Angles stepped to two degrees, so a slow motion is not repainted for every fraction.
    pub fn q2(mut degrees: f64) -> f64 {
        return (rt::round_even((degrees / 2.0)) * 2.0);
    }
    // A phase stepped to sixteenths.
    pub fn q16(mut phase: f64) -> f64 {
        return ((phase * 16.0).floor() / 16.0);
    }
}
