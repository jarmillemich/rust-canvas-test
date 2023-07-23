use crate::{
    components::{
        graphics::{Color, DrawCircle},
        physics::{Gravity, MovementReceiver, Position, Velocity},
    },
    core::{
        networking::NetworkControlTarget,
        scheduling::{Action, PlayerAction, ResTickQueue},
    },
    utils::log,
};
use bevy::prelude::*;

pub fn sys_fire_receive(
    world: &World,
    mut commands: Commands,
    tc: Res<ResTickQueue>,
    query: Query<(&MovementReceiver, &Position, &NetworkControlTarget)>,
) {
    for action in tc.current_tick_actions() {
        match action {
            Action::PlayerAction {
                action: PlayerAction::Fire,
                player_id,
            } => {
                for (_, pos, pid) in query.iter() {
                    if pid.player_id != *player_id {
                        continue;
                    }

                    // Spawn some more particles
                    for i in 0..8 {
                        let theta = i as f32 * 2. * std::f32::consts::PI / 8.;

                        commands.spawn((
                            Position::new_f32(
                                pos.x.to_num::<f32>() + 300. * f32::cos(theta),
                                pos.y.to_num::<f32>() + 300. * f32::sin(theta),
                            ),
                            Velocity::new_f32(-1. * f32::sin(theta), 1. * f32::cos(theta)),
                            Color::new(20 * i as u8, 255 - 16 * i as u8, 0, 255),
                            DrawCircle::new(16.),
                            Gravity,
                        ));
                    }
                }

                let ent_count = world.entities().total_count();
                log(format!("Have {ent_count} entities now"));
            }
            _ => continue,
        }
    }
}
