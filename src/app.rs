use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::sync::Mutex;

use std::thread::sleep;
use std::time::Duration;

use egui::vec2;
use egui::Color32;
use egui::Key;
use egui::Rect;
use egui::Sense;
use egui::Stroke;

use crate::chip8;
use crate::chip8::Chip8;

#[derive(Default)]
pub struct Emulator {
    chip8: Chip8,
    ops_per_second: usize,
    running: bool,
    context: egui::Context,
}

impl Emulator {
    pub fn new(program: &[u8], context: egui::Context) -> Self {
        Emulator {
            context,
            ops_per_second: 500,
            running: true,
            chip8: Chip8::with_program(program),
        }
    }

    pub fn reset_and_load_program(&mut self, program: &[u8]) {
        self.chip8 = Chip8::with_program(program);
        self.running = true;
    }

    pub fn main_loop(emulator: Arc<Mutex<Emulator>>) {
        let mut ops_per_second = emulator.lock().unwrap().ops_per_second;

        loop {
            if let Ok(ref mut emulator) = emulator.lock() {
                if emulator.running {
                    match emulator.chip8.emulate_tick() {
                        Ok(state) => match state {
                            chip8::State::Halt | chip8::State::InfiniteLoop => {
                                emulator.running = false
                            }
                            chip8::State::DisplayUpdated => {
                                emulator.context.request_repaint();
                            }
                            chip8::State::Ordinary => (),
                        },
                        Err(err) => {
                            println!("Fatal Error: {err}");
                            emulator.running = false;
                        }
                    }
                    ops_per_second = emulator.ops_per_second;
                }
            }
            if ops_per_second > 0 {
                sleep(Duration::from_secs_f32(1.0 / ops_per_second as f32));
            }
        }
    }
}

pub struct EmulatorApp {
    emulator: Arc<Mutex<Emulator>>,
    pixels: [[u8; 64]; 32],
}

impl EmulatorApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, emulator: Arc<Mutex<Emulator>>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        EmulatorApp {
            emulator,
            pixels: [[0u8; 64]; 32],
        }
    }

    pub fn render_display(&mut self, ui: &mut egui::Ui) {
        let desired_size = ui.available_size();
        let (whole_rect, _) =
            ui.allocate_exact_size(desired_size, Sense::focusable_noninteractive());

        let stroke = Stroke::new(0.5, Color32::from_gray(80));
        let pixel_width = whole_rect.width() / 64.0;
        let pixel_height = whole_rect.height() / 32.0;
        if ui.is_rect_visible(whole_rect) {
            {
                let new_pixels = &self.emulator.lock().unwrap().chip8.display.pixels;
                for (y, row) in new_pixels.iter().enumerate() {
                    for (x, pixel) in row.iter().enumerate() {
                        if *pixel {
                            self.pixels[y][x] = self.pixels[y][x].saturating_add(128);
                        } else {
                            self.pixels[y][x] = self.pixels[y][x].saturating_sub(16);
                        }
                    }
                }
            }
            for (y, row) in self.pixels.iter().enumerate() {
                for (x, pixel) in row.iter().enumerate() {
                    let min =
                        whole_rect.min + vec2(x as f32 * pixel_width, y as f32 * pixel_height);
                    let pixel_rect = Rect::from_min_size(min, vec2(pixel_width, pixel_height));
                    let color = Color32::from_gray(*pixel);
                    ui.painter().rect(pixel_rect, 0.0, color, stroke)
                }
            }
        }
    }
}

impl eframe::App for EmulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !ctx.input().raw.dropped_files.is_empty() {
            let drop = &ctx.input().raw.dropped_files[0];
            if let Some(path) = &drop.path {
                let mut data: Vec<u8> = Vec::new();
                File::open(path).unwrap().read_to_end(&mut data).unwrap();
                self.emulator.lock().unwrap().reset_and_load_program(&data);
            } else if let Some(bytes) = &drop.bytes {
                self.emulator
                    .lock()
                    .unwrap()
                    .reset_and_load_program(&*bytes);
            }
        }

        {
            let keys = &mut self.emulator.lock().unwrap().chip8.keys;
            keys[0x0] = ctx.input().key_down(Key::Num0);
            keys[0x1] = ctx.input().key_down(Key::Num1);
            keys[0x2] = ctx.input().key_down(Key::Num2);
            keys[0x3] = ctx.input().key_down(Key::Num3);

            keys[0x4] = ctx.input().key_down(Key::Num4);
            keys[0x5] = ctx.input().key_down(Key::Num5);
            keys[0x6] = ctx.input().key_down(Key::Num6);
            keys[0x7] = ctx.input().key_down(Key::Num7);

            keys[0x8] = ctx.input().key_down(Key::Num8);
            keys[0x9] = ctx.input().key_down(Key::Num9);
            keys[0xA] = ctx.input().key_down(Key::A);
            keys[0xB] = ctx.input().key_down(Key::B);

            keys[0xC] = ctx.input().key_down(Key::C);
            keys[0xD] = ctx.input().key_down(Key::D);
            keys[0xE] = ctx.input().key_down(Key::E);
            keys[0xF] = ctx.input().key_down(Key::F);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_display(ui);
        });
        ctx.request_repaint()
    }
}
