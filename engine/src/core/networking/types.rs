use bevy::prelude::Resource;
use serde_derive::{Deserialize, Serialize};

use crate::core::scheduling::Action;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum NetworkMessage {
    /// Request a Pong message
    Ping(usize),

    /// Respond to a Ping message
    Pong(usize),

    /// Client->Server initial message to indicate readiness to receive the world
    RequestWorldLoad,

    // Server->Client messages
    /// Initial world load
    World(WorldLoad),

    /// Server sends this message to the client to indicate that it a tick
    /// has been finalized and the actions in it should be used at the appropriate time
    // TODO should we use usize for networked types?
    FinalizedTick { tick: usize, actions: Vec<Action> },

    // Client->Server messages
    /// Client sends this message to the server to indicate that it wants to have an action scheduled
    ScheduleActions { actions: Vec<Action> },
}

#[derive(Serialize, Deserialize, Resource, Debug, Eq, PartialEq)]
pub struct WorldLoad {
    /// Serialized DynamicScene
    pub scene: Vec<u8>,
    /// Last tick that has processed as of scene being serialized
    pub last_finalized_tick: usize,
}
