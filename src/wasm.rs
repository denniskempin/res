#![cfg(target_arch = "wasm32")]

use crate::app::EmulatorApp;
use eframe::wasm_bindgen;
use eframe::wasm_bindgen::prelude::*;
use eframe::wasm_bindgen::JsValue;

#[wasm_bindgen]
pub fn start_app(canvas_id: &str) -> Result<(), JsValue> {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    eframe::start_web(canvas_id, Box::new(|cc| Box::new(EmulatorApp::new(cc))))
}
