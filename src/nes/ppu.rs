use anyhow::Result;
use bitflags::bitflags;
use image::GenericImage;
use image::GenericImageView;
use image::ImageBuffer;
use image::Rgb;
use image::RgbImage;
use image::SubImage;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

use super::cartridge::Cartridge;

const CONTROL_REGISTER_ADDR: u16 = 0x2000;
const STATUS_REGISTER_ADDR: u16 = 0x2002;
const ADDRESS_REGISTER_ADDR: u16 = 0x2006;
const DATA_REGISTER_ADDR: u16 = 0x2007;

pub struct Ppu {
    pub cartridge: Rc<RefCell<Cartridge>>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
    pub internal_data_buffer: u8,
    pub cycle: usize,
    pub scanline: usize,

    pub control_register: ControlRegister,
    pub status_register: StatusRegister,
    pub address_register: AddressRegister,

    pub nmi_interrupt: bool,
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
            cycle: 0,
            scanline: 0,

            control_register: ControlRegister::default(),
            status_register: StatusRegister::default(),
            address_register: AddressRegister::default(),

            nmi_interrupt: false,
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.cycle += cycles;
        if self.cycle >= 341 {
            self.cycle -= 341;
            self.scanline += 1;

            if self.scanline == 241 {
                self.status_register.insert(StatusRegister::VBLANK_STARTED);
                if self
                    .control_register
                    .contains(ControlRegister::GENERATE_NMI)
                {
                    self.nmi_interrupt = true;
                }
            }

            if self.scanline == 261 {
                self.status_register.remove(StatusRegister::VBLANK_STARTED);
            }

            if self.scanline >= 262 {
                self.scanline = 0;
            }
        }
    }

    pub fn get_palette_entry(&self, palette_id: usize, entry: usize) -> Rgb<u8> {
        if entry == 0 {
            SYSTEM_PALLETE[self.read_ppu_memory(0x3F00) as usize]
        } else {
            let addr = 0x3F00 + palette_id as u16 * 4 + entry as u16;
            SYSTEM_PALLETE[self.read_ppu_memory(addr) as usize]
        }
    }

    pub fn read_ppu_memory(&self, addr: u16) -> u8 {
        match addr {
            0..=0x1FFF => self.cartridge.borrow().chr[addr as usize],
            0x2000..=0x3FFF => self.vram[(addr - 0x2000) as usize % self.vram.len()],
            _ => panic!("Invalid PPU address read {addr:04X}"),
        }
    }

    pub fn write_ppu_memory(&mut self, addr: u16, value: u8) {
        match addr {
            0..=0x1FFF => self.cartridge.borrow_mut().chr[addr as usize] = value,
            0x2000..=0x3FFF => self.vram[(addr - 0x2000) as usize % self.vram.len()] = value,
            _ => panic!("Invalid PPU address read {addr:04X}"),
        };
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

    pub fn read_status_register(&mut self) -> u8 {
        let status = self.status_register.bits;
        self.status_register.remove(StatusRegister::VBLANK_STARTED);
        status
    }

    pub fn cpu_bus_peek(&self, addr: u16) -> u8 {
        match addr {
            CONTROL_REGISTER_ADDR => self.control_register.bits,
            STATUS_REGISTER_ADDR => self.status_register.bits,
            _ => {
                println!("Warning: Invalid peek/read from PPU at {addr:04X}");
                0
            }
        }
    }

    pub fn cpu_bus_read(&mut self, addr: u16) -> u8 {
        match addr {
            DATA_REGISTER_ADDR => self.read_data_register(),
            STATUS_REGISTER_ADDR => self.read_status_register(),
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

    pub fn poll_nmi_interrupt(&mut self) -> bool {
        if self.nmi_interrupt {
            self.nmi_interrupt = false;
            true
        } else {
            false
        }
    }

    pub fn render_chr(&self, palette_id: usize) -> Result<RgbImage> {
        let mut rendered: RgbImage = ImageBuffer::new(32 * 8, 16 * 8);
        self.render_tile_bank(0, palette_id, &mut rendered.sub_image(0, 0, 16 * 8, 16 * 8));
        self.render_tile_bank(
            1,
            palette_id,
            &mut rendered.sub_image(16 * 8, 0, 16 * 8, 16 * 8),
        );
        Ok(rendered)
    }

    pub fn render_tile_bank(
        &self,
        bank: usize,
        palette_id: usize,
        target: &mut SubImage<&mut RgbImage>,
    ) {
        for y in 0..16 {
            for x in 0..16 {
                let tile_num = (y * 16) + x;
                self.render_tile(
                    bank,
                    tile_num,
                    palette_id,
                    &mut target.sub_image((x * 8) as u32, (y * 8) as u32, 8, 8),
                );
            }
        }
    }

    pub fn render_tile(
        &self,
        bank: usize,
        tile_num: usize,
        palette_id: usize,
        target: &mut SubImage<&mut RgbImage>,
    ) {
        let bank_addr = (0x1000 * bank) as u16;
        let tile_addr = bank_addr + (tile_num * 16) as u16;
        let tile: Vec<u8> = (tile_addr..=(tile_addr + 15))
            .map(|addr| self.read_ppu_memory(addr))
            .collect();
        for y in 0..8 {
            let mut lower = tile[y];
            let mut upper = tile[y + 8];
            for x in (0..8_usize).rev() {
                let value = (1 & upper) << 1 | (1 & lower);
                let rgb = self.get_palette_entry(palette_id, value as usize);
                target.put_pixel(x as u32, y as u32, rgb);
                upper >>= 1;
                lower >>= 1;
            }
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
pub static SYSTEM_PALLETE: [Rgb<u8>; 64] = [
    Rgb([0x80, 0x80, 0x80]),
    Rgb([0x00, 0x3D, 0xA6]),
    Rgb([0x00, 0x12, 0xB0]),
    Rgb([0x44, 0x00, 0x96]),
    Rgb([0xA1, 0x00, 0x5E]),
    Rgb([0xC7, 0x00, 0x28]),
    Rgb([0xBA, 0x06, 0x00]),
    Rgb([0x8C, 0x17, 0x00]),
    Rgb([0x5C, 0x2F, 0x00]),
    Rgb([0x10, 0x45, 0x00]),
    Rgb([0x05, 0x4A, 0x00]),
    Rgb([0x00, 0x47, 0x2E]),
    Rgb([0x00, 0x41, 0x66]),
    Rgb([0x00, 0x00, 0x00]),
    Rgb([0x05, 0x05, 0x05]),
    Rgb([0x05, 0x05, 0x05]),
    Rgb([0xC7, 0xC7, 0xC7]),
    Rgb([0x00, 0x77, 0xFF]),
    Rgb([0x21, 0x55, 0xFF]),
    Rgb([0x82, 0x37, 0xFA]),
    Rgb([0xEB, 0x2F, 0xB5]),
    Rgb([0xFF, 0x29, 0x50]),
    Rgb([0xFF, 0x22, 0x00]),
    Rgb([0xD6, 0x32, 0x00]),
    Rgb([0xC4, 0x62, 0x00]),
    Rgb([0x35, 0x80, 0x00]),
    Rgb([0x05, 0x8F, 0x00]),
    Rgb([0x00, 0x8A, 0x55]),
    Rgb([0x00, 0x99, 0xCC]),
    Rgb([0x21, 0x21, 0x21]),
    Rgb([0x09, 0x09, 0x09]),
    Rgb([0x09, 0x09, 0x09]),
    Rgb([0xFF, 0xFF, 0xFF]),
    Rgb([0x0F, 0xD7, 0xFF]),
    Rgb([0x69, 0xA2, 0xFF]),
    Rgb([0xD4, 0x80, 0xFF]),
    Rgb([0xFF, 0x45, 0xF3]),
    Rgb([0xFF, 0x61, 0x8B]),
    Rgb([0xFF, 0x88, 0x33]),
    Rgb([0xFF, 0x9C, 0x12]),
    Rgb([0xFA, 0xBC, 0x20]),
    Rgb([0x9F, 0xE3, 0x0E]),
    Rgb([0x2B, 0xF0, 0x35]),
    Rgb([0x0C, 0xF0, 0xA4]),
    Rgb([0x05, 0xFB, 0xFF]),
    Rgb([0x5E, 0x5E, 0x5E]),
    Rgb([0x0D, 0x0D, 0x0D]),
    Rgb([0x0D, 0x0D, 0x0D]),
    Rgb([0xFF, 0xFF, 0xFF]),
    Rgb([0xA6, 0xFC, 0xFF]),
    Rgb([0xB3, 0xEC, 0xFF]),
    Rgb([0xDA, 0xAB, 0xEB]),
    Rgb([0xFF, 0xA8, 0xF9]),
    Rgb([0xFF, 0xAB, 0xB3]),
    Rgb([0xFF, 0xD2, 0xB0]),
    Rgb([0xFF, 0xEF, 0xA6]),
    Rgb([0xFF, 0xF7, 0x9C]),
    Rgb([0xD7, 0xE8, 0x95]),
    Rgb([0xA6, 0xED, 0xAF]),
    Rgb([0xA2, 0xF2, 0xDA]),
    Rgb([0x99, 0xFF, 0xFC]),
    Rgb([0xDD, 0xDD, 0xDD]),
    Rgb([0x11, 0x11, 0x11]),
    Rgb([0x11, 0x11, 0x11]),
];

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
