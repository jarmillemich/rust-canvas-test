use specs::prelude::*;
use web_sys::console;
use crate::{components::{physics::Position,graphics::Color}, renderer::Renderer, action::FixedPoint};

pub struct SysRenderer;

impl<'a> System<'a> for SysRenderer {
    type SystemData = (
        WriteExpect<'a, Renderer>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Color>,
    );

    fn run(&mut self, (mut renderer, pos, col): Self::SystemData) {
        // Arbitrarily chosen "center" for the moment
        let cx = 400.;
        let cy = 400.;
        
        renderer.draw_test(cx, cy, 6., [0., 0., 0., 1.]);
        
        for (pos, col) in (&pos, &col).join() {
            let xx = pos.x;
            let yy = pos.y;

            renderer.draw_test(
                xx.to_num::<f32>() + cx,
                yy.to_num::<f32>() + cy,
                16.,
                [
                    col.r as f32,
                    col.g as f32,
                    col.b as f32,
                    col.a as f32,
                ]
            );

        }
    }
}