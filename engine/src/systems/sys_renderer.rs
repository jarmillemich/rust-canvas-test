use crate::{
    components::{
        graphics::{Color, DrawCircle},
        physics::Position,
    },
    renderer::Renderer,
};
use bevy::prelude::*;

pub fn sys_renderer(
    mut renderer: NonSendMut<Renderer>,
    query: Query<(&Position, &Color, &DrawCircle)>,
) {
    // web_sys::console::log_1(&format!("Rendering {} circles", query.iter().count()).into());

    for (pos, col, circle) in query.iter() {
        // Arbitrarily chosen "center" for the moment
        let cx = 400.;
        let cy = 400.;

        //renderer.draw_test(cx, cy, 6., [0., 0., 0., 255.]);

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
