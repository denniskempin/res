#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use argh::FromArgs;
use egui::vec2;
use res::app::EmulatorApp;
use res::app::Rom;

/// Rust Entertainment System
#[derive(FromArgs)]
struct ResArgs {
    /// rom file to load
    #[argh(positional)]
    rom: Option<String>,
}

fn main() {
    let args: ResArgs = argh::from_env();

    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions {
        drag_and_drop_support: true,
        initial_window_size: Some(vec2(1400.0, 800.0)),
        ..Default::default()
    };

    let rom = args.rom.map(|path| {
        let path = PathBuf::from(path);
        Rom::load_from_file(&path)
    });

    eframe::run_native(
        "NES Emulator",
        native_options,
        Box::new(|cc| Box::new(EmulatorApp::new(cc, rom))),
    );
}
