use bevy::prelude::{Component, NonSendMut, ResMut, Resource};
use std::collections::HashMap;

pub mod types;
pub use types::*;

use self::channels::ResChannelManager;
use crate::core::scheduling::{ResActionQueue, ResTickQueue};

pub mod channels;
pub use channels::*;

mod set_client_connection;
pub use set_client_connection::*;

#[derive(Resource, Default)]
pub struct ResNetworkQueue {
    inbound_queue: HashMap<ChannelId, Vec<NetworkMessage>>,
    outbound_queue: HashMap<ChannelId, Vec<NetworkMessage>>,
}

// Sending
impl ResNetworkQueue {
    pub fn send(&mut self, channel_id: ChannelId, message: NetworkMessage) {
        self.outbound_queue
            .entry(channel_id)
            .or_default()
            .push(message);
    }

    /// Returns the outbound queue and clears it.
    pub fn take_outbound(&mut self) -> &mut HashMap<ChannelId, Vec<NetworkMessage>> {
        &mut self.outbound_queue
    }
}

// Receiving
impl ResNetworkQueue {
    /// Adds some messages to the inbound queue
    pub fn on_messages(&mut self, channel_id: ChannelId, messages: Vec<NetworkMessage>) {
        self.inbound_queue
            .entry(channel_id)
            .or_default()
            .extend(messages);
    }

    /// Takes messages matching the given predicate from the inbound queue and returns them, removing them from the queue.
    pub fn take_inbound<F>(&mut self, channel_id: &ChannelId, predicate: F) -> Vec<NetworkMessage>
    where
        F: Fn(&NetworkMessage) -> bool,
    {
        let mut messages = Vec::new();
        let queue = self.inbound_queue.entry(*channel_id).or_default();
        let mut i = queue.len();
        while i > 0 {
            i -= 1;
            if predicate(&queue[i]) {
                messages.push(queue.swap_remove(i));
            }
        }
        messages
    }
}

pub fn sys_network_sync(
    mut network_queue: ResMut<ResNetworkQueue>,
    mut channel_manager: NonSendMut<ResChannelManager>,
) {
    // Send messages to appropriate channels
    let outbound_queues = network_queue.take_outbound();
    for (channel_id, messages) in outbound_queues {
        if messages.is_empty() {
            continue;
        }

        let channel = channel_manager.get_channel_mut(*channel_id);
        let messages = std::mem::take(messages);
        channel.send(messages);
    }

    // Receive messages from all channels
    for (channel_id, channel) in channel_manager.iter_channels_mut() {
        let messages = channel.drain();
        network_queue.on_messages(*channel_id, messages);
    }
}

pub fn sys_host_scheduler(
    mut action_queue: ResMut<ResActionQueue>,
    mut network_queue: ResMut<ResNetworkQueue>,
    mut tick_queue: ResMut<ResTickQueue>,
) {
    // TODO
    // 1. Immediately schedule all actions from the action queue to the tick queue
    // 2. Create a NetworkMessage::FinalizedTick of the last finalized tick
    // 3. For each client,
    // 3a. Send them the NetworkMessage::FinalizedTick
    // 3c. receive a NetworkMessage::ScheduleAction from the client if available
    // 3d. Immediately schedule those actions
}

/// Keeps track of the channel to a particular client
#[derive(Component)]
pub struct ClientConnection {
    pub channel_id: ChannelId,
}

impl ClientConnection {
    pub fn new(channel_id: ChannelId) -> Self {
        Self { channel_id }
    }
}
