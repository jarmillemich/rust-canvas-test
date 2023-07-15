#![cfg(test)]
use crate::{
    core::{networking::serialize_world, scheduling::ResTickQueue},
    engine::{Engine, ResLastTickHash, ResLogger, SimulationSet},
};
use bevy::prelude::*;
use std::str;

#[test]
fn test_tick_queue_network_sync() {
    // Tests that a client tick queue will stay in sync with a host tick queue
}

#[test]
fn sync_test() {
    let (mut host, mut client) = make_host_client_pair();

    println!("=== Beginning sync test loop ===");

    for _ in 0..100 {
        host.test_tick();
        client.test_tick();

        get_client_in_sync(&mut host, &mut client);

        let host_hash = get_engine_hash(&mut host);
        let client_hash = get_engine_hash(&mut client);
        assert_eq!(
            host_hash,
            client_hash,
            "Out of sync at host={} client={}",
            get_last_simulated_tick(&mut host),
            get_last_simulated_tick(&mut client)
        );
    }
}

fn make_host_client_pair() -> (Engine, Engine) {
    use crate::core::networking::make_fake_network_channel_pair;

    let mut host = Engine::new();
    let mut client = Engine::new();

    // add_diagnostic_logger(&mut host);
    // add_diagnostic_logger(&mut client);

    host.connect_as_host();

    // Cannot start accepting clients immediately, TODO
    host.test_tick();

    let (host_channel, client_channel) = make_fake_network_channel_pair();

    host.add_client_as_host(host_channel);
    client.connect_as_client(client_channel);

    // Tick a couple more times to get in sync
    for _ in 0..5 {
        host.test_tick();
        client.test_tick();
    }

    get_client_in_sync(&mut host, &mut client);

    (host, client)
}

fn add_diagnostic_logger(engine: &mut Engine) {
    let app = engine.get_app();
    app.lock()
        .unwrap()
        .add_system(sys_diagnostic_logger.in_set(SimulationSet::AfterTick));
}

fn sys_diagnostic_logger(world: &mut World) {
    let tc = world.get_resource::<ResTickQueue>().unwrap();
    let current = tc.get_last_simulated_tick();

    let srlz = serialize_world(world);
    let srlz = str::from_utf8(srlz.as_slice()).unwrap();

    let logger = world.get_resource::<ResLogger>().unwrap();
    logger.log(format!("Tick {}\n{}", current, srlz).as_str())
}

fn get_engine_hash(engine: &mut Engine) -> u64 {
    let app = engine.get_app();
    let app = app.lock().unwrap();
    let world = &app.world;
    let last_hash = world.get_resource::<ResLastTickHash>().unwrap();
    last_hash.0
}

fn get_last_simulated_tick(engine: &mut Engine) -> usize {
    let app = engine.get_app();
    let app = app.lock().unwrap();
    let world = &app.world;
    let tick_queue = world.get_resource::<ResTickQueue>().unwrap();
    tick_queue.get_last_simulated_tick()
}

fn get_client_in_sync(host: &mut Engine, client: &mut Engine) {
    // Tick until the client is in sync with the host
    let target_host_tick = get_last_simulated_tick(host);
    let mut last_tick = get_last_simulated_tick(client);
    while get_last_simulated_tick(client) < target_host_tick {
        client.test_tick();

        let last_client_tick = get_last_simulated_tick(client);
        assert!(
            last_client_tick > last_tick,
            "Client did not proceed {} -> {}",
            last_client_tick,
            target_host_tick
        );
        last_tick = last_client_tick;
    }
}
