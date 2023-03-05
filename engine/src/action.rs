use bitmask_enum::bitmask;
use fixed::{types::extra::U12, FixedI64};

pub type FixedPoint = FixedI64<U12>;

#[bitmask(u8)]
#[derive(Default)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[allow(unused)]
#[derive(PartialEq)]
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
