use crate::{
    components::physics::{Gravity, GravityEmitter, Position, Velocity},
    fixed_point::FixedPoint,
};
use bevy::prelude::*;
extern crate web_sys;

pub fn sys_gravity(
    emitters: Query<(&Position, &GravityEmitter)>,
    mut receivers: Query<(&Position, &mut Velocity, &Gravity)>,
) {
    for (gravity_position, gravity) in &emitters {
        for (pos, mut vel, receiver) in &mut receivers {
            let dx = pos.x - gravity_position.x;
            let dy = pos.y - gravity_position.y;

            assert!(
                dx.to_num::<f32>() != 0.0 || dy.to_num::<f32>() != 0.0,
                "Two gravity entities in the same location"
            );

            let d = dx * dx + dy * dy;

            let g = FixedPoint::from_num(2);

            vel.vx -= g * dx / d;
            vel.vy -= g * dy / d;
        }
    }
}
