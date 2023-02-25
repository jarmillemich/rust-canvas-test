use crate::components::physics::{Position, Velocity};
use specs::prelude::*;
extern crate web_sys;

pub struct SysGravity;

impl<'a> System<'a> for SysGravity {
    type SystemData = (ReadStorage<'a, Position>, WriteStorage<'a, Velocity>);

    fn run(&mut self, (pos, mut vel): Self::SystemData) {
        for (pos, vel) in (&pos, &mut vel).join() {
            let d = pos.x * pos.x + pos.y * pos.y;

            vel.vx -= pos.x / d;
            vel.vy -= pos.y / d;
        }
    }
}
