pub mod bus;
pub mod cpu;
pub mod trace;

use anyhow::Result;
use std::fs;
use std::path::Path;

use self::bus::Bus;
use self::bus::RomDevice;
use self::cpu::Cpu;
use self::cpu::Operation;
use self::trace::Trace;

////////////////////////////////////////////////////////////////////////////////
// System

#[derive(Default)]
pub struct System {
    pub bus: Bus,
    pub cpu: Cpu,
    pub clock: u64,
}

impl System {
    pub fn tick(&mut self) -> Result<bool> {
        self.clock += 1;
        self.cpu.tick(self.clock, &mut self.bus)
    }

    pub fn trace(&self) -> Result<Trace> {
        if let Ok(operation) = Operation::read(&self.bus, self.cpu.program_counter) {
            Ok(Trace {
                pc: self.cpu.program_counter,
                opcode_raw: self
                    .bus
                    .slice(self.cpu.program_counter, operation.size())
                    .to_vec(),
                opcode_str: operation.format(&self.cpu, &self.bus),
                a: self.cpu.a,
                x: self.cpu.x,
                y: self.cpu.y,
                p: self.cpu.status_flags.bits(),
            })
        } else {
            Ok(Trace {
                pc: self.cpu.program_counter,
                opcode_raw: vec![self.bus.read_u8(self.cpu.program_counter)],
                opcode_str: "N/A".to_string(),
                a: self.cpu.a,
                x: self.cpu.x,
                y: self.cpu.y,
                p: self.cpu.status_flags.bits(),
            })
        }
    }

    pub fn execute_until_halt(&mut self) -> Result<()> {
        while self.cpu.execute_one(&mut self.bus)? {}
        Ok(())
    }

    pub fn with_program(program: &[u8]) -> System {
        let mut system = System::default();
        system.load_program(program);
        system
    }

    pub fn with_ines(path: &Path) -> Result<System> {
        let mut system = System::default();
        system.load_ines(path)?;
        Ok(system)
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.cpu.program_counter = RomDevice::START_ADDR;
        self.bus.rom.load_program(program);
    }

    pub fn load_ines(&mut self, path: &Path) -> Result<()> {
        let ines_file = fs::read(path)?;
        self.bus.rom.load_ines(&ines_file)?;
        self.reset()
    }

    pub fn reset(&mut self) -> Result<()> {
        self.cpu.program_counter = self.bus.read_u16(0xFFFC_u16);
        Ok(())
    }
}
