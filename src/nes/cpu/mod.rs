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

pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub status_flags: StatusFlags,
    pub program_counter: u16,
    pub halt: bool,
    pub sp: u8,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            status_flags: StatusFlags::from_bits_truncate(0x24),
            program_counter: 0,
            halt: false,
            sp: 0xFD,
        }
    }
}

impl Cpu {
    const STACK_ADDR: u16 = 0x0100;

    pub fn tick(&mut self, _clock: u64, bus: &mut Bus) -> Result<bool> {
        self.execute_one(bus)
    }

    pub fn execute_one(&mut self, bus: &mut Bus) -> Result<bool> {
        let operation = self.next_operation(bus)?;
        operation.execute(self, bus);
        Ok(!self.halt)
    }

    fn next_operation(&mut self, bus: &mut Bus) -> Result<Operation> {
        let operation = Operation::read(bus, self.program_counter)?;
        self.program_counter += operation.size() as u16;
        Ok(operation)
    }

    fn stack_push(&mut self, bus: &mut Bus, value: u8) {
        bus.write_u8(Self::STACK_ADDR + self.sp as u16, value);
        self.sp -= 1;
    }

    fn stack_pop(&mut self, bus: &mut Bus) -> u8 {
        self.sp += 1;
        bus.read_u8(Self::STACK_ADDR + self.sp as u16)
    }
}
