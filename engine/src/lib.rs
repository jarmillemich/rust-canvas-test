#![feature(box_syntax)]

mod components;
mod resources;
mod systems;
mod utils;

mod action;
mod input;

use wasm_bindgen::prelude::*;
mod engine;
mod renderer;
use engine::init_engine;

use crate::engine::Engine;

#[wasm_bindgen]
pub fn init(canvas: web_sys::HtmlCanvasElement) -> Result<Engine, JsValue> {
    utils::set_panic_hook();
    let engine = init_engine(canvas);

    Ok(engine)
}
