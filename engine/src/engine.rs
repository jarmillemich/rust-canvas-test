use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::components::graphics::{Color, DrawCircle};
use crate::components::physics::{Gravity, GravityEmitter, MovementReceiver, Position, Velocity};
use crate::input::EventQueue;
use crate::renderer::init_renderer;
use crate::resources::TickCoordinator;
use crate::systems;
use specs::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
pub struct Engine {
    is_running: Arc<AtomicBool>,
    world: Arc<Mutex<World>>,
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

impl Engine {
    pub fn new(canvas: &web_sys::HtmlCanvasElement) -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            world: Arc::new(Mutex::new(init_world(canvas))),
            //event_queue: Arc::new(Mutex::new(EventQueue::new()))
            //event_queue: EventQueue::new(),
            //dispatcher: Arc::new(init_dispatcher())
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
    world.insert(TickCoordinator::new());

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
