use std::{
    cell::RefCell,
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
            channels::ResChannelManager, ChannelId, ClientConnection, ConnectionToHost,
            NetworkMessage, RtcNetworkChannel, WorldLoad,
        },
        scheduling::{CoordinationState, ResEventQueue, ResTickQueue},
    },
    renderer::init_renderer,
    systems,
};
use bevy::{prelude::*, scene::ScenePlugin};
use futures::channel::oneshot::channel;
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
        }

        self.set_coord_state(CoordinationState::ConnectedLocal);
    }

    /// Starts a hosting session that others can join
    pub fn connect_as_host(&mut self) {
        self.assert_not_connected();
        let mut app = self.app.lock().unwrap();
        app.add_startup_system(sys_init_world);

        app.world
            .get_resource_mut::<NextState<SimulationState>>()
            .unwrap()
            .set(SimulationState::Running);

        self.set_coord_state(CoordinationState::Hosting);
    }

    /// Serializes the world, e.g. to send to a newly connected client
    pub fn serialize_world(&self) -> Vec<u8> {
        // Pack everything into a scene
        let world = &self.app.lock().unwrap().world;
        let type_registry = world.resource::<AppTypeRegistry>();
        let scene = DynamicScene::from_world(world, type_registry);

        // Serialize into a RON string
        // TODO we should perhaps just directly serialize into bytes, this method produces a prettified version
        let serialized = scene.serialize_ron(type_registry).unwrap();

        serialized.into_bytes()
    }

    fn register_channel(&self, connection: RtcDataChannel) -> ChannelId {
        let mut app = self.app.lock().unwrap();
        let mut channel_manager = app
            .world
            .get_non_send_resource_mut::<ResChannelManager>()
            .unwrap();
        let channel = RtcNetworkChannel::new(connection);
        channel_manager.register_channel(Box::new(channel))
    }

    /// Adds a new client to a hosting session
    pub fn add_client_as_host(&self, mut connection: RtcDataChannel) {
        assert!(
            matches!(self.get_coord_state(), CoordinationState::Hosting),
            "Must be hosting to accept clients"
        );

        // Add to the session
        let channel_id = self.register_channel(connection);
        let client_connection = ClientConnection::new(channel_id);
        self.app.lock().unwrap().world.spawn((client_connection,));
    }

    /// Connects to a remote session as a client
    pub fn connect_as_client(&mut self, connection: RtcDataChannel) {
        self.assert_not_connected();
        self.set_coord_state(CoordinationState::ConnectedToHost);

        {
            let channel_id = self.register_channel(connection);
            let client = ConnectionToHost::new(channel_id);
            let mut app = self.app.lock().unwrap();
            app.insert_resource(client);

            use crate::core::networking::ClientJoinState;
            app.world
                .get_resource_mut::<NextState<ClientJoinState>>()
                .unwrap()
                .set(ClientJoinState::WaitingForWorld);
        }
        self.start();
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

            app.lock().unwrap().update();

            request_animation_frame(f.borrow().as_ref().unwrap());
        }));

        request_animation_frame(g.borrow().as_ref().unwrap());
    }

    pub fn test_tick(&self) {
        self.app.lock().unwrap().update();
    }
}

// The JavaScript interface
#[wasm_bindgen]
impl Engine {
    pub fn start(&mut self) {
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
enum SimulationSet {
    BeforeTick,
    Tick,
    AfterTick,
}

fn init_app() -> App {
    let mut app = App::new();

    app.add_state::<SimulationState>();

    let sim_systems = (
        systems::sys_movement_receive,
        systems::sys_fire_receive,
        systems::sys_movement.after(systems::sys_movement_receive),
        systems::sys_gravity,
    );

    // Register systems
    app.add_systems(sim_systems.in_set(SimulationSet::Tick));
    app.add_system(sys_consume_tick.in_set(SimulationSet::AfterTick));

    app.configure_set(SimulationSet::BeforeTick.run_if(can_simulation_proceed));
    app.configure_set(SimulationSet::Tick.run_if(can_simulation_proceed));
    app.configure_set(SimulationSet::AfterTick.run_if(can_simulation_proceed));
    app.configure_sets(
        (
            SimulationSet::BeforeTick,
            SimulationSet::Tick,
            SimulationSet::AfterTick,
        )
            .chain(),
    );

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

fn sys_consume_tick(mut tc: ResMut<ResTickQueue>) {
    tc.advance();
}

/// Determines if we are currently able to advance a tick, as opposed to waiting
fn can_simulation_proceed(tc: Res<ResTickQueue>, sim_state: Res<State<SimulationState>>) -> bool {
    sim_state.0 == SimulationState::Running && tc.is_next_tick_finalized()
}
