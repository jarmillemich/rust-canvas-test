use specs::prelude::*;

use crate::action::FixedPoint;

pub struct Velocity {
    pub vx: FixedPoint,
    pub vy: FixedPoint
}

impl Velocity {
    pub fn new(vx: FixedPoint, vy: FixedPoint) -> Self {
        Self { vx, vy }
    }
}

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}