mod operations;

pub use operations::Operation;

use super::apu::Apu;
use super::cartridge::Cartridge;
use super::ppu::Ppu;
use anyhow::Result;
use bitflags::bitflags;

////////////////////////////////////////////////////////////////////////////////
// CpuBus

pub struct CpuBus {
    pub ram: [u8; 0x2000],
    pub cartridge: Cartridge,
    pub apu: Apu,
    pub ppu: Ppu,
}

impl Default for CpuBus {
    fn default() -> Self {
        Self {
            ram: [0; 0x2000],
            cartridge: Default::default(),
            apu: Default::default(),
            ppu: Default::default(),
        }
    }
}

impl CpuBus {
    pub fn new(cartridge: Cartridge, apu: Apu, ppu: Ppu) -> Self {
        Self {
            ram: [0; 0x2000],
            cartridge,
            apu,
            ppu,
        }
    }

    /// Read a single byte from the bus. Note that reads require a mutable bus
    /// as they may have side-effects.
    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0b0000_0111_1111_1111],
            0x2000..=0x3FFF => self.ppu.cpu_bus_read(addr),
            0x4000..=0x4017 => self.apu.cpu_bus_read(addr),
            0x8000..=0xFFFF => self.cartridge.cpu_bus_read(addr),
            _ => panic!("Warning. Illegal read from: ${:04X}", addr),
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
            0x4000..=0x4017 => self.apu.cpu_bus_write(addr, value),
            0x8000..=0xFFFF => self.cartridge.cpu_bus_write(addr, value),
            _ => panic!("Warning. Illegal write to: ${:04X}", addr),
        }
    }

    /// Allows immutable reads from the bus for display/debug purposes.
    pub fn peek(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0b0000_0111_1111_1111],
            0x2000..=0x3FFF => self.ppu.cpu_bus_peek(addr),
            0x4000..=0x4017 => self.apu.cpu_bus_peek(addr),
            0x8000..=0xFFFF => self.cartridge.cpu_bus_peek(addr),
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
}

////////////////////////////////////////////////////////////////////////////////
// StatusFlags

bitflags! {
    #[derive(Default)]
    pub struct StatusFlags: u8 {
        const NEGATIVE = 0b1000_0000;
        const OVERFLOW = 0b0100_0000;
        const UNUSED = 0b0010_0000;
        const BREAK = 0b0001_0000;
        const DECIMAL = 0b0000_1000;
        const INTERRUPT = 0b0000_0100;
        const ZERO = 0b0000_0010;
        const CARRY = 0b0000_0001;
    }
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

    pub bus: CpuBus,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            status_flags: StatusFlags::from_bits_truncate(0x24),
            program_counter: 0,
            halt: false,
            sp: 0xFD,
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

    pub fn tick(&mut self) -> Result<bool> {
        self.execute_one()
    }

    pub fn execute_one(&mut self) -> Result<bool> {
        let operation = self.next_operation()?;
        operation.execute(self);
        Ok(!self.halt)
    }

    fn next_operation(&mut self) -> Result<Operation> {
        let operation = Operation::load(self, self.program_counter)?;
        self.program_counter += operation.size() as u16;
        Ok(operation)
    }

    fn stack_push(&mut self, value: u8) {
        self.bus.write(Self::STACK_ADDR + self.sp as u16, value);
        self.sp -= 1;
    }

    fn stack_pop(&mut self) -> u8 {
        self.sp += 1;
        self.bus.read(Self::STACK_ADDR + self.sp as u16)
    }

    pub fn peek_stack(&mut self) -> impl Iterator<Item = u8> + '_ {
        let stack_entries = 0xFF_u16 - self.sp as u16;
        self.bus
            .peek_slice(Self::STACK_ADDR + self.sp as u16 + 1, stack_entries)
    }

    pub fn print_stack(&mut self) {
        let formatted: Vec<String> = self.peek_stack().map(|s| format!("{:02X}", s)).collect();
        println!("{:?}", formatted);
    }
}
