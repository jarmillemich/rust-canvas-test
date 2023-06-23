use bevy::prelude::World;

use crate::{action::Action, resources::tick_coordination::tick_queue::TickQueue};

use super::action_coordinator::ActionScheduler;

pub struct ConnectionLoopback {}

impl ConnectionLoopback {
    pub fn new() -> Self {
        Self {}
    }
}

impl ActionScheduler for ConnectionLoopback {
    fn add_action(&mut self, queue: &mut TickQueue, action: Action) {
        // Just schedule in the next tick
        queue.enqueue_action(action, queue.current_tick + 50);
    }

    fn synchronize(&mut self, queue: &mut TickQueue, world: &mut World) {
        // Just finalize the next tick
        queue.finalize_tick(queue.current_tick + 1);
    }
}
