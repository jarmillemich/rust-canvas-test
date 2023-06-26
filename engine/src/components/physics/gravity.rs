use bevy::{prelude::*, reflect::ReflectFromReflect};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]

pub struct Gravity;
