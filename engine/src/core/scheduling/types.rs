use bevy::{prelude::Resource, reflect::Reflect};
use bitmask_enum::bitmask;
use serde_derive::{Deserialize, Serialize};

use crate::fixed_point::FixedPoint;

#[bitmask(u8)]
#[derive(Default, Serialize, Deserialize, Reflect)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Reflect, Default)]
pub struct PlayerId(pub usize);

#[derive(Resource, Default)]
pub struct ResPlayerIdGenerator {
    next_id: usize,
}

impl ResPlayerIdGenerator {
    pub fn get_next_id(&mut self) -> PlayerId {
        let id = self.next_id;
        self.next_id += 1;
        PlayerId(id)
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ResLocalPlayerId(pub Option<PlayerId>);

/// An Action encompasses any outside stimulus that may impact the simulation
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Eq)]
pub enum Action {
    /// An action taken by a specific player
    PlayerAction {
        action: PlayerAction,
        player_id: PlayerId,
    },

    /// The spawning of a player
    SpawnPlayer { player_id: PlayerId },
}

#[allow(unused)]
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Eq)]
pub enum PlayerAction {
    /// Indicate that we are moving in some cardinal direction
    StartMoving { dir: Direction },

    /// Indicate that we are no longer moving in some cardinal direction
    StopMoving { dir: Direction },

    /// Indicate the initiation of a jump
    Jump,

    /// Indicate the movement of the cursor
    Cursor { x: FixedPoint, y: FixedPoint },

    /// Indicate firing a weapon/ability
    Fire,
}

impl PlayerAction {
    pub fn for_player(self, player_id: PlayerId) -> Action {
        Action::PlayerAction {
            action: self,
            player_id,
        }
    }
}
