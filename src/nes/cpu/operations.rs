use anyhow::Result;

use super::Cpu;
use super::ReadOrPeek;
use super::StatusFlags;

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

    pub fn peek(cpu: &Cpu, addr: u16) -> Result<Operation> {
        let raw_opcode = cpu.bus.peek(addr);
        let table_entry = OPCODE_TABLE[raw_opcode as usize];
        Ok(Operation { addr, table_entry })
    }

    pub fn load(cpu: &mut Cpu, addr: u16) -> Result<Operation> {
        let raw_opcode = cpu.bus.read(addr);
        let table_entry = OPCODE_TABLE[raw_opcode as usize];
        Ok(Operation { addr, table_entry })
    }

    pub fn execute(&self, cpu: &mut Cpu) {
        (self.table_entry.execute_fn)(cpu, self.addr);
    }

    pub fn format(self, cpu: &Cpu) -> String {
        (self.table_entry.format_fn)(cpu, self.addr)
    }
}

////////////////////////////////////////////////////////////////////////////////
// OpCode Table

macro_rules! opcode {
    ($code: literal, $method: ident, $address_mode: ident) => {
        OpCodeTableEntry {
            code: $code,
            operand_size: $address_mode::OPERAND_SIZE,
            execute_fn: |cpu, addr| $method::<$address_mode>(cpu, addr),
            format_fn: |cpu, addr| {
                format!(
                    "{}{}",
                    stringify!($method).to_uppercase(),
                    $address_mode::format(cpu, addr)
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
            opcode!(0x80, ill, Immediate),
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
            opcode!(0x82, ill, Immediate),
            opcode!(0x92, ill, Implicit),
            opcode!(0xA2, ldx, Immediate),
            opcode!(0xB2, ill, Implicit),
            opcode!(0xC2, ill, Immediate),
            opcode!(0xD2, ill, Implicit),
            opcode!(0xE2, ill, Immediate),
            opcode!(0xF2, ill, Implicit),

            // Codes ending in 3
            opcode!(0x03, ill, IndirectX),
            opcode!(0x13, ill, IndirectY),
            opcode!(0x23, ill, IndirectX),
            opcode!(0x33, ill, IndirectY),
            opcode!(0x43, ill, IndirectX),
            opcode!(0x53, ill, IndirectY),
            opcode!(0x63, ill, IndirectX),
            opcode!(0x73, ill, IndirectY),
            opcode!(0x83, ill, IndirectX),
            opcode!(0x93, ill, IndirectY),
            opcode!(0xA3, ill, IndirectX),
            opcode!(0xB3, ill, IndirectY),
            opcode!(0xC3, ill, IndirectX),
            opcode!(0xD3, ill, IndirectY),
            opcode!(0xE3, ill, IndirectX),
            opcode!(0xF3, ill, IndirectY),

            // Codes ending in 4
            opcode!(0x04, ill, ZeroPage),
            opcode!(0x14, ill, ZeroPageX),
            opcode!(0x24, bit, ZeroPage),
            opcode!(0x34, ill, ZeroPageX),
            opcode!(0x44, ill, ZeroPage),
            opcode!(0x54, ill, ZeroPageX),
            opcode!(0x64, ill, ZeroPage),
            opcode!(0x74, ill, ZeroPageX),
            opcode!(0x84, sty, ZeroPage),
            opcode!(0x94, sty, ZeroPageX),
            opcode!(0xA4, ldy, ZeroPage),
            opcode!(0xB4, ldy, ZeroPageX),
            opcode!(0xC4, cpy, ZeroPage),
            opcode!(0xD4, ill, ZeroPageX),
            opcode!(0xE4, cpx, ZeroPage),
            opcode!(0xF4, ill, ZeroPageX),

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
            opcode!(0x07, ill, ZeroPage),
            opcode!(0x17, ill, ZeroPageX),
            opcode!(0x27, ill, ZeroPage),
            opcode!(0x37, ill, ZeroPageX),
            opcode!(0x47, ill, ZeroPage),
            opcode!(0x57, ill, ZeroPageX),
            opcode!(0x67, ill, ZeroPage),
            opcode!(0x77, ill, ZeroPageX),
            opcode!(0x87, ill, ZeroPage),
            opcode!(0x97, ill, ZeroPageY),
            opcode!(0xA7, ill, ZeroPage),
            opcode!(0xB7, ill, ZeroPageY),
            opcode!(0xC7, ill, ZeroPage),
            opcode!(0xD7, ill, ZeroPageX),
            opcode!(0xE7, ill, ZeroPage),
            opcode!(0xF7, ill, ZeroPageX),

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
            opcode!(0x89, ill, Immediate),
            opcode!(0x99, sta, AbsoluteY),
            opcode!(0xA9, lda, Immediate),
            opcode!(0xB9, lda, AbsoluteY),
            opcode!(0xC9, cmp, Immediate),
            opcode!(0xD9, cmp, AbsoluteY),
            opcode!(0xE9, sbc, Immediate),
            opcode!(0xF9, sbc, AbsoluteY),

            // Codes ending in A
            opcode!(0x0A, asl, Acumulator),
            opcode!(0x1A, ill, Implicit),
            opcode!(0x2A, rol, Acumulator),
            opcode!(0x3A, ill, Implicit),
            opcode!(0x4A, lsr, Acumulator),
            opcode!(0x5A, ill, Implicit),
            opcode!(0x6A, ror, Acumulator),
            opcode!(0x7A, ill, Implicit),
            opcode!(0x8A, txa, Implicit),
            opcode!(0x9A, txs, Implicit),
            opcode!(0xAA, tax, Implicit),
            opcode!(0xBA, tsx, Implicit),
            opcode!(0xCA, dex, Implicit),
            opcode!(0xDA, ill, Implicit),
            opcode!(0xEA, nop, Implicit),
            opcode!(0xFA, ill, Implicit),

            // Codes ending in B
            opcode!(0x0B, ill, Immediate),
            opcode!(0x1B, ill, AbsoluteY),
            opcode!(0x2B, ill, Immediate),
            opcode!(0x3B, ill, AbsoluteY),
            opcode!(0x4B, ill, Immediate),
            opcode!(0x5B, ill, AbsoluteY),
            opcode!(0x6B, ill, Immediate),
            opcode!(0x7B, ill, AbsoluteY),
            opcode!(0x8B, ill, Immediate),
            opcode!(0x9B, ill, AbsoluteY),
            opcode!(0xAB, ill, Immediate),
            opcode!(0xBB, ill, AbsoluteY),
            opcode!(0xCB, ill, Immediate),
            opcode!(0xDB, ill, AbsoluteY),
            opcode!(0xEB, ill, Immediate),
            opcode!(0xFB, ill, AbsoluteY),

            // Codes ending in C
            opcode!(0x0C, ill, Absolute),
            opcode!(0x1C, ill, AbsoluteX),
            opcode!(0x2C, bit, Absolute),
            opcode!(0x3C, ill, AbsoluteX),
            opcode!(0x4C, jmp, Absolute),
            opcode!(0x5C, ill, AbsoluteX),
            opcode!(0x6C, jmp, Indirect),
            opcode!(0x7C, ill, AbsoluteX),
            opcode!(0x8C, sty, Absolute),
            opcode!(0x9C, ill, AbsoluteX),
            opcode!(0xAC, ldy, Absolute),
            opcode!(0xBC, ldy, AbsoluteX),
            opcode!(0xCC, cpy, Absolute),
            opcode!(0xDC, ill, AbsoluteX),
            opcode!(0xEC, cpx, Absolute),
            opcode!(0xFC, ill, AbsoluteX),


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
            opcode!(0x0F, ill, Absolute),
            opcode!(0x1F, ill, AbsoluteX),
            opcode!(0x2F, ill, Absolute),
            opcode!(0x3F, ill, AbsoluteX),
            opcode!(0x4F, ill, Absolute),
            opcode!(0x5F, ill, AbsoluteX),
            opcode!(0x6F, ill, Absolute),
            opcode!(0x7F, ill, AbsoluteX),
            opcode!(0x8F, ill, Absolute),
            opcode!(0x9F, ill, AbsoluteX),
            opcode!(0xAF, ill, Absolute),
            opcode!(0xBF, ill, AbsoluteY),
            opcode!(0xCF, ill, Absolute),
            opcode!(0xDF, ill, AbsoluteX),
            opcode!(0xEF, ill, Absolute),
            opcode!(0xFF, ill, AbsoluteX),
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
    pub operand_size: usize,
    pub execute_fn: fn(cpu: &mut Cpu, addr: u16),
    pub format_fn: fn(cpu: &Cpu, addr: u16) -> String,
}

impl Default for OpCodeTableEntry {
    fn default() -> Self {
        Self {
            code: Default::default(),
            operand_size: 0,
            execute_fn: |_, _| unimplemented!(),
            format_fn: |_, _| "N/A".to_string(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Address Modes

#[derive(PartialEq)]
enum MemoryAccessMode {
    /// Only reads operand address
    Addr,
    /// Reads operand
    Read,
    /// Writes to operand
    Write,
    /// Modifies operand in place
    Modify,
}

trait AddressMode {
    const OPERAND_SIZE: usize = 0;
    const BASE_CYCLE_COUNT: usize = 1;

    fn load_addr<T: ReadOrPeek>(mut _reader: T, _addr: u16) -> u16 {
        0
    }

    fn load_operand(cpu: &mut Cpu, addr: u16) -> u8 {
        let addr = Self::load_addr(cpu.mutable_bus_reader(), addr);
        cpu.bus.read(addr)
    }

    fn store_operand(cpu: &mut Cpu, addr: u16, value: u8) {
        let addr = Self::load_addr(cpu.mutable_bus_reader(), addr);
        cpu.bus.write(addr, value)
    }

    fn cycle_count(_cpu: &mut Cpu, _addr: u16, mode: MemoryAccessMode) -> usize {
        match mode {
            MemoryAccessMode::Addr => Self::BASE_CYCLE_COUNT,
            MemoryAccessMode::Read => Self::BASE_CYCLE_COUNT + 1,
            MemoryAccessMode::Write => Self::BASE_CYCLE_COUNT + 1,
            MemoryAccessMode::Modify => Self::BASE_CYCLE_COUNT + 3,
        }
    }

    fn advance_clock(cpu: &mut Cpu, addr: u16, mode: MemoryAccessMode) {
        let cycle_count = Self::cycle_count(cpu, addr, mode);
        cpu.advance_clock(cycle_count);
    }

    fn peek_operand(cpu: &Cpu, addr: u16) -> u8 {
        cpu.bus
            .peek(Self::load_addr(cpu.immutable_bus_reader(), addr))
    }

    fn format(cpu: &Cpu, addr: u16) -> String;
}

struct Immediate {}
impl AddressMode for Immediate {
    const OPERAND_SIZE: usize = 1;

    fn cycle_count(_cpu: &mut Cpu, _addr: u16, _mode: MemoryAccessMode) -> usize {
        2
    }

    fn load_operand(cpu: &mut Cpu, addr: u16) -> u8 {
        cpu.bus.read(addr + 1)
    }

    fn peek_operand(cpu: &Cpu, addr: u16) -> u8 {
        cpu.bus.peek(addr + 1)
    }

    fn format(cpu: &Cpu, addr: u16) -> String {
        return format!(" #{:02X}", Self::peek_operand(cpu, addr));
    }
}

struct Implicit {}
impl AddressMode for Implicit {
    const OPERAND_SIZE: usize = 0;

    fn cycle_count(_cpu: &mut Cpu, _addr: u16, _mode: MemoryAccessMode) -> usize {
        2
    }

    fn format(_cpu: &Cpu, _addr: u16) -> String {
        "".to_string()
    }
}

struct Acumulator {}
impl AddressMode for Acumulator {
    const OPERAND_SIZE: usize = 0;

    fn load_operand(cpu: &mut Cpu, _addr: u16) -> u8 {
        cpu.a
    }

    fn peek_operand(cpu: &Cpu, _addr: u16) -> u8 {
        cpu.a
    }

    fn store_operand(cpu: &mut Cpu, _addr: u16, value: u8) {
        cpu.a = value;
    }

    fn cycle_count(_cpu: &mut Cpu, _addr: u16, _mode: MemoryAccessMode) -> usize {
        2
    }

    fn format(_cpu: &Cpu, _addr: u16) -> String {
        " A".to_string()
    }
}

struct Absolute {}
impl AddressMode for Absolute {
    const OPERAND_SIZE: usize = 2;
    const BASE_CYCLE_COUNT: usize = 3;

    fn load_addr<T: ReadOrPeek>(mut reader: T, addr: u16) -> u16 {
        reader.read_or_peek_u16(addr + 1)
    }

    fn format(cpu: &Cpu, addr: u16) -> String {
        return format!(
            " {:04X} @{:02X}",
            Self::load_addr(cpu.immutable_bus_reader(), addr),
            Self::peek_operand(cpu, addr)
        );
    }
}

struct AbsoluteX {}
impl AddressMode for AbsoluteX {
    const OPERAND_SIZE: usize = 2;
    const BASE_CYCLE_COUNT: usize = 3;

    fn load_addr<T: ReadOrPeek>(mut reader: T, addr: u16) -> u16 {
        reader
            .read_or_peek_u16(addr + 1)
            .wrapping_add(reader.cpu().x as u16)
    }

    fn cycle_count(cpu: &mut Cpu, addr: u16, mode: MemoryAccessMode) -> usize {
        match mode {
            MemoryAccessMode::Addr => Self::BASE_CYCLE_COUNT,
            MemoryAccessMode::Read => {
                // Reads witgin the same page take one cycle less.
                let base_lo = cpu.bus.peek(addr + 1);
                let (_, overflow) = base_lo.overflowing_add(cpu.x);
                if overflow {
                    Self::BASE_CYCLE_COUNT + 2
                } else {
                    Self::BASE_CYCLE_COUNT + 1
                }
            }
            MemoryAccessMode::Write => Self::BASE_CYCLE_COUNT + 2,
            MemoryAccessMode::Modify => Self::BASE_CYCLE_COUNT + 4,
        }
    }

    fn format(cpu: &Cpu, addr: u16) -> String {
        return format!(
            " {:04X}+X ={:04X} @{:02X}",
            cpu.bus.peek_u16(addr + 1),
            Self::load_addr(cpu.immutable_bus_reader(), addr),
            Self::peek_operand(cpu, addr)
        );
    }
}

struct AbsoluteY {}
impl AddressMode for AbsoluteY {
    const OPERAND_SIZE: usize = 2;
    const BASE_CYCLE_COUNT: usize = 3;

    fn load_addr<T: ReadOrPeek>(mut reader: T, addr: u16) -> u16 {
        reader
            .read_or_peek_u16(addr + 1)
            .wrapping_add(reader.cpu().y as u16)
    }

    fn cycle_count(cpu: &mut Cpu, addr: u16, mode: MemoryAccessMode) -> usize {
        match mode {
            MemoryAccessMode::Addr => Self::BASE_CYCLE_COUNT,
            MemoryAccessMode::Read => {
                // Reads witgin the same page take one cycle less.
                let base_lo = cpu.bus.peek(addr + 1);
                let (_, overflow) = base_lo.overflowing_add(cpu.y);
                if overflow {
                    Self::BASE_CYCLE_COUNT + 2
                } else {
                    Self::BASE_CYCLE_COUNT + 1
                }
            }
            MemoryAccessMode::Write => Self::BASE_CYCLE_COUNT + 2,
            MemoryAccessMode::Modify => Self::BASE_CYCLE_COUNT + 4,
        }
    }

    fn format(cpu: &Cpu, addr: u16) -> String {
        return format!(
            " {:04X}+Y ={:04X} @{:02X}",
            cpu.bus.peek_u16(addr + 1),
            Self::load_addr(cpu.immutable_bus_reader(), addr),
            Self::peek_operand(cpu, addr)
        );
    }
}

struct ZeroPage {}
impl AddressMode for ZeroPage {
    const OPERAND_SIZE: usize = 1;
    const BASE_CYCLE_COUNT: usize = 2;

    fn load_addr<T: ReadOrPeek>(mut reader: T, addr: u16) -> u16 {
        reader.read_or_peek(addr + 1) as u16
    }

    fn format(cpu: &Cpu, addr: u16) -> String {
        return format!(
            " {:02X} @ {:02X}",
            Self::load_addr(cpu.immutable_bus_reader(), addr),
            Self::peek_operand(cpu, addr)
        );
    }
}

struct ZeroPageX {}
impl AddressMode for ZeroPageX {
    const OPERAND_SIZE: usize = 1;
    const BASE_CYCLE_COUNT: usize = 3;

    fn load_addr<T: ReadOrPeek>(mut reader: T, addr: u16) -> u16 {
        reader.read_or_peek(addr + 1).wrapping_add(reader.cpu().x) as u16
    }

    fn format(cpu: &Cpu, addr: u16) -> String {
        return format!(
            " {:02X}+X ={:04X} @ {:02X}",
            cpu.bus.peek(addr + 1),
            Self::load_addr(cpu.immutable_bus_reader(), addr),
            Self::peek_operand(cpu, addr)
        );
    }
}

struct ZeroPageY {}
impl AddressMode for ZeroPageY {
    const OPERAND_SIZE: usize = 1;
    const BASE_CYCLE_COUNT: usize = 3;

    fn load_addr<T: ReadOrPeek>(mut reader: T, addr: u16) -> u16 {
        reader.read_or_peek(addr + 1).wrapping_add(reader.cpu().y) as u16
    }

    fn format(cpu: &Cpu, addr: u16) -> String {
        return format!(
            " {:02X}+Y ={:04X} @ {:02X}",
            cpu.bus.peek(addr + 1),
            Self::load_addr(cpu.immutable_bus_reader(), addr),
            Self::peek_operand(cpu, addr)
        );
    }
}

struct Relative {}
impl AddressMode for Relative {
    const OPERAND_SIZE: usize = 1;
    const BASE_CYCLE_COUNT: usize = 2;

    fn load_addr<T: ReadOrPeek>(mut reader: T, addr: u16) -> u16 {
        let delta = reader.read_or_peek(addr + 1) as i8 as i16;
        let base_addr = addr + 1 + Self::OPERAND_SIZE as u16;
        if delta > 0 {
            base_addr.wrapping_add(delta.unsigned_abs())
        } else {
            base_addr.wrapping_sub(delta.unsigned_abs())
        }
    }

    fn format(cpu: &Cpu, addr: u16) -> String {
        let relative_addr = cpu.bus.peek(addr + 1) as i8;
        return format!(
            " {:+02X} ={:04X} @ {:02X}",
            relative_addr,
            Self::load_addr(cpu.immutable_bus_reader(), addr),
            Self::peek_operand(cpu, addr)
        );
    }
}

struct Indirect {}
impl AddressMode for Indirect {
    const OPERAND_SIZE: usize = 2;
    const BASE_CYCLE_COUNT: usize = 5;

    fn load_addr<T: ReadOrPeek>(mut reader: T, addr: u16) -> u16 {
        let indirect_addr = reader.read_or_peek_u16(addr + 1);
        let bytes = if indirect_addr & 0x00FF == 0x00FF {
            // CPU Bug: Address wraps around inside page.
            let page = indirect_addr & 0xFF00;
            [
                reader.read_or_peek(indirect_addr),
                reader.read_or_peek(page),
            ]
        } else {
            [
                reader.read_or_peek(indirect_addr),
                reader.read_or_peek(indirect_addr + 1),
            ]
        };
        u16::from_le_bytes(bytes)
    }

    fn format(cpu: &Cpu, addr: u16) -> String {
        let indirect_addr = cpu.bus.peek_u16(addr + 1);
        return format!(
            " (${:04X}) ={:04X} @ {:02X}",
            indirect_addr,
            Self::load_addr(cpu.immutable_bus_reader(), addr),
            Self::peek_operand(cpu, addr)
        );
    }
}

struct IndirectY {}
impl AddressMode for IndirectY {
    const OPERAND_SIZE: usize = 1;
    const BASE_CYCLE_COUNT: usize = 4;

    fn load_addr<T: ReadOrPeek>(mut reader: T, addr: u16) -> u16 {
        let indirect_addr = reader.read_or_peek(addr + 1);
        let target_addr = u16::from_le_bytes([
            reader.read_or_peek(indirect_addr as u16),
            reader.read_or_peek(indirect_addr.wrapping_add(1) as u16),
        ]);
        target_addr.wrapping_add(reader.cpu().y as u16)
    }

    fn cycle_count(cpu: &mut Cpu, addr: u16, mode: MemoryAccessMode) -> usize {
        match mode {
            MemoryAccessMode::Addr => Self::BASE_CYCLE_COUNT,
            MemoryAccessMode::Read => {
                // Reads witgin the same page take one cycle less.
                let indirect_addr = cpu.bus.peek(addr + 1);
                let base_lo = cpu.bus.zero_page_peek(indirect_addr);
                let (_, overflow) = base_lo.overflowing_add(cpu.y);
                if overflow {
                    Self::BASE_CYCLE_COUNT + 2
                } else {
                    Self::BASE_CYCLE_COUNT + 1
                }
            }
            MemoryAccessMode::Write => Self::BASE_CYCLE_COUNT + 2,
            MemoryAccessMode::Modify => Self::BASE_CYCLE_COUNT + 4,
        }
    }

    fn format(cpu: &Cpu, addr: u16) -> String {
        let indirect_addr = cpu.bus.peek(addr + 1);
        let indirect = cpu.bus.peek_u16(indirect_addr as u16);
        return format!(
            " (${:02X})={:04X}+Y ={:04X} @ {:02X}",
            indirect_addr,
            indirect,
            Self::load_addr(cpu.immutable_bus_reader(), addr),
            Self::peek_operand(cpu, addr)
        );
    }
}

struct IndirectX {}
impl AddressMode for IndirectX {
    const OPERAND_SIZE: usize = 1;
    const BASE_CYCLE_COUNT: usize = 5;

    fn load_addr<T: ReadOrPeek>(mut reader: T, addr: u16) -> u16 {
        // Note: Zero-page address lookup will wrap around from 0xFF to 0x00.
        // TODO: Implement zero page u16 reads in bus
        let indirect_addr = reader.read_or_peek(addr + 1).wrapping_add(reader.cpu().x);
        let bytes = [
            reader.read_or_peek(indirect_addr as u16),
            reader.read_or_peek(indirect_addr.wrapping_add(1) as u16),
        ];
        u16::from_le_bytes(bytes)
    }

    fn format(cpu: &Cpu, addr: u16) -> String {
        let indirect_addr = cpu.bus.peek(addr + 1);
        return format!(
            " (${:02X}+X) ={:04X} @{:02X}",
            indirect_addr,
            Self::load_addr(cpu.immutable_bus_reader(), addr),
            Self::peek_operand(cpu, addr)
        );
    }
}

////////////////////////////////////////////////////////////////////////////////
// Utilities shared by operations

pub fn update_negative_zero_flags(cpu: &mut Cpu, value: u8) {
    cpu.status_flags.set(StatusFlags::ZERO, value == 0);
    cpu.status_flags
        .set(StatusFlags::NEGATIVE, value & 0b1000_0000 != 0);
}

fn branch(cpu: &mut Cpu, target_addr: u16) {
    // Branch across pages take one more cycle
    if target_addr & 0xFF00 == cpu.program_counter & 0xFF00 {
        cpu.advance_clock(1);
    } else {
        cpu.advance_clock(2);
    }
    cpu.program_counter = target_addr;
}

////////////////////////////////////////////////////////////////////////////////
// Operation Implementations

// Experiment:
// Can reduce the boilerplate to:
//
// fn sta(cpu: &mut Cpu, operand_addr: u16) {
//   cpu.write_u8(operand_addr, cpu.a);
// }
//
// with this cpu method:
//
// fn store_operand(&mut self, addr: u16, value: u8) {
//    let cycles = self.bus.write_u8(operand_addr, value);
//    self.advance_clock(cycles);
// }
//
// So the CPU provides and API for operations that handle memory access and
// properly counts cycles to do so.
//
// The address is preloaded for each operation. That means we need a few
// special cases for accumulator

// J** (Jump) / RT* (Return)

fn jmp<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    cpu.program_counter = AM::load_addr(cpu.mutable_bus_reader(), addr);
    AM::advance_clock(cpu, addr, MemoryAccessMode::Addr);
}

fn jsr<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    cpu.stack_push_u16(cpu.program_counter - 1);
    cpu.program_counter = AM::load_addr(cpu.mutable_bus_reader(), addr);
    AM::advance_clock(cpu, addr, MemoryAccessMode::Addr);
    cpu.advance_clock(3);
}

fn rts<AM: AddressMode>(cpu: &mut Cpu, _addr: u16) {
    cpu.program_counter = cpu.stack_pop_u16() + 1;
    cpu.advance_clock(6);
}

fn rti<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    plp::<AM>(cpu, addr);
    cpu.program_counter = cpu.stack_pop_u16();
    cpu.advance_clock(2);
}

// ST* (Store)

fn sta<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Write);
    AM::store_operand(cpu, addr, cpu.a);
}

fn stx<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Write);
    AM::store_operand(cpu, addr, cpu.x);
}

fn sty<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Write);
    AM::store_operand(cpu, addr, cpu.y);
}

// LD* (Load)

fn lda<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    cpu.a = AM::load_operand(cpu, addr);
    update_negative_zero_flags(cpu, cpu.a);
}

fn ldy<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    cpu.y = AM::load_operand(cpu, addr);
    update_negative_zero_flags(cpu, cpu.y);
}

fn ldx<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    cpu.x = AM::load_operand(cpu, addr);
    update_negative_zero_flags(cpu, cpu.x);
}

// IN* (Increment)

fn inc<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Modify);
    let value = AM::load_operand(cpu, addr).wrapping_add(1);
    AM::store_operand(cpu, addr, value);
    update_negative_zero_flags(cpu, value);
}

fn inx<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Modify);
    cpu.x = cpu.x.wrapping_add(1);
    update_negative_zero_flags(cpu, cpu.x);
}

fn iny<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Modify);
    cpu.y = cpu.y.wrapping_add(1);
    update_negative_zero_flags(cpu, cpu.y);
}

// DE* (Decrement)

fn dec<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Modify);
    let value = AM::load_operand(cpu, addr).wrapping_sub(1);
    AM::store_operand(cpu, addr, value);
    update_negative_zero_flags(cpu, value);
}

fn dex<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Modify);
    cpu.x = cpu.x.wrapping_sub(1);
    update_negative_zero_flags(cpu, cpu.x);
}

fn dey<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Modify);
    cpu.y = cpu.y.wrapping_sub(1);
    update_negative_zero_flags(cpu, cpu.y);
}

// SE* / CL* (Set / clear status bits)

fn sed<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    cpu.status_flags.insert(StatusFlags::DECIMAL);
}

fn cld<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    cpu.status_flags.remove(StatusFlags::DECIMAL);
}

fn sec<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    cpu.status_flags.insert(StatusFlags::CARRY);
}

fn clc<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    cpu.status_flags.remove(StatusFlags::CARRY);
}

fn clv<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    cpu.status_flags.remove(StatusFlags::OVERFLOW);
}

fn cli<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    cpu.status_flags.remove(StatusFlags::INTERRUPT);
}

// B** (Branch)

fn bcs<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Addr);
    if cpu.status_flags.contains(StatusFlags::CARRY) {
        let target_addr = AM::load_addr(cpu.mutable_bus_reader(), addr);
        branch(cpu, target_addr);
    }
}

fn bcc<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Addr);
    if !cpu.status_flags.contains(StatusFlags::CARRY) {
        let target_addr = AM::load_addr(cpu.mutable_bus_reader(), addr);
        branch(cpu, target_addr);
    }
}

fn beq<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Addr);
    if cpu.status_flags.contains(StatusFlags::ZERO) {
        let target_addr = AM::load_addr(cpu.mutable_bus_reader(), addr);
        branch(cpu, target_addr);
    }
}

fn bne<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Addr);
    if !cpu.status_flags.contains(StatusFlags::ZERO) {
        let target_addr = AM::load_addr(cpu.mutable_bus_reader(), addr);
        branch(cpu, target_addr);
    }
}

fn bmi<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Addr);
    if cpu.status_flags.contains(StatusFlags::NEGATIVE) {
        let target_addr = AM::load_addr(cpu.mutable_bus_reader(), addr);
        branch(cpu, target_addr);
    }
}

fn bpl<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Addr);
    if !cpu.status_flags.contains(StatusFlags::NEGATIVE) {
        let target_addr = AM::load_addr(cpu.mutable_bus_reader(), addr);
        branch(cpu, target_addr);
    }
}

fn bvs<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Addr);
    if cpu.status_flags.contains(StatusFlags::OVERFLOW) {
        let target_addr = AM::load_addr(cpu.mutable_bus_reader(), addr);
        branch(cpu, target_addr);
    }
}

fn bvc<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Addr);
    if !cpu.status_flags.contains(StatusFlags::OVERFLOW) {
        let target_addr = AM::load_addr(cpu.mutable_bus_reader(), addr);
        branch(cpu, target_addr);
    }
}

// PH* (Push), PL* (Pull)

fn pha<AM: AddressMode>(cpu: &mut Cpu, _addr: u16) {
    cpu.stack_push(cpu.a);
    cpu.advance_clock(3);
}

fn php<AM: AddressMode>(cpu: &mut Cpu, _addr: u16) {
    let mut value = cpu.status_flags;
    value.insert(StatusFlags::BREAK);
    cpu.stack_push(value.bits);
    cpu.advance_clock(3);
}

fn pla<AM: AddressMode>(cpu: &mut Cpu, _addr: u16) {
    cpu.a = cpu.stack_pop();
    update_negative_zero_flags(cpu, cpu.a);
    cpu.advance_clock(4);
}

fn plp<AM: AddressMode>(cpu: &mut Cpu, _addr: u16) {
    let mut value = StatusFlags::from_bits_truncate(cpu.stack_pop());
    value.set(
        StatusFlags::BREAK,
        cpu.status_flags.contains(StatusFlags::BREAK),
    );
    value.insert(StatusFlags::UNUSED);
    cpu.status_flags = value;
    cpu.advance_clock(4);
}

// add / sub

fn adc<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    let carry: u16 = if cpu.status_flags.contains(StatusFlags::CARRY) {
        1
    } else {
        0
    };
    let operand = AM::load_operand(cpu, addr);
    let result = cpu.a as u16 + operand as u16 + carry;

    cpu.status_flags.set(StatusFlags::CARRY, result > 0xFF);
    cpu.status_flags.set(
        StatusFlags::OVERFLOW,
        (operand ^ result as u8) & (result as u8 ^ cpu.a) & 0x80 != 0,
    );
    cpu.a = result as u8;
    update_negative_zero_flags(cpu, cpu.a);
}

fn sbc<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    let carry: u16 = if cpu.status_flags.contains(StatusFlags::CARRY) {
        1
    } else {
        0
    };
    let operand = (AM::load_operand(cpu, addr) as i8)
        .wrapping_neg()
        .wrapping_sub(1) as u8;
    let result = cpu.a as u16 + operand as u16 + carry;

    cpu.status_flags.set(StatusFlags::CARRY, result > 0xFF);
    cpu.status_flags.set(
        StatusFlags::OVERFLOW,
        (operand ^ result as u8) & (result as u8 ^ cpu.a) & 0x80 != 0,
    );
    cpu.a = result as u8;
    update_negative_zero_flags(cpu, cpu.a);
}

// Bit-wise operations

fn and<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    cpu.a &= AM::load_operand(cpu, addr);
    update_negative_zero_flags(cpu, cpu.a);
}

fn ora<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    cpu.a |= AM::load_operand(cpu, addr);
    update_negative_zero_flags(cpu, cpu.a);
}

fn eor<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    cpu.a ^= AM::load_operand(cpu, addr);
    update_negative_zero_flags(cpu, cpu.a);
}

// C** (Compare)

fn cmp<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    let (value, overflow) = cpu.a.overflowing_sub(AM::load_operand(cpu, addr));
    update_negative_zero_flags(cpu, value);
    cpu.status_flags.set(StatusFlags::CARRY, !overflow);
}

fn cpx<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    let (value, overflow) = cpu.x.overflowing_sub(AM::load_operand(cpu, addr));
    update_negative_zero_flags(cpu, value);
    cpu.status_flags.set(StatusFlags::CARRY, !overflow);
}

fn cpy<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    let (value, overflow) = cpu.y.overflowing_sub(AM::load_operand(cpu, addr));
    update_negative_zero_flags(cpu, value);
    cpu.status_flags.set(StatusFlags::CARRY, !overflow);
}

// Shifts

fn lsr<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Modify);
    let operand = AM::load_operand(cpu, addr);
    let (result, _) = operand.overflowing_shr(1);
    AM::store_operand(cpu, addr, result);
    update_negative_zero_flags(cpu, result);
    cpu.status_flags
        .set(StatusFlags::CARRY, (operand & 0x01) != 0);
}

fn asl<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Modify);
    let operand = AM::load_operand(cpu, addr);
    let (result, _) = operand.overflowing_shl(1);
    AM::store_operand(cpu, addr, result);
    update_negative_zero_flags(cpu, result);
    cpu.status_flags
        .set(StatusFlags::CARRY, (operand & 0x80) != 0);
}

fn ror<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Modify);
    let operand = AM::load_operand(cpu, addr);
    let (mut result, _) = operand.overflowing_shr(1);
    if cpu.status_flags.contains(StatusFlags::CARRY) {
        result |= 0b1000_0000;
    }
    AM::store_operand(cpu, addr, result);
    update_negative_zero_flags(cpu, result);
    cpu.status_flags
        .set(StatusFlags::CARRY, (operand & 0x01) != 0);
}

fn rol<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Modify);
    let operand = AM::load_operand(cpu, addr);
    let (mut result, _) = operand.overflowing_shl(1);
    if cpu.status_flags.contains(StatusFlags::CARRY) {
        result |= 0b0000_0001;
    }
    AM::store_operand(cpu, addr, result);
    update_negative_zero_flags(cpu, result);
    cpu.status_flags
        .set(StatusFlags::CARRY, (operand & 0x80) != 0);
}

// Register Transfers

fn txa<AM: AddressMode>(cpu: &mut Cpu, _addr: u16) {
    cpu.a = cpu.x;
    update_negative_zero_flags(cpu, cpu.a);
    cpu.advance_clock(2);
}

fn tax<AM: AddressMode>(cpu: &mut Cpu, _addr: u16) {
    cpu.x = cpu.a;
    update_negative_zero_flags(cpu, cpu.x);
    cpu.advance_clock(2);
}

fn tay<AM: AddressMode>(cpu: &mut Cpu, _addr: u16) {
    cpu.y = cpu.a;
    update_negative_zero_flags(cpu, cpu.y);
    cpu.advance_clock(2);
}

fn tya<AM: AddressMode>(cpu: &mut Cpu, _addr: u16) {
    cpu.a = cpu.y;
    update_negative_zero_flags(cpu, cpu.a);
    cpu.advance_clock(2);
}

fn tsx<AM: AddressMode>(cpu: &mut Cpu, _addr: u16) {
    cpu.x = cpu.sp;
    update_negative_zero_flags(cpu, cpu.x);
    cpu.advance_clock(2);
}

fn txs<AM: AddressMode>(cpu: &mut Cpu, _addr: u16) {
    cpu.sp = cpu.x;
    cpu.advance_clock(2);
}

// Misc Operations

fn bit<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
    let value = AM::load_operand(cpu, addr);
    cpu.status_flags.set(
        StatusFlags::NEGATIVE,
        (value & StatusFlags::NEGATIVE.bits) > 0,
    );
    cpu.status_flags.set(
        StatusFlags::OVERFLOW,
        (value & StatusFlags::OVERFLOW.bits) > 0,
    );
    cpu.status_flags
        .set(StatusFlags::ZERO, (value & cpu.a) == 0);
}

fn hlt<AM: AddressMode>(cpu: &mut Cpu, _addr: u16) {
    cpu.halt = true;
}

fn sei<AM: AddressMode>(cpu: &mut Cpu, _addr: u16) {
    cpu.advance_clock(2);
}

fn nop<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    AM::advance_clock(cpu, addr, MemoryAccessMode::Read);
}

// Illegal Instruction
fn ill<AM: AddressMode>(cpu: &mut Cpu, addr: u16) {
    panic!("Illegal Opcode {:02X}", AM::load_operand(cpu, addr));
}
