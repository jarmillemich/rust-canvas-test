use crate::resources::TickCoordinator;
use bevy::prelude::*;
extern crate web_sys;

pub fn sys_tick_coordination(world: &mut World) {
    let mut tc = world
        .get_non_send_resource_mut::<TickCoordinator>()
        .unwrap();
    tc.on_tick_end(world);
}
