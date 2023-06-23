use bevy::scene::serde::SceneSerializer;
use serde_derive::{Deserialize, Serialize};

use crate::action::Action;

#[derive(Serialize, Deserialize)]
pub enum NetworkMessage {
    // Server->Client messages
    /// Initial world load
    World {
        scene: Vec<u8>,
        last_finalized_tick: usize,
    },

    /// Server sends this message to the client to indicate that it a tick
    /// has been finalized and the actions in it should be used at the appropriate time
    // TODO should we use usize for networked types?
    FinalizedTick { tick: usize, actions: Vec<Action> },

    // Client->Server messages
    /// Client sends this message to the server to indicate that it wants to have an action scheduled
    ScheduleAction { actions: Vec<Action> },
}
