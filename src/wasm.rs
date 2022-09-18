#![cfg(target_arch = "wasm32")]

use crate::app::EmulatorApp;
use crate::nes::System;
use base64;
use eframe::wasm_bindgen;
use eframe::wasm_bindgen::prelude::*;
use eframe::wasm_bindgen::JsValue;
use web_sys;

pub fn save_rom_in_local_storage(rom: &[u8]) {
    let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    storage.set_item("rom", &base64::encode(rom)).unwrap();
}

#[wasm_bindgen]
pub fn start_app(canvas_id: &str) -> Result<(), JsValue> {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    eframe::start_web(
        canvas_id,
        Box::new(|cc| {
            let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
            let initial_rom = storage.get_item("rom").unwrap();
            let initial_rom = initial_rom.map(|raw| base64::decode(raw).unwrap());
            Box::new(EmulatorApp::new(cc, initial_rom))
        }),
    )
}
