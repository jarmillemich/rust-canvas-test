use crate::{
    components::physics::{MovementReceiver, Velocity},
    core::{
        networking::NetworkControlTarget,
        scheduling::{Action, PlayerAction, ResTickQueue},
    },
};
use bevy::prelude::*;

pub fn sys_movement_receive(
    tc: Res<ResTickQueue>,
    mut query: Query<(&mut MovementReceiver, &mut Velocity, &NetworkControlTarget)>,
) {
    for action in tc.current_tick_actions() {
        match action {
            Action::PlayerAction {
                action: PlayerAction::StartMoving { dir },
                player_id,
            } => {
                for (mut mr, mut vel, pid) in &mut query {
                    if pid.player_id != *player_id {
                        continue;
                    }

                    mr.direction = mr.direction.or(*dir);
                    mr.apply(&mut vel);
                }
            }
            Action::PlayerAction {
                action: PlayerAction::StopMoving { dir },
                player_id,
            } => {
                for (mut mr, mut vel, pid) in &mut query {
                    if pid.player_id != *player_id {
                        continue;
                    }

                    mr.direction = mr.direction.and(dir.not());
                    mr.apply(&mut vel);
                }
            }
            _ => continue,
        }
    }
}
