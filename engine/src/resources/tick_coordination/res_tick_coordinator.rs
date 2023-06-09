use super::{action_coordinator::ActionScheduler, tick_queue::TickQueue};
use crate::action::Action;
use std::sync::{Arc, Mutex};

/// Manages when we are allowed to tick the simulation
/// and what actions are to be applied in a particular tick
pub struct TickCoordinator {
    /// Circular queue of actions, use n%128 slot for tick n and clear
    tick_queue: TickQueue,

    /// The current ActionScheduler used to coordinate when an action will occur
    coordinator: Arc<Mutex<dyn ActionScheduler>>,
}

impl TickCoordinator {
    pub fn new(coordinator: Arc<Mutex<dyn ActionScheduler>>) -> Self {
        Self {
            tick_queue: TickQueue::new(),
            coordinator,
        }
    }

    /// Requests that an action be applied.
    /// Note that it may not be applied immediately
    pub fn add_action(&mut self, action: Action) {
        self.coordinator
            .lock()
            .unwrap()
            .add_action(&mut self.tick_queue, action);
    }

    /// Returns the actions that should be applied in the current tick
    pub fn current_tick_actions(&self) -> &Vec<Action> {
        self.tick_queue.current_tick_actions()
    }

    /// Must be invoked whenever a tick ends to perform scheduling logic
    pub fn on_tick_end(&mut self) {
        self.coordinator
            .lock()
            .unwrap()
            .synchronize(&mut self.tick_queue);

        // Immediately advance whenever we have new finalized ticks
        if self.tick_queue.is_next_tick_finalized() {
            self.tick_queue.advance();
        }
    }
}
