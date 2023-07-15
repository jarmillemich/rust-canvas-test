use bevy::reflect::{FromReflect, Reflect};
use bitmask_enum::bitmask;
use serde_derive::{Deserialize, Serialize};

use crate::fixed_point::FixedPoint;

#[bitmask(u8)]
#[derive(Default, Serialize, Deserialize, Reflect, FromReflect)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[allow(unused)]
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Eq)]
pub enum Action {
    /// Indicate that we are moving in some cardinal direction
    StartMoving { dir: Direction },

    /// Indicate that we are no longer moving in some cardinal direction
    StopMoving { dir: Direction },

    /// Indicate the initiation of a jump
    Jump,

    /// Indicate the movement of the cursor
    Cursor { x: FixedPoint, y: FixedPoint },

    /// Indicate firing a weapon/ability
    Fire,
}