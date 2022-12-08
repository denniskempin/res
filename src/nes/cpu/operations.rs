use anyhow::anyhow;
use anyhow::Result;

use super::Cpu;
use super::CpuBus;
use super::StatusFlags;

////////////////////////////////////////////////////////////////////////////////
// Operation

#[derive(Default, Clone)]
pub struct Operation {
    pub addr: u16,
    pub table_entry: OpCodeTableEntry,
}

impl Operation {
    pub fn size(&self) -> usize {
        self.table_entry.operand_size + 1
    }

    pub fn peek(cpu: &Cpu, addr: u16) -> Option<Operation> {
        let raw_opcode = cpu.bus.peek(addr)?;
        let table_entry = OPCODE_TABLE[raw_opcode as usize];
        Some(Operation { addr, table_entry })
    }

    pub fn load(cpu: &mut Cpu, addr: u16) -> Result<Operation> {
        let raw_opcode = cpu.read(addr)?;
        let table_entry = OPCODE_TABLE[raw_opcode as usize];
        Ok(Operation { addr, table_entry })
    }

    pub fn execute(&self, cpu: &mut Cpu) -> Result<()> {
        cpu.advance_clock(self.table_entry.cycle_count_before)?;
        (self.table_entry.execute_fn)(cpu, self.addr)?;
        cpu.advance_clock(self.table_entry.cycle_count_after)
    }

    pub fn format(&self, cpu: &Cpu) -> String {
        (self.table_entry.format_fn)(cpu, self.addr)
    }
}

////////////////////////////////////////////////////////////////////////////////
// OpCode Table

macro_rules! opcode {
    (
        $code: literal,
        $method: ident,
        $address_mode: ident,
        $cycle_count: expr
    ) => {
        OpCodeTableEntry {
            code: $code,
            operand_size: $address_mode::OPERAND_SIZE,
            execute_fn: |cpu, addr| {
                let address_mode = $address_mode::load(cpu, addr)?;
                $method::<$address_mode>(cpu, address_mode)
            },
            format_fn: |cpu, addr| {
                if let Ok(address_mode) = $address_mode::load(cpu, addr) {
                    format!(
                        "{}{}",
                        stringify!($method).to_uppercase(),
                        address_mode.format(cpu)
                    )
                } else {
                    format!("INV")
                }
            },
            cycle_count_before: $cycle_count.0,
            cycle_count_after: $cycle_count.1,
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
            opcode!(0x00, hlt, Implicit, (2, 0)),
            opcode!(0x10, bpl, Relative, (2, 0)),
            opcode!(0x20, jsr, Absolute, (6, 0)),
            opcode!(0x30, bmi, Relative, (2, 0)),
            opcode!(0x40, rti, Implicit, (6, 0)),
            opcode!(0x50, bvc, Relative, (2, 0)),
            opcode!(0x60, rts, Implicit, (6, 0)),
            opcode!(0x70, bvs, Relative, (2, 0)),
            opcode!(0x80, ill, Immediate, (1, 0)),
            opcode!(0x90, bcc, Relative, (2, 0)),
            opcode!(0xA0, ldy, Immediate, (2, 0)),
            opcode!(0xB0, bcs, Relative, (2, 0)),
            opcode!(0xC0, cpy, Immediate, (2, 0)),
            opcode!(0xD0, bne, Relative, (2, 0)),
            opcode!(0xE0, cpx, Immediate, (2, 0)),
            opcode!(0xF0, beq, Relative, (2, 0)),

            // Codes ending in 1
            opcode!(0x01, ora, IndirectX, (6, 0)),
            opcode!(0x11, ora, IndirectY, (5, 0)),
            opcode!(0x21, and, IndirectX, (6, 0)),
            opcode!(0x31, and, IndirectY, (5, 0)),
            opcode!(0x41, eor, IndirectX, (6, 0)),
            opcode!(0x51, eor, IndirectY, (5, 0)),
            opcode!(0x61, adc, IndirectX, (6, 0)),
            opcode!(0x71, adc, IndirectY, (5, 0)),
            opcode!(0x81, sta, IndirectX, (6, 0)),
            opcode!(0x91, sta, IndirectY, (6, 0)),
            opcode!(0xA1, lda, IndirectX, (6, 0)),
            opcode!(0xB1, lda, IndirectY, (5, 0)),
            opcode!(0xC1, cmp, IndirectX, (6, 0)),
            opcode!(0xD1, cmp, IndirectY, (5, 0)),
            opcode!(0xE1, sbc, IndirectX, (6, 0)),
            opcode!(0xF1, sbc, IndirectY, (5, 0)),

            // Codes ending in 2
            opcode!(0x02, ill, Implicit, (1, 0)),
            opcode!(0x12, ill, Implicit, (1, 0)),
            opcode!(0x22, ill, Implicit, (1, 0)),
            opcode!(0x32, ill, Implicit, (1, 0)),
            opcode!(0x42, ill, Implicit, (1, 0)),
            opcode!(0x52, ill, Implicit, (1, 0)),
            opcode!(0x62, ill, Implicit, (1, 0)),
            opcode!(0x72, ill, Implicit, (1, 0)),
            opcode!(0x82, ill, Immediate, (1, 0)),
            opcode!(0x92, ill, Implicit, (1, 0)),
            opcode!(0xA2, ldx, Immediate, (2, 0)),
            opcode!(0xB2, ill, Implicit, (1, 0)),
            opcode!(0xC2, ill, Immediate, (1, 0)),
            opcode!(0xD2, ill, Implicit, (1, 0)),
            opcode!(0xE2, ill, Immediate, (1, 0)),
            opcode!(0xF2, ill, Implicit, (1, 0)),

            // Codes ending in 3
            opcode!(0x03, ill, IndirectX, (1, 0)),
            opcode!(0x13, ill, IndirectY, (1, 0)),
            opcode!(0x23, ill, IndirectX, (1, 0)),
            opcode!(0x33, ill, IndirectY, (1, 0)),
            opcode!(0x43, ill, IndirectX, (1, 0)),
            opcode!(0x53, ill, IndirectY, (1, 0)),
            opcode!(0x63, ill, IndirectX, (1, 0)),
            opcode!(0x73, ill, IndirectY, (1, 0)),
            opcode!(0x83, ill, IndirectX, (1, 0)),
            opcode!(0x93, ill, IndirectY, (1, 0)),
            opcode!(0xA3, ill, IndirectX, (1, 0)),
            opcode!(0xB3, ill, IndirectY, (1, 0)),
            opcode!(0xC3, ill, IndirectX, (1, 0)),
            opcode!(0xD3, ill, IndirectY, (1, 0)),
            opcode!(0xE3, ill, IndirectX, (1, 0)),
            opcode!(0xF3, ill, IndirectY, (1, 0)),

            // Codes ending in 4
            opcode!(0x04, ill, ZeroPage, (1, 0)),
            opcode!(0x14, ill, ZeroPageX, (1, 0)),
            opcode!(0x24, bit, ZeroPage, (3, 0)),
            opcode!(0x34, ill, ZeroPageX, (1, 0)),
            opcode!(0x44, ill, ZeroPage, (1, 0)),
            opcode!(0x54, ill, ZeroPageX, (1, 0)),
            opcode!(0x64, ill, ZeroPage, (1, 0)),
            opcode!(0x74, ill, ZeroPageX, (1, 0)),
            opcode!(0x84, sty, ZeroPage, (3, 0)),
            opcode!(0x94, sty, ZeroPageX, (4, 0)),
            opcode!(0xA4, ldy, ZeroPage, (3, 0)),
            opcode!(0xB4, ldy, ZeroPageX, (4, 0)),
            opcode!(0xC4, cpy, ZeroPage, (3, 0)),
            opcode!(0xD4, ill, ZeroPageX, (1, 0)),
            opcode!(0xE4, cpx, ZeroPage, (3, 0)),
            opcode!(0xF4, ill, ZeroPageX, (1, 0)),

            // Codes ending in 5
            opcode!(0x05, ora, ZeroPage, (3, 0)),
            opcode!(0x15, ora, ZeroPageX, (4, 0)),
            opcode!(0x25, and, ZeroPage, (3, 0)),
            opcode!(0x35, and, ZeroPageX, (4, 0)),
            opcode!(0x45, eor, ZeroPage, (3, 0)),
            opcode!(0x55, eor, ZeroPageX, (4, 0)),
            opcode!(0x65, adc, ZeroPage, (3, 0)),
            opcode!(0x75, adc, ZeroPageX, (4, 0)),
            opcode!(0x85, sta, ZeroPage, (3, 0)),
            opcode!(0x95, sta, ZeroPageX, (4, 0)),
            opcode!(0xA5, lda, ZeroPage, (3, 0)),
            opcode!(0xB5, lda, ZeroPageX, (4, 0)),
            opcode!(0xC5, cmp, ZeroPage, (3, 0)),
            opcode!(0xD5, cmp, ZeroPageX, (4, 0)),
            opcode!(0xE5, sbc, ZeroPage, (3, 0)),
            opcode!(0xF5, sbc, ZeroPageX, (4, 0)),

            // Codes ending in 6
            opcode!(0x06, asl, ZeroPage, (5, 0)),
            opcode!(0x16, asl, ZeroPageX, (6, 0)),
            opcode!(0x26, rol, ZeroPage, (5, 0)),
            opcode!(0x36, rol, ZeroPageX, (6, 0)),
            opcode!(0x46, lsr, ZeroPage, (5, 0)),
            opcode!(0x56, lsr, ZeroPageX, (6, 0)),
            opcode!(0x66, ror, ZeroPage, (5, 0)),
            opcode!(0x76, ror, ZeroPageX, (6, 0)),
            opcode!(0x86, stx, ZeroPage, (3, 0)),
            opcode!(0x96, stx, ZeroPageY, (4, 0)),
            opcode!(0xA6, ldx, ZeroPage, (3, 0)),
            opcode!(0xB6, ldx, ZeroPageY, (4, 0)),
            opcode!(0xC6, dec, ZeroPage, (5, 0)),
            opcode!(0xD6, dec, ZeroPageX, (6, 0)),
            opcode!(0xE6, inc, ZeroPage, (5, 0)),
            opcode!(0xF6, inc, ZeroPageX, (6, 0)),

            // Codes ending in 7
            opcode!(0x07, ill, ZeroPage, (1, 0)),
            opcode!(0x17, ill, ZeroPageX, (1, 0)),
            opcode!(0x27, ill, ZeroPage, (1, 0)),
            opcode!(0x37, ill, ZeroPageX, (1, 0)),
            opcode!(0x47, ill, ZeroPage, (1, 0)),
            opcode!(0x57, ill, ZeroPageX, (1, 0)),
            opcode!(0x67, ill, ZeroPage, (1, 0)),
            opcode!(0x77, ill, ZeroPageX, (1, 0)),
            opcode!(0x87, ill, ZeroPage, (1, 0)),
            opcode!(0x97, ill, ZeroPageY, (1, 0)),
            opcode!(0xA7, ill, ZeroPage, (1, 0)),
            opcode!(0xB7, ill, ZeroPageY, (1, 0)),
            opcode!(0xC7, ill, ZeroPage, (1, 0)),
            opcode!(0xD7, ill, ZeroPageX, (1, 0)),
            opcode!(0xE7, ill, ZeroPage, (1, 0)),
            opcode!(0xF7, ill, ZeroPageX, (1, 0)),

            // Codes ending in 8
            opcode!(0x08, php, Implicit, (3, 0)),
            opcode!(0x18, clc, Implicit, (2, 0)),
            opcode!(0x28, plp, Implicit, (4, 0)),
            opcode!(0x38, sec, Implicit, (2, 0)),
            opcode!(0x48, pha, Implicit, (3, 0)),
            opcode!(0x58, cli, Implicit, (2, 0)),
            opcode!(0x68, pla, Implicit, (4, 0)),
            opcode!(0x78, sei, Implicit, (2, 0)),
            opcode!(0x88, dey, Implicit, (2, 0)),
            opcode!(0x98, tya, Implicit, (2, 0)),
            opcode!(0xA8, tay, Implicit, (2, 0)),
            opcode!(0xB8, clv, Implicit, (2, 0)),
            opcode!(0xC8, iny, Implicit, (2, 0)),
            opcode!(0xD8, cld, Implicit, (2, 0)),
            opcode!(0xE8, inx, Implicit, (2, 0)),
            opcode!(0xF8, sed, Implicit, (2, 0)),

            // Codes ending in 9
            opcode!(0x09, ora, Immediate, (2, 0)),
            opcode!(0x19, ora, AbsoluteY, (4, 0)),
            opcode!(0x29, and, Immediate, (2, 0)),
            opcode!(0x39, and, AbsoluteY, (4, 0)),
            opcode!(0x49, eor, Immediate, (2, 0)),
            opcode!(0x59, eor, AbsoluteY, (4, 0)),
            opcode!(0x69, adc, Immediate, (2, 0)),
            opcode!(0x79, adc, AbsoluteY, (4, 0)),
            opcode!(0x89, ill, Immediate, (1, 0)),
            opcode!(0x99, sta, AbsoluteY, (5, 0)),
            opcode!(0xA9, lda, Immediate, (2, 0)),
            opcode!(0xB9, lda, AbsoluteY, (4, 0)),
            opcode!(0xC9, cmp, Immediate, (2, 0)),
            opcode!(0xD9, cmp, AbsoluteY, (4, 0)),
            opcode!(0xE9, sbc, Immediate, (2, 0)),
            opcode!(0xF9, sbc, AbsoluteY, (4, 0)),

            // Codes ending in A
            opcode!(0x0A, asl, Accumulator, (2, 0)),
            opcode!(0x1A, ill, Implicit, (1, 0)),
            opcode!(0x2A, rol, Accumulator, (2, 0)),
            opcode!(0x3A, ill, Implicit, (1, 0)),
            opcode!(0x4A, lsr, Accumulator, (2, 0)),
            opcode!(0x5A, ill, Implicit, (1, 0)),
            opcode!(0x6A, ror, Accumulator, (2, 0)),
            opcode!(0x7A, ill, Implicit, (1, 0)),
            opcode!(0x8A, txa, Implicit, (2, 0)),
            opcode!(0x9A, txs, Implicit, (2, 0)),
            opcode!(0xAA, tax, Implicit, (2, 0)),
            opcode!(0xBA, tsx, Implicit, (2, 0)),
            opcode!(0xCA, dex, Implicit, (2, 0)),
            opcode!(0xDA, ill, Implicit, (1, 0)),
            opcode!(0xEA, nop, Implicit, (2, 0)),
            opcode!(0xFA, ill, Implicit, (1, 0)),

            // Codes ending in B
            opcode!(0x0B, ill, Immediate, (1, 0)),
            opcode!(0x1B, ill, AbsoluteY, (1, 0)),
            opcode!(0x2B, ill, Immediate, (1, 0)),
            opcode!(0x3B, ill, AbsoluteY, (1, 0)),
            opcode!(0x4B, ill, Immediate, (1, 0)),
            opcode!(0x5B, ill, AbsoluteY, (1, 0)),
            opcode!(0x6B, ill, Immediate, (1, 0)),
            opcode!(0x7B, ill, AbsoluteY, (1, 0)),
            opcode!(0x8B, ill, Immediate, (1, 0)),
            opcode!(0x9B, ill, AbsoluteY, (1, 0)),
            opcode!(0xAB, ill, Immediate, (1, 0)),
            opcode!(0xBB, ill, AbsoluteY, (1, 0)),
            opcode!(0xCB, ill, Immediate, (1, 0)),
            opcode!(0xDB, ill, AbsoluteY, (1, 0)),
            opcode!(0xEB, ill, Immediate, (1, 0)),
            opcode!(0xFB, ill, AbsoluteY, (1, 0)),

            // Codes ending in C
            opcode!(0x0C, ill, Absolute, (1, 0)),
            opcode!(0x1C, ill, AbsoluteX, (1, 0)),
            opcode!(0x2C, bit, Absolute, (4, 0)),
            opcode!(0x3C, ill, AbsoluteX, (1, 0)),
            opcode!(0x4C, jmp, Absolute, (3, 0)),
            opcode!(0x5C, ill, AbsoluteX, (1, 0)),
            opcode!(0x6C, jmp, Indirect, (5, 0)),
            opcode!(0x7C, ill, AbsoluteX, (1, 0)),
            opcode!(0x8C, sty, Absolute, (4, 0)),
            opcode!(0x9C, ill, AbsoluteX, (1, 0)),
            opcode!(0xAC, ldy, Absolute, (4, 0)),
            opcode!(0xBC, ldy, AbsoluteX, (4, 0)),
            opcode!(0xCC, cpy, Absolute, (4, 0)),
            opcode!(0xDC, ill, AbsoluteX, (1, 0)),
            opcode!(0xEC, cpx, Absolute, (4, 0)),
            opcode!(0xFC, ill, AbsoluteX, (1, 0)),

            // Codes ending in D
            opcode!(0x0D, ora, Absolute, (4, 0)),
            opcode!(0x1D, ora, AbsoluteX, (4, 0)),
            opcode!(0x2D, and, Absolute, (4, 0)),
            opcode!(0x3D, and, AbsoluteX, (4, 0)),
            opcode!(0x4D, eor, Absolute, (4, 0)),
            opcode!(0x5D, eor, AbsoluteX, (4, 0)),
            opcode!(0x6D, adc, Absolute, (4, 0)),
            opcode!(0x7D, adc, AbsoluteX, (4, 0)),
            opcode!(0x8D, sta, Absolute, (4, 0)),
            opcode!(0x9D, sta, AbsoluteX, (5, 0)),
            opcode!(0xAD, lda, Absolute, (4, 0)),
            opcode!(0xBD, lda, AbsoluteX, (4, 0)),
            opcode!(0xCD, cmp, Absolute, (4, 0)),
            opcode!(0xDD, cmp, AbsoluteX, (4, 0)),
            opcode!(0xED, sbc, Absolute, (4, 0)),
            opcode!(0xFD, sbc, AbsoluteX, (4, 0)),

            // Codes endding in E
            opcode!(0x0E, asl, Absolute, (6, 0)),
            opcode!(0x1E, asl, AbsoluteX, (7, 0)),
            opcode!(0x2E, rol, Absolute, (6, 0)),
            opcode!(0x3E, rol, AbsoluteX, (7, 0)),
            opcode!(0x4E, lsr, Absolute, (6, 0)),
            opcode!(0x5E, lsr, AbsoluteX, (7, 0)),
            opcode!(0x6E, ror, Absolute, (6, 0)),
            opcode!(0x7E, ror, AbsoluteX, (7, 0)),
            opcode!(0x8E, stx, Absolute, (4, 0)),
            opcode!(0x9E, ill, AbsoluteX, (1, 0)),
            opcode!(0xAE, ldx, Absolute, (4, 0)),
            opcode!(0xBE, ldx, AbsoluteY, (4, 0)),
            opcode!(0xCE, dec, Absolute, (6, 0)),
            opcode!(0xDE, dec, AbsoluteX, (7, 0)),
            opcode!(0xEE, inc, Absolute, (6, 0)),
            opcode!(0xFE, inc, AbsoluteX, (7, 0)),

            // Codes endding in F
            opcode!(0x0F, ill, Absolute, (1, 0)),
            opcode!(0x1F, ill, AbsoluteX, (1, 0)),
            opcode!(0x2F, ill, Absolute, (1, 0)),
            opcode!(0x3F, ill, AbsoluteX, (1, 0)),
            opcode!(0x4F, ill, Absolute, (1, 0)),
            opcode!(0x5F, ill, AbsoluteX, (1, 0)),
            opcode!(0x6F, ill, Absolute, (1, 0)),
            opcode!(0x7F, ill, AbsoluteX, (1, 0)),
            opcode!(0x8F, ill, Absolute, (1, 0)),
            opcode!(0x9F, ill, AbsoluteX, (1, 0)),
            opcode!(0xAF, ill, Absolute, (1, 0)),
            opcode!(0xBF, ill, AbsoluteY, (1, 0)),
            opcode!(0xCF, ill, Absolute, (1, 0)),
            opcode!(0xDF, ill, AbsoluteX, (1, 0)),
            opcode!(0xEF, ill, Absolute, (1, 0)),
            opcode!(0xFF, ill, AbsoluteX, (1, 0)),

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
pub struct OpCodeTableEntry {
    pub code: u8,
    pub operand_size: usize,
    pub execute_fn: fn(cpu: &mut Cpu, addr: u16) -> Result<()>,
    pub format_fn: fn(cpu: &Cpu, addr: u16) -> String,
    pub cycle_count_before: usize,
    pub cycle_count_after: usize,
}

impl Default for OpCodeTableEntry {
    fn default() -> Self {
        Self {
            code: Default::default(),
            operand_size: 0,
            execute_fn: |_, _| unimplemented!(),
            format_fn: |_, _| "N/A".to_string(),
            cycle_count_before: 0,
            cycle_count_after: 0,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Address Modes

trait Operand {
    const OPERAND_SIZE: usize = 0;

    fn load(_cpu: &Cpu, _addr: u16) -> Result<Self>
    where
        Self: Sized;

    fn format(&self, cpu: &Cpu) -> String;

    fn operand_addr(&self) -> u16 {
        unimplemented!()
    }

    fn apply_page_cross_penality(&self, _cpu: &mut Cpu) -> Result<()> {
        Ok(())
    }

    fn load_operand(&self, cpu: &mut Cpu) -> Result<u8> {
        cpu.read(self.operand_addr())
    }

    fn store_operand(&self, cpu: &mut Cpu, value: u8) -> Result<()> {
        cpu.write(self.operand_addr(), value)
    }
}

struct Immediate {
    operand: u8,
}
impl Operand for Immediate {
    const OPERAND_SIZE: usize = 1;

    fn load(cpu: &Cpu, addr: u16) -> Result<Self> {
        Ok(Self {
            operand: peek_to_result(cpu.bus.peek(addr + 1))?,
        })
    }

    fn load_operand(&self, _cpu: &mut Cpu) -> Result<u8> {
        Ok(self.operand)
    }

    fn format(&self, _cpu: &Cpu) -> String {
        return format!(" #{:02X}", self.operand);
    }
}

struct Implicit {}
impl Operand for Implicit {
    const OPERAND_SIZE: usize = 0;

    fn load(_cpu: &Cpu, _addr: u16) -> Result<Self> {
        Ok(Self {})
    }

    fn format(&self, _cpu: &Cpu) -> String {
        "".to_string()
    }
}

struct Accumulator {}
impl Operand for Accumulator {
    const OPERAND_SIZE: usize = 0;

    fn load(_cpu: &Cpu, _addr: u16) -> Result<Self> {
        Ok(Self {})
    }

    fn load_operand(&self, cpu: &mut Cpu) -> Result<u8> {
        Ok(cpu.a)
    }

    fn store_operand(&self, cpu: &mut Cpu, value: u8) -> Result<()> {
        cpu.a = value;
        Ok(())
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

    fn load(cpu: &Cpu, addr: u16) -> Result<Self> {
        Ok(Self {
            operand_addr: peek_to_result(cpu.bus.peek_u16(addr + 1))?,
        })
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn format(&self, _cpu: &Cpu) -> String {
        return format!(" ${:04X}", self.operand_addr);
    }
}

struct AbsoluteX {
    base_addr: u16,
    operand_addr: u16,
    page_cross: bool,
}

impl Operand for AbsoluteX {
    const OPERAND_SIZE: usize = 2;

    fn load(cpu: &Cpu, addr: u16) -> Result<Self> {
        let base_addr = peek_to_result(cpu.bus.peek_u16(addr + 1))?;
        let operand_addr = base_addr.wrapping_add(cpu.x as u16);
        let page_cross = base_addr & 0xFF00 != operand_addr & 0xFF00;
        Ok(Self {
            base_addr,
            operand_addr,
            page_cross,
        })
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn apply_page_cross_penality(&self, cpu: &mut Cpu) -> Result<()> {
        if self.page_cross {
            cpu.advance_clock(1)
        } else {
            Ok(())
        }
    }

    fn format(&self, _cpu: &Cpu) -> String {
        return format!(" {:04X}+X = ${:04X}", self.base_addr, self.operand_addr);
    }
}

struct AbsoluteY {
    base_addr: u16,
    operand_addr: u16,
    page_cross: bool,
}
impl Operand for AbsoluteY {
    const OPERAND_SIZE: usize = 2;

    fn load(cpu: &Cpu, addr: u16) -> Result<Self> {
        let base_addr = peek_to_result(cpu.bus.peek_u16(addr + 1))?;
        let operand_addr = base_addr.wrapping_add(cpu.y as u16);
        let page_cross = base_addr & 0xFF00 != operand_addr & 0xFF00;
        Ok(Self {
            base_addr,
            operand_addr,
            page_cross,
        })
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn apply_page_cross_penality(&self, cpu: &mut Cpu) -> Result<()> {
        if self.page_cross {
            cpu.advance_clock(1)
        } else {
            Ok(())
        }
    }

    fn format(&self, _cpu: &Cpu) -> String {
        return format!(" {:04X}+Y = ${:04X}", self.base_addr, self.operand_addr);
    }
}

fn peek_to_result<T>(peek: Option<T>) -> Result<T> {
    peek.ok_or_else(|| anyhow!("Invalid operand address"))
}

struct ZeroPage {
    operand_addr: u8,
}
impl Operand for ZeroPage {
    const OPERAND_SIZE: usize = 1;

    fn load(cpu: &Cpu, addr: u16) -> Result<Self> {
        Ok(Self {
            operand_addr: peek_to_result(cpu.bus.peek(addr + 1))?,
        })
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr as u16
    }

    fn format(&self, _cpu: &Cpu) -> String {
        return format!(" $00{:02X}", self.operand_addr);
    }
}
struct ZeroPageX {
    base_addr: u8,
    operand_addr: u16,
}
impl Operand for ZeroPageX {
    const OPERAND_SIZE: usize = 1;

    fn load(cpu: &Cpu, addr: u16) -> Result<Self> {
        let base_addr = peek_to_result(cpu.bus.peek(addr + 1))?;
        let operand_addr = base_addr.wrapping_add(cpu.x) as u16;
        Ok(Self {
            base_addr,
            operand_addr,
        })
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn format(&self, _cpu: &Cpu) -> String {
        return format!(" {:02X}+X = ${:04X}", self.base_addr, self.operand_addr);
    }
}

struct ZeroPageY {
    base_addr: u8,
    operand_addr: u16,
}
impl Operand for ZeroPageY {
    const OPERAND_SIZE: usize = 1;

    fn load(cpu: &Cpu, addr: u16) -> Result<Self> {
        let base_addr = peek_to_result(cpu.bus.peek(addr + 1))?;
        let operand_addr = base_addr.wrapping_add(cpu.y) as u16;
        cpu.bus.peek(0x00).unwrap(); // Fake read for one extra cycle
        Ok(Self {
            base_addr,
            operand_addr,
        })
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn format(&self, _cpu: &Cpu) -> String {
        return format!(" {:02X}+Y = ${:04X}", self.base_addr, self.operand_addr);
    }
}

struct Relative {
    relative_addr: i8,
    operand_addr: u16,
}
impl Operand for Relative {
    const OPERAND_SIZE: usize = 1;

    fn load(cpu: &Cpu, addr: u16) -> Result<Self> {
        let relative_addr = peek_to_result(cpu.bus.peek(addr + 1))? as i8;
        let base_addr = addr + 1 + Self::OPERAND_SIZE as u16;
        let operand_addr = if relative_addr > 0 {
            base_addr.wrapping_add((relative_addr as i16).unsigned_abs())
        } else {
            base_addr.wrapping_sub((relative_addr as i16).unsigned_abs())
        };
        Ok(Self {
            relative_addr,
            operand_addr,
        })
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn format(&self, _cpu: &Cpu) -> String {
        return format!(" {:+02X} = ${:04X}", self.relative_addr, self.operand_addr);
    }
}

struct Indirect {
    indirect_addr: u16,
    operand_addr: u16,
}
impl Operand for Indirect {
    const OPERAND_SIZE: usize = 2;

    fn load(cpu: &Cpu, addr: u16) -> Result<Self> {
        let indirect_addr = peek_to_result(cpu.bus.peek_u16(addr + 1))?;
        let bytes = if indirect_addr & 0x00FF == 0x00FF {
            // CPU Bug: Address wraps around inside page.
            let page = indirect_addr & 0xFF00;
            [
                peek_to_result(cpu.bus.peek(indirect_addr))?,
                peek_to_result(cpu.bus.peek(page))?,
            ]
        } else {
            [
                peek_to_result(cpu.bus.peek(indirect_addr))?,
                peek_to_result(cpu.bus.peek(indirect_addr + 1))?,
            ]
        };
        let operand_addr = u16::from_le_bytes(bytes);
        Ok(Self {
            indirect_addr,
            operand_addr,
        })
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn format(&self, _cpu: &Cpu) -> String {
        return format!(
            " (${:04X}) = ${:04X}",
            self.indirect_addr, self.operand_addr
        );
    }
}

struct IndirectY {
    indirect_addr: u8,
    operand_addr: u16,
    page_cross: bool,
}
impl Operand for IndirectY {
    const OPERAND_SIZE: usize = 1;

    fn load(cpu: &Cpu, addr: u16) -> Result<Self> {
        let indirect_addr = peek_to_result(cpu.bus.peek(addr + 1))?;
        let base_addr = u16::from_le_bytes([
            cpu.bus.peek(indirect_addr as u16).unwrap(),
            cpu.bus.peek(indirect_addr.wrapping_add(1) as u16).unwrap(),
        ]);
        let operand_addr = base_addr.wrapping_add(cpu.y as u16);
        let page_cross = base_addr & 0xFF00 != operand_addr & 0xFF00;

        Ok(Self {
            indirect_addr,
            operand_addr,
            page_cross,
        })
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn apply_page_cross_penality(&self, cpu: &mut Cpu) -> Result<()> {
        if self.page_cross {
            cpu.advance_clock(1)
        } else {
            Ok(())
        }
    }

    fn format(&self, _cpu: &Cpu) -> String {
        return format!(
            " (${:02X})+Y = ${:04X}",
            self.indirect_addr, self.operand_addr
        );
    }
}

struct IndirectX {
    indirect_addr: u8,
    operand_addr: u16,
}
impl Operand for IndirectX {
    const OPERAND_SIZE: usize = 1;

    fn load(cpu: &Cpu, addr: u16) -> Result<Self> {
        let indirect_addr = peek_to_result(cpu.bus.peek(addr + 1))?.wrapping_add(cpu.x);
        let operand_addr = u16::from_le_bytes([
            cpu.bus.peek(indirect_addr as u16).unwrap(),
            cpu.bus.peek(indirect_addr.wrapping_add(1) as u16).unwrap(),
        ]);
        cpu.bus.peek(0x00).unwrap(); // Fake read for extra cycle.
        Ok(Self {
            indirect_addr,
            operand_addr,
        })
    }

    fn operand_addr(&self) -> u16 {
        self.operand_addr
    }

    fn format(&self, _cpu: &Cpu) -> String {
        return format!(
            " (${:02X}+X) = ${:04X}",
            self.indirect_addr, self.operand_addr
        );
    }
}

////////////////////////////////////////////////////////////////////////////////
// Utilities shared by operations

pub fn update_negative_zero_flags(cpu: &mut Cpu, value: u8) {
    cpu.status_flags.zero = value == 0;
    cpu.status_flags.negative = value & 0b1000_0000 != 0;
}

fn branch(cpu: &mut Cpu, target_addr: u16) -> Result<()> {
    // Branch across pages take one more cycle
    if target_addr & 0xFF00 == cpu.program_counter & 0xFF00 {
        cpu.advance_clock(1)?;
    } else {
        cpu.advance_clock(2)?;
    }
    cpu.program_counter = target_addr;
    Ok(())
}

fn pop_status_flags(cpu: &mut Cpu) -> Result<()> {
    let mut value = StatusFlags::from_bits(cpu.stack_pop()?);
    value.break_flag = cpu.status_flags.break_flag;
    value.unused = true;
    cpu.status_flags = value;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Operation Implementations

// J** (Jump) / RT* (Return)

fn jmp<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    cpu.program_counter = operand.operand_addr();
    Ok(())
}

fn jsr<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    cpu.stack_push_u16(cpu.program_counter - 1)?;
    cpu.program_counter = operand.operand_addr();
    Ok(())
}

fn rts<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.program_counter = cpu.stack_pop_u16()? + 1;
    Ok(())
}

fn rti<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    pop_status_flags(cpu)?;
    cpu.program_counter = cpu.stack_pop_u16()?;
    Ok(())
}

// ST* (Store)

fn sta<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    operand.store_operand(cpu, cpu.a)
}

fn stx<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    operand.store_operand(cpu, cpu.x)
}

fn sty<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    operand.store_operand(cpu, cpu.y)
}

// LD* (Load)

fn lda<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    cpu.a = operand.load_operand(cpu)?;
    update_negative_zero_flags(cpu, cpu.a);
    operand.apply_page_cross_penality(cpu)?;
    Ok(())
}

fn ldy<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    cpu.y = operand.load_operand(cpu)?;
    update_negative_zero_flags(cpu, cpu.y);
    operand.apply_page_cross_penality(cpu)?;
    Ok(())
}

fn ldx<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    cpu.x = operand.load_operand(cpu)?;
    update_negative_zero_flags(cpu, cpu.x);
    operand.apply_page_cross_penality(cpu)?;
    Ok(())
}

// IN* (Increment)

fn inc<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    let value = operand.load_operand(cpu)?.wrapping_add(1);
    operand.store_operand(cpu, value)?;
    update_negative_zero_flags(cpu, value);
    Ok(())
}

fn inx<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.x = cpu.x.wrapping_add(1);
    update_negative_zero_flags(cpu, cpu.x);
    Ok(())
}

fn iny<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.y = cpu.y.wrapping_add(1);
    update_negative_zero_flags(cpu, cpu.y);
    Ok(())
}

// DE* (Decrement)

fn dec<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    let value = operand.load_operand(cpu)?.wrapping_sub(1);
    operand.store_operand(cpu, value)?;
    update_negative_zero_flags(cpu, value);
    Ok(())
}

fn dex<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.x = cpu.x.wrapping_sub(1);
    update_negative_zero_flags(cpu, cpu.x);
    Ok(())
}

fn dey<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.y = cpu.y.wrapping_sub(1);
    update_negative_zero_flags(cpu, cpu.y);
    Ok(())
}

// SE* / CL* (Set / clear status bits)

fn sed<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.status_flags.decimal = true;
    Ok(())
}

fn cld<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.status_flags.decimal = false;
    Ok(())
}

fn sec<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.status_flags.carry = true;
    Ok(())
}

fn clc<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.status_flags.carry = false;
    Ok(())
}

fn clv<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.status_flags.overflow = false;
    Ok(())
}

fn cli<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.status_flags.interrupt = false;
    Ok(())
}

// B** (Branch)

fn bcs<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    if cpu.status_flags.carry {
        branch(cpu, operand.operand_addr())?;
    }
    Ok(())
}

fn bcc<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    if !cpu.status_flags.carry {
        branch(cpu, operand.operand_addr())?;
    }
    Ok(())
}

fn beq<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    if cpu.status_flags.zero {
        branch(cpu, operand.operand_addr())?;
    }
    Ok(())
}

fn bne<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    if !cpu.status_flags.zero {
        branch(cpu, operand.operand_addr())?;
    }
    Ok(())
}

fn bmi<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    if cpu.status_flags.negative {
        branch(cpu, operand.operand_addr())?;
    }
    Ok(())
}

fn bpl<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    if !cpu.status_flags.negative {
        branch(cpu, operand.operand_addr())?;
    }
    Ok(())
}

fn bvs<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    if cpu.status_flags.overflow {
        branch(cpu, operand.operand_addr())?;
    }
    Ok(())
}

fn bvc<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    if !cpu.status_flags.overflow {
        branch(cpu, operand.operand_addr())?;
    }
    Ok(())
}

// PH* (Push), PL* (Pull)

fn pha<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.stack_push(cpu.a)
}

fn php<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    let mut value = cpu.status_flags;
    value.break_flag = true;
    cpu.stack_push(value.bits())
}

fn pla<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.a = cpu.stack_pop()?;
    update_negative_zero_flags(cpu, cpu.a);
    Ok(())
}

fn plp<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    pop_status_flags(cpu)?;
    Ok(())
}

// add / sub

fn adc<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    let carry = cpu.status_flags.carry as u16;
    let value = operand.load_operand(cpu)?;
    let result = cpu.a as u16 + value as u16 + carry;

    // TODO: Learn the details behind the C and V flags and how they differ.
    cpu.status_flags.carry = result > 0xFF;
    cpu.status_flags.overflow = (value ^ result as u8) & (result as u8 ^ cpu.a) & 0x80 != 0;
    cpu.a = result as u8;
    update_negative_zero_flags(cpu, cpu.a);
    Ok(())
}

fn sbc<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    let carry = cpu.status_flags.carry as u16;
    let value = (operand.load_operand(cpu)? as i8)
        .wrapping_neg()
        .wrapping_sub(1) as u8;
    let result = cpu.a as u16 + value as u16 + carry;

    cpu.status_flags.carry = result > 0xFF;
    cpu.status_flags.overflow = (value ^ result as u8) & (result as u8 ^ cpu.a) & 0x80 != 0;
    cpu.a = result as u8;
    update_negative_zero_flags(cpu, cpu.a);
    Ok(())
}

// Bit-wise operations

fn and<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    cpu.a &= operand.load_operand(cpu)?;
    update_negative_zero_flags(cpu, cpu.a);
    Ok(())
}

fn ora<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    cpu.a |= operand.load_operand(cpu)?;
    update_negative_zero_flags(cpu, cpu.a);
    Ok(())
}

fn eor<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    cpu.a ^= operand.load_operand(cpu)?;
    update_negative_zero_flags(cpu, cpu.a);
    Ok(())
}

// C** (Compare)

fn cmp<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    let (value, overflow) = cpu.a.overflowing_sub(operand.load_operand(cpu)?);
    update_negative_zero_flags(cpu, value);
    cpu.status_flags.carry = !overflow;
    Ok(())
}

fn cpx<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    let (value, overflow) = cpu.x.overflowing_sub(operand.load_operand(cpu)?);
    update_negative_zero_flags(cpu, value);
    cpu.status_flags.carry = !overflow;
    Ok(())
}

fn cpy<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    let (value, overflow) = cpu.y.overflowing_sub(operand.load_operand(cpu)?);
    update_negative_zero_flags(cpu, value);
    cpu.status_flags.carry = !overflow;
    Ok(())
}

// Shifts

fn lsr<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    let value = operand.load_operand(cpu)?;
    let (result, _) = value.overflowing_shr(1);
    operand.store_operand(cpu, result)?;
    update_negative_zero_flags(cpu, result);
    cpu.status_flags.carry = (value & 0x01) != 0;
    Ok(())
}

fn asl<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    let value = operand.load_operand(cpu)?;
    let (result, _) = value.overflowing_shl(1);
    operand.store_operand(cpu, result)?;
    update_negative_zero_flags(cpu, result);
    cpu.status_flags.carry = (value & 0x80) != 0;
    Ok(())
}

fn ror<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    let operand2 = operand.load_operand(cpu)?;
    let (mut result, _) = operand2.overflowing_shr(1);
    if cpu.status_flags.carry {
        result |= 0b1000_0000;
    }
    operand.store_operand(cpu, result)?;
    update_negative_zero_flags(cpu, result);
    cpu.status_flags.carry = (operand2 & 0x01) != 0;
    Ok(())
}

fn rol<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    let operand2 = operand.load_operand(cpu)?;
    let (mut result, _) = operand2.overflowing_shl(1);
    if cpu.status_flags.carry {
        result |= 0b0000_0001;
    }
    operand.store_operand(cpu, result)?;
    update_negative_zero_flags(cpu, result);
    cpu.status_flags.carry = (operand2 & 0x80) != 0;
    Ok(())
}

// Register Transfers

fn txa<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.a = cpu.x;
    update_negative_zero_flags(cpu, cpu.a);
    Ok(())
}

fn tax<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.x = cpu.a;
    update_negative_zero_flags(cpu, cpu.x);
    Ok(())
}

fn tay<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.y = cpu.a;
    update_negative_zero_flags(cpu, cpu.y);
    Ok(())
}

fn tya<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.a = cpu.y;
    update_negative_zero_flags(cpu, cpu.a);
    Ok(())
}

fn tsx<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.x = cpu.sp;
    update_negative_zero_flags(cpu, cpu.x);
    Ok(())
}

fn txs<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.sp = cpu.x;
    Ok(())
}

// Misc Operations

fn bit<AM: Operand>(cpu: &mut Cpu, operand: AM) -> Result<()> {
    let value = operand.load_operand(cpu)?;
    let flags = StatusFlags::from_bits(value);
    cpu.status_flags.negative = flags.negative;
    cpu.status_flags.overflow = flags.overflow;
    cpu.status_flags.zero = (value & cpu.a) == 0;
    Ok(())
}

fn hlt<AM: Operand>(cpu: &mut Cpu, _operand: AM) -> Result<()> {
    cpu.halt = true;
    Ok(())
}

fn sei<AM: Operand>(_cpu: &mut Cpu, _operand: AM) -> Result<()> {
    Ok(())
}

fn nop<AM: Operand>(_cpu: &mut Cpu, _operand: AM) -> Result<()> {
    Ok(())
}

fn ill<AM: Operand>(_cpu: &mut Cpu, _operand: AM) -> Result<()> {
    Err(anyhow!("Invalid Opcode"))
}
