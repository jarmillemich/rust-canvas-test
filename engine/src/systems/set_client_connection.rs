use std::sync::{Arc, Mutex};

use bevy::{
    prelude::{AppTypeRegistry, Assets, World, *},
    scene::{serde::SceneDeserializer, DynamicScene, DynamicSceneBundle},
};
use serde::de::DeserializeSeed;

use crate::{
    engine::SimulationState,
    resources::{
        tick_coordination::{connection_to_host::ConnectionToHost, types::WorldLoad},
        TickCoordinator,
    },
};

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
    let mut tc = world
        .get_non_send_resource_mut::<TickCoordinator>()
        .unwrap();
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
        .get_non_send_resource_mut::<Arc<Mutex<ConnectionToHost>>>()
        .unwrap()
        .lock()
        .unwrap()
        .mark_world_received();

    web_sys::console::log_1(
        &format!(
            "[client] Loaded world, initial tick was {}",
            world_load.last_finalized_tick
        )
        .into(),
    );
}

pub fn attach_to_app(app: &mut App) {
    app.add_state::<ClientJoinState>().add_system(
        sys_try_load_world.run_if(
            state_exists::<ClientJoinState>()
                .and_then(in_state(ClientJoinState::WaitingForWorld))
                .and_then(resource_exists::<WorldLoad>()),
        ),
    );
}
