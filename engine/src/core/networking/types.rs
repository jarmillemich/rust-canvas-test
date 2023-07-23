use crate::core::scheduling::{Action, PlayerId};
use bevy::prelude::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
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

    /// Sets the clients configuration
    SetClientConfig { player_id: PlayerId },

    /// Server sends this message to the client to indicate that it a tick
    /// has been finalized and the actions in it should be used at the appropriate time
    // TODO should we use usize for networked types?
    FinalizedTick { tick: usize, actions: Vec<Action> },

    // Client->Server messages
    /// Client sends this message to the server to indicate that it wants to have an action scheduled
    ScheduleActions { actions: Vec<Action> },
}

#[derive(Serialize, Deserialize, Resource, Debug, Eq, PartialEq, Clone)]
pub struct WorldLoad {
    /// Serialized DynamicScene
    pub scene: Vec<u8>,

    /// Last tick that has simulated as of scene being serialized
    pub last_simulated_tick: usize,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct NetworkControlTarget {
    // Id of the player in control of this entity
    pub player_id: PlayerId,
}

impl NetworkControlTarget {
    pub fn new(player_id: &PlayerId) -> Self {
        Self {
            player_id: player_id.clone(),
        }
    }
}
