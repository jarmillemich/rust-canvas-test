use specs::prelude::*;

use crate::action::FixedPoint;

pub struct Velocity {
    pub vx: FixedPoint,
    pub vy: FixedPoint,
}

impl Velocity {
    pub fn new(vx: FixedPoint, vy: FixedPoint) -> Self {
        Self { vx, vy }
    }

    pub fn new_f32(vx: f32, vy: f32) -> Self {
        Self::new(FixedPoint::from_num(vx), FixedPoint::from_num(vy))
    }
}

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}
