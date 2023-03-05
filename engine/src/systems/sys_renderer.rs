use crate::{
    components::{
        graphics::{Color, DrawCircle},
        physics::Position,
    },
    renderer::Renderer,
};
use specs::prelude::*;

pub struct SysRenderer;

impl<'a> System<'a> for SysRenderer {
    type SystemData = (
        WriteExpect<'a, Renderer>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Color>,
        ReadStorage<'a, DrawCircle>,
    );

    fn run(&mut self, (mut renderer, pos, col, circle): Self::SystemData) {
        // Arbitrarily chosen "center" for the moment
        let cx = 400.;
        let cy = 400.;

        //renderer.draw_test(cx, cy, 6., [0., 0., 0., 255.]);

        for (pos, col, circle) in (&pos, &col, &circle).join() {
            let xx = pos.x;
            let yy = pos.y;

            renderer.draw_test(
                xx.to_num::<f32>() + cx,
                yy.to_num::<f32>() + cy,
                circle.radius,
                [
                    col.red as f32,
                    col.green as f32,
                    col.blue as f32,
                    col.alpha as f32,
                ],
            );
        }
    }
}
