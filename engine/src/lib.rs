#[macro_use]
extern crate custom_derive;
#[macro_use]
extern crate newtype_derive;

// Types
mod fixed_point;

// ECS
mod components;
mod resources;
mod systems;
mod utils;

// Modules
pub mod core;

// Web
use wasm_bindgen::prelude::*;
pub mod engine;
mod renderer;
use engine::init_engine;

use crate::engine::Engine;

#[wasm_bindgen]
pub fn init(canvas: web_sys::HtmlCanvasElement) -> Result<Engine, JsValue> {
    utils::set_panic_hook();
    let engine = init_engine(canvas);

    Ok(engine)
}
