use crate::{
    components::physics::{Gravity, GravityEmitter, Position, Velocity},
    fixed_point::FixedPoint,
};
use specs::prelude::*;
extern crate web_sys;

pub struct SysGravity;

impl<'a> System<'a> for SysGravity {
    type SystemData = (
        ReadStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, Gravity>,
        ReadStorage<'a, GravityEmitter>,
    );

    fn run(&mut self, (pos, mut vel, receiver, emitter): Self::SystemData) {
        for (gravity_position, gravity) in (&pos, &emitter).join() {
            for (pos, vel, rec) in (&pos, &mut vel, &receiver).join() {
                let dx = pos.x - gravity_position.x;
                let dy = pos.y - gravity_position.y;

                assert!(
                    dx != 0 || dy != 0,
                    "Two gravity entities in the same location"
                );

                let d = dx * dx + dy * dy;

                let g = FixedPoint::from_num(2);

                vel.vx -= g * dx / d;
                vel.vy -= g * dy / d;
            }
        }
    }
}
