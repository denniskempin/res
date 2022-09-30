use std::collections::VecDeque;

use eframe::CreationContext;
use egui::vec2;
use egui::ColorImage;
use egui::RichText;
use egui::Rounding;
use egui::Sense;
use egui::TextureHandle;
use egui::Ui;

use crate::nes::ppu::SYSTEM_PALETTE;
use crate::nes::System;

pub struct Debugger {
    nametable_texture: TextureHandle,

    run_mode: RunMode,
    selected_run_mode: RunMode,
    previous_states: VecDeque<Vec<u8>>,
}

impl Debugger {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext<'_>) -> Self {
        Debugger {
            nametable_texture: cc.egui_ctx.load_texture("Nametable", ColorImage::example()),
            run_mode: RunMode::Indefinitely,
            selected_run_mode: RunMode::NextFrame,
            previous_states: VecDeque::new(),
        }
    }

    pub fn run_emulator(&mut self, emulator: &mut System) {
        match self.run_mode {
            RunMode::Paused => (),
            RunMode::Indefinitely => {
                emulator.execute_one_frame().unwrap();
            }
            RunMode::NextScanline => {
                emulator.execute_one_frame().unwrap();
                self.run_mode = RunMode::Paused;
            }
            RunMode::NextVblank => {
                emulator.execute_one_frame().unwrap();
                self.run_mode = RunMode::Paused;
            }
            RunMode::NextFrame => {
                emulator.execute_one_frame().unwrap();
                self.run_mode = RunMode::Paused;
            }
        }
    }

    pub fn right_debug_panel(&mut self, ui: &mut Ui, emulator: &System) {
        ui.separator();
        self.debug_controls(ui);
        ui.separator();
        self.palette_table(ui, emulator);
        ui.separator();
        ui.label(RichText::new("Nametable").strong());

        self.nametable_texture
            .set(emulator.ppu().debug_render_nametable());
        ui.image(&self.nametable_texture, vec2(256.0, 240.0));
    }

    pub fn bottom_debug_panel(&mut self, ui: &mut Ui, _emulator: &System) {
        ui.label(RichText::new("Bottom Debug Panel").strong());
    }

    fn palette_table(&self, ui: &mut Ui, emulator: &System) {
        ui.label(RichText::new("Color Palette").strong());
        for palette_id in 0..8 {
            ui.columns(4, |cols| {
                for (color_id, col) in cols.iter_mut().enumerate() {
                    let desired_size = vec2(col.available_size().x, 16.0);
                    let (whole_rect, response) =
                        col.allocate_exact_size(desired_size, Sense::focusable_noninteractive());
                    response.on_hover_text(format!("Color {color_id} of palette {palette_id}"));

                    let color = emulator.ppu().get_palette_entry(palette_id, color_id);
                    col.painter().rect_filled(
                        whole_rect,
                        Rounding::none(),
                        SYSTEM_PALETTE[color as usize],
                    );
                }
            });
        }
    }

    fn debug_controls(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            if self.run_mode != RunMode::Paused {
                if ui.button("Pause").clicked() {
                    self.run_mode = RunMode::Paused;
                }
                return;
            }

            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.selected_run_mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.selected_run_mode,
                        RunMode::Indefinitely,
                        "Indefinitely",
                    );
                    ui.selectable_value(
                        &mut self.selected_run_mode,
                        RunMode::NextFrame,
                        "NextFrame",
                    );
                    ui.selectable_value(
                        &mut self.selected_run_mode,
                        RunMode::NextVblank,
                        "NextVblank",
                    );
                    ui.selectable_value(
                        &mut self.selected_run_mode,
                        RunMode::NextScanline,
                        "NextScanline",
                    );
                });

            if ui.button("Run").clicked() {
                self.run_mode = self.selected_run_mode;
            }

            if !self.previous_states.is_empty() && ui.button("Step Back").clicked() {}
        });
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum RunMode {
    Paused,
    Indefinitely,
    NextScanline,
    NextVblank,
    NextFrame,
}
