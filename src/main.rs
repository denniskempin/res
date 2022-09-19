#![cfg(not(target_arch = "wasm32"))]

use std::fs::File;
use std::io::Read;

use egui::vec2;
use res::app::EmulatorApp;

fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions {
        drag_and_drop_support: true,
        initial_window_size: Some(vec2(1400.0, 800.0)),
        ..Default::default()
    };

    let rom = std::env::args().nth(1).map(|path| {
        let mut data: Vec<u8> = Vec::new();
        File::open(path).unwrap().read_to_end(&mut data).unwrap();
        data
    });

    eframe::run_native(
        "NES Emulator",
        native_options,
        Box::new(|cc| Box::new(EmulatorApp::new(cc, rom))),
    );
}
