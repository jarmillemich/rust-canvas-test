use std::{collections::VecDeque, sync::{Arc, Mutex}};

use wasm_bindgen::{prelude::*, convert::FromWasmAbi};
use web_sys::{console, HtmlElement, HtmlCanvasElement};

pub enum InputEvent {
    // Keyboard
    KeyDown { key: char },
    KeyUp { key: char },

    // Mouse
    MouseMove { x: i32, y: i32 },
    MouseDown { button: u8 },
    MouseUp { button: u8 },
}

/// Structure to forward events from JS-land to Rust-land
#[wasm_bindgen]
pub struct EventQueue {
    queue: Arc<Mutex<VecDeque<InputEvent>>>,
}

#[wasm_bindgen]
impl EventQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new()))
        }
    }

    pub fn attach(&self, el: &HtmlCanvasElement) {
        // TODO probably some great leaking going on here
        self.listen(el, "mousemove", |queue, event: web_sys::MouseEvent| {
            queue.lock().unwrap().push_back(InputEvent::MouseMove { x: event.x(), y: event.y() });
            //console::log_1(&format!("Have {} events", queue.lock().unwrap().len()).into());
        });
    }

    fn listen<T: FromWasmAbi + 'static, F: FnMut(&Arc<Mutex<VecDeque<InputEvent>>>, T) -> () + 'static>(&self, el: &HtmlCanvasElement, event: &str, mut cb: F) {
        let queue = self.queue.clone();

        let closure = Closure::<dyn FnMut(_)>::new(move |event: T| {
            cb(&queue, event);
        });
        el.add_event_listener_with_callback(event, closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }
}