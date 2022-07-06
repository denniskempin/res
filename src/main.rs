#![cfg(not(target_arch = "wasm32"))]

use chip8emu::app::EmulatorApp;
use egui::vec2;

fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions {
        drag_and_drop_support: true,
        initial_window_size: Some(vec2(1400.0, 800.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Chip8 Emulator",
        native_options,
        Box::new(|cc| Box::new(EmulatorApp::new(cc))),
    );
}
