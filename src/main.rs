#![cfg(not(target_arch = "wasm32"))]

use chip8emu::app::EmulatorApp;

fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions {
        drag_and_drop_support: true,
        ..Default::default()
    };

    eframe::run_native(
        "Chip8 Emulator",
        native_options,
        Box::new(|cc| Box::new(EmulatorApp::new(cc))),
    );
}
