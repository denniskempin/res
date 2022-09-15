mod operations;

use std::cell::RefCell;
use std::rc::Rc;

pub use operations::Operation;

use super::apu::Apu;
use super::cartridge::Cartridge;
use super::joypad::Joypad;
use super::ppu::Ppu;
use anyhow::Result;

////////////////////////////////////////////////////////////////////////////////
// CpuBus

pub struct CpuBus {
    pub ram: [u8; 0x2000],
    pub cartridge: Rc<RefCell<Cartridge>>,
    pub apu: Apu,
    pub ppu: Ppu,
    pub joypad0: Joypad,
    pub joypad1: Joypad,
}

impl Default for CpuBus {
    fn default() -> Self {
        let cartridge = Rc::new(RefCell::new(Cartridge::default()));
        Self {
            ram: [0; 0x2000],
            cartridge: cartridge.clone(),
            apu: Default::default(),
            ppu: Ppu::new(cartridge),
            joypad0: Joypad::default(),
            joypad1: Joypad::default(),
        }
    }
}

impl CpuBus {
    pub fn tick(&mut self, cpu_cycles: usize) {
        self.ppu.tick(cpu_cycles * 3);
    }

    pub fn poll_nmi_interrupt(&mut self) -> bool {
        self.ppu.poll_nmi_interrupt()
    }

    /// Read a single byte from the bus. Note that reads require a mutable bus
    /// as they may have side-effects.
    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0b0000_0111_1111_1111],
            0x2000..=0x3FFF => self.ppu.cpu_bus_read(addr),
            0x4000..=0x4015 => self.apu.cpu_bus_read(addr),
            0x4016 => self.joypad0.cpu_bus_read(),
            0x4017 => self.joypad1.cpu_bus_read(),
            0x8000..=0xFFFF => self.cartridge.borrow_mut().cpu_bus_read(addr),
            _ => {
                println!("Warning. Illegal read from: ${:04X}", addr);
                0
            }
        }
    }

    /// Reads a little endian u16 word from the bus.
    pub fn read_u16(&mut self, addr: u16) -> u16 {
        u16::from_le_bytes([self.read(addr), self.read(addr + 1)])
    }

    pub fn zero_page_read(&mut self, addr: u8) -> u8 {
        self.read(addr as u16)
    }

    pub fn zero_page_read_u16(&mut self, addr: u8) -> u16 {
        u16::from_le_bytes([
            self.zero_page_read(addr),
            self.zero_page_read(addr.wrapping_add(1)),
        ])
    }

    /// Write a single byte to the bus.
    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0b0000_0111_1111_1111] = value,
            0x2000..=0x3FFF => self.ppu.cpu_bus_write(addr, value),
            0x4000..=0x4013 => self.apu.cpu_bus_write(addr, value),
            0x4014 => self.oam_dma(value),
            0x4015 => self.apu.cpu_bus_write(0x4015, value),
            0x4016 => self.joypad0.cpu_bus_write(value),
            0x4017 => self.joypad1.cpu_bus_write(value),
            0x6000..=0xFFFF => self.cartridge.borrow_mut().cpu_bus_write(addr, value),
            _ => panic!("Warning. Illegal write to: ${:04X}", addr),
        }
    }

    /// Allows immutable reads from the bus for display/debug purposes.
    pub fn peek(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0b0000_0111_1111_1111],
            0x2000..=0x3FFF => self.ppu.cpu_bus_peek(addr),
            0x4000..=0x4013 => self.apu.cpu_bus_peek(addr),
            0x4014 => 0,
            0x4015 => self.apu.cpu_bus_peek(0x4015),
            0x4016 => self.joypad0.cpu_bus_peek(),
            0x4017 => self.joypad1.cpu_bus_peek(),
            0x6000..=0xFFFF => self.cartridge.borrow().cpu_bus_peek(addr),
            _ => panic!("Warning. Illegal peek from: ${:04X}", addr),
        }
    }

    /// Peeks at a range of bytes from the bus
    pub fn peek_slice(&self, addr: u16, length: u16) -> impl Iterator<Item = u8> + '_ {
        (addr..(addr + length)).map(|addr| self.peek(addr))
    }

    /// Peeks at a little endian u16 word from the bus.
    pub fn peek_u16(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.peek(addr), self.peek(addr.wrapping_add(1))])
    }

    pub fn zero_page_peek(&self, addr: u8) -> u8 {
        self.peek(addr as u16)
    }

    pub fn zero_page_peek_u16(&self, addr: u8) -> u16 {
        u16::from_le_bytes([
            self.zero_page_peek(addr),
            self.zero_page_peek(addr.wrapping_add(1)),
        ])
    }

    pub fn oam_dma(&mut self, memory_page: u8) {
        let start_addr = (memory_page as u16) << 8;
        for i in 0x00..=0xFF_u8 {
            let value = self.read(start_addr + i as u16);
            self.ppu.write_oam(i, value);
        }
        // Hack.. we should be advancing the CPU clock, but don't have access
        // to it here. Instead just advance everything else on the bus.
        // Ideally, the bus could tell the CPU how long a read/write took.
        self.tick(512);
    }
}

////////////////////////////////////////////////////////////////////////////////
// StatusFlags

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct StatusFlags {
    carry: bool,
    zero: bool,
    interrupt: bool,
    decimal: bool,
    break_flag: bool,
    unused: bool,
    overflow: bool,
    negative: bool,
}

impl StatusFlags {
    pub fn bits(&self) -> u8 {
        (self.carry as u8)
            | (self.zero as u8) << 1
            | (self.interrupt as u8) << 2
            | (self.decimal as u8) << 3
            | (self.break_flag as u8) << 4
            | (self.unused as u8) << 5
            | (self.overflow as u8) << 6
            | (self.negative as u8) << 7
    }

    pub fn from_bits(bits: u8) -> StatusFlags {
        StatusFlags {
            carry: bits & 0b0000_0001 > 0,
            zero: bits & 0b0000_0010 > 0,
            interrupt: bits & 0b0000_0100 > 0,
            decimal: bits & 0b0000_1000 > 0,
            break_flag: bits & 0b0001_0000 > 0,
            unused: bits & 0b0010_0000 > 0,
            overflow: bits & 0b0100_0000 > 0,
            negative: bits & 0b1000_0000 > 0,
        }
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

pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub status_flags: StatusFlags,
    pub program_counter: u16,
    pub halt: bool,
    pub sp: u8,
    pub cycle: usize,

    pub bus: CpuBus,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            status_flags: StatusFlags::from_bits(0x24),
            program_counter: 0,
            halt: false,
            sp: 0xFD,
            cycle: 0,
            bus: Default::default(),
        }
    }
}

impl Cpu {
    const STACK_ADDR: u16 = 0x0100;

    pub fn new(bus: CpuBus) -> Self {
        Self {
            bus,
            ..Default::default()
        }
    }

    pub fn boot(&mut self) {
        self.cycle += 7;
        self.bus.tick(7);
    }

    pub fn execute_one(&mut self) -> Result<bool> {
        let operation = self.next_operation()?;
        operation.execute(self);
        if self.bus.poll_nmi_interrupt() {
            self.stack_push_u16(self.program_counter);
            self.stack_push(self.status_flags.bits());
            self.status_flags.interrupt = true;
            self.program_counter = self.bus.read_u16(InterruptVector::Nmi as u16);
            self.advance_clock(2);
        }
        Ok(!self.halt)
    }

    pub fn advance_clock(&mut self, cycles: usize) {
        self.cycle += cycles;
        self.bus.tick(cycles);
    }

    fn next_operation(&mut self) -> Result<Operation> {
        let operation = Operation::load(self, self.program_counter)?;
        self.program_counter += operation.size() as u16;
        Ok(operation)
    }

    fn stack_push_u16(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.stack_push(bytes[1]);
        self.stack_push(bytes[0]);
    }

    fn stack_pop_u16(&mut self) -> u16 {
        u16::from_le_bytes([self.stack_pop(), self.stack_pop()])
    }

    fn stack_push(&mut self, value: u8) {
        self.write(Self::STACK_ADDR + self.sp as u16, value);
        self.sp -= 1;
    }

    fn stack_pop(&mut self) -> u8 {
        self.sp += 1;
        self.read(Self::STACK_ADDR + self.sp as u16)
    }

    pub fn peek_stack(&mut self) -> impl Iterator<Item = u8> + '_ {
        let stack_entries = 0xFF_u16 - self.sp as u16;
        self.bus
            .peek_slice(Self::STACK_ADDR + self.sp as u16 + 1, stack_entries)
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        self.advance_clock(1);
        self.bus.read(addr)
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        self.advance_clock(1);
        self.bus.write(addr, value)
    }

    pub fn read_u16(&mut self, addr: u16) -> u16 {
        u16::from_le_bytes([self.read(addr), self.read(addr + 1)])
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
    fn advance_clock(&mut self, cycles: usize);
    fn read_or_peek(&mut self, addr: u16) -> u8;
    fn read_or_peek_u16(&mut self, addr: u16) -> u16;
}

impl<'a> MaybeMutableCpu for MutableCpuWrapper<'a> {
    fn immutable(&self) -> &Cpu {
        self.cpu
    }

    fn advance_clock(&mut self, cycles: usize) {
        self.cpu.advance_clock(cycles);
    }

    fn read_or_peek(&mut self, addr: u16) -> u8 {
        self.cpu.read(addr)
    }

    fn read_or_peek_u16(&mut self, addr: u16) -> u16 {
        self.cpu.read_u16(addr)
    }
}

impl<'a> MaybeMutableCpu for ImmutableCpuWrapper<'a> {
    fn immutable(&self) -> &Cpu {
        self.cpu
    }

    fn advance_clock(&mut self, _cycles: usize) {}

    fn read_or_peek(&mut self, addr: u16) -> u8 {
        self.cpu.bus.peek(addr)
    }

    fn read_or_peek_u16(&mut self, addr: u16) -> u16 {
        self.cpu.bus.peek_u16(addr)
    }
}
