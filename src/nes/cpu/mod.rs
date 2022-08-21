mod operations;

use std::{cell::RefCell, rc::Rc};

pub use operations::Operation;

use super::{apu::Apu, cartridge::Cartridge, memory_map::MemoryMap, ppu::Ppu};
use anyhow::Result;
use bitflags::bitflags;

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

pub const RAM_START_ADDR: u16 = 0x0000;
pub const RAM_END_ADDR: u16 = 0x1FFF;

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

    pub ram: [u8; 0x2000],
    pub cartridge: Rc<RefCell<Cartridge>>,
    pub apu: Rc<RefCell<Apu>>,
    pub ppu: Rc<RefCell<Ppu>>,
}

impl Cpu {
    const STACK_ADDR: u16 = 0x0100;

    pub fn new(
        cartridge: Rc<RefCell<Cartridge>>,
        apu: Rc<RefCell<Apu>>,
        ppu: Rc<RefCell<Ppu>>,
    ) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            status_flags: StatusFlags::from_bits_truncate(0x24),
            program_counter: 0,
            halt: false,
            sp: 0xFD,
            ram: [0; 0x2000],
            cartridge,
            apu,
            ppu,
        }
    }

    pub fn tick(&mut self, _clock: u64) -> Result<bool> {
        self.execute_one()
    }

    pub fn execute_one(&mut self) -> Result<bool> {
        let operation = self.next_operation()?;
        operation.execute(self);
        Ok(!self.halt)
    }

    fn next_operation(&mut self) -> Result<Operation> {
        let operation = Operation::read(self, self.program_counter)?;
        self.program_counter += operation.size() as u16;
        Ok(operation)
    }

    fn stack_push(&mut self, value: u8) {
        self.write(Self::STACK_ADDR + self.sp as u16, value);
        self.sp -= 1;
    }

    fn stack_pop(&mut self) -> u8 {
        self.sp += 1;
        self.read(Self::STACK_ADDR + self.sp as u16)
    }

    pub fn read_stack(&self) -> impl Iterator<Item = u8> + '_ {
        let stack_entries = 0xFF_u16 - self.sp as u16;
        self.slice(Self::STACK_ADDR + self.sp as u16 + 1, stack_entries)
    }

    pub fn print_stack(&self) {
        let formatted: Vec<String> = self.read_stack().map(|s| format!("{:02X}", s)).collect();
        println!("{:?}", formatted);
    }

    pub fn slice(&self, addr: u16, length: u16) -> impl Iterator<Item = u8> + '_ {
        (addr..(addr + length)).map(|addr| self.read(addr))
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.read(addr), self.read(addr + 1)])
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            RAM_START_ADDR..=RAM_END_ADDR => self.ram[addr as usize & 0b0000_0111_1111_1111],
            Cartridge::START_ADDR..=Cartridge::END_ADDR => self.cartridge.borrow().read(addr),
            Apu::START_ADDR..=Apu::END_ADDR => self.apu.borrow().read(addr),
            Ppu::START_ADDR..=Ppu::END_ADDR => self.ppu.borrow().read(addr),
            _ => panic!("Warning. Illegal read from: ${:04X}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            RAM_START_ADDR..=RAM_END_ADDR => {
                self.ram[addr as usize & 0b0000_0111_1111_1111] = value
            }
            Cartridge::START_ADDR..=Cartridge::END_ADDR => {
                self.cartridge.borrow_mut().write(addr, value)
            }
            Apu::START_ADDR..=Apu::END_ADDR => self.apu.borrow_mut().write(addr, value),
            Ppu::START_ADDR..=Ppu::END_ADDR => self.ppu.borrow_mut().write(addr, value),
            _ => panic!("Warning. Illegal write to: ${:04X}", addr),
        }
    }
}
