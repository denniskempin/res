use anyhow::Result;

use super::Cpu;
use super::MaybeMutableCpu;
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
        let raw_opcode = cpu.read(addr);
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
            execute_fn: |cpu, addr| {
                let address_mode = $address_mode::load(cpu.mutable_wrapper(), addr);
                $method::<$address_mode>(cpu, address_mode)
            },

            format_fn: |cpu, addr| {
                let address_mode = $address_mode::load(cpu.immutable_wrapper(), addr);
                format!(
                    "{}{}",
                    stringify!($method).to_uppercase(),
                    address_mode.format(cpu)
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
            opcode!(0x0A, asl, Accumulator),
            opcode!(0x1A, ill, Implicit),
            opcode!(0x2A, rol, Accumulator),
            opcode!(0x3A, ill, Implicit),
            opcode!(0x4A, lsr, Accumulator),
            opcode!(0x5A, ill, Implicit),
            opcode!(0x6A, ror, Accumulator),
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

trait Operand {
    const OPERAND_SIZE: usize = 0;

    fn load<T: MaybeMutableCpu>(_cpu: T, _addr: u16) -> Self;

    fn format(&self, cpu: &Cpu) -> String;

    fn operand_addr(&self) -> u16 {
        unimplemented!()
    }

    fn extra_write_cycle(&self) -> bool {
        false
    }

    fn load_operand(&self, cpu: &mut Cpu) -> u8 {
        cpu.read(self.operand_addr())
    }

    fn store_operand(&self, cpu: &mut Cpu, value: u8) {
        if self.extra_write_cycle() {
            cpu.advance_clock(1);
        }
        cpu.write(self.operand_addr(), value)
    }
}

struct Immediate {
    operand: u8,
}
impl Operand for Immediate {
    const OPERAND_SIZE: usize = 1;

    fn load<T: MaybeMutableCpu>(mut cpu: T, addr: u16) -> Self {
        Self {
            operand: cpu.read_or_peek(addr + 1),
        }
    }

    fn load_operand(&self, _cpu: &mut Cpu) -> u8 {
        self.operand
    }

    fn format(&self, _cpu: &Cpu) -> String {
        return format!(" #{:02X}", self.operand);
    }
}

struct Implicit {}
impl Operand for Implicit {
    const OPERAND_SIZE: usize = 0;

    fn load<T: MaybeMutableCpu>(mut cpu: T, _addr: u16) -> Self {
        // Even Implicit address modes seem to be spending an extra
        // cycle to load the operand.
        cpu.advance_clock(1);
        Self {}
    }

    fn format(&self, _cpu: &Cpu) -> String {
        "".to_string()
    }
}

struct Accumulator {}
impl Operand for Accumulator {
    const OPERAND_SIZE: usize = 0;

    fn load<T: MaybeMutableCpu>(_cpu: T, _addr: u16) -> Self {
        Self {}
    }

    fn load_operand(&self, cpu: &mut Cpu) -> u8 {
        cpu.a
    }

    fn store_operand(&self, cpu: &mut Cpu, value: u8) {
        cpu.a = value;
    }

    fn format(&self, _cpu: &Cpu) -> String {
        " A".to_string()
    }
}

struct Absolute {
    operand_addr: u16,
}
impl Operand for Absolute {
    const OPERAND_SIZE: usize = 2;

    fn load<T: MaybeMutableCpu>(mut cpu: T, addr: u16) -> Self {
        Self {
            operand_addr: cpu.read_or_peek_u16(addr + 1),
        }
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn format(&self, cpu: &Cpu) -> String {
        return format!(
            " {:04X} @{:02X}",
            self.operand_addr,
            cpu.bus.peek(self.operand_addr)
        );
    }
}

struct AbsoluteX {
    base_addr: u16,
    operand_addr: u16,
    extra_write_cycle: bool,
}

impl Operand for AbsoluteX {
    const OPERAND_SIZE: usize = 2;

    fn load<T: MaybeMutableCpu>(mut cpu: T, addr: u16) -> Self {
        let base_addr = cpu.read_or_peek_u16(addr + 1);
        let operand_addr = base_addr.wrapping_add(cpu.immutable().x as u16);
        let page_cross = base_addr & 0xFF00 != operand_addr & 0xFF00;
        if page_cross {
            cpu.advance_clock(1);
        }
        Self {
            base_addr,
            operand_addr,
            extra_write_cycle: !page_cross,
        }
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn extra_write_cycle(&self) -> bool {
        self.extra_write_cycle
    }

    fn format(&self, cpu: &Cpu) -> String {
        return format!(
            " {:04X}+X ={:04X} @{:02X}",
            self.base_addr,
            self.operand_addr,
            cpu.bus.peek(self.operand_addr)
        );
    }
}

struct AbsoluteY {
    base_addr: u16,
    operand_addr: u16,
    extra_write_cycle: bool,
}
impl Operand for AbsoluteY {
    const OPERAND_SIZE: usize = 2;

    fn load<T: MaybeMutableCpu>(mut cpu: T, addr: u16) -> Self {
        let base_addr = cpu.read_or_peek_u16(addr + 1);
        let operand_addr = base_addr.wrapping_add(cpu.immutable().y as u16);
        let page_cross = base_addr & 0xFF00 != operand_addr & 0xFF00;
        if page_cross {
            cpu.advance_clock(1);
        }
        Self {
            base_addr,
            operand_addr,
            extra_write_cycle: !page_cross,
        }
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn extra_write_cycle(&self) -> bool {
        self.extra_write_cycle
    }

    fn format(&self, cpu: &Cpu) -> String {
        return format!(
            " {:04X}+Y ={:04X} @{:02X}",
            self.base_addr,
            self.operand_addr,
            cpu.bus.peek(self.operand_addr)
        );
    }
}

struct ZeroPage {
    operand_addr: u8,
}
impl Operand for ZeroPage {
    const OPERAND_SIZE: usize = 1;

    fn load<T: MaybeMutableCpu>(mut cpu: T, addr: u16) -> Self {
        Self {
            operand_addr: cpu.read_or_peek(addr + 1),
        }
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr as u16
    }

    fn format(&self, cpu: &Cpu) -> String {
        return format!(
            " {:02X} @ {:02X}",
            self.operand_addr,
            cpu.bus.peek(self.operand_addr())
        );
    }
}

struct ZeroPageX {
    base_addr: u8,
    operand_addr: u16,
}
impl Operand for ZeroPageX {
    const OPERAND_SIZE: usize = 1;

    fn load<T: MaybeMutableCpu>(mut cpu: T, addr: u16) -> Self {
        let base_addr = cpu.read_or_peek(addr + 1);
        let operand_addr = base_addr.wrapping_add(cpu.immutable().x) as u16;
        cpu.read_or_peek(0x00); // Fake read for one extra cycle
        Self {
            base_addr,
            operand_addr,
        }
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn format(&self, cpu: &Cpu) -> String {
        return format!(
            " {:02X}+X ={:04X} @ {:02X}",
            self.base_addr,
            self.operand_addr,
            cpu.bus.peek(self.operand_addr)
        );
    }
}

struct ZeroPageY {
    base_addr: u8,
    operand_addr: u16,
}
impl Operand for ZeroPageY {
    const OPERAND_SIZE: usize = 1;

    fn load<T: MaybeMutableCpu>(mut cpu: T, addr: u16) -> Self {
        let base_addr = cpu.read_or_peek(addr + 1);
        let operand_addr = base_addr.wrapping_add(cpu.immutable().y) as u16;
        cpu.read_or_peek(0x00); // Fake read for one extra cycle
        Self {
            base_addr,
            operand_addr,
        }
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn format(&self, cpu: &Cpu) -> String {
        return format!(
            " {:02X}+Y ={:04X} @ {:02X}",
            self.base_addr,
            self.operand_addr,
            cpu.bus.peek(self.operand_addr)
        );
    }
}

struct Relative {
    relative_addr: i8,
    operand_addr: u16,
}
impl Operand for Relative {
    const OPERAND_SIZE: usize = 1;

    fn load<T: MaybeMutableCpu>(mut cpu: T, addr: u16) -> Self {
        let relative_addr = cpu.read_or_peek(addr + 1) as i8;
        let base_addr = addr + 1 + Self::OPERAND_SIZE as u16;
        let operand_addr = if relative_addr > 0 {
            base_addr.wrapping_add((relative_addr as i16).unsigned_abs())
        } else {
            base_addr.wrapping_sub((relative_addr as i16).unsigned_abs())
        };
        Self {
            relative_addr,
            operand_addr,
        }
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn format(&self, cpu: &Cpu) -> String {
        return format!(
            " {:+02X} ={:04X} @ {:02X}",
            self.relative_addr,
            self.operand_addr,
            cpu.bus.peek(self.operand_addr)
        );
    }
}

struct Indirect {
    indirect_addr: u16,
    operand_addr: u16,
}
impl Operand for Indirect {
    const OPERAND_SIZE: usize = 2;

    fn load<T: MaybeMutableCpu>(mut cpu: T, addr: u16) -> Self {
        let indirect_addr = cpu.read_or_peek_u16(addr + 1);
        let bytes = if indirect_addr & 0x00FF == 0x00FF {
            // CPU Bug: Address wraps around inside page.
            let page = indirect_addr & 0xFF00;
            [cpu.read_or_peek(indirect_addr), cpu.read_or_peek(page)]
        } else {
            [
                cpu.read_or_peek(indirect_addr),
                cpu.read_or_peek(indirect_addr + 1),
            ]
        };
        let operand_addr = u16::from_le_bytes(bytes);
        Self {
            indirect_addr,
            operand_addr,
        }
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn format(&self, cpu: &Cpu) -> String {
        return format!(
            " (${:04X}) ={:04X} @ {:02X}",
            self.indirect_addr,
            self.operand_addr,
            cpu.bus.peek(self.operand_addr)
        );
    }
}

struct IndirectY {
    indirect_addr: u8,
    operand_addr: u16,
    extra_write_cycle: bool,
}
impl Operand for IndirectY {
    const OPERAND_SIZE: usize = 1;

    fn load<T: MaybeMutableCpu>(mut cpu: T, addr: u16) -> Self {
        let indirect_addr = cpu.read_or_peek(addr + 1);
        let base_addr = u16::from_le_bytes([
            cpu.read_or_peek(indirect_addr as u16),
            cpu.read_or_peek(indirect_addr.wrapping_add(1) as u16),
        ]);
        let operand_addr = base_addr.wrapping_add(cpu.immutable().y as u16);
        let page_cross = base_addr & 0xFF00 != operand_addr & 0xFF00;

        if page_cross {
            cpu.advance_clock(1);
        }

        Self {
            indirect_addr,
            operand_addr,
            extra_write_cycle: !page_cross,
        }
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn extra_write_cycle(&self) -> bool {
        self.extra_write_cycle
    }

    fn format(&self, cpu: &Cpu) -> String {
        return format!(
            " (${:02X})+Y ={:04X} @ {:02X}",
            self.indirect_addr,
            self.operand_addr,
            cpu.bus.peek(self.operand_addr)
        );
    }
}

struct IndirectX {
    indirect_addr: u8,
    operand_addr: u16,
}
impl Operand for IndirectX {
    const OPERAND_SIZE: usize = 1;

    fn load<T: MaybeMutableCpu>(mut cpu: T, addr: u16) -> Self {
        let indirect_addr = cpu.read_or_peek(addr + 1).wrapping_add(cpu.immutable().x);
        let operand_addr = u16::from_le_bytes([
            cpu.read_or_peek(indirect_addr as u16),
            cpu.read_or_peek(indirect_addr.wrapping_add(1) as u16),
        ]);
        cpu.read_or_peek(0x00); // Fake read for extra cycle.
        Self {
            indirect_addr,
            operand_addr,
        }
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn format(&self, cpu: &Cpu) -> String {
        return format!(
            " (${:02X}+X) ={:04X} @{:02X}",
            self.indirect_addr,
            self.operand_addr,
            cpu.bus.peek(self.operand_addr)
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

fn pop_status_flags(cpu: &mut Cpu) {
    let mut value = StatusFlags::from_bits_truncate(cpu.stack_pop());
    value.set(
        StatusFlags::BREAK,
        cpu.status_flags.contains(StatusFlags::BREAK),
    );
    value.insert(StatusFlags::UNUSED);
    cpu.status_flags = value;
}

////////////////////////////////////////////////////////////////////////////////
// Operation Implementations

// J** (Jump) / RT* (Return)

fn jmp<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    cpu.program_counter = operand.operand_addr();
}

fn jsr<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    cpu.stack_push_u16(cpu.program_counter - 1);
    cpu.program_counter = operand.operand_addr();
    cpu.advance_clock(1);
}

fn rts<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.program_counter = cpu.stack_pop_u16() + 1;
    cpu.advance_clock(2);
}

fn rti<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    pop_status_flags(cpu);
    cpu.program_counter = cpu.stack_pop_u16();
    cpu.advance_clock(1);
}

// ST* (Store)

fn sta<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    operand.store_operand(cpu, cpu.a);
}

fn stx<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    operand.store_operand(cpu, cpu.x);
}

fn sty<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    operand.store_operand(cpu, cpu.y);
}

// LD* (Load)

fn lda<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    cpu.a = operand.load_operand(cpu);
    update_negative_zero_flags(cpu, cpu.a);
}

fn ldy<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    cpu.y = operand.load_operand(cpu);
    update_negative_zero_flags(cpu, cpu.y);
}

fn ldx<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    cpu.x = operand.load_operand(cpu);
    update_negative_zero_flags(cpu, cpu.x);
}

// IN* (Increment)

fn inc<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    let value = operand.load_operand(cpu).wrapping_add(1);
    operand.store_operand(cpu, value);
    update_negative_zero_flags(cpu, value);
    cpu.advance_clock(1);
}

fn inx<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.x = cpu.x.wrapping_add(1);
    update_negative_zero_flags(cpu, cpu.x);
}

fn iny<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.y = cpu.y.wrapping_add(1);
    update_negative_zero_flags(cpu, cpu.y);
}

// DE* (Decrement)

fn dec<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    let value = operand.load_operand(cpu).wrapping_sub(1);
    operand.store_operand(cpu, value);
    update_negative_zero_flags(cpu, value);
    cpu.advance_clock(1);
}

fn dex<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.x = cpu.x.wrapping_sub(1);
    update_negative_zero_flags(cpu, cpu.x);
}

fn dey<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.y = cpu.y.wrapping_sub(1);
    update_negative_zero_flags(cpu, cpu.y);
}

// SE* / CL* (Set / clear status bits)

fn sed<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.status_flags.insert(StatusFlags::DECIMAL);
}

fn cld<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.status_flags.remove(StatusFlags::DECIMAL);
}

fn sec<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.status_flags.insert(StatusFlags::CARRY);
}

fn clc<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.status_flags.remove(StatusFlags::CARRY);
}

fn clv<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.status_flags.remove(StatusFlags::OVERFLOW);
}

fn cli<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.status_flags.remove(StatusFlags::INTERRUPT);
}

// B** (Branch)

fn bcs<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    if cpu.status_flags.contains(StatusFlags::CARRY) {
        branch(cpu, operand.operand_addr());
    }
}

fn bcc<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    if !cpu.status_flags.contains(StatusFlags::CARRY) {
        branch(cpu, operand.operand_addr());
    }
}

fn beq<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    if cpu.status_flags.contains(StatusFlags::ZERO) {
        branch(cpu, operand.operand_addr());
    }
}

fn bne<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    if !cpu.status_flags.contains(StatusFlags::ZERO) {
        branch(cpu, operand.operand_addr());
    }
}

fn bmi<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    if cpu.status_flags.contains(StatusFlags::NEGATIVE) {
        branch(cpu, operand.operand_addr());
    }
}

fn bpl<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    if !cpu.status_flags.contains(StatusFlags::NEGATIVE) {
        branch(cpu, operand.operand_addr());
    }
}

fn bvs<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    if cpu.status_flags.contains(StatusFlags::OVERFLOW) {
        branch(cpu, operand.operand_addr());
    }
}

fn bvc<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    if !cpu.status_flags.contains(StatusFlags::OVERFLOW) {
        branch(cpu, operand.operand_addr());
    }
}

// PH* (Push), PL* (Pull)

fn pha<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.stack_push(cpu.a);
}

fn php<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    let mut value = cpu.status_flags;
    value.insert(StatusFlags::BREAK);
    cpu.stack_push(value.bits);
}

fn pla<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.a = cpu.stack_pop();
    update_negative_zero_flags(cpu, cpu.a);
    cpu.advance_clock(1);
}

fn plp<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    pop_status_flags(cpu);
    cpu.advance_clock(1);
}

// add / sub

fn adc<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    let carry = cpu.status_flags.carry() as u16;
    let value = operand.load_operand(cpu);
    let result = cpu.a as u16 + value as u16 + carry;

    // TODO: Learn the details behind the C and V flags and how they differ.
    cpu.status_flags.set_carry(result > 0xFF);
    cpu.status_flags
        .set_overflow((value ^ result as u8) & (result as u8 ^ cpu.a) & 0x80 != 0);
    cpu.a = result as u8;
    update_negative_zero_flags(cpu, cpu.a);
}

fn sbc<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    let carry = cpu.status_flags.carry() as u16;
    let value = (operand.load_operand(cpu) as i8)
        .wrapping_neg()
        .wrapping_sub(1) as u8;
    let result = cpu.a as u16 + value as u16 + carry;

    cpu.status_flags.set_carry(result > 0xFF);
    cpu.status_flags.set(
        StatusFlags::OVERFLOW,
        (value ^ result as u8) & (result as u8 ^ cpu.a) & 0x80 != 0,
    );
    cpu.a = result as u8;
    update_negative_zero_flags(cpu, cpu.a);
}

// Bit-wise operations

fn and<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    cpu.a &= operand.load_operand(cpu);
    update_negative_zero_flags(cpu, cpu.a);
}

fn ora<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    cpu.a |= operand.load_operand(cpu);
    update_negative_zero_flags(cpu, cpu.a);
}

fn eor<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    cpu.a ^= operand.load_operand(cpu);
    update_negative_zero_flags(cpu, cpu.a);
}

// C** (Compare)

fn cmp<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    let (value, overflow) = cpu.a.overflowing_sub(operand.load_operand(cpu));
    update_negative_zero_flags(cpu, value);
    cpu.status_flags.set_carry(!overflow);
}

fn cpx<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    let (value, overflow) = cpu.x.overflowing_sub(operand.load_operand(cpu));
    update_negative_zero_flags(cpu, value);
    cpu.status_flags.set_carry(!overflow);
}

fn cpy<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    let (value, overflow) = cpu.y.overflowing_sub(operand.load_operand(cpu));
    update_negative_zero_flags(cpu, value);
    cpu.status_flags.set_carry(!overflow);
}

// Shifts

fn lsr<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    let value = operand.load_operand(cpu);
    let (result, _) = value.overflowing_shr(1);
    operand.store_operand(cpu, result);
    update_negative_zero_flags(cpu, result);
    cpu.status_flags.set_carry((value & 0x01) != 0);
    cpu.advance_clock(1);
}

fn asl<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    let value = operand.load_operand(cpu);
    let (result, _) = value.overflowing_shl(1);
    operand.store_operand(cpu, result);
    update_negative_zero_flags(cpu, result);
    cpu.status_flags.set_carry((value & 0x80) != 0);
    cpu.advance_clock(1);
}

fn ror<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    let operand2 = operand.load_operand(cpu);
    let (mut result, _) = operand2.overflowing_shr(1);
    if cpu.status_flags.carry() {
        result |= 0b1000_0000;
    }
    operand.store_operand(cpu, result);
    update_negative_zero_flags(cpu, result);
    cpu.status_flags.set_carry((operand2 & 0x01) != 0);
    cpu.advance_clock(1);
}

fn rol<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    let operand2 = operand.load_operand(cpu);
    let (mut result, _) = operand2.overflowing_shl(1);
    if cpu.status_flags.carry() {
        result |= 0b0000_0001;
    }
    operand.store_operand(cpu, result);
    update_negative_zero_flags(cpu, result);
    cpu.status_flags.set_carry((operand2 & 0x80) != 0);
    cpu.advance_clock(1);
}

// Register Transfers

fn txa<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.a = cpu.x;
    update_negative_zero_flags(cpu, cpu.a);
}

fn tax<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.x = cpu.a;
    update_negative_zero_flags(cpu, cpu.x);
}

fn tay<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.y = cpu.a;
    update_negative_zero_flags(cpu, cpu.y);
}

fn tya<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.a = cpu.y;
    update_negative_zero_flags(cpu, cpu.a);
}

fn tsx<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.x = cpu.sp;
    update_negative_zero_flags(cpu, cpu.x);
}

fn txs<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.sp = cpu.x;
}

// Misc Operations

fn bit<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    let value = operand.load_operand(cpu);
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

fn hlt<AM: Operand>(cpu: &mut Cpu, _operand: AM) {
    cpu.halt = true;
}

fn sei<AM: Operand>(_cpu: &mut Cpu, _operand: AM) {}

fn nop<AM: Operand>(_cpu: &mut Cpu, _operand: AM) {}

fn ill<AM: Operand>(cpu: &mut Cpu, operand: AM) {
    panic!("Illegal Opcode {:02X}", operand.load_operand(cpu));
}
