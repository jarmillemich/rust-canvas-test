use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use bevy::prelude::{Res, ResMut, Resource};
use wasm_bindgen::{convert::FromWasmAbi, prelude::*};
use web_sys::{EventTarget, HtmlCanvasElement};

use super::{Action, Direction, ResActionQueue};

#[allow(unused)]
pub enum InputEvent {
    // Keyboard
    KeyDown { key: String },
    KeyUp { key: String },

    // Mouse
    MouseMove { x: i32, y: i32 },
    MouseDown { button: u8 },
    MouseUp { button: u8 },
}

/// Structure to forward events from JS-land to Rust-land
#[wasm_bindgen]
#[derive(Default, Resource)]
pub struct ResEventQueue {
    queue: Arc<Mutex<VecDeque<InputEvent>>>,
}

#[wasm_bindgen]
impl ResEventQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn new_with_canvas(el: &HtmlCanvasElement) -> Self {
        let queue = ResEventQueue::new();
        queue.attach(el);
        queue
    }

    pub fn attach(&self, el: &HtmlCanvasElement) {
        // TODO probably some great leaking going on here
        // self.listen(el, "mousemove", |queue, event: web_sys::MouseEvent| {
        //     queue.lock().unwrap().push_back(InputEvent::MouseMove {
        //         x: event.x(),
        //         y: event.y(),
        //     });
        // });

        self.listen(
            &el.owner_document().unwrap(),
            "keydown",
            |queue, event: web_sys::KeyboardEvent| {
                queue
                    .lock()
                    .unwrap()
                    .push_back(InputEvent::KeyDown { key: event.key() });
            },
        );

        self.listen(
            &el.owner_document().unwrap(),
            "keyup",
            |queue, event: web_sys::KeyboardEvent| {
                queue
                    .lock()
                    .unwrap()
                    .push_back(InputEvent::KeyUp { key: event.key() });
            },
        );
    }

    fn listen<
        T: FromWasmAbi + 'static,
        F: FnMut(&Arc<Mutex<VecDeque<InputEvent>>>, T) + 'static,
    >(
        &self,
        el: &EventTarget,
        event: &str,
        mut cb: F,
    ) {
        let queue = self.queue.clone();

        let closure = Closure::<dyn FnMut(_)>::new(move |event: T| {
            cb(&queue, event);
        });
        el.add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }
}

impl ResEventQueue {
    pub fn items(&self) -> &Arc<Mutex<VecDeque<InputEvent>>> {
        &self.queue
    }
}

pub fn sys_input(mut action_queue: ResMut<ResActionQueue>, event_queue: Res<ResEventQueue>) {
    // TODO does this keep the lock for the entire loop?
    for event in event_queue.items().lock().unwrap().drain(..) {
        let action: Action = match event {
            InputEvent::KeyDown { key } => match key.as_str() {
                "w" => Action::StartMoving { dir: Direction::Up },
                "a" => Action::StartMoving {
                    dir: Direction::Left,
                },
                "s" => Action::StartMoving {
                    dir: Direction::Down,
                },
                "d" => Action::StartMoving {
                    dir: Direction::Right,
                },
                " " => Action::Fire,
                _ => continue,
            },

            InputEvent::KeyUp { key } => match key.as_str() {
                "w" => Action::StopMoving { dir: Direction::Up },
                "a" => Action::StopMoving {
                    dir: Direction::Left,
                },
                "s" => Action::StopMoving {
                    dir: Direction::Down,
                },
                "d" => Action::StopMoving {
                    dir: Direction::Right,
                },
                _ => continue,
            },
            // Ignore unhandled events
            _ => continue,
        };

        // Request that the action be scheduled
        action_queue.add_action(action);
    }
}
