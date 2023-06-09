use bevy::prelude::*;

#[derive(Component)]
pub struct GravityEmitter;

impl GravityEmitter {
    pub fn new() -> Self {
        Self {}
    }
}
