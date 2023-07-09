use crate::components::physics::{Position, Velocity};
use bevy::prelude::*;

pub fn sys_movement(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in &mut query {
        pos.x += vel.vx;
        pos.y += vel.vy;
    }
}
