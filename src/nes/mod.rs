pub mod cartridge;
pub mod cpu;
pub mod trace;

use anyhow::Result;
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use self::cartridge::Cartridge;
use self::cpu::Cpu;
use self::cpu::Operation;
use self::trace::Trace;

pub struct System {
    pub cpu: Cpu,
    pub cartridge: Rc<RefCell<Cartridge>>,
    pub clock: u64,
}

impl Default for System {
    fn default() -> Self {
        let cartridge = Rc::new(RefCell::new(Cartridge::default()));
        Self {
            cpu: Cpu::new(cartridge.clone()),
            cartridge,
            clock: 0,
        }
    }
}

impl System {
    pub fn tick(&mut self) -> Result<bool> {
        self.clock += 1;
        self.cpu.tick(self.clock)
    }

    pub fn trace(&self) -> Result<Trace> {
        if let Ok(operation) = Operation::read(&self.cpu, self.cpu.program_counter) {
            Ok(Trace {
                pc: self.cpu.program_counter,
                opcode_raw: self
                    .cpu
                    .slice(self.cpu.program_counter, operation.size() as u16)
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
                opcode_raw: vec![self.cpu.read(self.cpu.program_counter)],
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

    pub fn execute_until_halt(&mut self) -> Result<()> {
        while self.cpu.execute_one()? {}
        Ok(())
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.cpu.program_counter = Cartridge::START_ADDR;
        self.cartridge.borrow_mut().load_program(program);
    }

    pub fn load_ines(&mut self, path: &Path) -> Result<()> {
        let ines_file = fs::read(path)?;
        self.cartridge.borrow_mut().load_ines(&ines_file)?;
        self.reset()
    }

    pub fn reset(&mut self) -> Result<()> {
        self.cpu.program_counter = self.cpu.read_u16(0xFFFC_u16);
        Ok(())
    }
}
