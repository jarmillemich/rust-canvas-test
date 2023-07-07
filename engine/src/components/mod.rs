use bevy::{
    prelude::App,
    reflect::{ReflectDeserialize, ReflectSerialize},
};

use crate::fixed_point::FixedPoint;

pub mod graphics;
pub mod physics;

pub fn register_components(app: &mut App) {
    // Any types that should be included in a world save/serialization must be registered here
    app.register_type::<FixedPoint>()
        .register_type_data::<FixedPoint, ReflectSerialize>()
        .register_type_data::<FixedPoint, ReflectDeserialize>()
        .register_type::<graphics::Color>()
        .register_type::<graphics::DrawCircle>()
        .register_type::<physics::Position>()
        .register_type::<physics::Velocity>()
        .register_type::<physics::Gravity>()
        .register_type::<physics::GravityEmitter>()
        .register_type::<physics::MovementReceiver>()
        .register_type::<crate::core::scheduling::Direction>();
}
