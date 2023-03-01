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

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn init(canvas: web_sys::HtmlCanvasElement) -> Result<Engine, JsValue> {
    utils::set_panic_hook();
    let engine = init_engine(canvas);

    Ok(engine)
}
