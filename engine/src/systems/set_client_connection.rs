use bevy::{
    ecs::entity::EntityMap,
    prelude::{AppTypeRegistry, AssetServer, Assets, Commands, Handle, World, *},
    scene::{
        serde::{SceneDeserializer, SceneSerializer},
        DynamicScene, DynamicSceneBundle, Scene, SceneBundle,
    },
};
use serde::de::DeserializeSeed;

use crate::resources::{tick_coordination::types::WorldLoad, TickCoordinator};

#[derive(States, PartialEq, Debug, Clone, Hash, Default, Eq)]
enum ClientJoinState {
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
    // Start afresh
    // TODO maybe have a marker component so we don't clear special things?
    world.clear_entities();

    let world_load = world.remove_resource::<WorldLoad>().unwrap();
    let scene = world_load.scene;

    // Convert from ron to DynamicScene we can insert
    let scene = {
        let srlz = String::from_utf8(scene).unwrap();
        let mut deserializer = ron::de::Deserializer::from_str(&srlz).unwrap();
        let scene_deserializer = SceneDeserializer {
            type_registry: &world.resource::<AppTypeRegistry>().read(),
        };
        scene_deserializer.deserialize(&mut deserializer).unwrap()
    };

    // Wrap in an asset
    let scene = world
        .get_resource_mut::<Assets<DynamicScene>>()
        .unwrap()
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

    web_sys::console::log_1(&"[client] Loaded world".into());
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
