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
        for (pos, col) in (&pos, &col).join() {
            let xx = pos.x;
            let yy = pos.y;

            let dp = FixedPoint::from_num(8);

            renderer.draw([
                (xx + dp).to_num::<f32>() / 1024.0, (yy + dp).to_num::<f32>() / 1024.0,
                (xx + dp).to_num::<f32>() / 1024.0, (yy - dp).to_num::<f32>() / 1024.0,
                (xx - dp).to_num::<f32>() / 1024.0, (yy - dp).to_num::<f32>() / 1024.0,
            ], [
                col.r as f32,
                col.g as f32,
                col.b as f32,
                col.a as f32,
            ]);

        }
    }
}