use std::fmt::Debug;

use eframe::emath::Align;
use eframe::CreationContext;
use egui::text::LayoutJob;
use egui::vec2;
use egui::Button;
use egui::Color32;
use egui::ColorImage;
use egui::FontFamily;
use egui::FontId;
use egui::Label;
use egui::Rect;
use egui::RichText;
use egui::Rounding;
use egui::ScrollArea;
use egui::Sense;
use egui::TextFormat;
use egui::TextStyle;
use egui::TextureHandle;
use egui::Ui;
use itertools::Itertools;

use crate::nes::cpu::Operation;
use crate::nes::ppu::SYSTEM_PALETTE;
use crate::nes::System;
use crate::util::RingBuffer;

pub struct Debugger {
    nametable_texture: TextureHandle,

    command: Option<DebugCommand>,
    previous_states: RingBuffer<System, 256>,

    inspector_is_open: bool,
    scroll_to_memory_location: Option<u16>,
}

impl Debugger {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        Debugger {
            nametable_texture: cc.egui_ctx.load_texture("Nametable", ColorImage::example()),
            command: None,
            previous_states: RingBuffer::default(),
            scroll_to_memory_location: None,
            inspector_is_open: false,
        }
    }

    pub fn run_emulator(&mut self, emulator: &mut System) {
        if let Some(command) = self.command {
            match command {
                DebugCommand::Run => {
                    if emulator.ppu().frame % 60 == 0 {
                        self.previous_states.push(emulator.clone());
                    }
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
                DebugCommand::StepInstructions(n) => {
                    emulator.cpu.execute_one().unwrap();
                    if n > 1 {
                        self.command = Some(DebugCommand::StepInstructions(n - 1));
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
        self.cpu_panel(ui, emulator);
        ui.separator();
        self.operations_panel(ui, emulator);

        egui::Window::new("CPU Bus")
            .open(&mut self.inspector_is_open)
            .show(ui.ctx(), |ui| {
                ui.style_mut().override_font_id = Some(FontId::monospace(14.0));
                ui.add(
                    Label::new(RichText::new(
                        "      00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F",
                    ))
                    .wrap(false),
                );

                let text_style = TextStyle::Body;
                let row_height = ui.text_style_height(&text_style);
                let bytes_per_line: usize = 16;
                let num_rows = 0xFFFF / bytes_per_line;

                let mut scroll = ScrollArea::vertical();
                if let Some(location) = self.scroll_to_memory_location {
                    let coarse_location = (location / bytes_per_line as u16) as f32;
                    self.scroll_to_memory_location = None;
                    scroll = scroll.vertical_scroll_offset(
                        coarse_location * (row_height + ui.spacing().item_spacing.y),
                    );
                }
                scroll.show_rows(ui, row_height, num_rows, |ui, row_range| {
                    for row in row_range {
                        let addr = (row * bytes_per_line) as u16;
                        let bytes = emulator.cpu().bus.peek_slice(addr, bytes_per_line as u16);
                        let bytes_str = bytes.map(|s| format!("{:02X}", s)).join(" ");
                        ui.add(
                            Label::new(RichText::new(format!("{:04X}: {}", addr, bytes_str)))
                                .wrap(false),
                        );
                    }
                });
            });
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

    fn cpu_panel(&self, ui: &mut Ui, emulator: &System) {
        ui.label(RichText::new("CPU").strong());
        let cpu = emulator.cpu();

        ui.horizontal(|ui| {
            ui.label(format!("A {:02X}", cpu.a));
            ui.separator();
            ui.label(format!("X {:02X}", cpu.x));
            ui.separator();
            ui.label(format!("Y {:02X}", cpu.y));
        });
        ui.horizontal(|ui| {
            ui.label("Status:");
            ui.label(cpu.status_flags.pretty_print());
        });
        ui.label(format!("Cycle: {}", cpu.cycle));
        ui.label(format!("PC: 0x{:04X}", cpu.program_counter));
        ui.label(RichText::new("Stack").strong());
        for line in &cpu.peek_stack().chunks(8) {
            let line_str = line.map(|s| format!("{:02X}", s)).join(" ");
            ui.label(line_str);
        }
    }

    fn operations_panel(&mut self, ui: &mut Ui, emulator: &System) {
        ui.label(RichText::new("Operations").strong());

        let last_ops = emulator.cpu().debug.last_ops.iter().take(20).rev();
        for addr in last_ops {
            self.operation_label(ui, *addr, emulator, false);
        }
        self.operation_label(ui, emulator.cpu().program_counter, emulator, true);
        for addr in emulator.cpu().peek_next_operations(10).skip(1) {
            self.operation_label(ui, addr, emulator, false);
        }
    }

    fn operation_label(&mut self, ui: &mut Ui, addr: u16, emulator: &System, current: bool) {
        let op = Operation::peek(emulator.cpu(), addr).unwrap();

        ui.horizontal(|ui| {
            let addr_str = if current {
                format!("> {:04X}", addr)
            } else {
                format!("  {:04X}", addr)
            };
            ui.label(RichText::new(addr_str));
            for part in op.format(emulator.cpu()).split(' ') {
                let mut text = RichText::new(part).strong();
                if part.starts_with('$') {
                    text = text.color(Color32::LIGHT_BLUE);
                    text = text.underline();
                    let widget = ui.add(Label::new(text).sense(Sense::click()));
                    if widget.clicked() {
                        let addr =
                            u16::from_str_radix(part.strip_prefix('$').unwrap(), 16).unwrap();
                        self.scroll_to_memory_location = Some(addr);
                        self.inspector_is_open = true;
                    }
                } else if part.starts_with('#') {
                    text = text.color(Color32::LIGHT_GREEN);
                    ui.label(text);
                } else if part.starts_with('+') {
                    text = text.color(Color32::LIGHT_RED);
                    ui.label(text);
                } else {
                    ui.label(text);
                }
            }
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
                        let rgb = if color < 64 {
                            SYSTEM_PALETTE[color as usize]
                        } else {
                            Color32::RED
                        };
                        col.painter().rect_filled(rect, Rounding::none(), rgb);
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
            if ui.add_enabled(paused, Button::new("Step")).clicked() {
                self.previous_states.push(emulator.clone());
                self.command = Some(DebugCommand::StepInstructions(1));
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
