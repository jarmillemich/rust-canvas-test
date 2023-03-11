use crate::{
    action::Action,
    components::{
        graphics::{Color, DrawCircle},
        physics::{Gravity, MovementReceiver, Position, Velocity},
    },
    resources::TickCoordinator,
};
use specs::prelude::*;
use web_sys::console;

pub struct SysFireReceiver;

impl<'a> System<'a> for SysFireReceiver {
    type SystemData = (
        ReadExpect<'a, TickCoordinator>,
        WriteStorage<'a, MovementReceiver>,
        ReadStorage<'a, Position>,
        Entities<'a>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (tc, mr, pos, entities, updater): Self::SystemData) {
        for action in tc.current_tick_actions() {
            match action {
                Action::Fire => {
                    for (_, pos) in (&mr, &pos).join() {
                        // Spawn some more particles
                        for i in 0..8 {
                            let entity = entities.create();

                            let theta = i as f32 * 2. * std::f32::consts::PI / 8.;

                            updater.insert(
                                entity,
                                Position::new_f32(
                                    pos.x.to_num::<f32>() + 300. * f32::cos(theta),
                                    pos.y.to_num::<f32>() + 300. * f32::sin(theta),
                                ),
                            );

                            updater.insert(
                                entity,
                                Velocity::new_f32(-1. * f32::sin(theta), 1. * f32::cos(theta)),
                            );
                            updater.insert(
                                entity,
                                Color::new(20 * i as u8, 255 - 16 * i as u8, 0, 255),
                            );
                            updater.insert(entity, DrawCircle::new(16.));
                            updater.insert(entity, Gravity);
                        }
                    }

                    let ent_count = entities.join().count();
                    console::log_1(&format!("Have {ent_count} entities now").into());
                }
                _ => continue,
            }
        }
    }
}
