use bevy::{
    ecs::entity::EntityMap,
    prelude::{AppTypeRegistry, Assets, World, *},
    scene::{serde::SceneDeserializer, DynamicScene, DynamicSceneBundle},
};
use serde::de::DeserializeSeed;

use crate::{
    core::{
        networking::WorldLoad,
        scheduling::{CoordinationState, ResActionQueue, ResTickQueue},
    },
    engine::{ResLogger, SimulationState},
    utils::log,
};

use super::{ChannelId, NetworkMessage, ResNetworkQueue};

pub fn sys_client_scheduler(
    mut action_queue: ResMut<ResActionQueue>,
    mut network_queue: ResMut<ResNetworkQueue>,
    mut tick_queue: ResMut<ResTickQueue>,
    mut connection_to_host: ResMut<ConnectionToHost>,
    join_state: Res<State<ClientJoinState>>,
    logger: Res<ResLogger>,
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
        ClientJoinState::WaitingForWorld | ClientJoinState::NeedsSendInitialPing => {
            // Buffer
            connection_to_host.buffered_messages.extend(finalized_ticks);
        }
        ClientJoinState::Connected => {
            // Play received messages and any buffered messages
            for msg in finalized_ticks
                .into_iter()
                .chain(connection_to_host.buffered_messages.drain(..))
            {
                if let NetworkMessage::FinalizedTick { tick, actions } = msg {
                    // logger.debug(format!("Received finalized tick {}", tick).as_str());
                    tick_queue.finalize_tick_with_actions(tick, actions);
                }
            }
        }
    }

    // Reset up to where we have simulated
    if matches!(join_state.0, ClientJoinState::Connected) {
        tick_queue.reset_simulated();
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
    /// We need to send a ping to the host to let them know we are ready to receive
    NeedsSendInitialPing,
    /// Waiting for the host to send us the world
    WaitingForWorld,
    /// In sync with the host, playing normally
    Connected,
}

fn sys_initial_connect(
    mut join_state: ResMut<NextState<ClientJoinState>>,
    mut network_queue: ResMut<ResNetworkQueue>,
    connection_to_host: Res<ConnectionToHost>,
) {
    // Send our initial ping
    log("[client] Sending initial ping".into());
    network_queue.send(&connection_to_host.channel_id, NetworkMessage::Ping(0));
    join_state.set(ClientJoinState::WaitingForWorld);
}

fn sys_receive_world_load(
    mut network_queue: ResMut<ResNetworkQueue>,
    client_connection: Res<ConnectionToHost>,
    mut commands: Commands,
) {
    // Receive the world load
    let mut world_load = network_queue.take_inbound(&client_connection.channel_id, |msg| {
        matches!(msg, NetworkMessage::World { .. })
    });

    if let Some(NetworkMessage::World(world_load)) = world_load.pop() {
        log(format!(
            "[client] Received world load at tick {}",
            world_load.last_simulated_tick
        ));

        // Store the world load
        commands.insert_resource(world_load);
    }
}

fn sys_try_load_world(world: &mut World) {
    log("[client] Loading world maybe".into());

    // Start afresh
    // TODO maybe have a marker component so we don't clear special things?
    world.clear_entities();

    let world_load = world.remove_resource::<WorldLoad>().unwrap();
    let scene = world_load.scene;

    // Convert from ron to DynamicScene we can insert
    let scene = {
        // let srlz = String::from_utf8(scene).unwrap();
        // log(&srlz.clone().into());
        let mut deserializer = ron::de::Deserializer::from_bytes(&scene).unwrap();
        let scene_deserializer = SceneDeserializer {
            type_registry: &world.resource::<AppTypeRegistry>().read(),
        };
        scene_deserializer.deserialize(&mut deserializer).unwrap()
    };

    log(format!(
        "[client] Initial scene from server has {} entities",
        scene.entities.len()
    ));

    let mut entity_map = EntityMap::default();
    scene.write_to_world(world, &mut entity_map).unwrap();

    // Double checking
    log(format!(
        "[client] Actually spawned {} entities",
        world.entities().len()
    ));

    // Align tick coordination
    let mut tc = world
        .get_resource_mut::<ResTickQueue>()
        .expect("Tick queue missing");
    tc.set_last_simulated_tick(world_load.last_simulated_tick);

    // Remove this once we are done
    world.remove_resource::<WorldLoad>();

    // TESTING just allow regular running last_simulated_tick
    world
        .get_resource_mut::<NextState<SimulationState>>()
        .unwrap()
        .set(SimulationState::Running);

    // Let our connection know it can let buffered ticks through
    world
        .get_resource_mut::<NextState<ClientJoinState>>()
        .unwrap()
        .set(ClientJoinState::Connected);

    log(format!(
        "[client] Loaded world, initial tick was {}",
        world_load.last_simulated_tick
    ));
}

pub fn attach_to_app(app: &mut App) {
    use crate::engine::SimulationSet;

    app.add_state::<ClientJoinState>()
        .add_system(
            sys_receive_world_load
                .run_if(
                    state_exists::<ClientJoinState>()
                        .and_then(in_state(ClientJoinState::WaitingForWorld)),
                )
                .in_set(SimulationSet::NetworkPre),
        )
        .add_system(
            sys_try_load_world
                .run_if(
                    state_exists::<ClientJoinState>()
                        .and_then(in_state(ClientJoinState::WaitingForWorld))
                        .and_then(resource_exists::<WorldLoad>()),
                )
                .after(SimulationSet::AfterTick),
        )
        .add_system(
            sys_client_scheduler.run_if(
                in_state(CoordinationState::ConnectedToHost)
                    .and_then(in_state(ClientJoinState::Connected)),
            ),
        )
        .add_system(sys_initial_connect.run_if(in_state(ClientJoinState::NeedsSendInitialPing)));
}
