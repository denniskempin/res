mod operations;

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::anyhow;
use anyhow::Result;
use bincode::Decode;
use bincode::Encode;
pub use operations::Operation;
use packed_struct::prelude::*;

use super::apu::Apu;
use super::cartridge::Cartridge;
use super::debugger::Debugger;
use super::debugger::MemoryAccess;
use super::joypad::Joypad;
use super::ppu::Ppu;

////////////////////////////////////////////////////////////////////////////////
// CpuBus

pub trait CpuBus {
    fn advance_clock(&mut self, cpu_cycles: usize) -> Result<()>;
    fn poll_nmi_interrupt(&mut self) -> bool;
    fn peek(&self, addr: u16) -> Option<u8>;
    fn read(&mut self, addr: u16) -> Result<u8>;
    fn write(&mut self, addr: u16, value: u8) -> Result<()>;
    fn oam_dma(&mut self, memory_page: u8) -> Result<()>;

    /// Peeks at a little endian u16 word from the bus.
    fn peek_u16(&self, addr: u16) -> Option<u16> {
        Some(u16::from_le_bytes([
            self.peek(addr)?,
            self.peek(addr.wrapping_add(1))?,
        ]))
    }

    fn zero_page_peek(&self, addr: u8) -> Option<u8> {
        self.peek(addr as u16)
    }

    fn zero_page_peek_u16(&self, addr: u8) -> Option<u16> {
        Some(u16::from_le_bytes([
            self.zero_page_peek(addr)?,
            self.zero_page_peek(addr.wrapping_add(1))?,
        ]))
    }

    fn read_u16(&mut self, addr: u16) -> Result<u16> {
        Ok(u16::from_le_bytes([self.read(addr)?, self.read(addr + 1)?]))
    }

    fn zero_page_read(&mut self, addr: u8) -> Result<u8> {
        self.read(addr as u16)
    }

    fn zero_page_read_u16(&mut self, addr: u8) -> Result<u16> {
        Ok(u16::from_le_bytes([
            self.zero_page_read(addr)?,
            self.zero_page_read(addr.wrapping_add(1))?,
        ]))
    }
}

#[derive(Default, Encode, Decode, Clone)]
pub struct ResCpuBus {
    pub ram: Vec<u8>,
    pub cartridge: Rc<RefCell<Cartridge>>,
    pub apu: Apu,
    pub ppu: Ppu,
    pub joypad0: Joypad,
    pub joypad1: Joypad,
    pub debugger: Rc<RefCell<Debugger>>,
}

impl ResCpuBus {
    pub fn new(debugger: Rc<RefCell<Debugger>>) -> Self {
        let cartridge = Rc::new(RefCell::new(Cartridge::default()));
        Self {
            ram: vec![0; 0x2000],
            debugger,
            ppu: Ppu::new(cartridge.clone()),
            cartridge,
            ..Default::default()
        }
    }

    /// Peeks at a range of bytes from the bus
    pub fn peek_slice(&self, addr: u16, length: u16) -> impl Iterator<Item = Option<u8>> + '_ {
        (addr..(addr + length)).map(|addr| self.peek(addr))
    }
}

impl CpuBus for ResCpuBus {
    fn advance_clock(&mut self, cpu_cycles: usize) -> Result<()> {
        self.ppu.advance_clock(cpu_cycles * 3)?;
        Ok(())
    }

    fn poll_nmi_interrupt(&mut self) -> bool {
        self.ppu.poll_nmi_interrupt()
    }

    /// Allows immutable reads from the bus for display/debug purposes.
    fn peek(&self, addr: u16) -> Option<u8> {
        match addr {
            0x0000..=0x1FFF => Some(self.ram[addr as usize & 0b0000_0111_1111_1111]),
            0x2000..=0x3FFF => Some(self.ppu.cpu_bus_peek(addr)?),
            0x4000..=0x4013 => Some(self.apu.cpu_bus_peek(addr)),
            0x4014 => Some(0),
            0x4015 => Some(self.apu.cpu_bus_peek(0x4015)),
            0x4016 => Some(self.joypad0.cpu_bus_peek()),
            0x4017 => Some(self.joypad1.cpu_bus_peek()),
            0x4020..=0xFFFF => self.cartridge.borrow().cpu_bus_peek(addr),
            _ => None,
        }
    }
    /// Read a single byte from the bus. Note that reads require a mutable bus
    /// as they may have side-effects.
    fn read(&mut self, addr: u16) -> Result<u8> {
        self.debugger
            .borrow_mut()
            .on_cpu_memory_access(MemoryAccess::Read(addr));
        match addr {
            0x0000..=0x1FFF => Ok(self.ram[addr as usize & 0b0000_0111_1111_1111]),
            0x2000..=0x3FFF => Ok(self.ppu.cpu_bus_read(addr)?),
            0x4000..=0x4015 => Ok(self.apu.cpu_bus_read(addr)),
            0x4016 => Ok(self.joypad0.cpu_bus_read()),
            0x4017 => Ok(self.joypad1.cpu_bus_read()),
            0x4020..=0xFFFF => Ok(self.cartridge.borrow_mut().cpu_bus_read(addr)?),
            _ => {
                self.debugger
                    .borrow_mut()
                    .on_cpu_memory_error(MemoryAccess::Read(addr));
                Ok(0)
            }
        }
    }

    fn write(&mut self, addr: u16, value: u8) -> Result<()> {
        self.debugger
            .borrow_mut()
            .on_cpu_memory_access(MemoryAccess::Write(addr, value));
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0b0000_0111_1111_1111] = value,
            0x2000..=0x3FFF => self.ppu.cpu_bus_write(addr, value)?,
            0x4000..=0x4013 => self.apu.cpu_bus_write(addr, value),
            0x4014 => self.oam_dma(value)?,
            0x4015 => self.apu.cpu_bus_write(0x4015, value),
            0x4016 => self.joypad0.cpu_bus_write(value),
            0x4017 => self.joypad1.cpu_bus_write(value),
            0x4020..=0xFFFF => self.cartridge.borrow_mut().cpu_bus_write(addr, value)?,
            _ => self
                .debugger
                .borrow_mut()
                .on_cpu_memory_error(MemoryAccess::Write(addr, value)),
        };
        Ok(())
    }

    fn oam_dma(&mut self, memory_page: u8) -> Result<()> {
        let start_addr = (memory_page as u16) << 8;
        for i in 0x00..=0xFF_u8 {
            let value = self.read(start_addr + i as u16)?;
            self.ppu.oam_data[i as usize] = value;
        }
        // Hack.. we should be advancing the CPU clock, but don't have access
        // to it here. Instead just advance everything else on the bus.
        // Ideally, the bus could tell the CPU how long a read/write took.
        self.advance_clock(512)?;
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// StatusFlags

#[derive(PackedStruct, Encode, Decode, Clone, Debug, Default, Copy, PartialEq)]
#[packed_struct(bit_numbering = "msb0")]
pub struct StatusFlags {
    negative: bool,
    overflow: bool,
    unused: bool,
    break_flag: bool,
    decimal: bool,
    interrupt: bool,
    zero: bool,
    carry: bool,
}

impl StatusFlags {
    pub fn bits(&self) -> u8 {
        self.pack().unwrap()[0]
    }

    pub fn from_bits(bits: u8) -> StatusFlags {
        StatusFlags::unpack(&[bits]).unwrap()
    }

    pub fn pretty_print(&self) -> String {
        let mut chars: Vec<char> = Vec::new();
        chars.push(if self.carry { 'C' } else { '.' });
        chars.push(if self.zero { 'Z' } else { '.' });
        chars.push(if self.interrupt { 'I' } else { '.' });
        chars.push(if self.decimal { 'D' } else { '.' });
        chars.push(if self.break_flag { 'B' } else { '.' });
        chars.push('.');
        chars.push(if self.overflow { 'O' } else { '.' });
        chars.push(if self.overflow { 'N' } else { '.' });
        chars.iter().collect()
    }
}

#[derive(Copy, Clone)]
enum InterruptVector {
    Nmi = 0xFFFA,
    #[allow(dead_code)]
    Reset = 0xFFFC,
    #[allow(dead_code)]
    Irq = 0xFFFE,
}

////////////////////////////////////////////////////////////////////////////////
// Cpu

#[derive(Encode, Decode, Clone, Default)]
pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub status_flags: StatusFlags,
    pub program_counter: u16,
    pub halt: bool,
    pub sp: u8,
    pub cycle: usize,

    pub bus: ResCpuBus,

    pub debugger: Rc<RefCell<Debugger>>,
}

impl Cpu {
    const STACK_ADDR: u16 = 0x0100;

    pub fn new() -> Self {
        let debugger = Rc::new(RefCell::new(Debugger::default()));
        Self {
            bus: ResCpuBus::new(debugger.clone()),
            status_flags: StatusFlags::from_bits(0x24),
            sp: 0xFD,
            debugger,
            ..Default::default()
        }
    }

    pub fn boot(&mut self) -> Result<()> {
        self.cycle += 7;
        self.bus.advance_clock(7)
    }

    pub fn execute_one(&mut self) -> Result<bool> {
        let operation = self.next_operation()?;
        operation.execute(self)?;
        if self.bus.poll_nmi_interrupt() {
            self.stack_push_u16(self.program_counter)?;
            self.stack_push(self.status_flags.bits())?;
            self.status_flags.interrupt = true;
            self.program_counter = self.bus.read_u16(InterruptVector::Nmi as u16).unwrap();
            self.advance_clock(2)?;
        }
        Ok(!self.halt)
    }

    pub fn advance_clock(&mut self, cycles: usize) -> Result<()> {
        self.cycle += cycles;
        self.bus.advance_clock(cycles)
    }

    pub fn next_operation(&mut self) -> Result<Operation> {
        let operation = Operation::load(self, self.program_counter)?;
        self.debugger.borrow_mut().on_instruction(&operation);
        self.program_counter += operation.size() as u16;
        Ok(operation)
    }

    pub fn peek_next_operations(&self, count: usize) -> impl Iterator<Item = u16> + '_ {
        let mut virtual_pc = self.program_counter;
        (0..count).map(move |_| {
            let op = Operation::peek(self, virtual_pc).unwrap_or_default();
            virtual_pc += op.size() as u16;
            op.addr
        })
    }

    fn stack_push_u16(&mut self, value: u16) -> Result<()> {
        let bytes = value.to_le_bytes();
        self.stack_push(bytes[1])?;
        self.stack_push(bytes[0])?;
        Ok(())
    }

    fn stack_pop_u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes([self.stack_pop()?, self.stack_pop()?]))
    }

    fn stack_push(&mut self, value: u8) -> Result<()> {
        self.write(Self::STACK_ADDR + self.sp as u16, value)?;
        if self.sp == 0 {
            return Err(anyhow!("Stack overflow"));
        }
        self.sp -= 1;
        Ok(())
    }

    fn stack_pop(&mut self) -> Result<u8> {
        if self.sp == 0xFF {
            return Err(anyhow!("Popping from empty stack."));
        }
        self.sp += 1;
        self.read(Self::STACK_ADDR + self.sp as u16)
    }

    pub fn peek_stack(&self) -> impl Iterator<Item = u8> + '_ {
        let stack_entries = 0xFF_u16 - self.sp as u16;
        self.bus
            .peek_slice(Self::STACK_ADDR + self.sp as u16 + 1, stack_entries)
            .map(|s| s.unwrap_or(0x00))
    }

    pub fn read(&mut self, addr: u16) -> Result<u8> {
        self.advance_clock(1)?;
        self.bus.read(addr)
    }

    pub fn write(&mut self, addr: u16, value: u8) -> Result<()> {
        self.advance_clock(1)?;
        self.bus.write(addr, value)
    }

    pub fn read_u16(&mut self, addr: u16) -> Result<u16> {
        Ok(u16::from_le_bytes([self.read(addr)?, self.read(addr + 1)?]))
    }

    /// Returns a struct that implements MaybeMutableCpu and performs
    /// mutable reads.
    fn mutable_wrapper(&mut self) -> MutableCpuWrapper {
        MutableCpuWrapper { cpu: self }
    }

    /// Returns a struct that implements MaybeMutableCpu and performs
    /// immutable peeks.
    fn immutable_wrapper(&self) -> ImmutableCpuWrapper {
        ImmutableCpuWrapper { cpu: self }
    }
}

/// Wrapper for abstraction over mutability. See ReadOrPeek.
struct MaybeMutableCpuWrapper<T> {
    cpu: T,
}
type MutableCpuWrapper<'a> = MaybeMutableCpuWrapper<&'a mut Cpu>;
type ImmutableCpuWrapper<'a> = MaybeMutableCpuWrapper<&'a Cpu>;

/// Allows abstraction over mutability for accessing the Cpu.
///
/// This trait is implemented in MutableCpuWrapper to do mutating reads and
/// account for clock advances during reads.
/// It also implemented as ImmutableCpuWrapper, which won't touch the clock
/// and do immutable peek's instead.
///
/// This is used to re-use logic from execution flow in debug output as well
/// (e.g. to display calculated addresses without modifying the CPU state).
trait MaybeMutableCpu {
    fn immutable(&self) -> &Cpu;
    fn advance_clock(&mut self, cycles: usize) -> Result<()>;
    fn read_or_peek(&mut self, addr: u16) -> Result<u8>;
    fn read_or_peek_u16(&mut self, addr: u16) -> Result<u16>;
}

impl<'a> MaybeMutableCpu for MutableCpuWrapper<'a> {
    fn immutable(&self) -> &Cpu {
        self.cpu
    }

    fn advance_clock(&mut self, cycles: usize) -> Result<()> {
        self.cpu.advance_clock(cycles)?;
        Ok(())
    }

    fn read_or_peek(&mut self, addr: u16) -> Result<u8> {
        self.cpu.read(addr)
    }

    fn read_or_peek_u16(&mut self, addr: u16) -> Result<u16> {
        self.cpu.read_u16(addr)
    }
}

impl<'a> MaybeMutableCpu for ImmutableCpuWrapper<'a> {
    fn immutable(&self) -> &Cpu {
        self.cpu
    }

    fn advance_clock(&mut self, _cycles: usize) -> Result<()> {
        Ok(())
    }

    fn read_or_peek(&mut self, addr: u16) -> Result<u8> {
        Ok(self.cpu.bus.peek(addr).unwrap_or_default())
    }

    fn read_or_peek_u16(&mut self, addr: u16) -> Result<u16> {
        Ok(self.cpu.bus.peek_u16(addr).unwrap_or_default())
    }
}
