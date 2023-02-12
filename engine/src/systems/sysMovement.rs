use specs::prelude::*;
use crate::components::physics::{Position, Velocity};
extern crate web_sys;

pub struct SysMovement;

impl<'a> System<'a> for SysMovement {
    type SystemData = (WriteStorage<'a, Position>, ReadStorage<'a, Velocity>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.x += vel.vx;
            pos.y += vel.vy;

            web_sys::console::log_1(&format!("Thing at {},{}", pos.x, pos.y).into());

        }
    }
}