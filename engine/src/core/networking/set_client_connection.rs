use bevy::{
    prelude::{AppTypeRegistry, Assets, World, *},
    scene::{serde::SceneDeserializer, DynamicScene, DynamicSceneBundle},
};
use serde::de::DeserializeSeed;

use crate::{
    core::{
        networking::WorldLoad,
        scheduling::{CoordinationState, ResActionQueue, ResTickQueue},
    },
    engine::SimulationState,
};

use super::{ChannelId, NetworkMessage, ResNetworkQueue};

pub fn sys_client_scheduler(
    mut action_queue: ResMut<ResActionQueue>,
    mut network_queue: ResMut<ResNetworkQueue>,
    mut tick_queue: ResMut<ResTickQueue>,
    mut connection_to_host: ResMut<ConnectionToHost>,
    join_state: Res<State<ClientJoinState>>,
) {
    // 1. Drain all actions from the action queue
    // 2. Wrap in a NetworkMessage::ScheduleActions
    // 3. Send to the host
    // 4. Receive NetworkMessage::FinalizedTick
    // 5. Add finalized actions to the tick queue

    let channel_id = &connection_to_host.channel_id;

    let actions = action_queue.take_queue();
    if !actions.is_empty() {
        network_queue.send(channel_id, NetworkMessage::ScheduleActions { actions });
    }

    let finalized_ticks = network_queue.take_inbound(channel_id, |msg| {
        matches!(msg, NetworkMessage::FinalizedTick { .. })
    });

    // Possible buffer messages instead?
    match join_state.0 {
        ClientJoinState::NonClient => {
            panic!("Non client but running client scheduler")
        }
        ClientJoinState::WaitingForWorld => {
            // Buffer
            connection_to_host.buffered_messages.extend(finalized_ticks);
        }
        ClientJoinState::CatchingUp | ClientJoinState::Connected => {
            // Play received messages and any buffered messages
            for msg in finalized_ticks
                .into_iter()
                .chain(connection_to_host.buffered_messages.drain(..))
            {
                if let NetworkMessage::FinalizedTick { tick, actions } = msg {
                    tick_queue.finalize_tick_with_actions(tick, actions);
                }
            }
        }
    }
}

/// Keeps track of our channel to the current host
#[derive(Resource)]
pub struct ConnectionToHost {
    pub channel_id: ChannelId,
    buffered_messages: Vec<NetworkMessage>,
}

impl ConnectionToHost {
    pub fn new(channel_id: ChannelId) -> Self {
        Self {
            channel_id,
            buffered_messages: Vec::new(),
        }
    }
}

#[derive(States, PartialEq, Debug, Clone, Hash, Default, Eq)]
pub enum ClientJoinState {
    /// Game is not currently a client of another game
    #[default]
    NonClient,
    /// Waiting for the host to send us the world
    WaitingForWorld,
    /// Have the world, replaying ticks rapidly to get in sync
    CatchingUp,
    /// In sync with the host, playing normally
    Connected,
}

fn sys_try_load_world(world: &mut World) {
    web_sys::console::log_1(&"[client] Loading world maybe".into());

    // Start afresh
    // TODO maybe have a marker component so we don't clear special things?
    world.clear_entities();

    let world_load = world.remove_resource::<WorldLoad>().unwrap();
    let scene = world_load.scene;

    // Convert from ron to DynamicScene we can insert
    let scene = {
        // let srlz = String::from_utf8(scene).unwrap();
        // web_sys::console::log_1(&srlz.clone().into());
        let mut deserializer = ron::de::Deserializer::from_bytes(&scene).unwrap();
        let scene_deserializer = SceneDeserializer {
            type_registry: &world.resource::<AppTypeRegistry>().read(),
        };
        scene_deserializer.deserialize(&mut deserializer).unwrap()
    };

    web_sys::console::log_1(
        &format!(
            "[client] Initial scene from server has {} entities",
            scene.entities.len()
        )
        .into(),
    );

    // Wrap in an asset
    let scene = world
        .get_resource_mut::<Assets<DynamicScene>>()
        .expect("Should be able to get a DynamicScene asset to deserialize into")
        .add(scene);

    // Spawn the scene
    world.spawn(DynamicSceneBundle {
        scene,
        ..Default::default()
    });

    // Align tick coordination
    let mut tc = world.get_non_send_resource_mut::<ResTickQueue>().unwrap();
    tc.set_last_finalized_tick(world_load.last_finalized_tick);

    // Remove this once we are done
    world.remove_resource::<WorldLoad>();

    // Next is to catch up
    world
        .get_resource_mut::<NextState<ClientJoinState>>()
        .unwrap()
        .set(ClientJoinState::CatchingUp);

    // TESTING just allow regular running
    world
        .get_resource_mut::<NextState<SimulationState>>()
        .unwrap()
        .set(SimulationState::Running);

    // Let our connection know it can let buffered ticks through
    world
        .get_resource_mut::<NextState<ClientJoinState>>()
        .unwrap()
        .set(ClientJoinState::Connected);

    web_sys::console::log_1(
        &format!(
            "[client] Loaded world, initial tick was {}",
            world_load.last_finalized_tick
        )
        .into(),
    );
}

pub fn attach_to_app(app: &mut App) {
    app.add_state::<ClientJoinState>()
        .add_system(
            sys_try_load_world.run_if(
                state_exists::<ClientJoinState>()
                    .and_then(in_state(ClientJoinState::WaitingForWorld))
                    .and_then(resource_exists::<WorldLoad>()),
            ),
        )
        .add_system(sys_client_scheduler.run_if(in_state(CoordinationState::ConnectedToHost)));
}
