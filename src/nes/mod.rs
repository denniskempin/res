pub mod apu;
pub mod cartridge;
pub mod cpu;
pub mod joypad;
pub mod ppu;
pub mod trace;
mod util;

use std::fs;
use std::path::Path;

use anyhow::Result;
use bincode::Decode;
use bincode::Encode;

use self::cpu::Cpu;
use self::cpu::Operation;
use self::trace::Trace;

#[derive(Default, Encode, Decode)]
pub struct System {
    pub cpu: Cpu,
    pub last_apu_cycle: u32,
}

impl System {
    pub fn tick(&mut self) -> Result<bool> {
        self.cpu.execute_one()
    }

    pub fn snapshot(&self) -> Vec<u8> {
        bincode::encode_to_vec(self, bincode::config::standard()).unwrap()
    }

    pub fn trace(&self) -> Result<Trace> {
        if let Ok(operation) = Operation::peek(&self.cpu, self.cpu.program_counter) {
            Ok(Trace {
                pc: self.cpu.program_counter,
                opcode_raw: self
                    .cpu
                    .bus
                    .peek_slice(self.cpu.program_counter, operation.size() as u16)
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
            })
        } else {
            Ok(Trace {
                pc: self.cpu.program_counter,
                opcode_raw: vec![self.cpu.bus.peek(self.cpu.program_counter)],
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
            })
        }
    }

    pub fn with_program(program: &[u8]) -> Result<System> {
        let mut system = System::default();
        system.load_program(program)?;
        system.cpu.boot();
        Ok(system)
    }

    pub fn with_ines(path: &Path) -> Result<System> {
        let mut system = System::default();
        let ines_file = fs::read(path)?;
        system.load_ines(&ines_file)?;
        system.cpu.boot();
        Ok(system)
    }

    pub fn with_ines_bytes(bytes: &[u8]) -> Result<System> {
        let mut system = System::default();
        system.load_ines(bytes)?;
        system.cpu.boot();
        Ok(system)
    }

    pub fn with_snapshot(snapshot: &[u8]) -> Result<System> {
        Ok(bincode::decode_from_slice(snapshot, bincode::config::standard())?.0)
    }

    pub fn execute_until_halt(&mut self) -> Result<()> {
        while self.cpu.execute_one()? {
            println!("{:?}", self.trace());
        }
        Ok(())
    }

    pub fn execute_one_frame(&mut self) -> Result<()> {
        // Finish current frame until we enter vblank
        while !self.cpu.bus.ppu.vblank {
            self.cpu.execute_one()?;
        }
        // Execute current vblank perior until we reach the next frame.
        while self.cpu.bus.ppu.vblank {
            self.cpu.execute_one()?;
        }
        Ok(())
    }

    pub fn execute_frames(&mut self, num_frames: usize) -> Result<()> {
        for _ in 0..num_frames {
            self.execute_one_frame()?;
        }
        Ok(())
    }

    pub fn load_program(&mut self, program: &[u8]) -> Result<()> {
        self.cpu.bus.cartridge.borrow_mut().load_program(program);
        self.reset()
    }

    pub fn load_ines(&mut self, bytes: &[u8]) -> Result<()> {
        self.cpu.bus.cartridge.borrow_mut().load_ines(bytes)?;
        self.reset()
    }

    pub fn reset(&mut self) -> Result<()> {
        self.cpu.program_counter = self.cpu.bus.read_u16(0xFFFC_u16);
        Ok(())
    }
}
