mod audio;
mod debugger_ui;

use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

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
use gilrs::Axis;
use gilrs::Button;
use gilrs::Gilrs;
use tracing::instrument;

use self::audio::AudioEngine;
use self::debugger_ui::DebuggerUi;
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

pub struct Rom {
    ines_data: Vec<u8>,
    persistent_data: Option<Vec<u8>>,
    persist_file_path: PathBuf,
}

impl Rom {
    pub fn load_from_file(path: &Path) -> Rom {
        let ines_data = fs::read(path).unwrap();
        let persist_file_path = path.with_extension("nes.persist");
        let persist_file = fs::read(&persist_file_path);
        let persistent_data = match persist_file {
            Ok(data) => Some(data),
            Err(_) => None,
        };
        Rom {
            ines_data,
            persistent_data,
            persist_file_path,
        }
    }

    pub fn load_from_bytes(name: &str, ines_data: &[u8]) -> Rom {
        let persist_file_path = PathBuf::from(name).with_extension("nes.persist");
        let persistent_data = if cfg!(target_arch = "wasm32") {
            None
        } else {
            match fs::read(&persist_file_path) {
                Ok(data) => Some(data),
                Err(_) => None,
            }
        };
        Rom {
            ines_data: ines_data.to_owned(),
            persistent_data,
            persist_file_path,
        }
    }

    pub fn save_persistent_data(&self, persistent_data: Vec<u8>) {
        if cfg!(target_arch = "wasm32") {
            // TODO: Implement persistent storage for wasm32
        } else {
            fs::write(&self.persist_file_path, &persistent_data).unwrap();
        }
    }
}

pub struct EmulatorApp {
    emulator: System,
    loaded_rom: Option<Rom>,
    framebuffer_texture: TextureHandle,
    debug_mode: bool,
    debugger_ui: DebuggerUi,
    audio_engine: AudioEngine,
    gilrs: Gilrs,
}

impl EmulatorApp {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext<'_>, rom: Option<Rom>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let mut app = EmulatorApp {
            emulator: System::new(),
            loaded_rom: None,
            framebuffer_texture: cc.egui_ctx.load_texture(
                "Framebuffer",
                ColorImage::example(),
                Default::default(),
            ),
            debug_mode: true,
            debugger_ui: DebuggerUi::new(cc),
            audio_engine: AudioEngine::new(),
            gilrs: Gilrs::new().unwrap(),
        };

        if let Some(rom) = rom {
            app.load_rom(rom);
        }
        app
    }

    fn load_rom(&mut self, rom: Rom) {
        self.emulator =
            System::with_ines_bytes(&rom.ines_data, rom.persistent_data.as_deref()).unwrap();
        self.loaded_rom = Some(rom);
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
                    self.load_rom(Rom::load_from_file(path));
                }
                _ => {
                    panic!("Unknown file type");
                }
            }
        } else if let Some(bytes) = &drop.bytes {
            #[cfg(target_arch = "wasm32")]
            crate::wasm::save_rom_in_local_storage(bytes);
            self.load_rom(Rom::load_from_bytes(&drop.name, bytes));
        }
    }

    fn update_keys(&mut self, input: &InputState) {
        let mut joypad0 = [false; 8];
        while self.gilrs.next_event().is_some() {}
        if let Some((_, gamepad)) = self.gilrs.gamepads().next() {
            joypad0[JoypadButton::Right as usize] = gamepad.is_pressed(Button::DPadRight)
                || gamepad.value(Axis::DPadX) > 0.5
                || gamepad.value(Axis::LeftStickX) > 0.5;
            joypad0[JoypadButton::Left as usize] = gamepad.is_pressed(Button::DPadLeft)
                || gamepad.value(Axis::DPadX) < -0.5
                || gamepad.value(Axis::LeftStickX) < -0.5;
            joypad0[JoypadButton::Down as usize] = gamepad.is_pressed(Button::DPadDown)
                || gamepad.value(Axis::DPadY) < -0.5
                || gamepad.value(Axis::LeftStickY) < -0.5;
            joypad0[JoypadButton::Up as usize] = gamepad.is_pressed(Button::DPadUp)
                || gamepad.value(Axis::DPadY) > 0.5
                || gamepad.value(Axis::LeftStickY) > 0.5;
            joypad0[JoypadButton::Start as usize] = gamepad.is_pressed(Button::Start);
            joypad0[JoypadButton::Select as usize] = gamepad.is_pressed(Button::Select);
            joypad0[JoypadButton::ButtonB as usize] = gamepad.is_pressed(Button::South);
            joypad0[JoypadButton::ButtonA as usize] = gamepad.is_pressed(Button::East);
        } else {
            joypad0[JoypadButton::Right as usize] = input.key_down(Key::ArrowRight);
            joypad0[JoypadButton::Left as usize] = input.key_down(Key::ArrowLeft);
            joypad0[JoypadButton::Down as usize] = input.key_down(Key::ArrowDown);
            joypad0[JoypadButton::Up as usize] = input.key_down(Key::ArrowUp);
            joypad0[JoypadButton::Start as usize] = input.key_down(Key::S);
            joypad0[JoypadButton::Select as usize] = input.key_down(Key::A);
            joypad0[JoypadButton::ButtonB as usize] = input.key_down(Key::Z);
            joypad0[JoypadButton::ButtonA as usize] = input.key_down(Key::X);
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
                            self.load_rom(Rom::load_from_bytes(program.0, program.1));
                        }
                    }
                });
                ui.menu_button("Games", |ui| {
                    for program in GAMES {
                        if ui.button(program.0).clicked() {
                            self.load_rom(Rom::load_from_bytes(program.0, program.1));
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

    fn save_persistent_data(&self) {
        if let Some(rom) = &self.loaded_rom {
            let cartridge = self.emulator.cartridge().borrow();
            if !cartridge.has_persistent_data {
                return;
            }

            if self.emulator.ppu().frame % 600 == 0 {
                rom.save_persistent_data(self.emulator.cartridge().borrow().persistent_data())
            }
        }
    }
}

impl eframe::App for EmulatorApp {
    #[instrument(skip_all)]
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // Load new program if a file is dropped on the app
        if !ctx.input().raw.dropped_files.is_empty() {
            self.load_dropped_file(&ctx.input().raw.dropped_files[0]);
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.menu_bar(ui);
        });

        if self.loaded_rom.is_none() {
            return;
        }

        self.save_persistent_data();
        self.update_keys(&ctx.input());

        if !self.debug_mode {
            self.emulator
                .execute_for_duration(ctx.input().unstable_dt as f64)
                .unwrap();
        } else {
            self.debugger_ui
                .run_emulator(&mut self.emulator, ctx.input().unstable_dt as f64);

            egui::SidePanel::right("right_debug_panel")
                .resizable(false)
                .show(ctx, |ui| {
                    ui.style_mut().override_font_id = Some(FontId::monospace(12.0));
                    self.debugger_ui.right_debug_panel(ui, &self.emulator);
                });

            egui::TopBottomPanel::bottom("bottom_debug_panel")
                .resizable(false)
                .height_range(250.0..=250.0)
                .show(ctx, |ui| {
                    ui.style_mut().override_font_id = Some(FontId::monospace(12.0));
                    self.debugger_ui.bottom_debug_panel(ui, &self.emulator);
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
