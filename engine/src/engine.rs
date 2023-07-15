use std::{
    cell::RefCell,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use crate::{
    components::{
        self,
        graphics::{Color, DrawCircle},
        physics::{Gravity, GravityEmitter, MovementReceiver, Position, Velocity},
    },
    core::{
        networking::{
            channels::ResChannelManager, serialize_world, ChannelId, ClientConnection,
            ConnectionToHost, NetworkChannel, RtcNetworkChannel,
        },
        scheduling::{CoordinationState, ResEventQueue, ResTickQueue},
    },
    renderer::init_renderer,
    systems,
    utils::{debug_variable, log},
};
use bevy::{
    ecs::{entity::Entities, schedule::ScheduleLabel},
    prelude::*,
    scene::ScenePlugin,
};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{HtmlCanvasElement, RtcDataChannel};

#[wasm_bindgen(js_name = Engine)]
pub struct Engine {
    is_running: Arc<AtomicBool>,
    app: Arc<Mutex<App>>,
}

// TESTING
impl Engine {
    pub fn get_app(&self) -> Arc<Mutex<App>> {
        self.app.clone()
    }
}

pub fn init_engine(canvas: web_sys::HtmlCanvasElement) -> Engine {
    let engine = Engine::new();
    engine.attach_canvas(&canvas);
    engine
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen(js_class = Engine)]
impl Engine {
    fn get_coord_state(&self) -> CoordinationState {
        let app = self.app.lock().unwrap();
        let world = &app.world;

        let coord_state = world
            .get_resource::<State<CoordinationState>>()
            .expect("Should have a CoordinationState resource");

        coord_state.0.clone()
    }

    fn set_coord_state(&self, state: CoordinationState) {
        let mut app = self.app.lock().unwrap();

        app.world
            .get_resource_mut::<NextState<CoordinationState>>()
            .unwrap()
            .set(state)
    }

    /// Asserts that we have not already started a session
    fn assert_not_connected(&self) {
        assert!(
            matches!(self.get_coord_state(), CoordinationState::Disconnected),
            "Already connected"
        );
    }

    /// Starts a local session
    pub fn connect_local(&mut self) {
        self.assert_not_connected();
        {
            let mut app = self.app.lock().unwrap();

            app.add_startup_system(sys_init_world);

            app.world
                .get_resource_mut::<NextState<SimulationState>>()
                .unwrap()
                .set(SimulationState::Running);

            app.insert_resource(ResLogger::new("LOCAL"));
        }

        self.set_coord_state(CoordinationState::ConnectedLocal);
    }

    /// Starts a hosting session that others can join
    pub fn connect_as_host(&mut self) {
        self.assert_not_connected();
        {
            let mut app = self.app.lock().unwrap();
            app.add_startup_system(sys_init_world);

            app.world
                .get_resource_mut::<NextState<SimulationState>>()
                .unwrap()
                .set(SimulationState::Running);

            app.insert_resource(ResLogger::new("HOST"));
        }

        self.set_coord_state(CoordinationState::Hosting);
    }

    pub fn add_client_as_host_rtc(&self, connection: RtcDataChannel) {
        let channel = Box::new(RtcNetworkChannel::new(connection));
        self.add_client_as_host(channel)
    }

    pub fn connect_as_client_rtc(&mut self, connection: RtcDataChannel) {
        let channel = Box::new(RtcNetworkChannel::new(connection));
        self.connect_as_client(channel)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            app: Arc::new(Mutex::new(init_app())),
        }
    }

    pub fn attach_canvas(&self, canvas: &HtmlCanvasElement) {
        let mut app = self.app.lock().unwrap();
        // Add resources
        app.insert_non_send_resource(init_renderer(canvas).unwrap())
            .insert_resource(ResEventQueue::new_with_canvas(canvas))
            .add_system(systems::sys_renderer)
            .add_system(crate::core::scheduling::sys_input);
    }

    fn tick(&self) {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        let is_running = self.is_running.clone();
        let app = self.app.clone();

        *g.borrow_mut() = Some(Closure::new(move || {
            if !is_running.load(Ordering::Relaxed) {
                f.borrow_mut().take();
                return;
            }

            let mut app = app.lock().unwrap();

            // Catch up ticks, if needed
            let tc = app.world.get_resource::<ResTickQueue>().unwrap();
            let current = tc.get_last_simulated_tick();
            let max = tc.get_last_finalized_tick();
            if max - current > 1 {
                // We are behind, catch up
                for _ in current..max {
                    app.update();
                }
            }

            app.update();

            request_animation_frame(f.borrow().as_ref().unwrap());
        }));

        request_animation_frame(g.borrow().as_ref().unwrap());
    }

    pub fn test_tick(&self) {
        self.app.lock().unwrap().update();
    }

    fn register_channel(&self, channel: Box<dyn NetworkChannel>) -> ChannelId {
        let mut app = self.app.lock().unwrap();
        let mut channel_manager = app
            .world
            .get_non_send_resource_mut::<ResChannelManager>()
            .unwrap();

        channel_manager.register_channel(channel)
    }

    /// Adds a new client to a hosting session
    pub fn add_client_as_host(&self, channel: Box<dyn NetworkChannel>) {
        assert!(
            matches!(self.get_coord_state(), CoordinationState::Hosting),
            "Must be hosting to accept clients"
        );

        // Add to the session
        let channel_id = self.register_channel(channel);
        let client_connection = ClientConnection::new(channel_id);
        self.app.lock().unwrap().world.spawn((client_connection,));
    }

    /// Connects to a remote session as a client
    pub fn connect_as_client(&mut self, channel: Box<dyn NetworkChannel>) {
        self.assert_not_connected();
        self.set_coord_state(CoordinationState::ConnectedToHost);

        {
            let channel_id = self.register_channel(channel);
            let client = ConnectionToHost::new(channel_id);
            let mut app = self.app.lock().unwrap();
            app.insert_resource(client);

            use crate::core::networking::ClientJoinState;
            app.world
                .get_resource_mut::<NextState<ClientJoinState>>()
                .unwrap()
                .set(ClientJoinState::NeedsSendInitialPing);

            app.insert_resource(ResLogger::new("CLIENT"));
        }
    }
}

// The JavaScript interface
#[wasm_bindgen]
impl Engine {
    pub fn start_web(&mut self) {
        let was_running = self.is_running.fetch_or(true, Ordering::Relaxed);
        if !was_running {
            self.tick();
        }
    }

    pub fn stop(&mut self) {
        self.is_running.fetch_and(false, Ordering::Relaxed);
    }
}

fn sys_init_world(mut commands: Commands) {
    // Some test entities that are affected by gravity
    for x in 0..8 {
        commands.spawn((
            Position::new_f32(-32. * x as f32, -8.),
            Velocity::new_f32(1., -1.),
            Color::new(20 * x as u8, 255 - 16 * x as u8, 0, 255),
            DrawCircle::new(16.),
            Gravity,
        ));
    }

    // A movable gravity emitter
    commands.spawn((
        Position::new_f32(0., 0.),
        Velocity::new_f32(0., 0.),
        GravityEmitter,
        MovementReceiver::new(),
        Color::new(0, 0, 0, 255),
        DrawCircle::new(8.),
    ));
}

#[derive(States, PartialEq, Debug, Clone, Hash, Default, Eq)]
pub enum SimulationState {
    #[default]
    Paused,
    Running,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SimulationSet {
    /// "Before simulation" network systems. Always runs. Use this to e.g. receive messages and queue anything up for the simulation
    NetworkPre,
    /// "Regular" network systems. Always runs
    NetworkPost,

    /// Systems to run before the main simulation set
    BeforeTick,
    /// Primary simulation systems
    Tick,
    /// Systems to run after the main simulation set
    AfterTick,
}

#[derive(ScheduleLabel, Hash, Debug, Eq, PartialEq, Clone)]
pub enum SomeLabelsMaybe {
    Simulation,
}

fn sys_tick_logger(tc: Res<ResTickQueue>) {
    let to_simulate = tc.get_next_tick_to_simulate();
    let finalized = tc.get_last_finalized_tick();
    let num_actions = tc.current_tick_actions().len();
    let can_proceed = tc.is_next_tick_finalized();

    debug_variable(
        "__debug_current_tick",
        format!(
            "Tick {}/{} with {} actions, can proceed: {}",
            to_simulate, finalized, num_actions, can_proceed
        ),
    );
}

fn sys_sim_runner(world: &mut World) {
    let sim_state = world.get_resource_mut::<State<SimulationState>>().unwrap();
    if sim_state.0 == SimulationState::Running {
        world.run_schedule(SomeLabelsMaybe::Simulation);
    }
}

#[derive(Resource, Default)]
pub struct ResLastTickHash(pub u64);

#[derive(Resource, Default)]
pub struct ResLogger {
    tag: String,
}

impl ResLogger {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
        }
    }

    pub fn log(&self, msg: &str) {
        self.log_inner(format!("[{}] LOG: {}", self.tag, msg).as_str());
    }

    pub fn debug(&self, msg: &str) {
        self.log_inner(format!("[{}] DBG: {}", self.tag, msg).as_str());
    }

    fn log_inner(&self, msg: &str) {
        // #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&msg.into());
        #[cfg(not(target_arch = "wasm32"))]
        println!("{msg}");
    }
}

fn init_app() -> App {
    let mut app = App::new();

    app.add_state::<SimulationState>();

    let sim_schedule = Schedule::default();
    app.add_schedule(SomeLabelsMaybe::Simulation, sim_schedule);

    let sim_systems = (
        systems::sys_movement_receive,
        systems::sys_fire_receive,
        systems::sys_movement.after(systems::sys_movement_receive),
        systems::sys_gravity,
    );

    // Register systems
    app.add_systems(
        sim_systems
            .in_set(SimulationSet::Tick)
            .in_schedule(SomeLabelsMaybe::Simulation),
    );
    app.add_system(sys_sim_runner.in_set(SimulationSet::Tick));

    app.add_system(
        sys_tick_logger
            .before(SimulationSet::Tick)
            .after(SimulationSet::BeforeTick),
    );

    // TODO no distributive_run_if here?
    app.configure_set(SimulationSet::BeforeTick.run_if(can_simulation_proceed));
    app.configure_set(SimulationSet::Tick.run_if(can_simulation_proceed));
    app.configure_set(SimulationSet::AfterTick.run_if(can_simulation_proceed));
    app.configure_sets(
        (
            SimulationSet::NetworkPre,
            SimulationSet::BeforeTick,
            SimulationSet::Tick,
            SimulationSet::AfterTick,
        )
            .chain(),
    );

    // app.add_system(sys_pretick_log.in_set(SimulationSet::BeforeTick));
    app.insert_resource(ResLastTickHash::default());
    app.add_systems(
        (sys_world_checksum, sys_consume_tick)
            .chain()
            .in_set(SimulationSet::AfterTick),
    );

    // NetworkPost is after NetworkPre but may run in parallel with the simulation
    app.configure_set(SimulationSet::NetworkPost.after(SimulationSet::NetworkPre));

    // Attach our modules
    crate::core::networking::attach_to_app(&mut app);
    crate::core::scheduling::attach_to_app(&mut app);

    // Needed for loading a DynamicScene, i.e. from a RON stream
    app.add_plugin(AssetPlugin { ..default() })
        .add_plugin(ScenePlugin);

    // Register any components that should be synced
    components::register_components(&mut app);

    app
}

// fn sys_pretick_log(logger: Res<ResLogger>, tc: Res<ResTickQueue>, entities: &Entities) {
//     logger.log(
//         format!(
//             "About to simulate tick {} with {}",
//             tc.get_next_tick_to_simulate(),
//             entities.len(),
//         )
//         .as_str(),
//     );
// }

fn sys_world_checksum(world: &mut World) {
    let tick = world
        .get_resource::<ResTickQueue>()
        .unwrap()
        .get_last_simulated_tick();
    // let logger = world.get_resource::<ResLogger>().unwrap();

    let mut query = world.query::<&Position>();
    let hash = query.iter(world).fold(0, |acc, at| {
        //(|at| at.hash(&mut hasher));
        let mut hasher = DefaultHasher::new();
        at.hash(&mut hasher);
        let hash = hasher.finish();
        // log(format!("  Hash: {:?} {:x}", at, hash));
        acc ^ hash
    });

    //debug_variable("__debug_world_hash", format!("{} 0x{:x}", tick, hash));
    // let logger = world.get_resource::<ResLogger>().unwrap();
    // logger.debug(format!("Checksum at {} 0x{:x}", tick, hash).as_str());

    if tick % 100 == 0 {
        log(format!("World hash @ {}: 0x{:x}", tick, hash));
    }

    world.get_resource_mut::<ResLastTickHash>().unwrap().0 = hash;
}

/// Schedule after we have finished simulating a tick
fn sys_consume_tick(
    mut tc: ResMut<ResTickQueue>,
    lth: Res<ResLastTickHash>,
    // logger: Res<ResLogger>,
) {
    // logger.log(format!("Done with tick {}", tc.get_next_tick_to_simulate()).as_str());
    tc.advance(lth.0);
}

/// Determines if we are currently able to advance a tick, as opposed to waiting
fn can_simulation_proceed(tc: Res<ResTickQueue>, sim_state: Res<State<SimulationState>>) -> bool {
    sim_state.0 == SimulationState::Running && tc.is_next_tick_finalized()
}
