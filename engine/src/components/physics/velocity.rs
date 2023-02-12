use specs::prelude::*;

pub struct Velocity {
    pub vx: u32,
    pub vy: u32
}

impl Velocity {
    pub fn new(vx: u32, vy: u32) -> Self {
        Self { vx, vy }
    }
}

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}