use crate::resources::{tick_coordination::tick_queue::TickQueue, TickCoordinator};
use bevy::prelude::*;

/// When connecting as a client, runs until we have loaded the map
pub fn sys_client_init(
    mut commands: Commands,
    type_registry: Res<AppTypeRegistry>,
    sceneAssets: Res<Assets<DynamicScene>>,
) {
    // So, the plan
    // We'll get scheduled when we do connect_as_client
    // We'll have some state from idle -> connecting -> conected
    // We'll run while connecting
    // Once we have the world load data, we'll:
    //  - load the world
    //  - set our finalized tick number
    //  - change to connected
    //   -> which will start scheduling the main simulation
}
