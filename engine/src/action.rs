use bitmask_enum::bitmask;
use serde_derive::{Deserialize, Serialize};

use crate::fixed_point::FixedPoint;

#[bitmask(u8)]
#[derive(Default, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[allow(unused)]
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub enum Action {
    /// Indicate that we are moving in some cardinal direction
    StartMoving { dir: Direction },

    /// Indicate that we are no longer moving in some cardinal direction
    StopMoving { dir: Direction },

    /// Indicate the initiation of a jump
    Jump,

    /// Indicate the movement of the cursor
    Cursor {
        #[serde(with = "crate::fixed_point")]
        x: FixedPoint,
        #[serde(with = "crate::fixed_point")]
        y: FixedPoint,
    },

    /// Indicate firing a weapon/ability
    Fire,
}
