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
        graphics::{Color, DrawCircle},
        physics::{Gravity, GravityEmitter, MovementReceiver, Position, Velocity},
    },
    input::EventQueue,
    renderer::init_renderer,
    resources::tick_coordination::{
        connection_loopback::ConnectionLoopback, connection_to_client::ConnectionToClient,
        connection_to_host::ConnectionToHost, hosting_session::HostingSession,
        res_tick_coordinator::TickCoordinator,
    },
    systems,
};
use specs::prelude::*;
use wasm_bindgen::{prelude::*, JsCast};

#[wasm_bindgen(js_name = Engine)]
pub struct Engine {
    is_running: Arc<AtomicBool>,
    world: Arc<Mutex<World>>,
    // Eh?
    loopback_session: Option<Arc<Mutex<ConnectionLoopback>>>,
    hosting_session: Option<Arc<Mutex<HostingSession>>>,
    client_session: Option<Arc<Mutex<ConnectionToHost>>>,
}

pub fn init_engine(canvas: web_sys::HtmlCanvasElement) -> Engine {
    Engine::new(&canvas)
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
    fn assert_not_connected(&self) {
        assert!(
            self.loopback_session.is_none()
                && self.hosting_session.is_none()
                && self.client_session.is_none(),
            "Already connected"
        );
    }

    pub fn connect_local(&mut self) {
        self.assert_not_connected();
        let mut world = self.world.lock().unwrap();
        let loopback = Arc::new(Mutex::new(ConnectionLoopback::new()));
        self.loopback_session = Some(loopback.clone());
        world.insert(TickCoordinator::new(loopback));
    }

    pub fn connect_as_host(&mut self) {
        self.assert_not_connected();
        let mut world = self.world.lock().unwrap();
        let session = Arc::new(Mutex::new(HostingSession::new()));
        self.hosting_session = Some(session.clone());
        world.insert(TickCoordinator::new(session));
    }

    pub fn add_client_as_host(&self, client: ConnectionToClient) {
        // Perform initial synchronization
        // TODO what?

        //self.world.lock().unwrap().

        // Add to the session
        self.hosting_session
            .as_ref()
            .expect("Must have hosting session to add client")
            .lock()
            .unwrap()
            .add_client(client);
    }

    pub fn connect_as_client(&mut self, connection: ConnectionToHost) {
        self.assert_not_connected();
        let mut world = self.world.lock().unwrap();
        let client = Arc::new(Mutex::new(connection));
        self.client_session = Some(client.clone());
        world.insert(TickCoordinator::new(client));
    }
}

impl Engine {
    pub fn new(canvas: &web_sys::HtmlCanvasElement) -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            world: Arc::new(Mutex::new(init_world(canvas))),
            //event_queue: Arc::new(Mutex::new(EventQueue::new()))
            //event_queue: EventQueue::new(),
            //dispatcher: Arc::new(init_dispatcher())
            loopback_session: None,
            hosting_session: None,
            client_session: None,
        }
    }

    fn tick(&self) {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        let is_running = self.is_running.clone();
        let world = self.world.clone();
        let mut dispatcher = init_dispatcher();
        //let systems = init_systems();

        *g.borrow_mut() = Some(Closure::new(move || {
            if !is_running.load(Ordering::Relaxed) {
                f.borrow_mut().take();
                return;
            }

            dispatcher.dispatch_seq(&world.lock().unwrap());
            world.lock().unwrap().maintain();

            request_animation_frame(f.borrow().as_ref().unwrap());
        }));

        request_animation_frame(g.borrow().as_ref().unwrap());
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

fn init_world(canvas: &web_sys::HtmlCanvasElement) -> World {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Velocity>();
    world.register::<Gravity>();
    world.register::<Color>();
    world.register::<DrawCircle>();
    world.register::<GravityEmitter>();
    world.register::<MovementReceiver>();

    // Some test entities that are affected by gravity
    for x in 0..8 {
        world
            .create_entity()
            .with(Position::new_f32(-32. * x as f32, -8.))
            .with(Velocity::new_f32(1., -1.))
            .with(Color::new(20 * x as u8, 255 - 16 * x as u8, 0, 255))
            .with(DrawCircle::new(16.))
            .with(Gravity)
            .build();
    }

    // A movable gravity emitter
    world
        .create_entity()
        .with(Position::new_f32(0., 0.))
        .with(Velocity::new_f32(0., 0.))
        .with(GravityEmitter::new())
        .with(MovementReceiver::new())
        .with(Color::new(0, 0, 0, 255))
        .with(DrawCircle::new(8.))
        .build();

    // Add resources
    world.insert(init_renderer(canvas).unwrap());
    let event_queue = EventQueue::new();
    event_queue.attach(canvas);
    world.insert(event_queue);
    //world.insert(TickCoordinator::new(Box::new(ConnectionLoopback::new())));

    world
}

fn init_dispatcher() -> Dispatcher<'static, 'static> {
    DispatcherBuilder::new()
        // Register systems
        .with(systems::SysInput, "Input", &[])
        .with(systems::SysMovementReceiver, "MovementReceiver", &[])
        .with(systems::SysFireReceiver, "FireReceiver", &[])
        .with(systems::SysMovement, "Movement", &["MovementReceiver"])
        .with(systems::SysGravity, "Gravity", &[])
        .with(systems::SysRenderer, "Renderer", &[])
        .with(systems::SysTickCoordinator, "TickCoordinator", &[])
        .build()
}
