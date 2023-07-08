use bevy::prelude::{in_state, App, IntoSystemConfig, ResMut, Resource, States};

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

    let next_tick = tick_queue.next_unfinalized_tick();
    tick_queue.finalize_tick_with_actions(next_tick, action_queue.take_queue());
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

pub fn attach_to_app(app: &mut App) {
    app.insert_resource(ResActionQueue::default())
        .insert_resource(ResTickQueue::default())
        .add_state::<CoordinationState>()
        .add_system(sys_local_scheduler.run_if(in_state(CoordinationState::ConnectedLocal)));
}
