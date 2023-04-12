use std::sync::{Arc, Mutex};

use crate::action::Action;

use super::{
    action_coordinator::ActionScheduler, connection_to_client::ConnectionToClient,
    tick_queue::TickQueue, types::NetworkMessage,
};

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
        // Just schedule in the next non-finalized tick, we're special!
        queue.enqueue_action(action, queue.next_unfinalized_tick());
    }

    fn synchronize(&mut self, queue: &mut TickQueue) {
        // TODO for now just adding all pending actions from all clients to the next unfinalized tick
        for client in &mut self.clients {
            for message in client.take_current_messages() {
                match message {
                    NetworkMessage::ScheduleAction { actions } => {
                        for action in actions {
                            queue.enqueue_action(action, queue.next_unfinalized_tick());
                        }
                    }

                    _ => panic!("Unexpected message from client!"),
                }
            }
        }

        // Finalize the next tick
        queue.finalize_tick(queue.next_unfinalized_tick());

        // Send any finalized ticks to all clients
        for client in &mut self.clients {
            client.synchronize_to_queue(queue);
        }
    }
}
