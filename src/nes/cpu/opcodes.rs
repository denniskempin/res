use crate::nes::bus::Bus;

use super::Cpu;

////////////////////////////////////////////////////////////////////////////////
// OpCode Table

macro_rules! opcode {
    ($method: ident, $address_mode: ident) => {
        Some(OpCodeTableEntry {
            address_mode: $address_mode,
            execute: $method,
            mnemonic: stringify!($method),
        })
    };
}

lazy_static! {
    pub static ref OPCODE_TABLE: [Option<OpCodeTableEntry>; 0x100] = {
        use AddressMode::*;
        let mut table = [Option::None; 0x100];
        table[0x65] = opcode!(adc, ZeroPage);
        table[0x85] = opcode!(sta, ZeroPage);
        table[0xA4] = opcode!(ldy, ZeroPage);
        table[0xA9] = opcode!(lda, Immediate);
        table[0xAA] = opcode!(tax, Implicit);
        table[0xC8] = opcode!(iny, Implicit);
        table[0xE6] = opcode!(inc, ZeroPage);
        table[0x00] = opcode!(hlt, Implicit);
        table
    };
}

#[derive(Copy, Clone)]
pub struct OpCodeTableEntry {
    pub address_mode: AddressMode,
    pub execute: fn(&mut OpContext),
    pub mnemonic: &'static str,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AddressMode {
    Implicit,
    Immediate,
    ZeroPage,
}

////////////////////////////////////////////////////////////////////////////////
// OpContext

pub struct OpContext<'a> {
    pub cpu: &'a mut Cpu,
    pub bus: &'a mut Bus,
    pub operand: Operand,
}

#[derive(Copy, Clone)]
pub enum Operand {
    Implicit,
    Immediate(u8),
    ZeroPage(u8),
}

impl OpContext<'_> {
    pub fn load_operand_u8(&mut self) -> u8 {
        match self.operand {
            Operand::Implicit => panic!("Implicit address mode cannot be loaded."),
            Operand::Immediate(value) => value,
            Operand::ZeroPage(addr) => self.bus.read_u8(addr),
        }
    }

    pub fn write_operand_u8(&mut self, value: u8) {
        match self.operand {
            Operand::Implicit => panic!("Implicit address mode cannot be written."),
            Operand::Immediate(_) => panic!("Immediate address mode cannot be written."),
            Operand::ZeroPage(addr) => self.bus.write_u8(addr, value),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Operation Implementations

fn sta(ctx: &mut OpContext) {
    ctx.write_operand_u8(ctx.cpu.registers.a);
}

fn adc(ctx: &mut OpContext) {
    ctx.cpu.registers.a += ctx.load_operand_u8();
}

fn inc(ctx: &mut OpContext) {
    let value = ctx.load_operand_u8() + 1;
    ctx.write_operand_u8(value);
}

fn lda(ctx: &mut OpContext) {
    ctx.cpu.registers.a = ctx.load_operand_u8();
    ctx.cpu.update_status_flags(ctx.cpu.registers.a);
}

fn tax(ctx: &mut OpContext) {
    ctx.cpu.registers.x = ctx.cpu.registers.a;
    ctx.cpu.update_status_flags(ctx.cpu.registers.x);
}

fn ldy(ctx: &mut OpContext) {
    ctx.cpu.registers.y = ctx.load_operand_u8();
}

fn iny(ctx: &mut OpContext) {
    ctx.cpu.registers.y += 1;
}

fn hlt(ctx: &mut OpContext) {
    ctx.cpu.halt = true;
}
