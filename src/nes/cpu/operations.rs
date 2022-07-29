use anyhow::Result;

use std::marker::PhantomData;

use super::Cpu;
use super::StatusFlags;
use crate::nes::bus::Bus;

////////////////////////////////////////////////////////////////////////////////
// Operation

pub struct Operation {
    addr: u16,
    table_entry: OpCodeTableEntry,
}

impl Operation {
    pub fn size(&self) -> usize {
        self.table_entry.operand_size + 1
    }

    pub fn read(bus: &Bus, addr: u16) -> Result<Operation> {
        let raw_opcode = bus.read_u8(addr);
        let table_entry = OPCODE_TABLE[raw_opcode as usize];
        Ok(Operation { addr, table_entry })
    }

    pub fn execute(&self, cpu: &mut Cpu, bus: &mut Bus) {
        (self.table_entry.execute_fn)(cpu, bus, self.addr);
    }

    pub fn format(self, cpu: &Cpu, bus: &Bus) -> String {
        (self.table_entry.format_fn)(cpu, bus, self.addr)
    }
}

////////////////////////////////////////////////////////////////////////////////
// OpCode Table

macro_rules! opcode {
    ($code: literal, $method: ident, $address_mode: ident) => {
        OpCodeTableEntry {
            code: $code,
            operand_size: $address_mode::OPERAND_SIZE,
            execute_fn: |cpu, bus, addr| {
                $method(&mut Context::<$address_mode> {
                    cpu,
                    bus,
                    addr,
                    phantom: PhantomData,
                })
            },
            format_fn: |cpu, bus, addr| {
                format!(
                    "{}{}",
                    stringify!($method).to_uppercase(),
                    $address_mode::format(cpu, bus, addr)
                )
            },
        }
    };
}

lazy_static! {
    static ref OPCODE_TABLE: [OpCodeTableEntry; 0x100] = {
        const OPCODE_LIST: &[OpCodeTableEntry] = &[
            //
            opcode!(0x00, hlt, Implicit),
            opcode!(0x00, hlt, Implicit),
            opcode!(0x10, bpl, Relative),
            opcode!(0x20, jsr, Absolute),
            opcode!(0x30, bmi, Relative),
            opcode!(0x60, rts, Implicit),
            opcode!(0x90, bcc, Relative),
            opcode!(0xA0, ldy, Immediate),
            opcode!(0xB0, bcs, Relative),
            opcode!(0xF0, beq, Relative),
            opcode!(0xD0, bne, Relative),

            //
            opcode!(0x91, sta, IndirectY),
            //
            opcode!(0xA2, ldx, Immediate),

            //
            opcode!(0x24, bit, ZeroPage),
            opcode!(0xA4, ldy, ZeroPage),
            opcode!(0x84, sty, ZeroPage),

            //
            opcode!(0x65, adc, ZeroPage),
            opcode!(0x85, sta, ZeroPage),
            opcode!(0x95, sta, ZeroPageX),

            //
            opcode!(0xE6, inc, ZeroPage),
            opcode!(0x86, stx, ZeroPage),

            //
            opcode!(0x18, clc, Implicit),
            opcode!(0x78, sei, Implicit),
            opcode!(0x38, sec, Implicit),
            opcode!(0x88, dey, Implicit),
            opcode!(0xC8, iny, Implicit),
            opcode!(0xD8, cld, Implicit),
            opcode!(0xE8, inx, Implicit),

            //
            opcode!(0xA9, lda, Immediate),

            //
            opcode!(0x9A, txs, Implicit),
            opcode!(0xAA, tax, Implicit),
            opcode!(0xBA, tsx, Implicit),
            opcode!(0xCA, dex, Implicit),
            opcode!(0xEA, nop, Implicit),

            //
            opcode!(0x4C, jmp, Absolute),
            opcode!(0x2C, bit, Absolute),

            //
            opcode!(0x8D, sta, Absolute),
            opcode!(0x9D, sta, AbsoluteX),

            //
            opcode!(0x8E, stx, Absolute),
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
    pub operand_size: usize,
    pub execute_fn: fn(cpu: &mut Cpu, bus: &mut Bus, addr: u16),
    pub format_fn: fn(cpu: &Cpu, bus: &Bus, addr: u16) -> String,
}

impl Default for OpCodeTableEntry {
    fn default() -> Self {
        Self {
            code: Default::default(),
            operand_size: 0,
            execute_fn: |_, _, _| unimplemented!(),
            format_fn: |_, _, _| "N/A".to_string(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Context

/// Context for executing an operation. Allows the operation to load and store
/// operands regardless of their address mode.
/// Makes use of monomorphization to build execute_fn's that are optimiezd at
/// compile time for their address mode and do not require additional branches
/// or lookups of which mode to use.
struct Context<'a, AM: AddressModeImpl> {
    cpu: &'a mut Cpu,
    bus: &'a mut Bus,
    addr: u16,
    phantom: PhantomData<AM>,
}

impl<AM: AddressModeImpl> Context<'_, AM> {
    pub fn operand_addr(&self) -> u16 {
        AM::addr(self.cpu, self.bus, self.addr)
    }

    pub fn load_operand(&self) -> u8 {
        AM::load(self.cpu, self.bus, self.addr)
    }

    pub fn store_operand(&mut self, value: u8) {
        AM::store(self.cpu, self.bus, self.addr, value)
    }

    pub fn update_negative_zero_flags(&mut self, value: u8) {
        self.cpu.status_flags.set(StatusFlags::ZERO, value == 0);
        self.cpu
            .status_flags
            .set(StatusFlags::NEGATIVE, value & 0b1000_0000 != 0);
    }
}

////////////////////////////////////////////////////////////////////////////////
// Address Modes

struct Immediate {}
impl AddressModeImpl for Immediate {
    const OPERAND_SIZE: usize = 1;

    fn addr(_cpu: &Cpu, _bus: &Bus, _addr: u16) -> u16 {
        unimplemented!();
    }

    fn load(_cpu: &Cpu, bus: &Bus, addr: u16) -> u8 {
        bus.read_u8(addr + 1)
    }

    fn store(_cpu: &Cpu, _bus: &mut Bus, _addr: u16, _value: u8) {
        unimplemented!();
    }

    fn format(cpu: &Cpu, bus: &Bus, addr: u16) -> String {
        return format!(" #{:02X}", Self::load(cpu, bus, addr));
    }
}

struct Implicit {}
impl AddressModeImpl for Implicit {
    const OPERAND_SIZE: usize = 0;

    fn addr(_cpu: &Cpu, _bus: &Bus, _addr: u16) -> u16 {
        unimplemented!();
    }

    fn load(_cpu: &Cpu, _bus: &Bus, _addr: u16) -> u8 {
        unimplemented!();
    }

    fn store(_cpu: &Cpu, _bus: &mut Bus, _addr: u16, _value: u8) {
        unimplemented!();
    }

    fn format(_cpu: &Cpu, _bus: &Bus, _addr: u16) -> String {
        "".to_string()
    }
}

struct Absolute {}
impl AddressModeImpl for Absolute {
    const OPERAND_SIZE: usize = 2;

    fn addr(_cpu: &Cpu, bus: &Bus, addr: u16) -> u16 {
        bus.read_u16(addr + 1)
    }

    fn format(cpu: &Cpu, bus: &Bus, addr: u16) -> String {
        return format!(
            " {:04X} @{:02X}",
            Self::addr(cpu, bus, addr),
            Self::load(cpu, bus, addr)
        );
    }
}

struct AbsoluteX {}
impl AddressModeImpl for AbsoluteX {
    const OPERAND_SIZE: usize = 2;

    fn addr(cpu: &Cpu, bus: &Bus, addr: u16) -> u16 {
        bus.read_u16(addr + 1) + cpu.x as u16
    }

    fn format(cpu: &Cpu, bus: &Bus, addr: u16) -> String {
        return format!(
            " {:04X}+X ={:04X} @{:02X}",
            bus.read_u16(addr + 1),
            Self::addr(cpu, bus, addr),
            Self::load(cpu, bus, addr)
        );
    }
}

struct AbsoluteY {}
impl AddressModeImpl for AbsoluteY {
    const OPERAND_SIZE: usize = 2;

    fn addr(cpu: &Cpu, bus: &Bus, addr: u16) -> u16 {
        bus.read_u16(addr + 1) + cpu.y as u16
    }

    fn format(cpu: &Cpu, bus: &Bus, addr: u16) -> String {
        return format!(
            " {:04X}+Y ={:04X} @{:02X}",
            bus.read_u16(addr + 1),
            Self::addr(cpu, bus, addr),
            Self::load(cpu, bus, addr)
        );
    }
}

struct ZeroPage {}
impl AddressModeImpl for ZeroPage {
    const OPERAND_SIZE: usize = 1;

    fn addr(_cpu: &Cpu, bus: &Bus, addr: u16) -> u16 {
        bus.read_u8(addr + 1) as u16
    }

    fn format(cpu: &Cpu, bus: &Bus, addr: u16) -> String {
        return format!(
            " {:02X} @ {:02X}",
            Self::addr(cpu, bus, addr),
            Self::load(cpu, bus, addr)
        );
    }
}

struct ZeroPageX {}
impl AddressModeImpl for ZeroPageX {
    const OPERAND_SIZE: usize = 1;

    fn addr(cpu: &Cpu, bus: &Bus, addr: u16) -> u16 {
        bus.read_u8(addr + 1) as u16 + cpu.x as u16
    }

    fn format(cpu: &Cpu, bus: &Bus, addr: u16) -> String {
        return format!(
            " {:02X}+X ={:04X} @ {:02X}",
            bus.read_u8(addr + 1),
            Self::addr(cpu, bus, addr),
            Self::load(cpu, bus, addr)
        );
    }
}

struct ZeroPageY {}
impl AddressModeImpl for ZeroPageY {
    const OPERAND_SIZE: usize = 1;

    fn addr(cpu: &Cpu, bus: &Bus, addr: u16) -> u16 {
        bus.read_u8(addr + 1) as u16 + cpu.y as u16
    }

    fn format(cpu: &Cpu, bus: &Bus, addr: u16) -> String {
        return format!(
            " {:02X}+Y ={:04X} @ {:02X}",
            bus.read_u8(addr + 1),
            Self::addr(cpu, bus, addr),
            Self::load(cpu, bus, addr)
        );
    }
}

struct Relative {}
impl AddressModeImpl for Relative {
    const OPERAND_SIZE: usize = 1;

    fn addr(_cpu: &Cpu, bus: &Bus, addr: u16) -> u16 {
        let delta = bus.read_u8(addr + 1) as i8 as i16;
        let base_addr = addr + 1 + Self::OPERAND_SIZE as u16;
        if delta > 0 {
            base_addr.wrapping_add(delta.unsigned_abs())
        } else {
            base_addr.wrapping_sub(delta.unsigned_abs())
        }
    }

    fn format(cpu: &Cpu, bus: &Bus, addr: u16) -> String {
        let relative_addr = bus.read_u8(addr + 1) as i8;
        return format!(
            " {:+02X} ={:04X} @ {:02X}",
            relative_addr,
            Self::addr(cpu, bus, addr),
            Self::load(cpu, bus, addr)
        );
    }
}

struct IndirectY {}
impl AddressModeImpl for IndirectY {
    const OPERAND_SIZE: usize = 1;

    fn addr(cpu: &Cpu, bus: &Bus, addr: u16) -> u16 {
        bus.read_u16(bus.read_u8(addr + 1) as u16) as u16 + cpu.y as u16
    }

    fn format(cpu: &Cpu, bus: &Bus, addr: u16) -> String {
        let indirect_addr = bus.read_u8(addr + 1);
        let indirect = bus.read_u16(indirect_addr as u16);
        return format!(
            " (${:02X})={:04X}+Y ={:04X} @ {:02X}",
            indirect_addr,
            indirect,
            Self::addr(cpu, bus, addr),
            Self::load(cpu, bus, addr)
        );
    }
}

trait AddressModeImpl {
    const OPERAND_SIZE: usize = 0;

    fn addr(cpu: &Cpu, bus: &Bus, addr: u16) -> u16;
    fn format(cpu: &Cpu, bus: &Bus, addr: u16) -> String;

    fn load(cpu: &Cpu, bus: &Bus, addr: u16) -> u8 {
        bus.read_u8(Self::addr(cpu, bus, addr))
    }

    fn store(cpu: &Cpu, bus: &mut Bus, addr: u16, value: u8) {
        bus.write_u8(Self::addr(cpu, bus, addr), value)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Operation Implementations

// J** (Jump)

fn jmp<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.program_counter = ctx.operand_addr();
}

fn jsr<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    let bytes = ctx.cpu.program_counter.to_le_bytes();
    ctx.cpu.stack_push(ctx.bus, bytes[1]);
    ctx.cpu.stack_push(ctx.bus, bytes[0]);
    ctx.cpu.program_counter = ctx.operand_addr();
}

fn rts<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    let bytes = [ctx.cpu.stack_pop(ctx.bus), ctx.cpu.stack_pop(ctx.bus)];
    ctx.cpu.program_counter = u16::from_le_bytes(bytes);
}

// ST* (Store)

fn sta<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.store_operand(ctx.cpu.a);
}

fn stx<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.store_operand(ctx.cpu.x);
}

fn sty<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.store_operand(ctx.cpu.y);
}

// LD* (Load)

fn lda<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.a = ctx.load_operand();
    ctx.update_negative_zero_flags(ctx.cpu.a);
}

fn ldy<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.y = ctx.load_operand();
    ctx.update_negative_zero_flags(ctx.cpu.y);
}

fn ldx<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.x = ctx.load_operand();
    ctx.update_negative_zero_flags(ctx.cpu.x);
}

// IN* (Increment)

fn inc<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    let value = ctx.load_operand() + 1;
    ctx.store_operand(value);
    ctx.update_negative_zero_flags(value);
}

fn inx<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.x = ctx.cpu.x.wrapping_add(1);
    ctx.update_negative_zero_flags(ctx.cpu.x);
}

fn iny<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.y = ctx.cpu.y.wrapping_add(1);
    ctx.update_negative_zero_flags(ctx.cpu.y);
}

// DE* (Decrement)

fn dec<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    let value = ctx.load_operand() - 1;
    ctx.store_operand(value);
    ctx.update_negative_zero_flags(value);
}

fn dex<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.x = ctx.cpu.x.wrapping_sub(1);
    ctx.update_negative_zero_flags(ctx.cpu.x);
}

fn dey<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.y = ctx.cpu.y.wrapping_sub(1);
    ctx.update_negative_zero_flags(ctx.cpu.y);
}

// SE* / CL* (Set / clear status bits)

fn sec<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.status_flags.insert(StatusFlags::CARRY);
}

fn clc<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.status_flags.remove(StatusFlags::CARRY);
}

fn cld<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.status_flags.remove(StatusFlags::DECIMAL);
}

// B** (Branch)

fn bcs<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    if ctx.cpu.status_flags.contains(StatusFlags::CARRY) {
        ctx.cpu.program_counter = ctx.operand_addr();
    }
}

fn bcc<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    if !ctx.cpu.status_flags.contains(StatusFlags::CARRY) {
        ctx.cpu.program_counter = ctx.operand_addr();
    }
}

fn beq<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    if ctx.cpu.status_flags.contains(StatusFlags::ZERO) {
        ctx.cpu.program_counter = ctx.operand_addr();
    }
}

fn bne<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    if !ctx.cpu.status_flags.contains(StatusFlags::ZERO) {
        ctx.cpu.program_counter = ctx.operand_addr();
    }
}

fn bmi<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    if ctx.cpu.status_flags.contains(StatusFlags::NEGATIVE) {
        ctx.cpu.program_counter = ctx.operand_addr();
    }
}

fn bpl<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    if !ctx.cpu.status_flags.contains(StatusFlags::NEGATIVE) {
        ctx.cpu.program_counter = ctx.operand_addr();
    }
}

// misc

fn bit<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    let value = ctx.load_operand();
    ctx.cpu.status_flags.set(
        StatusFlags::NEGATIVE,
        (value & StatusFlags::NEGATIVE.bits) > 0,
    );
    ctx.cpu.status_flags.set(
        StatusFlags::OVERFLOW,
        (value & StatusFlags::OVERFLOW.bits) > 0,
    );
    ctx.cpu
        .status_flags
        .set(StatusFlags::ZERO, (value & ctx.cpu.a) == 0);
}

fn hlt<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.halt = true;
}

fn adc<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.a += ctx.load_operand();
}

fn tax<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.x = ctx.cpu.a;
    ctx.update_negative_zero_flags(ctx.cpu.x);
}

fn tsx<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.x = ctx.cpu.sp;
    ctx.update_negative_zero_flags(ctx.cpu.x);
}

fn txs<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.sp = ctx.cpu.x;
}

fn sei<AM: AddressModeImpl>(_ctx: &mut Context<AM>) {}

fn nop<AM: AddressModeImpl>(_ctx: &mut Context<AM>) {}
