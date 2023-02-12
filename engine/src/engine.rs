use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use specs::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use crate::components::physics::{Gravity, Position, Velocity};
use crate::renderer::init_renderer;
use crate::systems::{self, SysMovement};


#[wasm_bindgen]
pub struct Engine {
    is_running: Arc<AtomicBool>,
    world: Arc<Mutex<World>>,
    //dispatcher: Arc<Dispatcher<'static, 'static>>,
}

pub fn init_engine(canvas: web_sys::HtmlCanvasElement) -> Engine {
    Engine::new(canvas)
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
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            world: Arc::new(Mutex::new(init_world(canvas))),
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

fn init_world(canvas: web_sys::HtmlCanvasElement) -> World {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Velocity>();
    world.register::<Gravity>();

    world.create_entity()
        .with(Position::new(0, 0))
        .with(Velocity::new(1, 1))
        .with(Gravity)
        .build();

    world.insert(init_renderer(canvas));

    world
}

fn init_dispatcher() -> Dispatcher<'static, 'static> {
    DispatcherBuilder::new()
        .with(systems::SysMovement, "Movement", &[])
        .build()
}

fn init_systems() -> Vec<Box<dyn RunNow<'static>>> {
    vec![
        box SysMovement
    ]
}