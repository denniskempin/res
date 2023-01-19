pub mod apu;
pub mod cartridge;
pub mod cpu;
pub mod debugger;
pub mod joypad;
pub mod ppu;
pub mod trace;

use std::cell::RefCell;
use std::fs;
use std::path::Path;

use anyhow::anyhow;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use tracing::instrument;

use self::cartridge::Cartridge;
use self::cpu::Cpu;
use self::cpu::CpuBus;
use self::cpu::Operation;
use self::ppu::Ppu;
use self::trace::Trace;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct RecordEntry {
    pub frame: usize,
    pub joypad_0: [bool; 8],
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Record {
    pub entries: Vec<RecordEntry>,
}

#[derive(Clone)]
pub struct System {
    pub cpu: Cpu,
    pub record_to: Option<Record>,
    pub playback_from: Option<Record>,
    pub delta_t_accumulator: f64,
    pub audio_buffer: Vec<f32>,
    pub audio_sample_rate: usize,
    pub s_since_last_sample: f64,
}

impl System {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            record_to: None,
            playback_from: None,
            delta_t_accumulator: 0.0,
            audio_buffer: Vec::new(),
            audio_sample_rate: 45_100,
            s_since_last_sample: 0.0,
        }
    }
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn ppu(&self) -> &Ppu {
        &self.cpu.bus.ppu
    }

    pub fn cartridge(&self) -> &RefCell<Cartridge> {
        &self.cpu.bus.cartridge
    }

    pub fn tick(&mut self) -> Result<bool> {
        self.cpu.execute_one()
    }

    pub fn playback_from_file(&mut self, file: &Path) {
        self.playback_from = Some(serde_json::from_slice(&fs::read(file).unwrap()).unwrap());
    }

    pub fn update_buttons(&mut self, joypad0: [bool; 8]) {
        if let Some(record) = &self.playback_from {
            for entry in &record.entries {
                if entry.frame == self.ppu().frame {
                    println!("Playback: {:?}", entry);
                    self.cpu.bus.joypad0.update_buttons(entry.joypad_0);
                }
            }
        } else if self.cpu.bus.joypad0.update_buttons(joypad0) {
            if let Some(record) = &mut self.record_to {
                record.entries.push(RecordEntry {
                    frame: self.cpu.bus.ppu.frame,
                    joypad_0: joypad0,
                });
                println!("Recorded: {:?}", record.entries.last());
            }
        }
    }

    pub fn snapshot(&self) -> Vec<u8> {
        bincode::encode_to_vec(&self.cpu, bincode::config::standard()).unwrap()
    }

    pub fn trace(&self) -> Trace {
        if let Some(operation) = Operation::peek(&self.cpu, self.cpu.program_counter) {
            Trace {
                pc: self.cpu.program_counter,
                opcode_raw: self
                    .cpu
                    .bus
                    .peek_slice(self.cpu.program_counter, operation.size() as u16)
                    .map(|b| b.unwrap_or(0))
                    .collect(),
                legal: true,
                opcode_str: operation.format(&self.cpu),
                a: self.cpu.a,
                x: self.cpu.x,
                y: self.cpu.y,
                p: self.cpu.status_flags,
                sp: self.cpu.sp,
                ppu_scanline: self.cpu.bus.ppu.scanline,
                ppu_cycle: self.cpu.bus.ppu.cycle,
                cpu_cycle: self.cpu.cycle,
            }
        } else {
            Trace {
                pc: self.cpu.program_counter,
                opcode_raw: vec![self.cpu.bus.peek(self.cpu.program_counter).unwrap_or(0)],
                legal: false,
                opcode_str: "N/A".to_string(),
                a: self.cpu.a,
                x: self.cpu.x,
                y: self.cpu.y,
                p: self.cpu.status_flags,
                sp: self.cpu.sp,
                ppu_scanline: self.cpu.bus.ppu.scanline,
                ppu_cycle: self.cpu.bus.ppu.cycle,
                cpu_cycle: self.cpu.cycle,
            }
        }
    }

    pub fn with_program(program: &[u8]) -> Result<System> {
        let mut system = System::new();
        system
            .cpu
            .bus
            .cartridge
            .borrow_mut()
            .load_nrom_with_data(program, &[]);
        system.reset()?;
        system.cpu.boot()?;
        Ok(system)
    }

    pub fn with_ines(path: &Path) -> Result<System> {
        let ines_file = fs::read(path).unwrap();
        System::with_ines_bytes(&ines_file, None)
    }

    pub fn with_ines_bytes(bytes: &[u8], persistent_data: Option<&[u8]>) -> Result<System> {
        let mut system = System::new();
        system
            .cpu
            .bus
            .cartridge
            .borrow_mut()
            .load_ines(bytes, persistent_data)
            .unwrap();
        system.reset()?;
        system.cpu.boot()?;
        Ok(system)
    }

    pub fn with_snapshot(snapshot: &[u8]) -> Result<System> {
        Ok(System {
            cpu: bincode::decode_from_slice(snapshot, bincode::config::standard())
                .unwrap()
                .0,
            ..System::new()
        })
    }

    pub fn execute_until<F>(&mut self, should_break: F) -> Result<()>
    where
        F: Fn(&Cpu) -> bool,
    {
        const S_PER_CYCLE: f64 = 1.0 / (1.789773 * 1000.0 * 1000.0);
        let s_per_sample: f64 = 1.0 / self.audio_sample_rate as f64;

        self.cpu.debugger.borrow_mut().start_execution();
        loop {
            if self.cpu.halt {
                return Err(anyhow!("CPU halted"));
            }

            let cycles_before = self.cpu.cycle;
            if let Err(e) = self.cpu.execute_one() {
                return Err(anyhow!("Execution failed: {:?}", e));
            }

            self.s_since_last_sample += (self.cpu.cycle - cycles_before) as f64 * S_PER_CYCLE;
            if self.s_since_last_sample > s_per_sample {
                self.s_since_last_sample -= s_per_sample;
                self.audio_buffer.push(self.cpu.bus.apu.sample());
            }

            if should_break(&self.cpu) {
                return Ok(());
            }

            if self.cpu.debugger.borrow().should_break() {
                println!("Breakpoint: {}", self.cpu.debugger.borrow().break_message());
                return Err(anyhow!(
                    "Breakpoint: {}",
                    self.cpu.debugger.borrow().break_message()
                ));
            }
        }
    }

    pub fn execute_until_halt(&mut self) -> Result<()> {
        self.execute_until(|cpu| cpu.halt)
    }

    #[instrument(skip_all)]
    pub fn execute_one_frame(&mut self) -> Result<()> {
        let current_frame = self.ppu().frame;
        self.execute_until(|cpu| cpu.bus.ppu.frame > current_frame)
    }

    pub fn execute_frames(&mut self, num_frames: usize) -> Result<()> {
        for _ in 0..num_frames {
            self.execute_one_frame()?;
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn execute_for_duration(&mut self, seconds: f64) -> Result<()> {
        self.delta_t_accumulator += seconds;
        const S_PER_FRAME: f64 = 0.016639260956557062;
        let num_frames = (self.delta_t_accumulator / S_PER_FRAME).floor() as usize;
        self.delta_t_accumulator -= num_frames as f64 * S_PER_FRAME;
        self.execute_frames(num_frames)
    }

    pub fn reset(&mut self) -> Result<()> {
        self.cpu.program_counter = self.cpu.bus.read_u16(0xFFFC_u16)?;
        Ok(())
    }
}
