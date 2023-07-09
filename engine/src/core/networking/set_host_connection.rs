use crate::core::scheduling::{ResActionQueue, ResTickQueue};
use bevy::prelude::*;

use super::{ChannelId, NetworkMessage, ResNetworkQueue, WorldLoad};

pub fn sys_host_scheduler(
    mut action_queue: ResMut<ResActionQueue>,
    mut network_queue: ResMut<ResNetworkQueue>,
    mut tick_queue: ResMut<ResTickQueue>,
    mut clients: Query<&mut ClientConnection>,
) {
    // 1. Immediately schedule all actions from the action queue to the tick queue
    // 2. Create a NetworkMessage::FinalizedTick of the last finalized tick
    // 3. For each client,
    // 3a. Send them the NetworkMessage::FinalizedTick
    // 3c. receive a NetworkMessage::ScheduleAction from the client if available
    // 3d. Immediately schedule those actions

    let next_tick = tick_queue.next_unfinalized_tick();
    tick_queue.finalize_tick_with_actions(next_tick, action_queue.take_queue());

    for mut client in &mut clients.iter_mut().filter(|c| c.is_connected()) {
        // Send ours
        let channel_id = client.channel_id;
        let (last_finalized_tick, messages) =
            tick_queue.make_tick_finalization_messages(client.last_finalized_tick + 1);

        network_queue.send_many(&channel_id, messages);
        client.last_finalized_tick = last_finalized_tick;

        // Schedule theirs
        let messages = network_queue.take_inbound(&channel_id, |message| {
            matches!(message, NetworkMessage::ScheduleActions { .. })
        });

        for message in messages {
            if let NetworkMessage::ScheduleActions { actions } = message {
                tick_queue.enqueue_actions_immediately(actions);
            }
        }
    }
}

/// Keeps track of the channel to a particular client
#[derive(Component)]
pub struct ClientConnection {
    pub channel_id: ChannelId,
    pub last_finalized_tick: usize,
    connection_state: ClientConnectionState,
}

#[derive(Default)]
enum ClientConnectionState {
    #[default]
    Disconnected,
    WaitingForHello,
    NeedsWorldSend,
    Connected,
}

impl ClientConnection {
    pub fn new(channel_id: ChannelId) -> Self {
        Self {
            channel_id,
            last_finalized_tick: 0,
            connection_state: ClientConnectionState::WaitingForHello,
        }
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.connection_state, ClientConnectionState::Connected)
    }

    pub fn on_ping(&mut self, _ping_id: usize) {
        // If we were waiting for our hello, we got it
        if matches!(
            self.connection_state,
            ClientConnectionState::WaitingForHello
        ) {
            web_sys::console::log_1(&"[conn] Client sent initial ping".into());
            self.connection_state = ClientConnectionState::NeedsWorldSend;
        }
    }

    pub fn needs_world_send(&self) -> bool {
        matches!(self.connection_state, ClientConnectionState::NeedsWorldSend)
    }

    pub fn on_world_send(&mut self, tick: usize) {
        self.connection_state = ClientConnectionState::Connected;
        self.last_finalized_tick = tick - 1;
    }
}

pub fn sys_send_world(world: &mut World) {
    let mut query = world.query::<&mut ClientConnection>();

    if query.iter(world).any(|client| client.needs_world_send()) {
        web_sys::console::log_1(&"[conn] Sending world to one or more clients".into());

        let tick_queue = world.get_resource::<ResTickQueue>().unwrap();

        let last_finalized_tick = tick_queue.current_tick;

        let world_load: WorldLoad = WorldLoad {
            scene: serialize_world(world),
            last_finalized_tick,
        };
        let message = NetworkMessage::World(world_load);

        let mut to_send = Vec::new();

        query.for_each_mut(world, |mut client_connection| {
            if client_connection.needs_world_send() {
                to_send.push((client_connection.channel_id, message.clone()));
                client_connection.on_world_send(last_finalized_tick);
            }
        });

        let mut network_queue = world.get_resource_mut::<ResNetworkQueue>().unwrap();
        for (channel_id, message) in to_send {
            network_queue.send(&channel_id, message);
        }
    }
}

/// Serializes the world, e.g. to send to a newly connected client
pub fn serialize_world(world: &World) -> Vec<u8> {
    // Pack everything into a scene
    let type_registry = world.resource::<AppTypeRegistry>();
    let scene = DynamicScene::from_world(world, type_registry);

    // Serialize into a RON string
    // TODO we should perhaps just directly serialize into bytes, this method produces a prettified version
    let serialized = scene.serialize_ron(type_registry).unwrap();

    serialized.into_bytes()
}
