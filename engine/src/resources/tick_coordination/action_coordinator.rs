use bevy::prelude::World;

use crate::action::Action;

use super::tick_queue::TickQueue;

/// Interface for either a client->server or server->client connection
pub trait ActionScheduler {
    fn add_action(&mut self, queue: &mut TickQueue, action: Action);
    fn synchronize(&mut self, queue: &mut TickQueue, world: &mut World);
}
