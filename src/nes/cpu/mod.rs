mod operations;

pub use self::operations::Operation;

use super::bus::Bus;
use anyhow::Result;
use bitflags::bitflags;

////////////////////////////////////////////////////////////////////////////////
// StatusFlags

bitflags! {
    #[derive(Default)]
    pub struct StatusFlags: u8 {
        const NEGATIVE = 0b1000_0000;
        const OVERFLOW = 0b0100_0000;
        const UNUSED = 0b0010_0000;
        const BREAK = 0b0001_0000;
        const DECIMAL = 0b0000_1000;
        const INTERRUPT = 0b0000_0100;
        const ZERO = 0b0000_0010;
        const CARRY = 0b0000_0001;
    }
}

////////////////////////////////////////////////////////////////////////////////
// Cpu

#[derive(Default)]
pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub status_flags: StatusFlags,
    pub program_counter: u16,
    pub halt: bool,
    pub stack: Vec<u16>,
}

impl Cpu {
    pub fn tick(&mut self, _clock: u64, bus: &mut Bus) -> Result<bool> {
        self.execute_one(bus)
    }

    pub fn execute_one(&mut self, bus: &mut Bus) -> Result<bool> {
        let operation = self.next_operation(bus)?;
        operation.execute(self, bus);
        Ok(!self.halt)
    }

    pub fn update_status_flags(&mut self, value: u8) {
        self.status_flags.set(StatusFlags::ZERO, value == 0);
        self.status_flags
            .set(StatusFlags::NEGATIVE, value & 0b1000_0000 != 0);
    }

    fn next_operation(&mut self, bus: &mut Bus) -> Result<Operation> {
        let operation = Operation::read(bus, self.program_counter)?;
        self.program_counter += operation.size;
        Ok(operation)
    }
}
