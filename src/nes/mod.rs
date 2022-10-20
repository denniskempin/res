pub mod apu;
pub mod cartridge;
pub mod cpu;
pub mod joypad;
pub mod ppu;
pub mod trace;

use std::cell::RefCell;
use std::fs;
use std::path::Path;

use bincode::Decode;
use bincode::Encode;

use self::cartridge::Cartridge;
use self::cpu::Cpu;
use self::cpu::ExecResult;
use self::cpu::Operation;
use self::joypad::Joypad;
use self::ppu::Ppu;
use self::trace::Trace;

#[derive(Default, Encode, Decode, Clone)]
pub struct System {
    pub cpu: Cpu,
}

impl System {
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn ppu(&self) -> &Ppu {
        &self.cpu.bus.ppu
    }

    pub fn cartridge(&self) -> &RefCell<Cartridge> {
        &self.cpu.bus.cartridge
    }

    pub fn joypad0_mut(&mut self) -> &mut Joypad {
        &mut self.cpu.bus.joypad0
    }

    pub fn tick(&mut self) -> ExecResult<bool> {
        self.cpu.execute_one()
    }

    pub fn snapshot(&self) -> Vec<u8> {
        bincode::encode_to_vec(self, bincode::config::standard()).unwrap()
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

    pub fn with_program(program: &[u8]) -> ExecResult<System> {
        let mut system = System::default();
        system.cpu.bus.cartridge.borrow_mut().load_program(program);
        system.reset()?;
        system.cpu.boot()?;
        Ok(system)
    }

    pub fn with_ines(path: &Path) -> ExecResult<System> {
        let ines_file = fs::read(path).unwrap();
        System::with_ines_bytes(&ines_file)
    }

    pub fn with_ines_bytes(bytes: &[u8]) -> ExecResult<System> {
        let mut system = System::default();
        system
            .cpu
            .bus
            .cartridge
            .borrow_mut()
            .load_ines(bytes)
            .unwrap();
        system.reset()?;
        system.cpu.boot()?;
        Ok(system)
    }

    pub fn with_snapshot(snapshot: &[u8]) -> ExecResult<System> {
        Ok(
            bincode::decode_from_slice(snapshot, bincode::config::standard())
                .unwrap()
                .0,
        )
    }

    pub fn execute_until_halt(&mut self) -> ExecResult<()> {
        while self.cpu.execute_one()? {
            println!("{:?}", self.trace());
        }
        Ok(())
    }

    pub fn execute_one_frame(&mut self) -> ExecResult<()> {
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

    pub fn execute_frames(&mut self, num_frames: usize) -> ExecResult<()> {
        for _ in 0..num_frames {
            self.execute_one_frame()?;
        }
        Ok(())
    }

    pub fn reset(&mut self) -> ExecResult<()> {
        self.cpu.program_counter = self.cpu.bus.read_u16(0xFFFC_u16)?;
        Ok(())
    }
}
