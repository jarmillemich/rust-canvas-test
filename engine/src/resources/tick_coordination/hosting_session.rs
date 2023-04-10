use std::sync::{Arc, Mutex};

use crate::action::Action;

use super::{connection_to_client::ConnectionToClient, action_coordinator::ActionScheduler, tick_queue::TickQueue};

pub struct HostingSession {
    clients: Vec<ConnectionToClient>,
}

impl HostingSession {
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
        }
    }
}

impl ActionScheduler for HostingSession {
    fn add_action(&mut self, queue: &mut TickQueue, action: Action) {
        todo!();
    }

    fn synchronize(&mut self, queue: &mut TickQueue) {
        todo!();
    }
}