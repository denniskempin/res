mod debugger;

use std::fs::File;
use std::io::Read;

use eframe::CreationContext;
use eframe::Frame;
use egui::ColorImage;
use egui::Context;
use egui::DroppedFile;
use egui::Image;
use egui::InputState;
use egui::Key;
use egui::Layout;
use egui::Sense;
use egui::TextureHandle;
use egui::Ui;

use self::debugger::Debugger;
use crate::nes::joypad::JoypadButton;
use crate::nes::System;

pub struct EmulatorApp {
    emulator: System,
    loaded: bool,
    framebuffer_texture: TextureHandle,
    debug_mode: bool,
    debug_state: Debugger,
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
                System::default()
            },
            loaded,
            framebuffer_texture: cc
                .egui_ctx
                .load_texture("Framebuffer", ColorImage::example()),
            debug_mode: true,
            debug_state: Debugger::new(cc),
        }
    }

    fn load_dropped_file(&mut self, drop: &DroppedFile) {
        if let Some(path) = &drop.path {
            let mut data: Vec<u8> = Vec::new();
            File::open(path).unwrap().read_to_end(&mut data).unwrap();
            self.emulator = System::with_ines_bytes(&data).unwrap();
        } else if let Some(bytes) = &drop.bytes {
            #[cfg(target_arch = "wasm32")]
            crate::wasm::save_rom_in_local_storage(bytes);
            self.emulator = System::with_ines_bytes(&*bytes).unwrap();
        }
        self.loaded = true;
    }

    fn update_keys(&mut self, input: &InputState) {
        let joypad0 = self.emulator.joypad0_mut();
        joypad0.set_button(JoypadButton::Right, input.key_down(Key::ArrowRight));
        joypad0.set_button(JoypadButton::Left, input.key_down(Key::ArrowLeft));
        joypad0.set_button(JoypadButton::Down, input.key_down(Key::ArrowDown));
        joypad0.set_button(JoypadButton::Up, input.key_down(Key::ArrowUp));
        joypad0.set_button(JoypadButton::Start, input.key_down(Key::S));
        joypad0.set_button(JoypadButton::Select, input.key_down(Key::A));
        joypad0.set_button(JoypadButton::ButtonB, input.key_down(Key::Z));
        joypad0.set_button(JoypadButton::ButtonA, input.key_down(Key::X));
    }

    fn menu_bar(&mut self, ui: &mut Ui) {
        ui.columns(2, |columns| {
            columns[0].with_layout(Layout::left_to_right(), |ui| {
                ui.menu_button("Programs", |_ui| {});
                ui.menu_button("Games", |_ui| {});
                ui.label("(Or drop a .nes file to load it)");
            });
            columns[1].with_layout(Layout::right_to_left(), |ui| {
                if ui.button("Debug").clicked() {
                    self.debug_mode = !self.debug_mode;
                }
            });
        });
    }

    fn main_display(&mut self, ui: &mut Ui) {
        self.framebuffer_texture
            .set(self.emulator.ppu().framebuffer.as_color_image());

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
                    self.debug_state.right_debug_panel(ui, &self.emulator);
                });

            egui::TopBottomPanel::bottom("bottom_debug_panel")
                .resizable(false)
                .height_range(240.0..=240.0)
                .show(ctx, |ui| {
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
