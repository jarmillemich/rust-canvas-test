use bevy::prelude::{ResMut, Resource, States};

pub mod input;
pub use input::*;

pub mod tick_queue;
pub use tick_queue::*;

pub mod types;
pub use types::*;

pub fn sys_local_scheduler(
    mut action_queue: ResMut<ResActionQueue>,
    mut tick_queue: ResMut<ResTickQueue>,
) {
    // 1. Schedule everything immediately
    // 2. That's all!

    for action in action_queue.take_queue() {
        tick_queue.enqueue_action_immediately(action);
    }
}

#[derive(States, Debug, Default, Hash, PartialEq, Eq, Clone)]
pub enum CoordinationState {
    #[default]
    Disconnected,
    ConnectedLocal,
    Hosting,
    ConnectedToHost,
}

#[derive(Default, Resource)]
pub struct ResActionQueue {
    queue: Vec<Action>,
}

impl ResActionQueue {
    pub fn add_action(&mut self, action: Action) {
        self.queue.push(action);
    }

    pub fn take_queue(&mut self) -> Vec<Action> {
        // TODO probably churning this every frame is hilariously inefficient
        std::mem::take(&mut self.queue)
    }
}
