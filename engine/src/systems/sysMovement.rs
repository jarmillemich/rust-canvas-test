use specs::prelude::*;
use crate::components::physics::{Position, Velocity};
extern crate web_sys;

pub struct SysMovement;

fn w2f(p: i32) -> f32 { p as f32 / 256.0 }
fn f2w(p: f32) -> i32 { (p * 256.0) as i32 }

impl<'a> System<'a> for SysMovement {
    type SystemData = (WriteStorage<'a, Position>, ReadStorage<'a, Velocity>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.x += vel.vx;
            pos.y += vel.vy;
        }
    }
}