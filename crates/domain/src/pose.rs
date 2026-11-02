// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

//
// One frame's worth of body positions. Angles are in degrees, lengths
// in units (a Minion is 18 units tall, Mr. Bean 28); the painter turns
// it all into pixels. The head and reach fields only mean something to
// a figure with a neck and long arms (Mr. Bean); the Minion painter
// ignores them.
//
#[derive(Clone, Default, Copy)]
pub struct Pose {
    pub face: Expression,
    pub gaze_x: f64,
    pub gaze_y: f64,
    pub yaw: f64,
    pub walking: bool,
    pub walk_phase: f64,
    pub leg_tuck: f64,
    pub arm_near: f64,
    pub arm_far: f64,
    pub fore_near: f64,
    pub fore_far: f64,
    pub reach_near: f64,
    pub reach_far: f64,
    pub point_near: bool,
    pub lean: f64,
    pub body_dy: f64,
    pub stretch_x: f64,
    pub stretch_y: f64,
    pub head_turn: f64,
    pub head_dip: f64,
    pub tiptoe: f64,
    pub teddy: bool,
    pub teddy_wave: f64,
    pub wheel: bool,
    pub wheel_turn: f64,
    pub camera: bool,
    pub flash: bool,
    pub egg: bool,
    pub puff_up: f64,
    pub blast_flash: bool,
    pub bubble: bool,
    pub sweat: f64,
    pub zzz: bool,
    pub note: bool,
    pub heart: bool,
    pub decor_phase: f64,
}

impl Pose {
    pub fn standing() -> Pose {
        let mut pose: Pose = Pose::default();
        pose.face = (*EXPRESSION_NEUTRAL);
        pose.gaze_x = 0.5;
        pose.stretch_x = 1.0;
        pose.stretch_y = 1.0;
        return pose;
    }
    // True if drawing this pose would give the same picture as drawing other.
    pub fn same_as(self, mut o: Pose) -> bool {
        return (((((((((((((((((((((((((((((((((((self
            .face
            .same_as(o.face)
            && (self.gaze_x == o.gaze_x))
            && (self.gaze_y == o.gaze_y))
            && (self.yaw == o.yaw))
            && (self.walking == o.walking))
            && (self.walk_phase == o.walk_phase))
            && (self.leg_tuck == o.leg_tuck))
            && (self.arm_near == o.arm_near))
            && (self.arm_far == o.arm_far))
            && (self.fore_near == o.fore_near))
            && (self.fore_far == o.fore_far))
            && (self.reach_near == o.reach_near))
            && (self.reach_far == o.reach_far))
            && (self.point_near == o.point_near))
            && (self.lean == o.lean))
            && (self.body_dy == o.body_dy))
            && (self.stretch_x == o.stretch_x))
            && (self.stretch_y == o.stretch_y))
            && (self.head_turn == o.head_turn))
            && (self.head_dip == o.head_dip))
            && (self.tiptoe == o.tiptoe))
            && (self.teddy == o.teddy))
            && (self.teddy_wave == o.teddy_wave))
            && (self.wheel == o.wheel))
            && (self.wheel_turn == o.wheel_turn))
            && (self.camera == o.camera))
            && (self.flash == o.flash))
            && (self.egg == o.egg))
            && (self.puff_up == o.puff_up))
            && (self.blast_flash == o.blast_flash))
            && (self.bubble == o.bubble))
            && (self.sweat == o.sweat))
            && (self.zzz == o.zzz))
            && (self.note == o.note))
            && (self.heart == o.heart))
            && (self.decor_phase == o.decor_phase));
    }
}
