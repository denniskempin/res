use std::fs::File;
use std::io::Read;

use egui::vec2;
use egui::Color32;
use egui::DroppedFile;
use egui::FontFamily;
use egui::InputState;
use egui::Key;
use egui::Modifiers;
use egui::Rect;
use egui::RichText;
use egui::Sense;
use egui::Stroke;

use crate::chip8::Chip8;

pub struct Program {
    name: &'static str,
    data: &'static [u8],
}


macro_rules! embed_roms {
    ( $( $x:expr ),* ) => {
        &[
            $(
            Program {
                name: $x,
                data: include_bytes!(concat!("../chip8-roms/", $x)).as_slice(),
            },
            )*
        ]
    };
}

const PROGRAMS: &[Program] = embed_roms!(
    "programs/Jumping X and O [Harry Kleinberg, 1977].ch8",
    "programs/Keypad Test [Hap, 2006].ch8",
    "programs/Framed MK1 [GV Samways, 1980].ch8",
    "programs/Delay Timer Test [Matthew Mikolay, 2010].ch8",
    "programs/Minimal game [Revival Studios, 2007].ch8",
    "programs/IBM Logo.ch8",
    "programs/BMP Viewer - Hello (C8 example) [Hap, 2005].ch8",
    "programs/Framed MK2 [GV Samways, 1980].ch8",
    "programs/Chip8 emulator Logo [Garstyciuks].ch8",
    "programs/Random Number Test [Matthew Mikolay, 2010].ch8",
    "programs/Chip8 Picture.ch8",
    "programs/Division Test [Sergey Naydenov, 2010].ch8",
    "programs/Clock Program [Bill Fisher, 1981].ch8",
    "programs/Fishie [Hap, 2005].ch8",
    "programs/Life [GV Samways, 1980].ch8",
    "programs/SQRT Test [Sergey Naydenov, 2010].ch8"
);

const GAMES: &[Program] = embed_roms!(
    "games/Timebomb.ch8",
    "games/Paddles.ch8",
    "games/Sum Fun [Joyce Weisbecker].ch8",
    "games/Pong [Paul Vervalin, 1990].ch8",
    "games/Syzygy [Roy Trevino, 1990].ch8",
    "games/Soccer.ch8",
    "games/Breakout (Brix hack) [David Winter, 1997].ch8",
    "games/Breakout [Carmelo Cortez, 1979].ch8",
    "games/Puzzle.ch8",
    "games/Blinky [Hans Christian Egeberg, 1991].ch8",
    "games/Lunar Lander (Udo Pernisz, 1979).ch8",
    "games/Squash [David Winter].ch8",
    "games/15 Puzzle [Roger Ivie] (alt).ch8",
    "games/Addition Problems [Paul C. Moews].ch8",
    "games/Rush Hour [Hap, 2006] (alt).ch8",
    "games/Blitz [David Winter].ch8",
    "games/Space Flight.ch8",
    "games/Connect 4 [David Winter].ch8",
    "games/Brix [Andreas Gustafsson, 1990].ch8",
    "games/Tron.ch8",
    "games/Bowling [Gooitzen van der Wal].ch8",
    "games/Missile [David Winter].ch8",
    "games/Pong 2 (Pong hack) [David Winter, 1997].ch8",
    "games/Reversi [Philip Baltzer].ch8",
    "games/Astro Dodge [Revival Studios, 2008].ch8",
    "games/Russian Roulette [Carmelo Cortez, 1978].ch8",
    "games/Vertical Brix [Paul Robson, 1996].ch8",
    "games/Hi-Lo [Jef Winsor, 1978].ch8",
    "games/Kaleidoscope [Joseph Weisbecker, 1978].ch8",
    "games/Guess [David Winter].ch8",
    "games/Pong (1 player).ch8",
    "games/Rocket Launcher.ch8",
    "games/Guess [David Winter] (alt).ch8",
    "games/Programmable Spacefighters [Jef Winsor].ch8",
    "games/Wipe Off [Joseph Weisbecker].ch8",
    "games/Vers [JMN, 1991].ch8",
    "games/Tapeworm [JDR, 1999].ch8",
    "games/Nim [Carmelo Cortez, 1978].ch8",
    "games/Tank.ch8",
    "games/Worm V4 [RB-Revival Studios, 2007].ch8",
    "games/Shooting Stars [Philip Baltzer, 1978].ch8",
    "games/Brick (Brix hack, 1990).ch8",
    "games/Wall [David Winter].ch8",
    "games/Hidden [David Winter, 1996].ch8",
    "games/Coin Flipping [Carmelo Cortez, 1978].ch8",
    "games/Rocket Launch [Jonas Lindstedt].ch8",
    "games/Figures.ch8",
    "games/Biorhythm [Jef Winsor].ch8",
    "games/Blinky [Hans Christian Egeberg] (alt).ch8",
    "games/Tic-Tac-Toe [David Winter].ch8",
    "games/Craps [Camerlo Cortez, 1978].ch8",
    "games/Slide [Joyce Weisbecker].ch8",
    "games/Animal Race [Brian Astle].ch8",
    "games/Space Invaders [David Winter].ch8",
    "games/Most Dangerous Game [Peter Maruhnic].ch8",
    "games/Rush Hour [Hap, 2006].ch8",
    "games/Filter.ch8",
    "games/Mastermind FourRow (Robert Lindley, 1978).ch8",
    "games/Tetris [Fran Dachille, 1991].ch8",
    "games/Deflection [John Fort].ch8",
    "games/Rocket [Joseph Weisbecker, 1978].ch8",
    "games/Space Intercept [Joseph Weisbecker, 1978].ch8",
    "games/UFO [Lutz V, 1992].ch8",
    "games/ZeroPong [zeroZshadow, 2007].ch8",
    "games/Spooky Spot [Joseph Weisbecker, 1978].ch8",
    "games/Pong (alt).ch8",
    "games/X-Mirror.ch8",
    "games/15 Puzzle [Roger Ivie].ch8",
    "games/Submarine [Carmelo Cortez, 1978].ch8",
    "games/Landing.ch8",
    "games/Airplane.ch8",
    "games/Merlin [David Winter].ch8",
    "games/Cave.ch8",
    "games/Sequence Shoot [Joyce Weisbecker].ch8",
    "games/Space Invaders [David Winter] (alt).ch8"
);

const DEMOS: &[Program] = embed_roms!(
    "demos/Trip8 Demo (2008) [Revival Studios].ch8",
    "demos/Stars [Sergey Naydenov, 2010].ch8",
    "demos/Maze (alt) [David Winter, 199x].ch8",
    "demos/Sierpinski [Sergey Naydenov, 2010].ch8",
    "demos/Sirpinski [Sergey Naydenov, 2010].ch8",
    "demos/Maze [David Winter, 199x].ch8",
    "demos/Zero Demo [zeroZshadow, 2007].ch8",
    "demos/Particle Demo [zeroZshadow, 2008].ch8"
);

pub struct EmulatorApp {
    emulator: Chip8,
    pixels: [[u8; 64]; 32],
    paused: bool,
}

impl EmulatorApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        EmulatorApp {
            emulator: Chip8::with_program(include_bytes!("ibm_logo.ch8")),
            pixels: [[0u8; 64]; 32],
            paused: false,
        }
    }

    pub fn render_registers(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("CPU State"));
        egui::Grid::new("register_debug")
            .num_columns(5)
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                let register = &self.emulator.register;
                ui.label(RichText::new("0-3:").strong());
                ui.label(format!("{:02x}", register[0x0]));
                ui.label(format!("{:02x}", register[0x1]));
                ui.label(format!("{:02x}", register[0x2]));
                ui.label(format!("{:02x}", register[0x3]));
                ui.end_row();
                ui.label(RichText::new("4-7:").strong());
                ui.label(format!("{:02x}", register[0x4]));
                ui.label(format!("{:02x}", register[0x5]));
                ui.label(format!("{:02x}", register[0x6]));
                ui.label(format!("{:02x}", register[0x7]));
                ui.end_row();
                ui.label(RichText::new("8-B:").strong());
                ui.label(format!("{:02x}", register[0x8]));
                ui.label(format!("{:02x}", register[0x9]));
                ui.label(format!("{:02x}", register[0xA]));
                ui.label(format!("{:02x}", register[0xB]));
                ui.end_row();
                ui.label(RichText::new("C-F:").strong());
                ui.label(format!("{:02x}", register[0xC]));
                ui.label(format!("{:02x}", register[0xD]));
                ui.label(format!("{:02x}", register[0xE]));
                ui.label(format!("{:02x}", register[0xF]));
                ui.end_row();
                ui.label(RichText::new("IDX:").strong());
                ui.label(format!("{:03x}", self.emulator.index));
                ui.label(RichText::new("PC:").strong());
                ui.label(format!("{:03x}", self.emulator.pc));
                ui.end_row();
                ui.label(RichText::new("Delay:").strong());
                ui.label(format!("{:02x}", self.emulator.delay_timer.read()));
                ui.label(RichText::new("Sound:").strong());
                ui.label(format!("{:02x}", self.emulator.sound_timer.read()));
                ui.end_row();
                ui.label(RichText::new("Stack:").strong());
                for addr in &self.emulator.stack {
                    ui.label(format!("0x{:03x}", addr));
                }
            });
        ui.separator();
    }

    pub fn render_instructions(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Instructions").strong());
        egui::Grid::new("instructions_debug")
            .num_columns(1)
            .striped(true)
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                let start_idx = self.emulator.pc - 10;
                let end_idx = start_idx + 30;

                for i in (start_idx..end_idx).step_by(2) {
                    let mut instruction =
                        RichText::new(format!("{:03x}: {}", i, self.emulator.instruction_at(i)))
                            .family(FontFamily::Monospace);
                    if i == self.emulator.pc {
                        instruction = instruction.strong();
                    }
                    ui.label(instruction);
                    ui.end_row()
                }
            });
    }

    pub fn render_display(&mut self, ui: &mut egui::Ui) {
        let desired_size = ui.available_size();
        let (whole_rect, _) =
            ui.allocate_exact_size(desired_size, Sense::focusable_noninteractive());

        // Create fading effect by adding/substracting from internal buffer of
        // pixel brightness.
        let new_pixels = &self.emulator.display.pixels;
        for (y, row) in new_pixels.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                if *pixel {
                    self.pixels[y][x] = self.pixels[y][x].saturating_add(128);
                } else {
                    self.pixels[y][x] = self.pixels[y][x].saturating_sub(16);
                }
            }
        }

        // Draw pixels as rects
        let stroke = Stroke::new(0.5, Color32::from_gray(40));
        let pixel_width = whole_rect.width() / 64.0;
        let pixel_height = whole_rect.height() / 32.0;
        if ui.is_rect_visible(whole_rect) {
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

    fn load_dropped_file(&mut self, drop: &DroppedFile) {
        if let Some(path) = &drop.path {
            let mut data: Vec<u8> = Vec::new();
            File::open(path).unwrap().read_to_end(&mut data).unwrap();
            self.emulator = Chip8::with_program(&data);
        } else if let Some(bytes) = &drop.bytes {
            self.emulator = Chip8::with_program(&*bytes);
        }
    }

    fn update_keys(&mut self, input: &InputState) {
        let keys = &mut self.emulator.keys;
        keys[0x0] = input.key_down(Key::Num0);
        keys[0x1] = input.key_down(Key::Num1);
        keys[0x2] = input.key_down(Key::Num2);
        keys[0x3] = input.key_down(Key::Num3);

        keys[0x4] = input.key_down(Key::Num4);
        keys[0x5] = input.key_down(Key::Num5);
        keys[0x6] = input.key_down(Key::Num6);
        keys[0x7] = input.key_down(Key::Num7);

        keys[0x8] = input.key_down(Key::Num8);
        keys[0x9] = input.key_down(Key::Num9);
        keys[0xA] = input.key_down(Key::A);
        keys[0xB] = input.key_down(Key::B);

        keys[0xC] = input.key_down(Key::C);
        keys[0xD] = input.key_down(Key::D);
        keys[0xE] = input.key_down(Key::E);
        keys[0xF] = input.key_down(Key::F);
    }
}

impl eframe::App for EmulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input_mut().consume_key(Modifiers::NONE, Key::P) {
            self.paused = !self.paused;
        }
        let step = ctx.input_mut().consume_key(Modifiers::NONE, Key::S);

        // Load new program if a file is dropped on the app
        if !ctx.input().raw.dropped_files.is_empty() {
            self.load_dropped_file(&ctx.input().raw.dropped_files[0]);
        }
        self.update_keys(&ctx.input());

        // egui is rendering at 60Hz, Chip8 runs at 500Hz, so we need to run
        // 8-ish cpu cycles for each frame.
        if !self.paused {
            for _ in 0..8 {
                self.emulator.emulate_tick().unwrap();
            }
        } else if step || ctx.input().key_down(Key::C) {
            self.emulator.emulate_tick().unwrap();
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Programs", |ui| {
                    for program in PROGRAMS {
                        if ui.button(program.name).clicked() {
                            self.emulator = Chip8::with_program(program.data);
                        }
                    }
                });
                ui.menu_button("Games", |ui| {
                    for program in GAMES {
                        if ui.button(program.name).clicked() {
                            self.emulator = Chip8::with_program(program.data);
                        }
                    }
                });
                ui.menu_button("Demos", |ui| {
                    for program in DEMOS {
                        if ui.button(program.name).clicked() {
                            self.emulator = Chip8::with_program(program.data);
                        }
                    }
                });
                ui.label("(Or drop a .ch8 file to load it)")
            });

        });

        // Render debug display
        egui::SidePanel::right("debug_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.separator();
                self.render_registers(ui);
                self.render_instructions(ui);
                ui.separator();
                ui.label(RichText::new("Debug Keys:").strong());
                ui.label("P: Pause");
                ui.label("S: Step");
                ui.label("C (hold): Continue");
            });

        // Render emulator display
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_display(ui);
        });

        // Always repaint to keep rendering at 60Hz.
        ctx.request_repaint()
    }
}
