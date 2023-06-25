use crate::resources::TickCoordinator;
use bevy::prelude::*;
extern crate web_sys;

pub fn sys_tick_coordination(
    mut tick_coordinator: NonSendMut<TickCoordinator>,
    commands: Commands,
) {
    tick_coordinator.on_tick_end(commands);
}
