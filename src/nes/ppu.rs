use bitflags::bitflags;
use std::cell::RefCell;
use std::rc::Rc;

use super::cartridge::Cartridge;

const CONTROL_REGISTER_ADDR: u16 = 0x2000;
const ADDRESS_REGISTER_ADDR: u16 = 0x2006;
const DATA_REGISTER_ADDR: u16 = 0x2007;

pub struct Ppu {
    pub cartridge: Rc<RefCell<Cartridge>>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
    pub internal_data_buffer: u8,

    pub control_register: ControlRegister,
    pub status_register: StatusRegister,
    pub address_register: AddressRegister,
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new(Rc::new(RefCell::new(Cartridge::default())))
    }
}

impl Ppu {
    pub fn new(cartridge: Rc<RefCell<Cartridge>>) -> Self {
        Self {
            cartridge,
            vram: [0; 2048],
            oam_data: [0; 256],
            palette_table: [0; 32],
            internal_data_buffer: 0,
            control_register: ControlRegister::default(),
            status_register: StatusRegister::default(),
            address_register: AddressRegister::default(),
        }
    }

    pub fn tick(&mut self) {}

    pub fn read_ppu_memory(&self, addr: u16) -> u8 {
        match addr {
            0..=0x1FFF => self.cartridge.borrow().chr[addr as usize],
            0x2000..=0x3FFF => self.vram[addr as usize % self.vram.len()],
            _ => panic!("Invalid PPU address read {addr:04X}"),
        }
    }

    pub fn read_data_register(&mut self) -> u8 {
        let addr = self.address_register.address();
        let inc = if self
            .control_register
            .contains(ControlRegister::VRAM_ADD_INCREMENT)
        {
            32
        } else {
            1
        };
        self.address_register.increment(inc);
        let buffer = self.internal_data_buffer;
        self.internal_data_buffer = self.read_ppu_memory(addr);
        buffer
    }

    pub fn cpu_bus_peek(&self, addr: u16) -> u8 {
        match addr {
            CONTROL_REGISTER_ADDR => self.control_register.bits,
            _ => {
                println!("Warning: Invalid peek/read from PPU at {addr:04X}");
                0
            }
        }
    }

    pub fn cpu_bus_read(&mut self, addr: u16) -> u8 {
        match addr {
            DATA_REGISTER_ADDR => self.read_data_register(),
            _ => self.cpu_bus_peek(addr),
        }
    }

    pub fn cpu_bus_write(&mut self, addr: u16, value: u8) {
        match addr {
            CONTROL_REGISTER_ADDR => {
                self.control_register = ControlRegister::from_bits_truncate(value)
            }
            ADDRESS_REGISTER_ADDR => self.address_register.write(value),
            _ => println!("Warning: Invalid write to PPU at {addr:04X}"),
        }
    }
}

#[derive(Default)]
pub struct AddressRegister {
    value: [u8; 2],
    write_high: bool,
}

impl AddressRegister {
    pub fn write(&mut self, value: u8) {
        if self.write_high {
            self.value[1] = value;
        } else {
            self.value[0] = value;
        }
        self.write_high = !self.write_high;
    }

    pub fn increment(&mut self, inc: u8) {
        self.set_address(self.address().wrapping_add(inc as u16));
    }

    pub fn set_address(&mut self, addr: u16) {
        self.value = addr.to_be_bytes();
    }

    pub fn address(&self) -> u16 {
        u16::from_be_bytes(self.value)
    }
}

bitflags! {
    #[derive(Default)]
    pub struct ControlRegister: u8 {
       const NAMETABLE1              = 0b00000001;
       const NAMETABLE2              = 0b00000010;
       const VRAM_ADD_INCREMENT      = 0b00000100;
       const SPRITE_PATTERN_ADDR     = 0b00001000;
       const BACKROUND_PATTERN_ADDR  = 0b00010000;
       const SPRITE_SIZE             = 0b00100000;
       const MASTER_SLAVE_SELECT     = 0b01000000;
       const GENERATE_NMI            = 0b10000000;
   }
}

bitflags! {
    #[derive(Default)]
    pub struct StatusRegister: u8 {
        const NOTUSED          = 0b00000001;
        const NOTUSED2         = 0b00000010;
        const NOTUSED3         = 0b00000100;
        const NOTUSED4         = 0b00001000;
        const NOTUSED5         = 0b00010000;
        const SPRITE_OVERFLOW  = 0b00100000;
        const SPRITE_ZERO_HIT  = 0b01000000;
        const VBLANK_STARTED   = 0b10000000;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_address_register_write() {
        let mut reg = AddressRegister::default();
        reg.write(0x01);
        reg.write(0x02);
        assert_eq!(reg.address(), 0x0102);
        reg.write(0x03);
        assert_eq!(reg.address(), 0x0302);
    }

    #[test]
    pub fn test_address_register_increment() {
        let mut reg = AddressRegister::default();
        reg.write(0xFF);
        reg.write(0xFF);
        assert_eq!(reg.address(), 0xFFFF);
        reg.increment(0x02);
        assert_eq!(reg.address(), 0x0001);
    }

    #[test]
    pub fn test_data_register() {
        let mut ppu = Ppu::default();
        let mut chr = vec![0; 0x2000];
        chr[0x1000] = 0x12;
        chr[0x1001] = 0x34;
        ppu.cartridge.borrow_mut().chr = chr;

        ppu.cpu_bus_write(ADDRESS_REGISTER_ADDR, 0x10);
        ppu.cpu_bus_write(ADDRESS_REGISTER_ADDR, 0x00);
        assert_eq!(ppu.cpu_bus_read(DATA_REGISTER_ADDR), 0x00);
        assert_eq!(ppu.cpu_bus_read(DATA_REGISTER_ADDR), 0x12);
        assert_eq!(ppu.cpu_bus_read(DATA_REGISTER_ADDR), 0x34);
    }
}
