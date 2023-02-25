use specs::prelude::*;

use crate::action::FixedPoint;

pub struct Position {
    pub x: FixedPoint,
    pub y: FixedPoint,
}

impl Position {
    pub fn new(x: FixedPoint, y: FixedPoint) -> Self {
        Self { x, y }
    }
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}
