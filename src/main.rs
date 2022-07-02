#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use chip8emu::app::Emulator;
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
        Box::new(|cc| {
            let emulator = Arc::new(Mutex::new(Emulator::new(
                include_bytes!("ibm_logo.ch8"),
                cc.egui_ctx.clone(),
            )));

            {
                let emulator = emulator.clone();
                thread::spawn(move || {
                    Emulator::main_loop(emulator);
                });
            }

            Box::new(EmulatorApp::new(cc, emulator))
        }),
    );
}
