use specs::prelude::*;
use crate::components::physics::{Position, Velocity};
extern crate web_sys;

pub struct SysGravity;

fn w2f(p: i32) -> f32 { p as f32 / 256.0 }
fn f2w(p: f32) -> i32 { (p * 256.0) as i32 }

impl<'a> System<'a> for SysGravity {
    type SystemData = (ReadStorage<'a, Position>, WriteStorage<'a, Velocity>);

    fn run(&mut self, (pos, mut vel): Self::SystemData) {
        for (pos, vel) in (&pos, &mut vel).join() {
            let xx = w2f(pos.x);
            let yy = w2f(pos.y);

            let d = xx * xx + yy * yy;

            let mut vx = w2f(vel.vx);
            let mut vy = w2f(vel.vy);

            vx -= xx / d;
            vy -= yy / d;

            vel.vx = f2w(vx);
            vel.vy = f2w(vy);
        }
    }
}