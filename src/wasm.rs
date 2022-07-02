#![cfg(target_arch = "wasm32")]

use crate::app::Emulator;
use crate::app::EmulatorApp;
use eframe::wasm_bindgen;
use eframe::wasm_bindgen::prelude::*;
use eframe::wasm_bindgen::JsValue;
use std::sync::Arc;
use std::sync::Mutex;

#[wasm_bindgen]
pub fn start_app(canvas_id: &str) -> Result<(), JsValue> {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    eframe::start_web(
        canvas_id,
        Box::new(|cc| {
            let emulator = Arc::new(Mutex::new(Emulator::new(
                include_bytes!("ibm_logo.ch8"),
                cc.egui_ctx.clone(),
            )));

            // TODO: Need to use WebWorker to do threading.

            Box::new(EmulatorApp::new(cc, emulator.clone()))
        }),
    )
}
