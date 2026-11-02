// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

// A speck of dust kicked up by a landing or a skid.
#[derive(Clone, Default)]
pub struct PuffData {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub life: f64,
    pub max_life: f64,
}
rt::handle!(Puff, PuffData);

impl Puff {
    fn alloc__() -> Puff {
        let mut this = Puff::from_data(PuffData::default());
        this
    }
    pub fn new() -> Puff {
        Self::alloc__()
    }
    // 0 when fresh, 1 when about to vanish.
    pub fn age(mut self) -> f64 {
        return (1.0 - (self.life / self.max_life));
    }
}
