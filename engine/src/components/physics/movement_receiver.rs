use specs::prelude::*;
use web_sys::console;

use crate::action::{Direction, FixedPoint};

use super::Velocity;

/// TODO Testing receiving actions to a VelocityComponent
pub struct MovementReceiver {
    pub direction: Direction,
}

impl MovementReceiver {
    pub fn new() -> Self {
        Self {
            direction: Direction::none(),
        }
    }
}

impl Component for MovementReceiver {
    type Storage = VecStorage<Self>;
}

impl MovementReceiver {
    pub fn apply(&self, vel: &mut Velocity) {
        let negative: FixedPoint = FixedPoint::from_num(-1.);
        let positive: FixedPoint = FixedPoint::from_num(1.);
        let zero: FixedPoint = FixedPoint::from_num(0.);

        if self.direction.contains(Direction::Left) {
            vel.vx = negative;
        } else if self.direction.contains(Direction::Right) {
            vel.vx = positive;
        } else {
            vel.vx = zero;
        }

        if self.direction.contains(Direction::Down) {
            vel.vy = negative;
        } else if self.direction.contains(Direction::Up) {
            vel.vy = positive;
        } else {
            vel.vy = zero;
        }
    }
}
