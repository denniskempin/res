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
use egui::Sense;
use egui::TextureHandle;
use image::GenericImage;
use image::ImageBuffer;

use image::RgbaImage;

use crate::nes::joypad::JoypadButton;
use crate::nes::System;

pub struct EmulatorApp {
    emulator: System,
    texture: TextureHandle,
    loaded: bool,
}

impl EmulatorApp {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        EmulatorApp {
            emulator: System::default(),
            loaded: false,
            texture: cc
                .egui_ctx
                .load_texture("Framebuffer", ColorImage::example()),
        }
    }

    fn load_dropped_file(&mut self, drop: &DroppedFile) {
        if let Some(path) = &drop.path {
            let mut data: Vec<u8> = Vec::new();
            File::open(path).unwrap().read_to_end(&mut data).unwrap();
            self.emulator = System::with_ines_bytes(&data).unwrap();
        } else if let Some(bytes) = &drop.bytes {
            self.emulator = System::with_ines_bytes(&*bytes).unwrap();
        }
        self.loaded = true;
    }

    pub fn render_display(&mut self, ui: &mut egui::Ui) {
        let desired_size = ui.available_size();
        let (whole_rect, _) =
            ui.allocate_exact_size(desired_size, Sense::focusable_noninteractive());

        let image = Image::new(&self.texture, self.texture.size_vec2());
        image.paint_at(ui, whole_rect);
    }

    fn update_display(&mut self) {
        let mut image: RgbaImage = ImageBuffer::new(32 * 8, 30 * 8);
        self.emulator
            .cpu
            .bus
            .ppu
            .render_nametable(&mut image.sub_image(0, 0, 32 * 8, 30 * 8));
        let size = [image.width() as usize, image.height() as usize];
        let egui_image =
            ColorImage::from_rgba_unmultiplied(size, image.as_flat_samples().as_slice());
        self.texture.set(egui_image);
    }

    fn update_keys(&mut self, input: &InputState) {
        let joypad0 = &mut self.emulator.cpu.bus.joypad0;
        joypad0.set_button(JoypadButton::Right, input.key_down(Key::ArrowRight));
        joypad0.set_button(JoypadButton::Left, input.key_down(Key::ArrowLeft));
        joypad0.set_button(JoypadButton::Down, input.key_down(Key::ArrowDown));
        joypad0.set_button(JoypadButton::Up, input.key_down(Key::ArrowUp));
        joypad0.set_button(JoypadButton::Start, input.key_down(Key::S));
        joypad0.set_button(JoypadButton::Select, input.key_down(Key::A));
        joypad0.set_button(JoypadButton::ButtonB, input.key_down(Key::Z));
        joypad0.set_button(JoypadButton::ButtonA, input.key_down(Key::X));
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
            self.update_display();
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
            .show(ctx, |_ui| {});

        // Render emulator display
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_display(ui);
        });

        // Always repaint to keep rendering at 60Hz.
        ctx.request_repaint()
    }
}
