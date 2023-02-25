use bitmask_enum::bitmask;
use fixed::{types::extra::U12, FixedI64};

pub type FixedPoint = FixedI64<U12>;

#[bitmask(u8)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[allow(unused)]
pub enum Action {
    /// Indicate that we are moving in some combination of cardinal directions,
    /// or to stop moving if no flags are set
    SetMoving { dir: Direction },

    /// Indicate the initiation of a jump
    Jump,

    /// Indicate the movement of the cursor
    Cursor { x: FixedPoint, y: FixedPoint },

    /// Indicate firing a weapon/ability
    Fire,
}
