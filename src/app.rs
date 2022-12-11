mod audio;
mod debugger_ui;

use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::Read;

use eframe::CreationContext;
use eframe::Frame;
use egui::ColorImage;
use egui::Context;
use egui::DroppedFile;
use egui::FontId;
use egui::Image;
use egui::InputState;
use egui::Key;
use egui::Layout;
use egui::Sense;
use egui::TextureHandle;
use egui::Ui;

use self::audio::AudioEngine;
use self::debugger_ui::Debugger;
use crate::nes::joypad::JoypadButton;
use crate::nes::Record;
use crate::nes::System;

const PROGRAMS: &[(&str, &[u8])] = &[
    (
        "nestest",
        include_bytes!("../roms/programs/nestest.nes").as_slice(),
    ),
    (
        "instr_test_v5",
        include_bytes!("../roms/programs/instr_test_v5.nes").as_slice(),
    ),
    (
        "scanline",
        include_bytes!("../roms/programs/scanline.nes").as_slice(),
    ),
];

const GAMES: &[(&str, &[u8])] = &[
    (
        "Blaster",
        include_bytes!("../roms/games/blaster.nes").as_slice(),
    ),
    (
        "Alter Ego",
        include_bytes!("../roms/games/alter_ego.nes").as_slice(),
    ),
    (
        "Lan Master",
        include_bytes!("../roms/games/lan_master.nes").as_slice(),
    ),
];

pub struct EmulatorApp {
    emulator: System,
    loaded: bool,
    framebuffer_texture: TextureHandle,
    debug_mode: bool,
    debug_state: Debugger,
    audio_engine: AudioEngine,
}

impl EmulatorApp {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext<'_>, rom: Option<Vec<u8>>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let loaded = rom.is_some();
        EmulatorApp {
            emulator: if let Some(rom) = rom {
                System::with_ines_bytes(&rom).unwrap()
            } else {
                System::new()
            },
            loaded,
            framebuffer_texture: cc.egui_ctx.load_texture(
                "Framebuffer",
                ColorImage::example(),
                Default::default(),
            ),
            debug_mode: true,
            debug_state: Debugger::new(cc),
            audio_engine: AudioEngine::new(),
        }
    }

    fn load_rom(&mut self, data: &[u8]) {
        self.emulator = System::with_ines_bytes(data).unwrap();
        self.loaded = true;
    }

    fn load_dropped_file(&mut self, drop: &DroppedFile) {
        if let Some(path) = &drop.path {
            match path.extension().and_then(OsStr::to_str) {
                Some("json") => {
                    let data = fs::read_to_string(path).unwrap();
                    let record: Record = serde_json::from_str(&data).unwrap();
                    self.emulator.playback_from = Some(record);
                }
                Some("nes") => {
                    let mut data: Vec<u8> = Vec::new();
                    File::open(path).unwrap().read_to_end(&mut data).unwrap();
                    self.load_rom(&data);
                }
                _ => {
                    panic!("Unknown file type");
                }
            }
        } else if let Some(bytes) = &drop.bytes {
            #[cfg(target_arch = "wasm32")]
            crate::wasm::save_rom_in_local_storage(bytes);
            self.load_rom(bytes);
        }
    }

    fn update_keys(&mut self, input: &InputState) {
        let mut joypad0 = [false; 8];
        if input.key_down(Key::ArrowRight) {
            joypad0[JoypadButton::Right as usize] = true;
        }
        if input.key_down(Key::ArrowLeft) {
            joypad0[JoypadButton::Left as usize] = true;
        }
        if input.key_down(Key::ArrowDown) {
            joypad0[JoypadButton::Down as usize] = true;
        }
        if input.key_down(Key::ArrowUp) {
            joypad0[JoypadButton::Up as usize] = true;
        }
        if input.key_down(Key::S) {
            joypad0[JoypadButton::Start as usize] = true;
        }
        if input.key_down(Key::A) {
            joypad0[JoypadButton::Select as usize] = true;
        }
        if input.key_down(Key::Z) {
            joypad0[JoypadButton::ButtonB as usize] = true;
        }
        if input.key_down(Key::X) {
            joypad0[JoypadButton::ButtonA as usize] = true;
        }
        self.emulator.update_buttons(joypad0);
    }

    fn menu_bar(&mut self, ui: &mut Ui) {
        ui.columns(2, |columns| {
            columns[0].with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                if ui.button("Play Audio").clicked() {
                    self.audio_engine.start();
                }
                ui.menu_button("Programs", |ui| {
                    for program in PROGRAMS {
                        if ui.button(program.0).clicked() {
                            self.load_rom(program.1);
                        }
                    }
                });
                ui.menu_button("Games", |ui| {
                    for program in GAMES {
                        if ui.button(program.0).clicked() {
                            self.load_rom(program.1);
                        }
                    }
                });
                ui.label("(Or drop a .nes file to load it)");
            });
            columns[1].with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                if ui.button("Debug").clicked() {
                    self.debug_mode = !self.debug_mode;
                }
                if let Some(record) = &mut self.emulator.record_to {
                    if ui.button("Save Recording").clicked() {
                        std::fs::write(
                            "recording.json",
                            serde_json::to_string_pretty(&record).unwrap(),
                        )
                        .unwrap();
                        self.emulator.record_to = None;
                    }
                } else if ui.button("Record").clicked() {
                    self.emulator.record_to = Some(Record::default());
                }
            });
        });
    }

    fn main_display(&mut self, ui: &mut Ui) {
        self.framebuffer_texture.set(
            self.emulator.ppu().framebuffer.as_color_image(),
            Default::default(),
        );

        let desired_size = ui.available_size();
        let (whole_rect, _) =
            ui.allocate_exact_size(desired_size, Sense::focusable_noninteractive());

        let image = Image::new(
            &self.framebuffer_texture,
            self.framebuffer_texture.size_vec2(),
        );
        image.paint_at(ui, whole_rect);
    }
}

impl eframe::App for EmulatorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // Load new program if a file is dropped on the app
        if !ctx.input().raw.dropped_files.is_empty() {
            self.load_dropped_file(&ctx.input().raw.dropped_files[0]);
        }
        self.update_keys(&ctx.input());

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.menu_bar(ui);
        });

        if !self.loaded {
            return;
        }

        if !self.debug_mode {
            self.emulator.execute_one_frame().unwrap();
        } else {
            self.debug_state.run_emulator(&mut self.emulator);

            egui::SidePanel::right("right_debug_panel")
                .resizable(false)
                .show(ctx, |ui| {
                    ui.style_mut().override_font_id = Some(FontId::monospace(14.0));
                    self.debug_state.right_debug_panel(ui, &self.emulator);
                });

            egui::TopBottomPanel::bottom("bottom_debug_panel")
                .resizable(false)
                .height_range(250.0..=250.0)
                .show(ctx, |ui| {
                    ui.style_mut().override_font_id = Some(FontId::monospace(14.0));
                    self.debug_state.bottom_debug_panel(ui, &self.emulator);
                });
        }

        // Render emulator display
        egui::CentralPanel::default().show(ctx, |ui| {
            self.main_display(ui);
        });

        // Always repaint to keep rendering at 60Hz.
        ctx.request_repaint()
    }
}
