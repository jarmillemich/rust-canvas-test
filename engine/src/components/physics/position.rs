use bevy::prelude::*;

use crate::fixed_point::FixedPoint;

#[derive(Clone, Component)]
pub struct Position {
    pub x: FixedPoint,
    pub y: FixedPoint,
}

impl Position {
    pub fn new(x: FixedPoint, y: FixedPoint) -> Self {
        Self { x, y }
    }

    pub fn new_f32(x: f32, y: f32) -> Self {
        Self::new(FixedPoint::from_num(x), FixedPoint::from_num(y))
    }
}
