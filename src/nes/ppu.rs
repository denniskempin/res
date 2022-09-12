use anyhow::Result;
use bitflags::bitflags;
use image::GenericImage;

use image::ImageBuffer;
use image::Rgba;
use image::RgbaImage;
use image::SubImage;
use std::cell::RefCell;

use std::rc::Rc;

use super::cartridge::Cartridge;

const CONTROL_REGISTER_ADDR: u16 = 0x2000;
const STATUS_REGISTER_ADDR: u16 = 0x2002;
const OAM_ADDR: u16 = 0x2003;
const OAM_DATA: u16 = 0x2004;
const PPU_SCROLL: u16 = 0x2005;
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
    pub oam_addr: u8,
    pub scroll: u8,

    pub control_register: ControlRegister,
    pub status_register: StatusRegister,
    pub address_register: AddressRegister,

    pub nmi_interrupt: bool,
    pub vblank: bool,
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
            oam_addr: 0,
            scroll: 0,

            control_register: ControlRegister::default(),
            status_register: StatusRegister::default(),
            address_register: AddressRegister::default(),

            nmi_interrupt: false,
            vblank: false,
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.cycle += cycles;
        if self.cycle >= 341 {
            self.cycle -= 341;
            self.scanline += 1;

            if self.scanline == 241 {
                self.status_register.insert(StatusRegister::VBLANK_STARTED);
                self.vblank = true;
                if self
                    .control_register
                    .contains(ControlRegister::GENERATE_NMI)
                {
                    self.nmi_interrupt = true;
                }
            }

            if self.scanline == 261 {
                self.status_register.remove(StatusRegister::VBLANK_STARTED);
                self.vblank = false;
            }

            if self.scanline >= 262 {
                self.scanline = 0;
            }
        }
    }

    pub fn get_palette_entry(&self, palette_id: usize, entry: usize) -> Rgba<u8> {
        if entry == 0 {
            SYSTEM_PALLETE[self.read_ppu_memory(0x3F00 + (palette_id as u16 * 4)) as usize]
        } else {
            
            let addr = 0x3F00 + (palette_id as u16 * 4) + entry as u16;
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
            _ => println!("Warning: Invalid PPU address write {addr:04X}"),
        };
    }

    pub fn read_oam(&mut self, addr: u8) -> u8 {
        self.oam_data[addr as usize]
    }

    pub fn write_oam(&mut self, addr: u8, value: u8) {
        self.oam_data[addr as usize] = value;
    }

    fn increment_address_register(&mut self) -> u16 {
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
        addr
    }

    pub fn read_data_register(&mut self) -> u8 {
        let addr = self.increment_address_register();
        let buffer = self.internal_data_buffer;
        self.internal_data_buffer = self.read_ppu_memory(addr);
        buffer
    }

    pub fn write_data_register(&mut self, value: u8) {
        let addr = self.increment_address_register();
        self.write_ppu_memory(addr, value);
    }

    pub fn read_status_register(&mut self) -> u8 {
        let status = self.status_register.bits;
        self.status_register.remove(StatusRegister::VBLANK_STARTED);
        status
    }

    pub fn cpu_bus_peek(&self, addr: u16) -> u8 {
        match addr {
            OAM_ADDR => self.oam_addr,
            OAM_DATA => self.oam_data[self.oam_addr as usize],
            PPU_SCROLL => self.scroll,
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
            OAM_DATA => {
                let value = self.oam_data[self.oam_addr as usize];
                self.oam_addr = self.oam_addr.wrapping_add(1);
                value
            }
            DATA_REGISTER_ADDR => self.read_data_register(),
            STATUS_REGISTER_ADDR => self.read_status_register(),
            _ => self.cpu_bus_peek(addr),
        }
    }

    pub fn cpu_bus_write(&mut self, addr: u16, value: u8) {
        match addr {
            0x2001 => (),
            OAM_ADDR => self.oam_addr = value,
            OAM_DATA => self.oam_data[self.oam_addr as usize] = value,
            PPU_SCROLL => self.scroll = value,
            CONTROL_REGISTER_ADDR => {
                self.control_register = ControlRegister::from_bits_truncate(value)
            }
            ADDRESS_REGISTER_ADDR => self.address_register.write(value),
            DATA_REGISTER_ADDR => self.write_data_register(value),
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

    pub fn render_sprites(&self, target: &mut SubImage<&mut RgbaImage>) {
        for sprite_num in 0..64 {
            let oam_addr = sprite_num * 4;
            let y = self.oam_data[oam_addr + 0];
            if y > 0xEF {
                continue;
            }
            let idx = self.oam_data[oam_addr + 1];
            let attr = self.oam_data[oam_addr + 2];
            let x = self.oam_data[oam_addr + 3];
            let palette_id = attr & 0b0000_0011;
            if x < 31 * 8 && y < 29 * 8 {
                self.render_tile(
                    0,
                    idx as usize,
                    palette_id as usize + 4,
                    &mut target.sub_image(x.into(), y.into(), 8, 8),
                    true,
                );
            }
        }
    }

    pub fn get_tile_attribute(&self, x: u32, y: u32) -> u8 {
        let attr_table_idx = y / 4 * 8 + x / 4;
        let attr_byte = self.read_ppu_memory(0x23C0 + attr_table_idx as u16);
        match (x % 4 / 2, y % 4 / 2) {
            (0, 0) => attr_byte & 0b11,
            (1, 0) => (attr_byte >> 2) & 0b11,
            (0, 1) => (attr_byte >> 4) & 0b11,
            (1, 1) => (attr_byte >> 6) & 0b11,
            (_, _) => panic!("should not happen"),
        }
    }

    pub fn render_nametable(&mut self, target: &mut SubImage<&mut RgbaImage>) {
        let bank = if self
            .control_register
            .contains(ControlRegister::BACKROUND_PATTERN_ADDR)
        {
            1
        } else {
            0
        };

        for y in 0..30_u32 {
            for x in 0..32 {
                let addr = 0x2000 + y * 0x20 + x;
                let tile_num: usize = self.read_ppu_memory(addr as u16).into();
                self.render_tile(
                    bank,
                    tile_num,
                    self.get_tile_attribute(x, y).into(),
                    &mut target.sub_image(x * 8, y * 8, 8, 8),
                    false,
                )
            }
        }
    }

    pub fn render_pattern_table(&self, palette_id: usize) -> Result<RgbaImage> {
        let mut rendered: RgbaImage = ImageBuffer::new(32 * 8, 16 * 8);
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
        target: &mut SubImage<&mut RgbaImage>,
    ) {
        for y in 0..16 {
            for x in 0..16 {
                let tile_num = (y * 16) + x;
                self.render_tile(
                    bank,
                    tile_num,
                    palette_id,
                    &mut target.sub_image((x * 8) as u32, (y * 8) as u32, 8, 8),
                    false,
                );
            }
        }
    }

    pub fn render_tile(
        &self,
        bank: usize,
        tile_num: usize,
        palette_id: usize,
        target: &mut SubImage<&mut RgbaImage>,
        is_sprite: bool,
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
                if !(value == 0 && is_sprite) {
                    let rgb = self.get_palette_entry(palette_id, value as usize);
                    target.put_pixel(x as u32, y as u32, rgb);
                }
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
pub static SYSTEM_PALLETE: [Rgba<u8>; 64] = [
    Rgba([0x80, 0x80, 0x80, 0xFF]),
    Rgba([0x00, 0x3D, 0xA6, 0xFF]),
    Rgba([0x00, 0x12, 0xB0, 0xFF]),
    Rgba([0x44, 0x00, 0x96, 0xFF]),
    Rgba([0xA1, 0x00, 0x5E, 0xFF]),
    Rgba([0xC7, 0x00, 0x28, 0xFF]),
    Rgba([0xBA, 0x06, 0x00, 0xFF]),
    Rgba([0x8C, 0x17, 0x00, 0xFF]),
    Rgba([0x5C, 0x2F, 0x00, 0xFF]),
    Rgba([0x10, 0x45, 0x00, 0xFF]),
    Rgba([0x05, 0x4A, 0x00, 0xFF]),
    Rgba([0x00, 0x47, 0x2E, 0xFF]),
    Rgba([0x00, 0x41, 0x66, 0xFF]),
    Rgba([0x00, 0x00, 0x00, 0xFF]),
    Rgba([0x05, 0x05, 0x05, 0xFF]),
    Rgba([0x05, 0x05, 0x05, 0xFF]),
    Rgba([0xC7, 0xC7, 0xC7, 0xFF]),
    Rgba([0x00, 0x77, 0xFF, 0xFF]),
    Rgba([0x21, 0x55, 0xFF, 0xFF]),
    Rgba([0x82, 0x37, 0xFA, 0xFF]),
    Rgba([0xEB, 0x2F, 0xB5, 0xFF]),
    Rgba([0xFF, 0x29, 0x50, 0xFF]),
    Rgba([0xFF, 0x22, 0x00, 0xFF]),
    Rgba([0xD6, 0x32, 0x00, 0xFF]),
    Rgba([0xC4, 0x62, 0x00, 0xFF]),
    Rgba([0x35, 0x80, 0x00, 0xFF]),
    Rgba([0x05, 0x8F, 0x00, 0xFF]),
    Rgba([0x00, 0x8A, 0x55, 0xFF]),
    Rgba([0x00, 0x99, 0xCC, 0xFF]),
    Rgba([0x21, 0x21, 0x21, 0xFF]),
    Rgba([0x09, 0x09, 0x09, 0xFF]),
    Rgba([0x09, 0x09, 0x09, 0xFF]),
    Rgba([0xFF, 0xFF, 0xFF, 0xFF]),
    Rgba([0x0F, 0xD7, 0xFF, 0xFF]),
    Rgba([0x69, 0xA2, 0xFF, 0xFF]),
    Rgba([0xD4, 0x80, 0xFF, 0xFF]),
    Rgba([0xFF, 0x45, 0xF3, 0xFF]),
    Rgba([0xFF, 0x61, 0x8B, 0xFF]),
    Rgba([0xFF, 0x88, 0x33, 0xFF]),
    Rgba([0xFF, 0x9C, 0x12, 0xFF]),
    Rgba([0xFA, 0xBC, 0x20, 0xFF]),
    Rgba([0x9F, 0xE3, 0x0E, 0xFF]),
    Rgba([0x2B, 0xF0, 0x35, 0xFF]),
    Rgba([0x0C, 0xF0, 0xA4, 0xFF]),
    Rgba([0x05, 0xFB, 0xFF, 0xFF]),
    Rgba([0x5E, 0x5E, 0x5E, 0xFF]),
    Rgba([0x0D, 0x0D, 0x0D, 0xFF]),
    Rgba([0x0D, 0x0D, 0x0D, 0xFF]),
    Rgba([0xFF, 0xFF, 0xFF, 0xFF]),
    Rgba([0xA6, 0xFC, 0xFF, 0xFF]),
    Rgba([0xB3, 0xEC, 0xFF, 0xFF]),
    Rgba([0xDA, 0xAB, 0xEB, 0xFF]),
    Rgba([0xFF, 0xA8, 0xF9, 0xFF]),
    Rgba([0xFF, 0xAB, 0xB3, 0xFF]),
    Rgba([0xFF, 0xD2, 0xB0, 0xFF]),
    Rgba([0xFF, 0xEF, 0xA6, 0xFF]),
    Rgba([0xFF, 0xF7, 0x9C, 0xFF]),
    Rgba([0xD7, 0xE8, 0x95, 0xFF]),
    Rgba([0xA6, 0xED, 0xAF, 0xFF]),
    Rgba([0xA2, 0xF2, 0xDA, 0xFF]),
    Rgba([0x99, 0xFF, 0xFC, 0xFF]),
    Rgba([0xDD, 0xDD, 0xDD, 0xFF]),
    Rgba([0x11, 0x11, 0x11, 0xFF]),
    Rgba([0x11, 0x11, 0x11, 0xFF]),
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
