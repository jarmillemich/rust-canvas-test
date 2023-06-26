use bevy::{prelude::*, reflect::ReflectFromReflect};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct GravityEmitter;

impl GravityEmitter {
    pub fn new() -> Self {
        Self {}
    }
}
