use bevy::prelude::{
    in_state, App, IntoSystemConfig, IntoSystemConfigs, NonSendMut, Query, Res, ResMut, Resource,
};
use std::collections::HashMap;

pub mod types;
pub use types::*;

use crate::{engine::SimulationSet, utils::log};

use self::{channels::ResChannelManager, set_host_connection::sys_host_scheduler};

pub mod channels;
pub use channels::*;

mod set_client_connection;
pub use set_client_connection::*;
mod set_host_connection;
pub use set_host_connection::*;

use super::scheduling::CoordinationState;

#[derive(Resource, Default)]
pub struct ResNetworkQueue {
    inbound_queue: HashMap<ChannelId, Vec<NetworkMessage>>,
    outbound_queue: HashMap<ChannelId, Vec<NetworkMessage>>,
}

// Sending
impl ResNetworkQueue {
    pub fn send(&mut self, channel_id: &ChannelId, message: NetworkMessage) {
        self.outbound_queue
            .entry(*channel_id)
            .or_default()
            .push(message);
    }

    pub fn send_many(&mut self, channel_id: &ChannelId, messages: Vec<NetworkMessage>) {
        self.outbound_queue
            .entry(*channel_id)
            .or_default()
            .extend(messages);
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

/// Multiplexes NetworkMessages between the channel manager and network manager
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

        if messages.is_empty() {
            continue;
        }

        network_queue.on_messages(*channel_id, messages);
    }
}

fn sys_ping_responder(
    mut network_queue: ResMut<ResNetworkQueue>,
    mut clients: Query<&mut ClientConnection>,
    connection_to_host: Option<Res<ConnectionToHost>>,
) {
    // Host side
    for mut client in &mut clients {
        let channel_id = client.channel_id;
        for message in network_queue.take_inbound(&client.channel_id, |message| {
            matches!(message, NetworkMessage::Ping(_))
        }) {
            log("Received ping".into());
            match message {
                NetworkMessage::Ping(id) => {
                    network_queue.send(&channel_id, NetworkMessage::Pong(id));

                    client.on_ping(id);
                }
                _ => unreachable!(),
            }
        }
    }

    // Client side
    if let Some(connection_to_host) = connection_to_host {
        for message in network_queue.take_inbound(&connection_to_host.channel_id, |message| {
            matches!(message, NetworkMessage::Ping(_))
        }) {
            match message {
                NetworkMessage::Ping(id) => {
                    network_queue.send(&connection_to_host.channel_id, NetworkMessage::Pong(id));
                }
                _ => unreachable!(),
            }
        }
    }
}

pub fn attach_to_app(app: &mut App) {
    self::set_client_connection::attach_to_app(app);

    app.insert_resource(ResNetworkQueue::default())
        .insert_non_send_resource(ResChannelManager::default())
        .add_systems(
            (sys_network_sync, sys_ping_responder, sys_send_world)
                .before(SimulationSet::BeforeTick),
        )
        // XXX Adding another sync after the scheduler so we can send out stuff before starting a new tick and making everything out of date again
        .add_system(
            sys_network_sync
                .in_set(SimulationSet::NetworkPost)
                .after(sys_host_scheduler),
        );

    app.add_system(
        sys_host_scheduler
            .before(SimulationSet::BeforeTick)
            .run_if(in_state(CoordinationState::Hosting)),
    );
    app.add_system(
        sys_client_scheduler
            .before(SimulationSet::BeforeTick)
            .run_if(in_state(CoordinationState::ConnectedToHost)),
    );
}
