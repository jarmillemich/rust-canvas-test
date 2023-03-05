use crate::{
    action::Action,
    components::physics::{MovementReceiver, Velocity},
    resources::TickCoordinator,
};
use specs::prelude::*;
use web_sys::console;

pub struct SysMovementReceiver;

impl<'a> System<'a> for SysMovementReceiver {
    type SystemData = (
        ReadExpect<'a, TickCoordinator>,
        WriteStorage<'a, MovementReceiver>,
        WriteStorage<'a, Velocity>,
    );

    fn run(&mut self, (tc, mut mr, mut vel): Self::SystemData) {
        for action in tc.current_tick_actions() {
            match action {
                Action::StartMoving { dir } => {
                    for (mr, vel) in (&mut mr, &mut vel).join() {
                        mr.direction = mr.direction.or(*dir);
                        mr.apply(vel);
                    }
                }
                Action::StopMoving { dir } => {
                    for (mr, vel) in (&mut mr, &mut vel).join() {
                        mr.direction = mr.direction.and(dir.not());
                        mr.apply(vel);
                    }
                }
                _ => continue,
            }
        }
    }
}
