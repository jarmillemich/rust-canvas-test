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
use bevy::prelude::*;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::HtmlCanvasElement;

#[wasm_bindgen(js_name = Engine)]
pub struct Engine {
    is_running: Arc<AtomicBool>,
    app: Arc<Mutex<App>>,
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
        let mut app = self.app.lock().unwrap();
        let loopback = Arc::new(Mutex::new(ConnectionLoopback::new()));
        self.loopback_session = Some(loopback.clone());
        app.insert_non_send_resource(TickCoordinator::new(loopback));
    }

    pub fn connect_as_host(&mut self) {
        self.assert_not_connected();
        let mut app = self.app.lock().unwrap();
        let session = Arc::new(Mutex::new(HostingSession::new()));
        self.hosting_session = Some(session.clone());
        app.insert_non_send_resource(TickCoordinator::new(session));
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
        let mut app = self.app.lock().unwrap();
        let client = Arc::new(Mutex::new(connection));
        self.client_session = Some(client.clone());
        app.insert_non_send_resource(TickCoordinator::new(client));
    }
}

impl Engine {
    pub fn new(canvas: &web_sys::HtmlCanvasElement) -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            app: Arc::new(Mutex::new(init_app(canvas))),
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
        let app = self.app.clone();

        *g.borrow_mut() = Some(Closure::new(move || {
            if !is_running.load(Ordering::Relaxed) {
                f.borrow_mut().take();
                return;
            }

            // dispatcher.dispatch_seq(&app.lock().unwrap());
            app.lock().unwrap().update();

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

fn init_app(canvas: &HtmlCanvasElement) -> App {
    let mut app = App::new();

    // Register systems
    app.add_system(systems::sys_input)
        .add_system(systems::sys_movement_receive)
        .add_system(systems::sys_fire_receive)
        .add_system(systems::sys_movement.after(systems::sys_movement_receive)) // After mvmt receiver
        .add_system(systems::sys_gravity)
        .add_system(systems::sys_renderer)
        .add_system(systems::sys_tick_coordination)
        .add_startup_system(sys_init_world);

    // Add resources
    app.insert_non_send_resource(init_renderer(canvas).unwrap())
        .insert_resource(EventQueue::new_with_canvas(canvas));

    app
}
