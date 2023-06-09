use bevy::prelude::*;

#[derive(Component)]
pub struct DrawCircle {
    pub radius: f32,
}

impl DrawCircle {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}
