use crate::{
    components::physics::{MovementReceiver, Velocity},
    core::scheduling::{Action, ResTickQueue},
};
use bevy::prelude::*;

pub fn sys_movement_receive(
    tc: Res<ResTickQueue>,
    mut query: Query<(&mut MovementReceiver, &mut Velocity)>,
) {
    for action in tc.current_tick_actions() {
        match action {
            Action::StartMoving { dir } => {
                for (mut mr, mut vel) in &mut query {
                    mr.direction = mr.direction.or(*dir);
                    mr.apply(&mut vel);
                }
            }
            Action::StopMoving { dir } => {
                for (mut mr, mut vel) in &mut query {
                    mr.direction = mr.direction.and(dir.not());
                    mr.apply(&mut vel);
                }
            }
            _ => continue,
        }
    }
}
