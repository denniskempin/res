#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use argh::FromArgs;
use egui::vec2;
use res_egui::EmulatorApp;
use res_egui::Rom;
use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::prelude::*;

/// Rust Entertainment System
#[derive(FromArgs)]
struct ResArgs {
    /// rom file to load
    #[argh(positional)]
    rom: Option<String>,

    /// enable generation of trace files
    #[argh(option)]
    trace_file: Option<PathBuf>,
}

fn main() {
    let args: ResArgs = argh::from_env();

    let _tracing_guard = if let Some(trace_file) = args.trace_file {
        let (chrome_layer, guard) = ChromeLayerBuilder::new().file(trace_file).build();
        tracing_subscriber::registry().with(chrome_layer).init();
        Some(guard)
    } else {
        None
    };

    let native_options = eframe::NativeOptions {
        drag_and_drop_support: true,
        initial_window_size: Some(vec2(1600.0, 1000.0)),
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
