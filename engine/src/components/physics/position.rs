use specs::prelude::*;

pub struct Position {
    pub x: u32,
    pub y: u32
}

impl Position {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}