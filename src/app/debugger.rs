use std::collections::VecDeque;
use std::fmt::Debug;

use eframe::CreationContext;
use egui::vec2;
use egui::Button;
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

    command: Option<DebugCommand>,
    previous_states: StateStack,
}

impl Debugger {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext<'_>) -> Self {
        Debugger {
            nametable_texture: cc.egui_ctx.load_texture("Nametable", ColorImage::example()),
            command: None,
            previous_states: StateStack::default(),
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
        ui.separator();
        self.debug_controls(ui, emulator);
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

#[derive(Default)]
pub struct StateStack {
    stack: VecDeque<System>,
}

impl StateStack {
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
    pub fn pop(&mut self) -> System {
        self.stack.pop_front().unwrap()
    }

    pub fn push(&mut self, system: System) {
        self.stack.push_front(system);
        self.stack.truncate(256);
    }
}
