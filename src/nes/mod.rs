pub mod apu;
pub mod cartridge;
pub mod cpu;
pub mod ppu;
pub mod trace;

use self::cpu::Cpu;
use self::cpu::Operation;
use self::trace::Trace;
use anyhow::Result;
use std::fs;
use std::path::Path;

#[derive(Default)]
pub struct System {
    pub cpu: Cpu,
    pub clock: u64,
}

impl System {
    pub fn tick(&mut self) -> Result<bool> {
        if !self.cpu.tick()? {
            return Ok(false);
        }
        self.cpu.bus.ppu.tick();
        self.cpu.bus.ppu.tick();
        self.cpu.bus.ppu.tick();
        Ok(true)
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
                legal: operation.is_legal(),
                opcode_str: operation.format(&self.cpu),
                a: self.cpu.a,
                x: self.cpu.x,
                y: self.cpu.y,
                p: self.cpu.status_flags,
                sp: self.cpu.sp,
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
            })
        }
    }

    pub fn with_program(program: &[u8]) -> Result<System> {
        let mut system = System::default();
        system.load_program(program)?;
        Ok(system)
    }

    pub fn with_ines(path: &Path) -> Result<System> {
        let mut system = System::default();
        system.load_ines(path)?;
        Ok(system)
    }

    pub fn execute_until_halt(&mut self) -> Result<()> {
        while self.cpu.execute_one()? {}
        Ok(())
    }

    pub fn load_program(&mut self, program: &[u8]) -> Result<()> {
        self.cpu.bus.cartridge.load_program(program);
        self.reset()
    }

    pub fn load_ines(&mut self, path: &Path) -> Result<()> {
        let ines_file = fs::read(path)?;
        self.cpu.bus.cartridge.load_ines(&ines_file)?;
        self.reset()
    }

    pub fn reset(&mut self) -> Result<()> {
        self.cpu.program_counter = self.cpu.bus.read_u16(0xFFFC_u16);
        Ok(())
    }
}
