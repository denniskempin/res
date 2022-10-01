use std::fmt::Debug;

use eframe::CreationContext;
use egui::vec2;
use egui::Button;
use egui::ColorImage;
use egui::FontFamily;
use egui::RichText;
use egui::Rounding;
use egui::Sense;
use egui::TextureHandle;
use egui::Ui;
use itertools::Itertools;

use crate::nes::ppu::SYSTEM_PALETTE;
use crate::nes::System;
use crate::util::RingBuffer;

pub struct Debugger {
    nametable_texture: TextureHandle,

    command: Option<DebugCommand>,
    previous_states: RingBuffer<System, 256>,
}

impl Debugger {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        Debugger {
            nametable_texture: cc.egui_ctx.load_texture("Nametable", ColorImage::example()),
            command: None,
            previous_states: RingBuffer::default(),
        }
    }

    pub fn run_emulator(&mut self, emulator: &mut System) {
        if let Some(command) = self.command {
            match command {
                DebugCommand::Run => {
                    emulator.execute_one_frame().unwrap();
                }
                DebugCommand::StepFrames(n) => {
                    emulator.execute_one_frame().unwrap();
                    if n > 1 {
                        self.command = Some(DebugCommand::StepFrames(n - 1));
                    } else {
                        self.command = None
                    }
                }
                DebugCommand::StepBack => {
                    *emulator = self.previous_states.pop();
                    self.command = None
                }
                _ => (),
            }
        }
    }

    pub fn right_debug_panel(&mut self, ui: &mut Ui, emulator: &System) {
        self.debug_controls(ui, emulator);
        ui.separator();
        ui.label(RichText::new("CPU").strong());
        let cpu = emulator.cpu();

        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("A {:02X}", cpu.a)).family(FontFamily::Monospace));
            ui.separator();
            ui.label(RichText::new(format!("X {:02X}", cpu.x)).family(FontFamily::Monospace));
            ui.separator();
            ui.label(RichText::new(format!("Y {:02X}", cpu.y)).family(FontFamily::Monospace));
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Status:").family(FontFamily::Monospace));
            ui.label(RichText::new(cpu.status_flags.pretty_print()).family(FontFamily::Monospace));
        });
        ui.label(RichText::new(format!("Cycle: {}", cpu.cycle)).family(FontFamily::Monospace));
        ui.label(
            RichText::new(format!("PC: 0x{:04X}", cpu.program_counter))
                .family(FontFamily::Monospace),
        );

        ui.label(RichText::new("Stack").strong());
        for line in &cpu.peek_stack().chunks(8) {
            let line_str = line.map(|s| format!("{:02X}", s)).join(" ");
            ui.label(RichText::new(line_str).family(FontFamily::Monospace));
        }
    }

    pub fn bottom_debug_panel(&mut self, ui: &mut Ui, emulator: &System) {
        ui.horizontal(|ui| {
            self.palette_table(ui, emulator);
            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Nametable").strong());

                self.nametable_texture
                    .set(emulator.ppu().debug_render_nametable());
                ui.image(&self.nametable_texture, vec2(420.0, 210.0));
            });
        });
    }

    fn palette_table(&self, ui: &mut Ui, emulator: &System) {
        ui.vertical(|ui| {
            ui.set_max_width(160.0);
            ui.label(RichText::new("Color Palette").strong());
            for palette_id in 0..8 {
                ui.columns(4, |cols| {
                    for (color_id, col) in cols.iter_mut().enumerate() {
                        let (rect, response) = col.allocate_exact_size(
                            vec2(32.0, 24.0),
                            Sense::focusable_noninteractive(),
                        );
                        response.on_hover_text(format!("Color {color_id} of palette {palette_id}"));

                        let color = emulator.ppu().get_palette_entry(palette_id, color_id);
                        col.painter().rect_filled(
                            rect,
                            Rounding::none(),
                            SYSTEM_PALETTE[color as usize],
                        );
                    }
                });
            }
        });
    }

    fn debug_controls(&mut self, ui: &mut Ui, emulator: &System) {
        ui.horizontal_wrapped(|ui| {
            let paused = self.command.is_none();

            if ui.button(if paused { "Run" } else { "Pause" }).clicked() {
                if paused {
                    self.previous_states.push(emulator.clone());
                    self.command = Some(DebugCommand::Run);
                } else {
                    self.command = None;
                }
            }

            if ui.add_enabled(paused, Button::new("Step Frame")).clicked() {
                self.previous_states.push(emulator.clone());
                self.command = Some(DebugCommand::StepFrames(1));
            }

            if ui
                .add_enabled(
                    paused && !self.previous_states.is_empty(),
                    Button::new("Step Back"),
                )
                .clicked()
            {
                self.command = Some(DebugCommand::StepBack);
            }
        });
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum DebugCommand {
    Run,
    StepFrames(u32),
    StepInstructions(u32),
    StepScanlines(u32),
    StepBack,
    RunToNextVblankStart,
    RunToNextVblankEnd,
}
