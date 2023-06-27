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
    input::EventQueue,
    renderer::init_renderer,
    resources::tick_coordination::{
        connection_loopback::ConnectionLoopback,
        connection_to_client::ConnectionToClient,
        connection_to_host::ConnectionToHost,
        hosting_session::HostingSession,
        res_tick_coordinator::TickCoordinator,
        types::{NetworkMessage, WorldLoad},
    },
    systems,
};
use bevy::{prelude::*, scene::ScenePlugin};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::HtmlCanvasElement;

#[wasm_bindgen(js_name = Engine)]
pub struct Engine {
    is_running: Arc<AtomicBool>,
    app: Arc<Mutex<App>>,
    // TODO perhaps make an enum out of this
    loopback_session: Option<Arc<Mutex<ConnectionLoopback>>>,
    hosting_session: Option<Arc<Mutex<HostingSession>>>,
    client_session: Option<Arc<Mutex<ConnectionToHost>>>,
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
    /// Asserts that we have not already started a session
    fn assert_not_connected(&self) {
        assert!(
            self.loopback_session.is_none()
                && self.hosting_session.is_none()
                && self.client_session.is_none(),
            "Already connected"
        );
    }

    /// Starts a local session
    pub fn connect_local(&mut self) {
        self.assert_not_connected();
        let mut app = self.app.lock().unwrap();
        let loopback = Arc::new(Mutex::new(ConnectionLoopback::new()));
        self.loopback_session = Some(loopback.clone());
        app.insert_non_send_resource(TickCoordinator::new(loopback))
            .add_startup_system(sys_init_world);

        app.world
            .get_resource_mut::<NextState<SimulationState>>()
            .unwrap()
            .set(SimulationState::Running);
    }

    /// Starts a hosting session that others can join
    pub fn connect_as_host(&mut self) {
        self.assert_not_connected();
        let mut app = self.app.lock().unwrap();
        let session = Arc::new(Mutex::new(HostingSession::new()));
        self.hosting_session = Some(session.clone());
        app.insert_non_send_resource(TickCoordinator::new(session))
            .add_startup_system(sys_init_world);

        app.world
            .get_resource_mut::<NextState<SimulationState>>()
            .unwrap()
            .set(SimulationState::Running);
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

    /// Adds a new client to a hosting session
    pub fn add_client_as_host(&self, mut client: ConnectionToClient) {
        assert!(
            self.hosting_session.is_some(),
            "Must be hosting to accept clients"
        );

        let lft = self
            .app
            .lock()
            .unwrap()
            .world
            .get_non_send_resource::<TickCoordinator>()
            .unwrap()
            .get_last_finalized_tick();
        client.set_sync(lft);

        let serialized = self.serialize_world();
        let message = NetworkMessage::World(WorldLoad {
            scene: serialized,
            last_finalized_tick: lft,
        });
        let serialized_message = flexbuffers::to_vec(vec![message]).unwrap();
        client.send_message(serialized_message);

        // Add to the session
        self.hosting_session
            .as_ref()
            .expect("Must have hosting session to add client")
            .lock()
            .unwrap()
            .add_client(client);
    }

    /// Connects to a remote session as a client
    pub fn connect_as_client(&mut self, connection: ConnectionToHost) {
        self.assert_not_connected();
        {
            let mut app = self.app.lock().unwrap();
            let client = Arc::new(Mutex::new(connection));
            self.client_session = Some(client.clone());
            app.insert_non_send_resource(TickCoordinator::new(client.clone()));
            app.insert_non_send_resource(client);

            use systems::set_client_connection::ClientJoinState;
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
            //event_queue: Arc::new(Mutex::new(EventQueue::new()))
            //event_queue: EventQueue::new(),
            //dispatcher: Arc::new(init_dispatcher())
            loopback_session: None,
            hosting_session: None,
            client_session: None,
        }
    }

    pub fn attach_canvas(&self, canvas: &HtmlCanvasElement) {
        let mut app = self.app.lock().unwrap();
        // Add resources
        app.insert_non_send_resource(init_renderer(canvas).unwrap())
            .insert_resource(EventQueue::new_with_canvas(canvas))
            .add_system(systems::sys_renderer)
            .add_system(systems::sys_input);
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
        GravityEmitter::new(),
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
    app.add_systems(sim_systems.distributive_run_if(can_simulation_proceed))
        .add_system(systems::sys_tick_coordination);

    systems::setup_systems(&mut app);

    // Eh
    app.add_plugin(AssetPlugin { ..default() })
        .add_plugin(ScenePlugin);

    // Register any components that should be synced
    components::register_components(&mut app);

    app
}

/// Determines if we are currently able to advance a tick, as opposed to waiting
fn can_simulation_proceed(
    tc: NonSend<TickCoordinator>,
    sim_state: Res<State<SimulationState>>,
) -> bool {
    sim_state.0 == SimulationState::Running // && tc.is_next_tick_finalized()
}
