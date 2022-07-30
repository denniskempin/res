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
            // Codes ending in 0
            opcode!(0x00, hlt, Implicit),  // TODO: It's BRK not HLT
            opcode!(0x10, bpl, Relative),
            opcode!(0x20, jsr, Absolute),
            opcode!(0x30, bmi, Relative),
            opcode!(0x40, rti, Implicit),
            opcode!(0x50, bvc, Relative),
            opcode!(0x60, rts, Implicit),
            opcode!(0x70, bvs, Relative),
            opcode!(0x80, nop, Implicit),
            opcode!(0x90, bcc, Relative),
            opcode!(0xA0, ldy, Immediate),
            opcode!(0xB0, bcs, Relative),
            opcode!(0xC0, cpy, Immediate),
            opcode!(0xD0, bne, Relative),
            opcode!(0xE0, cpx, Immediate),
            opcode!(0xF0, beq, Relative),

            // Codes ending in 1
            opcode!(0x01, ora, IndirectX),
            opcode!(0x11, ora, IndirectY),
            opcode!(0x21, and, IndirectX),
            opcode!(0x31, and, IndirectY),
            opcode!(0x41, eor, IndirectX),
            opcode!(0x51, eor, IndirectY),
            opcode!(0x61, adc, IndirectX),
            opcode!(0x71, adc, IndirectY),
            opcode!(0x81, sta, IndirectX),
            opcode!(0x91, sta, IndirectY),
            opcode!(0xA1, lda, IndirectX),
            opcode!(0xB1, lda, IndirectY),
            opcode!(0xC1, cmp, IndirectX),
            opcode!(0xD1, cmp, IndirectY),
            opcode!(0xE1, sbc, IndirectX),
            opcode!(0xF1, sbc, IndirectY),

            // Codes ending in 2
            opcode!(0xA2, ldx, Immediate),

            // Codes ending in 3
            // ... only illegal opcodes

            // Codes ending in 4
            opcode!(0x24, bit, ZeroPage),
            opcode!(0x84, sty, ZeroPage),
            opcode!(0x94, sty, ZeroPageX),
            opcode!(0xA4, ldy, ZeroPage),
            opcode!(0xB4, ldy, ZeroPageX),
            opcode!(0xC4, cpy, ZeroPage),
            opcode!(0xE4, cpx, ZeroPage),

            // Codes ending in 5
            opcode!(0x05, ora, ZeroPage),
            opcode!(0x15, ora, ZeroPageX),
            opcode!(0x25, and, ZeroPage),
            opcode!(0x35, and, ZeroPageX),
            opcode!(0x45, eor, ZeroPage),
            opcode!(0x55, eor, ZeroPageX),
            opcode!(0x65, adc, ZeroPage),
            opcode!(0x75, adc, ZeroPageX),
            opcode!(0x85, sta, ZeroPage),
            opcode!(0x95, sta, ZeroPageX),
            opcode!(0xA5, lda, ZeroPage),
            opcode!(0xB5, lda, ZeroPageX),
            opcode!(0xC5, cmp, ZeroPage),
            opcode!(0xD5, cmp, ZeroPageX),
            opcode!(0xE5, sbc, ZeroPage),
            opcode!(0xF5, sbc, ZeroPageX),

            // Codes ending in 6
            opcode!(0x06, asl, ZeroPage),
            opcode!(0x16, asl, ZeroPageX),
            opcode!(0x26, rol, ZeroPage),
            opcode!(0x36, rol, ZeroPageX),
            opcode!(0x46, lsr, ZeroPage),
            opcode!(0x56, lsr, ZeroPageX),
            opcode!(0x66, ror, ZeroPage),
            opcode!(0x76, ror, ZeroPageX),
            opcode!(0x86, stx, ZeroPage),
            opcode!(0x96, stx, ZeroPageX),
            opcode!(0xA6, ldx, ZeroPage),
            opcode!(0xB6, ldx, ZeroPageY),
            opcode!(0xC6, dec, ZeroPage),
            opcode!(0xD6, dec, ZeroPageY),
            opcode!(0xE6, inc, ZeroPage),
            opcode!(0xF6, inc, ZeroPageY),

            // Codes ending in 7
            // ... only illegal opcodes

            // Codes ending in 8
            opcode!(0x08, php, Implicit),
            opcode!(0x18, clc, Implicit),
            opcode!(0x28, plp, Implicit),
            opcode!(0x38, sec, Implicit),
            opcode!(0x48, pha, Implicit),
            opcode!(0x58, cli, Implicit),
            opcode!(0x68, pla, Implicit),
            opcode!(0x78, sei, Implicit),
            opcode!(0x88, dey, Implicit),
            opcode!(0x98, tya, Implicit),
            opcode!(0xA8, tay, Implicit),
            opcode!(0xB8, clv, Implicit),
            opcode!(0xC8, iny, Implicit),
            opcode!(0xD8, cld, Implicit),
            opcode!(0xE8, inx, Implicit),
            opcode!(0xF8, sed, Implicit),

            // Codes ending in 9
            opcode!(0x09, ora, Immediate),
            opcode!(0x19, ora, AbsoluteY),
            opcode!(0x29, and, Immediate),
            opcode!(0x39, and, AbsoluteY),
            opcode!(0x49, eor, Immediate),
            opcode!(0x59, eor, AbsoluteY),
            opcode!(0x69, adc, Immediate),
            opcode!(0x79, adc, AbsoluteY),
            opcode!(0x89, nop, Immediate),
            opcode!(0x99, sta, AbsoluteY),
            opcode!(0xA9, lda, Immediate),
            opcode!(0xB9, lda, AbsoluteY),
            opcode!(0xC9, cmp, Immediate),
            opcode!(0xD9, cmp, AbsoluteY),
            opcode!(0xE9, sbc, Immediate),
            opcode!(0xF9, sbc, AbsoluteY),

            // Codes ending in A
            opcode!(0x0A, asl, Acumulator),
            opcode!(0x1A, nop, Implicit),
            opcode!(0x2A, rol, Acumulator),
            opcode!(0x3A, nop, Implicit),
            opcode!(0x4A, lsr, Acumulator),
            opcode!(0x5A, nop, Implicit),
            opcode!(0x6A, ror, Acumulator),
            opcode!(0x7A, nop, Implicit),
            opcode!(0x8A, txa, Implicit),
            opcode!(0x9A, txs, Implicit),
            opcode!(0xAA, tax, Implicit),
            opcode!(0xBA, tsx, Implicit),
            opcode!(0xCA, dex, Implicit),
            opcode!(0xDA, nop, Implicit),
            opcode!(0xEA, nop, Implicit),
            opcode!(0xFA, nop, Implicit),

            // Codes ending in B
            // ... only illegal opcodes

            // Codes ending in C
            opcode!(0x2C, bit, Absolute),
            opcode!(0x4C, jmp, Absolute),
            opcode!(0x6C, jmp, Indirect),
            opcode!(0x8C, sty, Absolute),
            opcode!(0xAC, ldy, Absolute),
            opcode!(0xBC, ldy, AbsoluteX),
            opcode!(0xCC, cpy, Absolute),
            opcode!(0xEC, cpx, Absolute),


            //
            opcode!(0x0D, ora, Absolute),
            opcode!(0x1D, ora, AbsoluteX),
            opcode!(0x2D, and, Absolute),
            opcode!(0x3D, and, AbsoluteX),
            opcode!(0x4D, eor, Absolute),
            opcode!(0x5D, eor, AbsoluteX),
            opcode!(0x6D, adc, Absolute),
            opcode!(0x7D, adc, AbsoluteX),
            opcode!(0x8D, sta, Absolute),
            opcode!(0x9D, sta, AbsoluteX),
            opcode!(0xAD, lda, Absolute),
            opcode!(0xBD, lda, AbsoluteX),
            opcode!(0xCD, cmp, Absolute),
            opcode!(0xDD, cmp, AbsoluteX),
            opcode!(0xED, sbc, Absolute),
            opcode!(0xFD, sbc, AbsoluteX),

            //
            opcode!(0x0E, asl, Absolute),
            opcode!(0x1E, asl, AbsoluteX),
            opcode!(0x2E, rol, Absolute),
            opcode!(0x3E, rol, AbsoluteX),
            opcode!(0x4E, lsr, Absolute),
            opcode!(0x5E, lsr, AbsoluteX),
            opcode!(0x6E, ror, Absolute),
            opcode!(0x7E, ror, AbsoluteX),
            opcode!(0x8E, stx, Absolute),
            opcode!(0x9E, nop, AbsoluteX),
            opcode!(0xAE, ldx, Absolute),
            opcode!(0xBE, ldx, AbsoluteY),
            opcode!(0xCE, dec, Absolute),
            opcode!(0xDE, dec, AbsoluteY),
            opcode!(0xEE, inc, Absolute),
            opcode!(0xFE, inc, AbsoluteY),
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

    fn store(_cpu: &mut Cpu, _bus: &mut Bus, _addr: u16, _value: u8) {
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

    fn store(_cpu: &mut Cpu, _bus: &mut Bus, _addr: u16, _value: u8) {
        unimplemented!();
    }

    fn format(_cpu: &Cpu, _bus: &Bus, _addr: u16) -> String {
        "".to_string()
    }
}

struct Acumulator {}
impl AddressModeImpl for Acumulator {
    const OPERAND_SIZE: usize = 0;

    fn addr(_cpu: &Cpu, _bus: &Bus, _addr: u16) -> u16 {
        unimplemented!();
    }

    fn load(cpu: &Cpu, _bus: &Bus, _addr: u16) -> u8 {
        cpu.a
    }

    fn store(cpu: &mut Cpu, _bus: &mut Bus, _addr: u16, value: u8) {
        cpu.a = value;
    }

    fn format(_cpu: &Cpu, _bus: &Bus, _addr: u16) -> String {
        " A".to_string()
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

struct Indirect {}
impl AddressModeImpl for Indirect {
    const OPERAND_SIZE: usize = 2;

    fn addr(cpu: &Cpu, bus: &Bus, addr: u16) -> u16 {
        bus.read_u16(bus.read_u16(addr + 1))
    }

    fn format(cpu: &Cpu, bus: &Bus, addr: u16) -> String {
        let indirect_addr = bus.read_u16(addr + 1);
        return format!(
            " (${:04X}) ={:04X} @ {:02X}",
            indirect_addr,
            Self::addr(cpu, bus, addr),
            Self::load(cpu, bus, addr)
        );
    }
}

struct IndirectY {}
impl AddressModeImpl for IndirectY {
    const OPERAND_SIZE: usize = 1;

    fn addr(cpu: &Cpu, bus: &Bus, addr: u16) -> u16 {
        // Note: Zero-page address lookup will wrap around from 0xFF to 0x00.
        // TODO: Implement zero page u16 reads in bus
        let indirect_addr = bus.read_u8(addr + 1);
        let bytes = [
            bus.read_u8(indirect_addr as u16),
            bus.read_u8(indirect_addr.wrapping_add(1) as u16),
        ];
        u16::from_le_bytes(bytes).wrapping_add(cpu.y as u16)
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

struct IndirectX {}
impl AddressModeImpl for IndirectX {
    const OPERAND_SIZE: usize = 1;

    fn addr(cpu: &Cpu, bus: &Bus, addr: u16) -> u16 {
        // Note: Zero-page address lookup will wrap around from 0xFF to 0x00.
        // TODO: Implement zero page u16 reads in bus
        let indirect_addr = bus.read_u8(addr + 1).wrapping_add(cpu.x);
        let bytes = [
            bus.read_u8(indirect_addr as u16),
            bus.read_u8(indirect_addr.wrapping_add(1) as u16),
        ];
        u16::from_le_bytes(bytes)
    }

    fn format(cpu: &Cpu, bus: &Bus, addr: u16) -> String {
        let indirect_addr = bus.read_u8(addr + 1);
        return format!(
            " (${:02X}+X) ={:04X} @{:02X}",
            indirect_addr,
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

    fn store(cpu: &mut Cpu, bus: &mut Bus, addr: u16, value: u8) {
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
    let bytes = (ctx.cpu.program_counter - 1).to_le_bytes();
    ctx.cpu.stack_push(ctx.bus, bytes[1]);
    ctx.cpu.stack_push(ctx.bus, bytes[0]);
    ctx.cpu.program_counter = ctx.operand_addr();
}

fn rts<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    let bytes = [ctx.cpu.stack_pop(ctx.bus), ctx.cpu.stack_pop(ctx.bus)];
    // We are reading the address back in inverse order, hence big endian.
    ctx.cpu.program_counter = u16::from_le_bytes(bytes) + 1;
}

fn rti<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    plp(ctx);
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
    let value = ctx.load_operand().wrapping_add(1);
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
    let value = ctx.load_operand().wrapping_sub(1);
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

fn sed<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.status_flags.insert(StatusFlags::DECIMAL);
}

fn cld<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.status_flags.remove(StatusFlags::DECIMAL);
}

fn sec<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.status_flags.insert(StatusFlags::CARRY);
}

fn clc<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.status_flags.remove(StatusFlags::CARRY);
}

fn clv<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.status_flags.remove(StatusFlags::OVERFLOW);
}

fn cli<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.status_flags.remove(StatusFlags::INTERRUPT);
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

fn bvs<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    if ctx.cpu.status_flags.contains(StatusFlags::OVERFLOW) {
        ctx.cpu.program_counter = ctx.operand_addr();
    }
}

fn bvc<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    if !ctx.cpu.status_flags.contains(StatusFlags::OVERFLOW) {
        ctx.cpu.program_counter = ctx.operand_addr();
    }
}

// PH* (Push), PL* (Pull)

fn pha<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.stack_push(ctx.bus, ctx.cpu.a);
}

fn php<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    let mut value = ctx.cpu.status_flags;
    value.insert(StatusFlags::BREAK);
    ctx.cpu.stack_push(ctx.bus, value.bits);
}

fn pla<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.a = ctx.cpu.stack_pop(ctx.bus);
    ctx.update_negative_zero_flags(ctx.cpu.a);
}

fn plp<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    let mut value = StatusFlags::from_bits_truncate(ctx.cpu.stack_pop(ctx.bus));
    value.set(
        StatusFlags::BREAK,
        ctx.cpu.status_flags.contains(StatusFlags::BREAK),
    );
    value.insert(StatusFlags::UNUSED);
    ctx.cpu.status_flags = value;
}

// Basic Arithmetic

fn adc<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    let carry: u16 = if ctx.cpu.status_flags.contains(StatusFlags::CARRY) {
        1
    } else {
        0
    };
    let operand = ctx.load_operand();
    let result = ctx.cpu.a as u16 + operand as u16 + carry;

    ctx.cpu.status_flags.set(StatusFlags::CARRY, result > 0xFF);
    ctx.cpu.status_flags.set(
        StatusFlags::OVERFLOW,
        (operand ^ result as u8) & (result as u8 ^ ctx.cpu.a) & 0x80 != 0,
    );
    ctx.cpu.a = result as u8;
    ctx.update_negative_zero_flags(ctx.cpu.a);
}

fn sbc<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    let carry: u16 = if ctx.cpu.status_flags.contains(StatusFlags::CARRY) {
        1
    } else {
        0
    };
    let operand = (ctx.load_operand() as i8).wrapping_neg().wrapping_sub(1) as u8;
    let result = ctx.cpu.a as u16 + operand as u16 + carry;

    ctx.cpu.status_flags.set(StatusFlags::CARRY, result > 0xFF);
    ctx.cpu.status_flags.set(
        StatusFlags::OVERFLOW,
        (operand ^ result as u8) & (result as u8 ^ ctx.cpu.a) & 0x80 != 0,
    );
    ctx.cpu.a = result as u8;
    ctx.update_negative_zero_flags(ctx.cpu.a);
}

fn and<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.a = ctx.cpu.a & ctx.load_operand();
    ctx.update_negative_zero_flags(ctx.cpu.a);
}

fn ora<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.a = ctx.cpu.a | ctx.load_operand();
    ctx.update_negative_zero_flags(ctx.cpu.a);
}

fn eor<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.a = ctx.cpu.a ^ ctx.load_operand();
    ctx.update_negative_zero_flags(ctx.cpu.a);
}

fn cmp<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    let (value, overflow) = ctx.cpu.a.overflowing_sub(ctx.load_operand());
    ctx.update_negative_zero_flags(value);
    ctx.cpu.status_flags.set(StatusFlags::CARRY, !overflow);
}

fn cpx<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    let (value, overflow) = ctx.cpu.x.overflowing_sub(ctx.load_operand());
    ctx.update_negative_zero_flags(value);
    ctx.cpu.status_flags.set(StatusFlags::CARRY, !overflow);
}

fn cpy<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    let (value, overflow) = ctx.cpu.y.overflowing_sub(ctx.load_operand());
    ctx.update_negative_zero_flags(value);
    ctx.cpu.status_flags.set(StatusFlags::CARRY, !overflow);
}

fn lsr<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    let operand = ctx.load_operand();
    let (result, _) = operand.overflowing_shr(1);
    ctx.store_operand(result);
    ctx.update_negative_zero_flags(result);
    ctx.cpu
        .status_flags
        .set(StatusFlags::CARRY, (operand & 0x01) != 0);
}

fn asl<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    let operand = ctx.load_operand();
    let (result, _) = operand.overflowing_shl(1);
    ctx.store_operand(result);
    ctx.update_negative_zero_flags(result);
    ctx.cpu
        .status_flags
        .set(StatusFlags::CARRY, (operand & 0x80) != 0);
}

fn ror<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    let operand = ctx.load_operand();
    let (mut result, _) = operand.overflowing_shr(1);
    if ctx.cpu.status_flags.contains(StatusFlags::CARRY) {
        result |= 0b1000_0000;
    }
    ctx.store_operand(result);
    ctx.update_negative_zero_flags(result);
    ctx.cpu
        .status_flags
        .set(StatusFlags::CARRY, (operand & 0x01) != 0);
}

fn rol<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    let operand = ctx.load_operand();
    let (mut result, _) = operand.overflowing_shl(1);
    if ctx.cpu.status_flags.contains(StatusFlags::CARRY) {
        result |= 0b0000_0001;
    }
    ctx.store_operand(result);
    ctx.update_negative_zero_flags(result);
    ctx.cpu
        .status_flags
        .set(StatusFlags::CARRY, (operand & 0x80) != 0);
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

fn txa<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.a = ctx.cpu.x;
    ctx.update_negative_zero_flags(ctx.cpu.a);
}

fn tax<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.x = ctx.cpu.a;
    ctx.update_negative_zero_flags(ctx.cpu.x);
}

fn tay<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.y = ctx.cpu.a;
    ctx.update_negative_zero_flags(ctx.cpu.y);
}

fn tya<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.a = ctx.cpu.y;
    ctx.update_negative_zero_flags(ctx.cpu.a);
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
