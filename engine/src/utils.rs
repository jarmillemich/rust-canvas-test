use wasm_bindgen::JsValue;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn log(s: String) {
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&s.into());
    #[cfg(not(target_arch = "wasm32"))]
    println!("[DEBUG] {}", s);
}

/// In web environments, sets a global variable. Otherwise, logs to stdout.
pub fn debug_variable(name: &str, value: String) {
    #[cfg(target_arch = "wasm32")]
    js_sys::Reflect::set(
        &web_sys::window().unwrap(),
        &JsValue::from(name),
        &value.into(),
    )
    .unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    println!("[VALUE] {}: {}", name, value);
}
