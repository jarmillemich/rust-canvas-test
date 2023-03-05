use specs::prelude::*;

pub struct DrawCircle {
    pub radius: f32,
}

impl DrawCircle {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Component for DrawCircle {
    type Storage = VecStorage<Self>;
}
