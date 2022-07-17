mod opcodes;

use super::bus::Bus;
use anyhow::anyhow;
use anyhow::Ok;
use anyhow::Result;
use opcodes::AddressMode;
use opcodes::OpCodeTableEntry;
use opcodes::OpContext;
use opcodes::Operand;
use opcodes::OPCODE_TABLE;

////////////////////////////////////////////////////////////////////////////////
// StatusFlags

#[derive(Default)]
pub struct StatusFlags {
    pub flags: u8,
}

impl StatusFlags {
    pub fn set_zero(&mut self, value: bool) {
        self.set_bit(0b0000_0010, value);
    }

    pub fn set_negative(&mut self, value: bool) {
        self.set_bit(0b1000_0000, value);
    }

    fn set_bit(&mut self, mask: u8, value: bool) {
        if value {
            self.flags |= mask;
        } else {
            self.flags &= !mask;
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Cpu

#[derive(Default)]
pub struct Cpu {
    pub registers: Registers,
    pub status_flags: StatusFlags,
    pub program_counter: u16,
    pub halt: bool,
}

#[derive(Default)]
pub struct Registers {
    pub a: u8,
    pub x: u8,
    pub y: u8,
}

impl Cpu {
    pub fn execute_one(&mut self, bus: &mut Bus) -> Result<bool> {
        let (operation, operand) = self.next_operation(bus)?;
        let mnemonic = match operand {
            Operand::Implicit => operation.mnemonic.to_string(),
            Operand::Immediate(value) => format!("{} #{:02X}", operation.mnemonic, value),
            Operand::ZeroPage(value) => format!("{} d{:02X}", operation.mnemonic, value),
        };
        println!("{}", mnemonic);

        let mut ctx = OpContext {
            cpu: self,
            bus,
            operand,
        };
        (operation.execute)(&mut ctx);
        Ok(!self.halt)
    }

    pub fn update_status_flags(&mut self, value: u8) {
        self.status_flags.set_zero(value == 0);
        self.status_flags.set_negative(value & 0b1000_0000 != 0);
    }

    fn next_program_byte(&mut self, bus: &mut Bus) -> u8 {
        let byte = bus.read_u8(self.program_counter);
        self.program_counter += 1;
        byte
    }

    fn next_operation(&mut self, bus: &mut Bus) -> Result<(OpCodeTableEntry, Operand)> {
        let opcode = self.next_program_byte(bus);
        let operation = OPCODE_TABLE[opcode as usize];
        if let Some(operation) = operation {
            let operand = match operation.address_mode {
                AddressMode::Implicit => Operand::Implicit,
                AddressMode::Immediate => Operand::Immediate(self.next_program_byte(bus)),
                AddressMode::ZeroPage => Operand::ZeroPage(self.next_program_byte(bus)),
            };
            Ok((operation, operand))
        } else {
            Err(anyhow!("Unsupported opcode {opcode:2x}"))
        }
    }
}
