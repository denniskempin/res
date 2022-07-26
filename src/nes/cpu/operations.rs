
use anyhow::Result;
use std::fmt::Display;

use super::Cpu;
use super::StatusFlags;
use crate::nes::bus::Bus;

////////////////////////////////////////////////////////////////////////////////
// Operation

pub struct Operation {
    pub size: u16,
    opcode: OpCodeTableEntry,
    raw_opcode: u8,
    operand: Operand,
}

impl Operation {
    pub fn read(bus: &Bus, addr: u16) -> Result<Operation> {
        let raw_opcode = bus.read_u8(addr);
        let opcode = OPCODE_TABLE[raw_opcode as usize];
        let (operand, size) = match opcode.address_mode {
            AddressMode::Absolute => (Operand::Absolute(bus.read_u16(addr + 1)), 2_u16),
            AddressMode::Relative => (Operand::Relative(bus.read_u8(addr + 1)), 1_u16),
            AddressMode::Implicit => (Operand::Implicit, 0_u16),
            AddressMode::Immediate => (Operand::Immediate(bus.read_u8(addr + 1)), 1_u16),
            AddressMode::ZeroPage => (Operand::ZeroPage(bus.read_u8(addr + 1)), 1_u16),
        };
        Ok(Operation {
            opcode,
            raw_opcode,
            operand,
            size: size + 1,
        })
    }

    pub fn execute(&self, cpu: &mut Cpu, bus: &mut Bus) {
        let mut ctx = Context {
            opcode: &self.opcode,
            bus,
            cpu,
            operand: self.operand,
        };
        (self.opcode.execute)(&mut ctx);
    }

    pub fn raw(&self) -> Vec<u8> {
        let mut raw = vec![self.raw_opcode];
        raw.append(&mut self.operand.raw());
        raw
    }

    pub fn format(self, _bus: &Bus) -> String {
        let mnemonic = self.opcode.mnemonic.to_uppercase();
        let operand_str = format!("{}", self.operand);
        if operand_str.is_empty() {
            self.opcode.mnemonic.to_uppercase()
        } else {
            format!("{mnemonic} {operand_str}")
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.opcode.mnemonic.to_uppercase(),
            self.operand
        )
    }
}

////////////////////////////////////////////////////////////////////////////////
// Operand

#[derive(Copy, Clone)]
pub enum Operand {
    Absolute(u16),
    Relative(u8),
    Implicit,
    Immediate(u8),
    ZeroPage(u8),
}

impl Operand {
    pub fn addr(self, cpu: &Cpu) -> u16 {
        match self {
            Operand::Absolute(addr) => addr,
            Operand::Relative(addr) => cpu.program_counter + addr as u16,
            Operand::ZeroPage(addr) => addr as u16,
            _ => unimplemented!(),
        }
    }

    pub fn load(self, bus: &mut Bus) -> u8 {
        match self {
            Operand::Immediate(value) => value,
            Operand::ZeroPage(addr) => bus.read_u8(addr),
            _ => unimplemented!(),
        }
    }

    pub fn write(self, bus: &mut Bus, value: u8) {
        match self {
            Operand::Absolute(addr) => bus.write_u8(addr, value),
            Operand::ZeroPage(addr) => bus.write_u8(addr, value),
            _ => unimplemented!(),
        }
    }

    pub fn raw(&self) -> Vec<u8> {
        match self {
            Operand::Absolute(value) => value.to_le_bytes().to_vec(),
            Operand::Relative(value) => vec![*value],
            Operand::Implicit => vec![],
            Operand::Immediate(value) => vec![*value],
            Operand::ZeroPage(value) => vec![*value],
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Operand::Absolute(value) => write!(f, "${:04X}", value),
            Operand::Relative(value) => write!(f, "d{:02X}", value),
            Operand::Implicit => Ok(()),
            Operand::Immediate(value) => write!(f, "#${:02X}", value),
            Operand::ZeroPage(value) => write!(f, "${:02X}", value),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// OpCode Table

macro_rules! opcode {
    ($code: literal, $method: ident, $address_mode: ident) => {
        OpCodeTableEntry {
            code: $code,
            address_mode: $address_mode,
            execute: $method,
            mnemonic: stringify!($method),
        }
    };
}

lazy_static! {
    static ref OPCODE_TABLE: [OpCodeTableEntry; 0x100] = {
        use AddressMode::*;
        const OPCODE_LIST: &[OpCodeTableEntry] = &[
            opcode!(0x00, hlt, Implicit),
            opcode!(0x00, hlt, Implicit),
            opcode!(0x20, jsr, Absolute),
            opcode!(0x90, bcc, Relative),
            opcode!(0xB0, bcs, Relative),
            opcode!(0xF0, beq, Relative),
            opcode!(0xD0, bne, Relative),
            opcode!(0xA2, ldx, Immediate),
            opcode!(0xA4, ldy, ZeroPage),
            opcode!(0x65, adc, ZeroPage),
            opcode!(0x85, sta, ZeroPage),
            opcode!(0xE6, inc, ZeroPage),
            opcode!(0x86, stx, ZeroPage),
            opcode!(0x18, clc, Implicit),
            opcode!(0x78, sei, Implicit),
            opcode!(0x38, sec, Implicit),
            opcode!(0xC8, iny, Implicit),
            opcode!(0xA9, lda, Immediate),
            opcode!(0xAA, tax, Implicit),
            opcode!(0xEA, nop, Implicit),
            opcode!(0x4C, jmp, Absolute),
            opcode!(0x8D, sta, Absolute),
        ];

        // Turn list of codes into opcode lookup table
        let mut table = [OpCodeTableEntry::default(); 0x100];
        for entry in OPCODE_LIST {
            table[entry.code as usize] = *entry;
        }
        table
    };
}

#[derive(Copy, Clone)]
struct OpCodeTableEntry {
    pub code: u8,
    pub address_mode: AddressMode,
    pub execute: fn(&mut Context),
    pub mnemonic: &'static str,
}

impl Default for OpCodeTableEntry {
    fn default() -> Self {
        Self {
            code: Default::default(),
            address_mode: AddressMode::Implicit,
            execute: not_implemented,
            mnemonic: "N/A",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum AddressMode {
    Absolute,
    Relative,
    Implicit,
    Immediate,
    ZeroPage,
}

////////////////////////////////////////////////////////////////////////////////
// OpContext

struct Context<'a> {
    opcode: &'a OpCodeTableEntry,
    cpu: &'a mut Cpu,
    bus: &'a mut Bus,
    operand: Operand,
}

impl Context<'_> {
    fn operand_addr(&self) -> u16 {
        self.operand.addr(self.cpu)
    }

    fn load_operand(&mut self) -> u8 {
        self.operand.load(self.bus)
    }

    fn write_operand(&mut self, value: u8) {
        self.operand.write(self.bus, value)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Operation Implementations

fn jmp(ctx: &mut Context) {
    ctx.cpu.program_counter = ctx.operand_addr();
}

fn jsr(ctx: &mut Context) {
    ctx.cpu.stack.push(ctx.cpu.program_counter);
    ctx.cpu.program_counter = ctx.operand_addr();
}

fn sta(ctx: &mut Context) {
    ctx.write_operand(ctx.cpu.a);
}

fn stx(ctx: &mut Context) {
    ctx.write_operand(ctx.cpu.x);
}

fn adc(ctx: &mut Context) {
    ctx.cpu.a += ctx.load_operand();
}

fn inc(ctx: &mut Context) {
    let value = ctx.load_operand() + 1;
    ctx.write_operand(value);
}

fn lda(ctx: &mut Context) {
    ctx.cpu.a = ctx.load_operand();
    ctx.cpu.update_status_flags(ctx.cpu.a);
}

fn tax(ctx: &mut Context) {
    ctx.cpu.x = ctx.cpu.a;
    ctx.cpu.update_status_flags(ctx.cpu.x);
}

fn ldy(ctx: &mut Context) {
    ctx.cpu.y = ctx.load_operand();
}

fn ldx(ctx: &mut Context) {
    ctx.cpu.x = ctx.load_operand();
}

fn iny(ctx: &mut Context) {
    ctx.cpu.y += 1;
}

fn hlt(ctx: &mut Context) {
    ctx.cpu.halt = true;
}

fn sec(ctx: &mut Context) {
    ctx.cpu.status_flags.insert(StatusFlags::CARRY);
}

fn clc(ctx: &mut Context) {
    ctx.cpu.status_flags.remove(StatusFlags::CARRY);
}

fn bcs(ctx: &mut Context) {
    if ctx.cpu.status_flags.contains(StatusFlags::CARRY) {
        ctx.cpu.program_counter = ctx.operand_addr();
    }
}

fn bcc(ctx: &mut Context) {
    if !ctx.cpu.status_flags.contains(StatusFlags::CARRY) {
        ctx.cpu.program_counter = ctx.operand_addr();
    }
}

fn beq(ctx: &mut Context) {
    if ctx.cpu.status_flags.contains(StatusFlags::ZERO) {
        ctx.cpu.program_counter = ctx.operand_addr();
    }
}

fn bne(ctx: &mut Context) {
    if !ctx.cpu.status_flags.contains(StatusFlags::ZERO) {
        ctx.cpu.program_counter = ctx.operand_addr();
    }
}

fn sei(_ctx: &mut Context) {}

fn nop(_ctx: &mut Context) {}

fn not_implemented(ctx: &mut Context) {
    unimplemented!("Opcode {} is not implemented", ctx.opcode.code);
}
