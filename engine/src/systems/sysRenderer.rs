use specs::prelude::*;
use web_sys::console;
use crate::{components::physics::{Position}, renderer::Renderer, action::FixedPoint};

pub struct SysRenderer;

impl<'a> System<'a> for SysRenderer {
    type SystemData = (
        WriteExpect<'a, Renderer>,
        ReadStorage<'a, Position>
    );

    fn run(&mut self, (mut renderer, pos): Self::SystemData) {
        for pos in (&pos).join() {
            let xx = pos.x;
            let yy = pos.y;

            let dp = FixedPoint::from_num(8);

            console::log_1(&format!("At {xx},{yy}").into());
            
            renderer.draw([
                (xx + dp).to_num::<f32>() / 1024.0, (yy + dp).to_num::<f32>() / 1024.0,
                (xx + dp).to_num::<f32>() / 1024.0, (yy - dp).to_num::<f32>() / 1024.0,
                (xx - dp).to_num::<f32>() / 1024.0, (yy - dp).to_num::<f32>() / 1024.0,
            ]);

        }
    }
}