use specs::prelude::*;
use crate::{components::physics::{Position}, renderer::Renderer};

pub struct SysRenderer;

fn w2f(p: i32) -> f32 { p as f32 / 256.0 }
fn f2w(p: f32) -> i32 { (p * 256.0) as i32 }

impl<'a> System<'a> for SysRenderer {
    type SystemData = (
        WriteExpect<'a, Renderer>,
        ReadStorage<'a, Position>
    );

    fn run(&mut self, (mut renderer, pos): Self::SystemData) {
        for pos in (&pos).join() {
            let xx = w2f(pos.x) / 64.0;
            let yy = w2f(pos.y) / 64.0;
            
            renderer.draw([
                xx + 0.00, yy + 0.01,
                xx + 0.01, yy - 0.01,
                xx - 0.01, yy - 0.01,
            ]);

        }
    }
}