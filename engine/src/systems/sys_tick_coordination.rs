use crate::resources::TickCoordinator;
use bevy::prelude::*;
extern crate web_sys;

pub fn sys_tick_coordination(mut tc: NonSendMut<TickCoordinator>) {
    tc.on_tick_end();
}
