// Types
mod action;
mod fixed_point;
mod input;

// ECS
mod components;
mod resources;
mod systems;
mod utils;

use js_sys::Array;
// Web
use wasm_bindgen::prelude::*;
mod engine;
mod renderer;
use engine::init_engine;

use crate::{
    engine::Engine,
    resources::tick_coordination::{
        connection_to_client::ConnectionToClient, connection_to_host::ConnectionToHost,
    },
};

#[wasm_bindgen]
pub fn init(canvas: web_sys::HtmlCanvasElement) -> Result<Engine, JsValue> {
    utils::set_panic_hook();
    let engine = init_engine(canvas);

    Ok(engine)
}

// #[wasm_bindgen]
// pub fn connect_local(engine: &mut Engine) {
//     engine.connect_local();
// }

// #[wasm_bindgen]
// pub fn connect_as_host(engine: &mut Engine, clients: &[Rtc]) {
//     let clients = clients
//         .iter()
//         .map(|client| {
//             client
//                 .dyn_into::<Box<ConnectionToClient>>()
//                 .expect("Pass in an array of ConnectionToClient")
//         })
//         .collect();
//     engine.connect_as_host(clients);
// }

// #[wasm_bindgen]
// pub fn connect_as_client(engine: &mut Engine, connection: &ConnectionToHost) {
//     engine.connect_as_client(connection);
// }
