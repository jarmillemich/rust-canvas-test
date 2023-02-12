use specs::prelude::*;

pub struct Velocity {
    pub vx: i32,
    pub vy: i32
}

impl Velocity {
    pub fn new(vx: i32, vy: i32) -> Self {
        Self { vx, vy }
    }
}

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}