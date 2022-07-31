use anyhow::Result;
use konst::eq_str;

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

    pub fn is_legal(&self) -> bool {
        self.table_entry.legal
    }
}

////////////////////////////////////////////////////////////////////////////////
// OpCode Table

macro_rules! opcode {
    ($code: literal, $method: ident, $address_mode: ident) => {
        OpCodeTableEntry {
            code: $code,
            legal: is_legal($code, stringify!($method)),
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

// TODOS:
// 0x00 is BRK not HLT

lazy_static! {
    static ref OPCODE_TABLE: [OpCodeTableEntry; 0x100] = {
        // Specify opcodes out of order to better organize them.
        const OPCODE_LIST: &[OpCodeTableEntry] = &[
            // Codes ending in 0
            opcode!(0x00, hlt, Implicit),
            opcode!(0x10, bpl, Relative),
            opcode!(0x20, jsr, Absolute),
            opcode!(0x30, bmi, Relative),
            opcode!(0x40, rti, Implicit),
            opcode!(0x50, bvc, Relative),
            opcode!(0x60, rts, Implicit),
            opcode!(0x70, bvs, Relative),
            opcode!(0x80, nop, Immediate),
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
            opcode!(0x02, ill, Implicit),
            opcode!(0x12, ill, Implicit),
            opcode!(0x22, ill, Implicit),
            opcode!(0x32, ill, Implicit),
            opcode!(0x42, ill, Implicit),
            opcode!(0x52, ill, Implicit),
            opcode!(0x62, ill, Implicit),
            opcode!(0x72, ill, Implicit),
            opcode!(0x82, nop, Immediate),
            opcode!(0x92, ill, Implicit),
            opcode!(0xA2, ldx, Immediate),
            opcode!(0xB2, ill, Implicit),
            opcode!(0xC2, nop, Immediate),
            opcode!(0xD2, ill, Implicit),
            opcode!(0xE2, nop, Immediate),
            opcode!(0xF2, ill, Implicit),

            // Codes ending in 3
            opcode!(0x03, slo, IndirectX),
            opcode!(0x13, slo, IndirectY),
            opcode!(0x23, rla, IndirectX),
            opcode!(0x33, rla, IndirectY),
            opcode!(0x43, sre, IndirectX),
            opcode!(0x53, sre, IndirectY),
            opcode!(0x63, rra, IndirectX),
            opcode!(0x73, rra, IndirectY),
            opcode!(0x83, sax, IndirectX),
            opcode!(0x93, sha, IndirectY),
            opcode!(0xA3, lax, IndirectX),
            opcode!(0xB3, lax, IndirectY),
            opcode!(0xC3, dcp, IndirectX),
            opcode!(0xD3, dcp, IndirectY),
            opcode!(0xE3, isc, IndirectX),
            opcode!(0xF3, isc, IndirectY),

            // Codes ending in 4
            opcode!(0x04, nop, ZeroPage),
            opcode!(0x14, nop, ZeroPageX),
            opcode!(0x24, bit, ZeroPage),
            opcode!(0x34, nop, ZeroPageX),
            opcode!(0x44, nop, ZeroPage),
            opcode!(0x54, nop, ZeroPageX),
            opcode!(0x64, nop, ZeroPage),
            opcode!(0x74, nop, ZeroPageX),
            opcode!(0x84, sty, ZeroPage),
            opcode!(0x94, sty, ZeroPageX),
            opcode!(0xA4, ldy, ZeroPage),
            opcode!(0xB4, ldy, ZeroPageX),
            opcode!(0xC4, cpy, ZeroPage),
            opcode!(0xD4, nop, ZeroPageX),
            opcode!(0xE4, cpx, ZeroPage),
            opcode!(0xF4, nop, ZeroPageX),

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
            opcode!(0x96, stx, ZeroPageY),
            opcode!(0xA6, ldx, ZeroPage),
            opcode!(0xB6, ldx, ZeroPageY),
            opcode!(0xC6, dec, ZeroPage),
            opcode!(0xD6, dec, ZeroPageX),
            opcode!(0xE6, inc, ZeroPage),
            opcode!(0xF6, inc, ZeroPageX),

            // Codes ending in 7
            opcode!(0x07, slo, ZeroPage),
            opcode!(0x17, slo, ZeroPageX),
            opcode!(0x27, rla, ZeroPage),
            opcode!(0x37, rla, ZeroPageX),
            opcode!(0x47, sre, ZeroPage),
            opcode!(0x57, sre, ZeroPageX),
            opcode!(0x67, rra, ZeroPage),
            opcode!(0x77, rra, ZeroPageX),
            opcode!(0x87, sax, ZeroPage),
            opcode!(0x97, sax, ZeroPageY),
            opcode!(0xA7, lax, ZeroPage),
            opcode!(0xB7, lax, ZeroPageY),
            opcode!(0xC7, dcp, ZeroPage),
            opcode!(0xD7, dcp, ZeroPageX),
            opcode!(0xE7, isc, ZeroPage),
            opcode!(0xF7, isc, ZeroPageX),

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
            opcode!(0x0B, anc, Immediate),
            opcode!(0x1B, slo, AbsoluteY),
            opcode!(0x2B, anc, Immediate),
            opcode!(0x3B, rla, AbsoluteY),
            opcode!(0x4B, alr, Immediate),
            opcode!(0x5B, sre, AbsoluteY),
            opcode!(0x6B, arr, Immediate),
            opcode!(0x7B, rra, AbsoluteY),
            opcode!(0x8B, ane, Immediate),
            opcode!(0x9B, tas, AbsoluteY),
            opcode!(0xAB, lxa, Immediate),
            opcode!(0xBB, las, AbsoluteY),
            opcode!(0xCB, sbx, Immediate),
            opcode!(0xDB, dcp, AbsoluteY),
            opcode!(0xEB, sbc, Immediate),
            opcode!(0xFB, isc, AbsoluteY),

            // Codes ending in C
            opcode!(0x0C, nop, Absolute),
            opcode!(0x1C, nop, AbsoluteX),
            opcode!(0x2C, bit, Absolute),
            opcode!(0x3C, nop, AbsoluteX),
            opcode!(0x4C, jmp, Absolute),
            opcode!(0x5C, nop, AbsoluteX),
            opcode!(0x6C, jmp, Indirect),
            opcode!(0x7C, nop, AbsoluteX),
            opcode!(0x8C, sty, Absolute),
            opcode!(0x9C, ill, AbsoluteX),
            opcode!(0xAC, ldy, Absolute),
            opcode!(0xBC, ldy, AbsoluteX),
            opcode!(0xCC, cpy, Absolute),
            opcode!(0xDC, nop, AbsoluteX),
            opcode!(0xEC, cpx, Absolute),
            opcode!(0xFC, nop, AbsoluteX),


            // Codes ending in D
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

            // Codes endding in E
            opcode!(0x0E, asl, Absolute),
            opcode!(0x1E, asl, AbsoluteX),
            opcode!(0x2E, rol, Absolute),
            opcode!(0x3E, rol, AbsoluteX),
            opcode!(0x4E, lsr, Absolute),
            opcode!(0x5E, lsr, AbsoluteX),
            opcode!(0x6E, ror, Absolute),
            opcode!(0x7E, ror, AbsoluteX),
            opcode!(0x8E, stx, Absolute),
            opcode!(0x9E, ill, AbsoluteX),
            opcode!(0xAE, ldx, Absolute),
            opcode!(0xBE, ldx, AbsoluteY),
            opcode!(0xCE, dec, Absolute),
            opcode!(0xDE, dec, AbsoluteX),
            opcode!(0xEE, inc, Absolute),
            opcode!(0xFE, inc, AbsoluteX),

            // Codes endding in F
            opcode!(0x0F, slo, Absolute),
            opcode!(0x1F, slo, AbsoluteX),
            opcode!(0x2F, rla, Absolute),
            opcode!(0x3F, rla, AbsoluteX),
            opcode!(0x4F, sre, Absolute),
            opcode!(0x5F, sre, AbsoluteX),
            opcode!(0x6F, rra, Absolute),
            opcode!(0x7F, rra, AbsoluteX),
            opcode!(0x8F, sax, Absolute),
            opcode!(0x9F, sha, AbsoluteX),
            opcode!(0xAF, lax, Absolute),
            opcode!(0xBF, lax, AbsoluteY),
            opcode!(0xCF, dcp, Absolute),
            opcode!(0xDF, dcp, AbsoluteX),
            opcode!(0xEF, isc, Absolute),
            opcode!(0xFF, isc, AbsoluteX),
        ];

        // Turn list of codes into opcode lookup table
        let mut table = [OpCodeTableEntry::default(); 0x100];
        for entry in OPCODE_LIST {
            table[entry.code as usize] = *entry;
        }
        // Verify all opcodes are specified
        for (i, entry) in table.iter().enumerate() {
            assert!(i == entry.code as usize);
        }
        table
    };
}

#[derive(Copy, Clone)]
struct OpCodeTableEntry {
    pub code: u8,
    pub legal: bool,
    pub operand_size: usize,
    pub execute_fn: fn(cpu: &mut Cpu, bus: &mut Bus, addr: u16),
    pub format_fn: fn(cpu: &Cpu, bus: &Bus, addr: u16) -> String,
}

impl Default for OpCodeTableEntry {
    fn default() -> Self {
        Self {
            code: Default::default(),
            legal: false,
            operand_size: 0,
            execute_fn: |_, _, _| unimplemented!(),
            format_fn: |_, _, _| "N/A".to_string(),
        }
    }
}

const fn is_legal(code: u8, method: &str) -> bool {
    match method {
        _ if eq_str(method, "nop") => code == 0xEA,
        _ if eq_str(method, "ill") => false,
        _ if eq_str(method, "sax") => false,
        _ if eq_str(method, "dcp") => false,
        _ if eq_str(method, "lax") => false,
        _ if eq_str(method, "isc") => false,
        _ if eq_str(method, "sla") => false,
        _ if eq_str(method, "slo") => false,
        _ if eq_str(method, "rla") => false,
        _ if eq_str(method, "sre") => false,
        _ if eq_str(method, "rra") => false,
        _ if eq_str(method, "sha") => false,
        _ if eq_str(method, "anc") => false,
        _ if eq_str(method, "alr") => false,
        _ => code != 0xEB,
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
        bus.read_u16(addr + 1).wrapping_add(cpu.y as u16)
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
        bus.read_u8(addr + 1).wrapping_add(cpu.x) as u16
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
        bus.read_u8(addr + 1).wrapping_add(cpu.y) as u16
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

    fn addr(_cpu: &Cpu, bus: &Bus, addr: u16) -> u16 {
        let indirect_addr = bus.read_u16(addr + 1);
        let bytes = if indirect_addr & 0x00FF == 0x00FF {
            // CPU Bug: Address wraps around inside page.
            let page = indirect_addr & 0xFF00;
            [bus.read_u8(indirect_addr), bus.read_u8(page)]
        } else {
            [bus.read_u8(indirect_addr), bus.read_u8(indirect_addr + 1)]
        };
        u16::from_le_bytes(bytes)
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

// J** (Jump) / RT* (Return)

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

// add / sub

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

// Bit-wise operations

fn and<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.a &= ctx.load_operand();
    ctx.update_negative_zero_flags(ctx.cpu.a);
}

fn ora<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.a |= ctx.load_operand();
    ctx.update_negative_zero_flags(ctx.cpu.a);
}

fn eor<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.cpu.a ^= ctx.load_operand();
    ctx.update_negative_zero_flags(ctx.cpu.a);
}

// C** (Compare)

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

// Shifts

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

// Register Transfers

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

// Misc Operations

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

fn sei<AM: AddressModeImpl>(_ctx: &mut Context<AM>) {}

fn nop<AM: AddressModeImpl>(_ctx: &mut Context<AM>) {}

// Illegal Instructions

fn ill<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    panic!("Illegal Opcode {:02X}", ctx.bus.read_u8(ctx.addr));
}

fn lax<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    lda(ctx);
    ldx(ctx);
}

fn sax<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ctx.store_operand(ctx.cpu.a & ctx.cpu.x);
}

fn dcp<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    dec(ctx);
    cmp(ctx);
}

fn isc<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    inc(ctx);
    sbc(ctx);
}

fn slo<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    asl(ctx);
    ora(ctx);
}

fn rla<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    rol(ctx);
    and(ctx);
}

fn rra<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    ror(ctx);
    adc(ctx);
}

fn sre<AM: AddressModeImpl>(ctx: &mut Context<AM>) {
    lsr(ctx);
    eor(ctx);
}

fn sha<AM: AddressModeImpl>(_ctx: &mut Context<AM>) {
    unimplemented!();
}

fn alr<AM: AddressModeImpl>(_ctx: &mut Context<AM>) {
    unimplemented!();
}

fn anc<AM: AddressModeImpl>(_ctx: &mut Context<AM>) {
    unimplemented!();
}

fn arr<AM: AddressModeImpl>(_ctx: &mut Context<AM>) {
    unimplemented!();
}

fn ane<AM: AddressModeImpl>(_ctx: &mut Context<AM>) {
    unimplemented!();
}

fn tas<AM: AddressModeImpl>(_ctx: &mut Context<AM>) {
    unimplemented!();
}

fn lxa<AM: AddressModeImpl>(_ctx: &mut Context<AM>) {
    unimplemented!();
}

fn las<AM: AddressModeImpl>(_ctx: &mut Context<AM>) {
    unimplemented!();
}

fn sbx<AM: AddressModeImpl>(_ctx: &mut Context<AM>) {
    unimplemented!();
}
