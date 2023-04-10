use serde_derive::{Deserialize, Serialize};

use crate::action::Action;

#[derive(Serialize, Deserialize)]
pub enum NetworkMessage {
    // TODO should we use usize for networked types?
    FinalizedTick { tick: usize, actions: Vec<Action> },
}
