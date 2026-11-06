// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

//
// A face, as numbers: how far the lids are down, where the brows sit,
// how the mouth is shaped. Because it is all continuous, one expression
// can melt into another (`Towards`) instead of snapping.
//
#[derive(Clone, Default, Copy)]
pub struct Expression {
    pub lid: f64,
    pub squint: f64,
    pub wink: f64,
    pub brow_raise: f64,
    pub brow_angle: f64,
    pub brow_asym: f64,
    pub pupil_size: f64,
    pub pupil_dx: f64,
    pub pupil_dy: f64,
    pub cross_eye: f64,
    pub mouth_curve: f64,
    pub mouth_open: f64,
    pub mouth_width: f64,
    pub mouth_dx: f64,
    pub teeth: f64,
    pub wobble: f64,
    pub tongue: f64,
}

// ---- Resting faces ---------------------------------------------------------
// Minding its own business.
pub static EXPRESSION_NEUTRAL: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.15, 0.0, 1.7, 0.0, 0.0, 0.0, 0.0,
    )
});

// A contented smile.
pub static EXPRESSION_HAPPY: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.08, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.85, 0.0, 1.9, 0.0, 0.0, 0.0, 0.0,
    )
});

// A wide, open-mouthed smile that never quite goes away.
pub static EXPRESSION_CHEERFUL: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.05, 0.0, 0.0, 0.15, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.9, 0.2, 2.0, 0.0, 0.6, 0.0, 0.0,
    )
});

// Brows down, lids heavy, mouth turned down.
pub static EXPRESSION_GRUMPY: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.3, 0.0, 0.0, 0.0, 0.8, 0.0, 1.0, 0.0, 0.0, 0.0, -0.6, 0.0, 1.7, 0.0, 0.0, 0.0, 0.0,
    )
});

// A hard stare and a flat mouth: not looking for friends.
pub static EXPRESSION_STERN: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.25, 0.0, 0.0, 0.0, 0.6, 0.0, 1.0, 0.0, 0.0, 0.0, -0.25, 0.0, 1.6, 0.0, 0.0, 0.0, 0.0,
    )
});

// Lids relaxed, a lazy half-smile to one side: too cool to care.
pub static EXPRESSION_COOL: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.45, 0.0, 0.0, -0.1, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.45, 0.0, 1.5, 0.3, 0.0, 0.0, 0.0,
    )
});

// Brows a little down, mouth straight: all business.
pub static EXPRESSION_SERIOUS: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.2, 0.0, 0.0, 0.0, 0.35, 0.0, 1.0, 0.0, 0.0, 0.0, -0.1, 0.0, 1.5, 0.0, 0.0, 0.0, 0.0,
    )
});

// Slightly cross-eyed, tongue out of a wide grin: the clown.
pub static EXPRESSION_SILLY: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.0, 0.0, 0.0, 0.3, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 0.8, 0.5, 2.0, 0.0, 0.0, 0.0, 0.6,
    )
});

// Brows up in the middle, pupils shrunk a little, lip trembling: expecting the worst.
pub static EXPRESSION_NERVOUS: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.0, 0.0, 0.0, 0.4, -0.5, 0.0, 0.85, 0.0, 0.0, 0.0, -0.2, 0.0, 1.2, 0.0, 0.0, 0.4, 0.0,
    )
});

// Lids heavy, mouth hanging open in a vacant smile.
pub static EXPRESSION_DOPEY: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.45, 0.0, 0.0, 0.25, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.5, 0.3, 1.5, 0.0, 0.0, 0.0, 0.3,
    )
});

// Lids lowered, one brow up, mouth pulled aside: a sideways look at someone not to be trusted.
pub static EXPRESSION_SUSPICIOUS: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.4, 0.25, 0.0, 0.0, 0.45, 0.4, 1.0, 0.0, 0.0, 0.0, -0.15, 0.0, 1.4, 0.3, 0.0, 0.0, 0.0,
    )
});

// Brows up and worried, pupils small, mouth tight: checking over both shoulders.
pub static EXPRESSION_SHIFTY: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.15, 0.0, 0.0, 0.3, -0.3, 0.0, 0.8, 0.0, 0.0, 0.0, -0.1, 0.0, 1.1, 0.0, 0.0, 0.0, 0.0,
    )
});

// Lids a touch lowered, brows a little up, a small closed smile: a look straight at you.
pub static EXPRESSION_KNOWING: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.2, 0.0, 0.0, 0.25, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.35, 0.0, 1.6, 0.0, 0.0, 0.0, 0.0,
    )
});

// Brows up in the middle, bottom lip out: a sulk.
pub static EXPRESSION_SULKY: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.3, 0.0, 0.0, -0.2, -0.5, 0.0, 1.0, 0.0, 0.0, 0.0, -0.55, 0.05, 1.2, 0.0, 0.0, 0.0, 0.0,
    )
});

// ---- Pleased ---------------------------------------------------------------
// A toothy grin.
pub static EXPRESSION_GRIN: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.0, 0.2, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.9, 0.45, 2.1, 0.0, 1.0, 0.0, 0.0,
    )
});

// Head back, mouth wide, eyes creased.
pub static EXPRESSION_LAUGH: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.3, 0.5, 0.0, 0.4, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.85, 2.2, 0.0, 1.0, 0.0, 0.0,
    )
});

// Eyes creased and a smaller, sillier laugh.
pub static EXPRESSION_GIGGLE: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.5, 0.4, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.9, 0.35, 1.8, 0.0, 0.8, 0.0, 0.0,
    )
});

// Eyes wide and mouth open with glee, mid-air.
pub static EXPRESSION_EXCITED: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.0, 0.0, 0.0, 0.6, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.8, 0.6, 1.9, 0.0, 0.5, 0.0, 0.0,
    )
});

// A wink and a smile.
pub static EXPRESSION_WINKING: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.8, 0.0, 1.9, 0.0, 0.0, 0.0, 0.0,
    )
});

// One brow up, a smirk to the side.
pub static EXPRESSION_SMUG: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.35, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.55, 0.0, 1.3, 0.7, 0.0, 0.0, 0.0,
    )
});

// Chin up, lids lowered, a closed-lipped smile: admiring itself.
pub static EXPRESSION_PROUD: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.3, 0.2, 0.0, 0.1, 0.2, 0.0, 1.0, 0.0, 0.0, 0.0, 0.6, 0.0, 1.6, 0.0, 0.0, 0.0, 0.0,
    )
});

// Narrowed eyes, one brow up and a lopsided grin: up to something.
pub static EXPRESSION_MISCHIEVOUS: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.3, 0.35, 0.0, 0.0, 0.5, 0.6, 1.0, 0.0, 0.0, 0.0, 0.7, 0.2, 1.6, 0.5, 0.3, 0.0, 0.0,
    )
});

// Squinting, brows down and tongue out: blowing a raspberry.
pub static EXPRESSION_RASPBERRY: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.0, 0.5, 0.0, 0.0, 0.6, 0.0, 1.0, 0.0, 0.0, 0.0, 0.3, 0.4, 1.3, 0.0, 0.0, 0.0, 1.0,
    )
});

// Lids lowered, lips pursed in a tiny "o": blowing a kiss.
pub static EXPRESSION_SMOOCH: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.7, 0.0, 0.0, 0.2, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.2, 0.35, 0.5, 0.0, 0.0, 0.0, 0.0,
    )
});

// Half-lidded, eyes rolled up and away, a soft smile: miles away.
pub static EXPRESSION_DREAMY: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.5, 0.0, 0.0, 0.3, 0.0, 0.0, 1.0, 0.2, 0.45, 0.0, 0.65, 0.0, 1.4, 0.0, 0.0, 0.0, 0.0,
    )
});

// ---- Thoughtful and tired ----------------------------------------------------
// One brow raised, lips slightly parted.
pub static EXPRESSION_CURIOUS: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.0, 0.0, 0.0, 0.0, 0.0, 0.8, 1.0, 0.0, 0.0, 0.0, 0.2, 0.15, 1.2, 0.0, 0.0, 0.0, 0.0,
    )
});

// Eyes rolled up and aside, mouth pulled to one side: working something out.
pub static EXPRESSION_THINKING: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.0, 0.0, 0.0, 0.2, 0.0, 0.6, 1.0, 0.3, 0.35, 0.0, -0.1, 0.0, 1.1, -0.6, 0.0, 0.0, 0.0,
    )
});

// Lips pursed in a small "o", eyes rolled innocently upward.
pub static EXPRESSION_WHISTLING: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.55, 0.0, 0.0, 0.35, 0.0, 0.0, 1.0, 0.2, 0.45, 0.0, 0.0, 0.4, 0.55, 0.0, 0.0, 0.0, 0.0,
    )
});

// Squinting in concentration before a jump.
pub static EXPRESSION_FOCUSED: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.25, 0.4, 0.0, 0.0, 0.4, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.4, 0.0, 0.0, 0.0, 0.0,
    )
});

// Half-lidded, looking off to the side: nothing worth watching.
pub static EXPRESSION_BORED: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.25, 0.2, 0.0, -0.15, 0.0, 1.3, 0.0, 0.0, 0.0, 0.0,
    )
});

// Lids drooping, gaze sinking.
pub static EXPRESSION_SLEEPY: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.7, 0.15, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -0.3, 0.0, 0.1, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0,
    )
});

// Eyes screwed shut, brows up, mouth as wide as it goes.
pub static EXPRESSION_YAWN: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.95, 0.0, 0.0, 0.5, -0.3, 0.0, 1.0, 0.0, 0.0, 0.0, -0.15, 1.0, 1.3, 0.0, 0.0, 0.0, 0.0,
    )
});

// Eyes shut, mouth slack, a hint of a smile.
pub static EXPRESSION_ASLEEP: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.2, 0.12, 1.0, 0.0, 0.0, 0.0, 0.0,
    )
});

// Out of breath: lids heavy, mouth open, tongue showing.
pub static EXPRESSION_PANTING: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.5, 0.2, 0.0, 0.0, -0.3, 0.0, 1.0, 0.0, 0.0, 0.0, 0.2, 0.5, 1.5, 0.0, 0.0, 0.0, 0.5,
    )
});

// ---- Alarmed ---------------------------------------------------------------
// Eyes wide, brows up, mouth a small "o".
pub static EXPRESSION_SURPRISED: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.0, 0.0, 0.0, 0.9, 0.0, 0.0, 0.75, 0.0, 0.0, 0.0, 0.0, 0.65, 1.0, 0.0, 0.0, 0.0, 0.0,
    )
});

// Brows as high as they go, pupils tiny, jaw dropped.
pub static EXPRESSION_SHOCKED: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.55, 0.0, 0.0, 0.0, -0.2, 1.0, 1.2, 0.0, 0.0, 0.0, 0.0,
    )
});

// Pupils shrunk, brows up and worried, lips trembling.
pub static EXPRESSION_SCARED: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.0, 0.0, 0.0, 0.7, -0.6, 0.0, 0.6, 0.0, 0.0, 0.0, -0.3, 0.45, 1.6, 0.0, 0.0, 1.0, 0.0,
    )
});

// Brows up, eyes closing, mouth open: the breath before a sneeze.
pub static EXPRESSION_INHALE: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.6, 0.0, 0.0, 0.8, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -0.1, 0.7, 1.2, 0.0, 0.0, 0.0, 0.0,
    )
});

// Eyes shut tight, mouth thrown open: the sneeze itself.
pub static EXPRESSION_SNEEZE: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        1.0, 0.0, 0.0, 0.3, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.9, 1.6, 0.0, 0.5, 0.0, 0.0,
    )
});

// Cross-eyed and reeling.
pub static EXPRESSION_DIZZY: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.35, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.3, 1.0, -0.2, 0.15, 1.7, 0.0, 0.0, 1.0, 0.0,
    )
});

// ---- Cross and sad ---------------------------------------------------------
// Teeth gritted, eyes narrowed: on the attack.
pub static EXPRESSION_DETERMINED: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.4, 0.0, 0.0, 0.0, 0.9, 0.0, 1.0, 0.0, 0.0, 0.0, -0.3, 0.2, 1.9, 0.0, 1.0, 0.0, 0.0,
    )
});

// Brows slammed down, teeth bared: a tantrum.
pub static EXPRESSION_FURIOUS: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.2, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, -0.6, 0.3, 1.9, 0.0, 1.0, 0.0, 0.0,
    )
});

// Lids half down, mouth turned down: not impressed.
pub static EXPRESSION_DISAPPROVING: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.45, 0.0, 0.0, 0.0, 0.3, 0.0, 1.0, 0.0, 0.0, 0.0, -0.4, 0.0, 1.5, 0.0, 0.0, 0.0, 0.0,
    )
});

// Brows up in the middle, mouth down.
pub static EXPRESSION_SAD: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.35, 0.0, 0.0, 0.0, -0.8, 0.0, 1.0, 0.0, 0.0, 0.0, -0.7, 0.0, 1.4, 0.0, 0.0, 0.0, 0.0,
    )
});

// Brows up and worried, mouth down and trembling: something bad is coming.
pub static EXPRESSION_WORRIED: rt::Global<Expression> = rt::Global::new(|| {
    Expression::new(
        0.0, 0.0, 0.0, 0.5, -0.7, 0.0, 1.0, 0.0, 0.0, 0.0, -0.4, 0.15, 1.3, 0.0, 0.0, 0.7, 0.0,
    )
});

impl Expression {
    pub fn new(
        mut lid: f64,
        mut squint: f64,
        mut wink: f64,
        mut brow_raise: f64,
        mut brow_angle: f64,
        mut brow_asym: f64,
        mut pupil_size: f64,
        mut pupil_dx: f64,
        mut pupil_dy: f64,
        mut cross_eye: f64,
        mut mouth_curve: f64,
        mut mouth_open: f64,
        mut mouth_width: f64,
        mut mouth_dx: f64,
        mut teeth: f64,
        mut wobble: f64,
        mut tongue: f64,
    ) -> Expression {
        let mut this = Expression::default();
        this.ctor(
            lid,
            squint,
            wink,
            brow_raise,
            brow_angle,
            brow_asym,
            pupil_size,
            pupil_dx,
            pupil_dy,
            cross_eye,
            mouth_curve,
            mouth_open,
            mouth_width,
            mouth_dx,
            teeth,
            wobble,
            tongue,
        );
        this
    }
    fn ctor(
        &mut self,
        mut lid: f64,
        mut squint: f64,
        mut wink: f64,
        mut brow_raise: f64,
        mut brow_angle: f64,
        mut brow_asym: f64,
        mut pupil_size: f64,
        mut pupil_dx: f64,
        mut pupil_dy: f64,
        mut cross_eye: f64,
        mut mouth_curve: f64,
        mut mouth_open: f64,
        mut mouth_width: f64,
        mut mouth_dx: f64,
        mut teeth: f64,
        mut wobble: f64,
        mut tongue: f64,
    ) {
        self.lid = lid;
        self.squint = squint;
        self.wink = wink;
        self.brow_raise = brow_raise;
        self.brow_angle = brow_angle;
        self.brow_asym = brow_asym;
        self.pupil_size = pupil_size;
        self.pupil_dx = pupil_dx;
        self.pupil_dy = pupil_dy;
        self.cross_eye = cross_eye;
        self.mouth_curve = mouth_curve;
        self.mouth_open = mouth_open;
        self.mouth_width = mouth_width;
        self.mouth_dx = mouth_dx;
        self.teeth = teeth;
        self.wobble = wobble;
        self.tongue = tongue;
    }
    // ---- Blending ------------------------------------------------------------
    // Moves fraction k of the way from a to b, snapping when close.
    pub fn towards(mut a: Expression, mut b: Expression, mut k: f64) -> Expression {
        let mut r: Expression = <Expression>::default();
        r.lid = Expression::step(a.lid, b.lid, k);
        r.squint = Expression::step(a.squint, b.squint, k);
        r.wink = Expression::step(a.wink, b.wink, k);
        r.brow_raise = Expression::step(a.brow_raise, b.brow_raise, k);
        r.brow_angle = Expression::step(a.brow_angle, b.brow_angle, k);
        r.brow_asym = Expression::step(a.brow_asym, b.brow_asym, k);
        r.pupil_size = Expression::step(a.pupil_size, b.pupil_size, k);
        r.pupil_dx = Expression::step(a.pupil_dx, b.pupil_dx, k);
        r.pupil_dy = Expression::step(a.pupil_dy, b.pupil_dy, k);
        r.cross_eye = Expression::step(a.cross_eye, b.cross_eye, k);
        r.mouth_curve = Expression::step(a.mouth_curve, b.mouth_curve, k);
        r.mouth_open = Expression::step(a.mouth_open, b.mouth_open, k);
        r.mouth_width = Expression::step(a.mouth_width, b.mouth_width, k);
        r.mouth_dx = Expression::step(a.mouth_dx, b.mouth_dx, k);
        r.teeth = Expression::step(a.teeth, b.teeth, k);
        r.wobble = Expression::step(a.wobble, b.wobble, k);
        r.tongue = Expression::step(a.tongue, b.tongue, k);
        return r;
    }
    pub fn step(mut a: f64, mut b: f64, mut k: f64) -> f64 {
        let mut next: f64 = (a + ((b - a) * k));
        return (if ((b - next).abs() < 0.004) { b } else { next });
    }
    pub fn same_as(self, mut o: Expression) -> bool {
        return (((((((((((((((((self.lid == o.lid) && (self.squint == o.squint))
            && (self.wink == o.wink))
            && (self.brow_raise == o.brow_raise))
            && (self.brow_angle == o.brow_angle))
            && (self.brow_asym == o.brow_asym))
            && (self.pupil_size == o.pupil_size))
            && (self.pupil_dx == o.pupil_dx))
            && (self.pupil_dy == o.pupil_dy))
            && (self.cross_eye == o.cross_eye))
            && (self.mouth_curve == o.mouth_curve))
            && (self.mouth_open == o.mouth_open))
            && (self.mouth_width == o.mouth_width))
            && (self.mouth_dx == o.mouth_dx))
            && (self.teeth == o.teeth))
            && (self.wobble == o.wobble))
            && (self.tongue == o.tongue));
    }
}
