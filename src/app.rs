use std::fs::File;
use std::io::Read;

use eframe::CreationContext;
use eframe::Frame;
use egui::vec2;
use egui::ColorImage;
use egui::Context;
use egui::DroppedFile;
use egui::Image;
use egui::InputState;
use egui::Key;
use egui::RichText;
use egui::Rounding;
use egui::Sense;
use egui::TextureHandle;
use egui::Ui;
use image::RgbaImage;

use crate::nes::joypad::JoypadButton;
use crate::nes::ppu::SYSTEM_PALETTE;
use crate::nes::System;

pub struct EmulatorApp {
    emulator: System,
    loaded: bool,
    framebuffer_texture: TextureHandle,
    nametable_texture: TextureHandle,
}

pub fn set_texture_from_image(handle: &mut TextureHandle, image: &RgbaImage) {
    let egui_image = ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        image.as_flat_samples().as_slice(),
    );
    handle.set(egui_image);
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
            nametable_texture: cc.egui_ctx.load_texture("Nametable", ColorImage::example()),
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

    fn update_framebuffer(&mut self) {
        self.framebuffer_texture
            .set(self.emulator.ppu().framebuffer.as_color_image());
    }

    fn update_debug_textures(&mut self) {
        self.nametable_texture
            .set(self.emulator.ppu().debug_render_nametable())
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

    fn palette_table(&self, ui: &mut Ui) {
        ui.label(RichText::new("Color Palette").strong());
        for palette_id in 0..8 {
            ui.columns(4, |cols| {
                for (color_id, col) in cols.iter_mut().enumerate() {
                    let desired_size = vec2(col.available_size().x, 16.0);
                    let (whole_rect, response) =
                        col.allocate_exact_size(desired_size, Sense::focusable_noninteractive());
                    response.on_hover_text(format!("Color {color_id} of palette {palette_id}"));

                    let color = self.emulator.ppu().get_palette_entry(palette_id, color_id);
                    col.painter().rect_filled(
                        whole_rect,
                        Rounding::none(),
                        SYSTEM_PALETTE[color as usize],
                    );
                }
            });
        }
    }
}

impl eframe::App for EmulatorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // Load new program if a file is dropped on the app
        if !ctx.input().raw.dropped_files.is_empty() {
            self.load_dropped_file(&ctx.input().raw.dropped_files[0]);
        }
        self.update_keys(&ctx.input());
        if self.loaded {
            self.emulator.execute_one_frame().unwrap();
            self.update_framebuffer();
            self.update_debug_textures();
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Programs", |_ui| {});
                ui.menu_button("Games", |_ui| {});
                ui.label("(Or drop a .nes file to load it)")
            });
        });

        // Render debug display
        egui::SidePanel::right("debug_panel")
            .resizable(false)
            .show(ctx, |ui| {
                self.palette_table(ui);
                ui.separator();
                ui.label(RichText::new("Nametable").strong());
                ui.image(&self.nametable_texture, vec2(256.0, 240.0));
            });

        // Render emulator display
        egui::CentralPanel::default().show(ctx, |ui| {
            let desired_size = ui.available_size();
            let (whole_rect, _) =
                ui.allocate_exact_size(desired_size, Sense::focusable_noninteractive());

            let image = Image::new(
                &self.framebuffer_texture,
                self.framebuffer_texture.size_vec2(),
            );
            image.paint_at(ui, whole_rect);
        });

        // Always repaint to keep rendering at 60Hz.
        ctx.request_repaint()
    }
}
