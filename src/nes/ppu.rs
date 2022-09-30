use std::cell::RefCell;
use std::rc::Rc;

use bincode::Decode;
use bincode::Encode;
use egui::Color32;
use egui::ColorImage;
use image::RgbaImage;
use packed_struct::prelude::*;

use super::cartridge::Cartridge;

const CONTROL_REGISTER_ADDR: u16 = 0x2000;
const STATUS_REGISTER_ADDR: u16 = 0x2002;
const OAM_ADDR: u16 = 0x2003;
const OAM_DATA: u16 = 0x2004;
const PPU_SCROLL: u16 = 0x2005;
const ADDRESS_REGISTER_ADDR: u16 = 0x2006;
const DATA_REGISTER_ADDR: u16 = 0x2007;

const FRAME_WIDTH: usize = 32 * 8;
const FRAME_HEIGHT: usize = 30 * 8;

////////////////////////////////////////////////////////////////////////////////
// PPU

#[derive(Encode, Decode)]
pub struct Ppu {
    pub cartridge: Rc<RefCell<Cartridge>>,
    pub palette_table: [u8; 32],
    pub vram: Vec<u8>,
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

    pub framebuffer: Framebuffer,
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
            vram: vec![0; 2048],
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

            framebuffer: Framebuffer::default(),
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.cycle += cycles;
        while self.cycle >= 341 {
            self.cycle -= 341;
            self.scanline += 1;

            if self.scanline == 241 {
                self.status_register.vblank_started = true;
                self.vblank = true;
                if self.control_register.generate_nmi {
                    self.nmi_interrupt = true;
                }
            }

            if self.scanline == 261 {
                self.status_register.vblank_started = false;
                self.status_register.sprite_zero_hit = false;
                self.vblank = false;
            }

            if self.scanline >= 262 {
                self.scanline = 0;
            }

            if self.scanline < 240 {
                let sprite_0_hit = self.render_scanline();
                if sprite_0_hit {
                    self.status_register.sprite_zero_hit = true;
                }
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Scanline Rendering

    fn collect_sprites_on_scanline(&self, scanline: usize) -> impl Iterator<Item = Sprite> + '_ {
        (0..64)
            .filter_map(move |i| {
                let sprite = Sprite::new(self, i);
                let delta_y = scanline as i32 - sprite.data.y as i32;
                if (0..8).contains(&delta_y) {
                    Some(sprite)
                } else {
                    None
                }
            })
            .rev()
    }

    pub fn get_nametable_entry(&self, coarse_x: usize, coarse_y: usize) -> usize {
        let addr = 0x2000 + coarse_y * 0x20 + coarse_x;
        self.read_ppu_memory(addr as u16) as usize
    }

    pub fn render_scanline(&mut self) -> bool {
        let screen_y = self.scanline as usize;
        let coarse_y = screen_y / 8;
        let fine_y = screen_y % 8;
        let mut sprite_0_hit = false;

        // Temporary buffer of pixels as (color, palette_id) pairs.
        let mut pixels = [(0_u8, 0_u8); 32 * 8];

        // Write background pixels to buffer
        for coarse_x in 0..32 {
            let background = NametableEntry::new(self, coarse_x, coarse_y);
            for (fine_x, pixel) in background.pattern.row_pixels(self, fine_y).enumerate() {
                let screen_x = coarse_x * 8 + fine_x as usize;
                pixels[screen_x as usize] = (pixel, background.palette_id);
            }
        }

        // Add sprite pixels
        for sprite in self.collect_sprites_on_scanline(self.scanline) {
            let sprite_row = screen_y - sprite.data.y as usize;
            for (fine_x, pixel) in sprite.row_pixels(self, sprite_row).enumerate() {
                let screen_x = sprite.data.x as usize + fine_x as usize;
                if screen_x >= 32 * 8 {
                    break;
                }
                let (bg_pixel, _) = pixels[screen_x as usize];
                if bg_pixel == 0 || (pixel > 0 && !sprite.data.attr.priority) {
                    if sprite.id == 0 {
                        sprite_0_hit = true;
                    }
                    pixels[screen_x as usize] = (pixel, sprite.data.attr.palette_id + 4);
                }
            }
        }

        // Convert into RGBA and write into framebuffer
        for (screen_x, (color, palette)) in pixels.into_iter().enumerate() {
            self.framebuffer[(screen_x, screen_y)] =
                self.get_palette_entry(palette as usize, color as usize);
        }
        sprite_0_hit
    }

    pub fn get_palette_entry(&self, palette_id: usize, entry: usize) -> u8 {
        if entry == 0 {
            self.read_ppu_memory(0x3F00)
        } else {
            let addr = 0x3F00 + (palette_id as u16 * 4) + entry as u16;
            self.read_ppu_memory(addr)
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    // PPU Bus

    pub fn read_ppu_memory(&self, addr: u16) -> u8 {
        match addr {
            0..=0x1FFF => self.cartridge.borrow().chr[addr as usize],
            0x2000..=0x3FFF => self.vram[(addr - 0x2000) as usize % self.vram.len()],
            _ => panic!("Invalid PPU address read {addr:04X}"),
        }
    }

    pub fn write_ppu_memory(&mut self, addr: u16, value: u8) {
        // Map memory addresses
        let addr = match addr {
            0x3F10 => 0x3F00,
            addr => addr,
        };

        match addr {
            0..=0x1FFF => self.cartridge.borrow_mut().chr[addr as usize] = value,
            0x2000..=0x3FFF => self.vram[(addr - 0x2000) as usize % 2048] = value,
            _ => println!("Warning: Invalid PPU address write {addr:04X}"),
        };
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Registers exposed to CPU bus

    fn increment_address_register(&mut self) -> u16 {
        let addr = self.address_register.address();
        let inc = if self.control_register.vram_add_increment {
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
        let status = self.status_register.pack().unwrap()[0];
        self.status_register.vblank_started = false;
        status
    }

    pub fn cpu_bus_peek(&self, addr: u16) -> u8 {
        match addr {
            OAM_ADDR => self.oam_addr,
            OAM_DATA => self.oam_data[self.oam_addr as usize],
            PPU_SCROLL => self.scroll,
            CONTROL_REGISTER_ADDR => self.control_register.pack().unwrap()[0],
            STATUS_REGISTER_ADDR => self.status_register.pack().unwrap()[0],
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
                self.control_register = ControlRegister::unpack(&[value]).unwrap();
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

    ////////////////////////////////////////////////////////////////////////////////
    // Debug API

    pub fn debug_render_nametable(&self) -> ColorImage {
        let mut image = ColorImage::new([32 * 8, 30 * 8], Color32::TRANSPARENT);
        for coarse_y in 0..30 {
            for coarse_x in 0..32 {
                let background = NametableEntry::new(self, coarse_x, coarse_y);
                for fine_y in 0..8 {
                    for (fine_x, pixel) in background.pattern.row_pixels(self, fine_y).enumerate() {
                        let color =
                            self.get_palette_entry(background.palette_id as usize, pixel as usize);
                        image[(coarse_x * 8 + fine_x, coarse_y * 8 + fine_y)] =
                            SYSTEM_PALETTE[color as usize];
                    }
                }
            }
        }
        image
    }
}

////////////////////////////////////////////////////////////////////////////////
// Framebuffer

#[derive(Decode, Encode)]
pub struct Framebuffer {
    pixels: Vec<u8>,
}

impl Default for Framebuffer {
    fn default() -> Self {
        Self {
            pixels: vec![0; FRAME_WIDTH * FRAME_HEIGHT],
        }
    }
}

impl Framebuffer {
    pub const SIZE: [usize; 2] = [FRAME_WIDTH, FRAME_HEIGHT];

    pub fn as_rgba_image(&self) -> RgbaImage {
        RgbaImage::from_vec(
            FRAME_WIDTH as u32,
            FRAME_HEIGHT as u32,
            self.pixels
                .iter()
                .flat_map(|c| {
                    let color32 = SYSTEM_PALETTE[*c as usize];
                    [color32.r(), color32.g(), color32.b(), color32.a()]
                })
                .collect(),
        )
        .unwrap()
    }

    pub fn as_color_image(&self) -> ColorImage {
        ColorImage {
            size: Framebuffer::SIZE,
            pixels: self
                .pixels
                .iter()
                .map(|c| SYSTEM_PALETTE[*c as usize])
                .collect(),
        }
    }
}

impl std::ops::Index<(usize, usize)> for Framebuffer {
    type Output = u8;

    fn index(&self, (x, y): (usize, usize)) -> &u8 {
        &self.pixels[y * FRAME_WIDTH + x]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Framebuffer {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut u8 {
        &mut self.pixels[y * FRAME_WIDTH + x]
    }
}

////////////////////////////////////////////////////////////////////////////////
// NametableEntry

pub struct NametableEntry {
    pattern: Pattern,
    palette_id: u8,
}

impl NametableEntry {
    pub fn new(ppu: &Ppu, coarse_x: usize, coarse_y: usize) -> NametableEntry {
        let addr = 0x2000 + coarse_y * 0x20 + coarse_x;
        let nametable_value = ppu.read_ppu_memory(addr as u16);

        let attr_table_idx = coarse_y / 4 * 8 + coarse_x / 4;
        let attr_byte = ppu.read_ppu_memory(0x23C0 + attr_table_idx as u16);
        let attribute = match (coarse_x % 4 / 2, coarse_y % 4 / 2) {
            (0, 0) => attr_byte & 0b11,
            (1, 0) => (attr_byte >> 2) & 0b11,
            (0, 1) => (attr_byte >> 4) & 0b11,
            (1, 1) => (attr_byte >> 6) & 0b11,
            (_, _) => panic!("should not happen"),
        };

        NametableEntry {
            pattern: Pattern::new(
                ppu.control_register.background_pattern_addr as u8,
                nametable_value,
            ),
            palette_id: attribute,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Sprite / OAM

pub struct Sprite {
    id: usize,
    data: OamEntry,
    pattern: Pattern,
}

impl Sprite {
    pub fn new(ppu: &Ppu, id: usize) -> Sprite {
        let sprite_addr = id * 4;
        let data =
            OamEntry::unpack_from_slice(&ppu.oam_data[sprite_addr..sprite_addr + 4]).unwrap();
        Sprite {
            id,
            data,
            pattern: Pattern::new(ppu.control_register.sprite_pattern_addr as u8, data.index),
        }
    }

    pub fn row_pixels(&self, ppu: &Ppu, mut y: usize) -> impl Iterator<Item = u8> {
        if self.data.attr.flip_v {
            y = 7 - y;
        }
        let mut row: Vec<u8> = self.pattern.row_pixels(ppu, y).collect();
        if self.data.attr.flip_h {
            row.reverse();
        }
        row.into_iter()
    }
}

#[derive(PackedStruct, Default, Debug, Copy, Clone)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "4")]
pub struct OamEntry {
    y: u8,
    index: u8,
    #[packed_field(size_bytes = "1")]
    attr: OamSpriteAttr,
    x: u8,
}

#[derive(PackedStruct, Default, Debug, Copy, Clone)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct OamSpriteAttr {
    flip_v: bool,
    flip_h: bool,
    priority: bool,
    #[packed_field(bits = "6..=7")]
    palette_id: u8,
}

////////////////////////////////////////////////////////////////////////////////
// Pattern

pub struct Pattern {
    addr: u16,
}

impl Pattern {
    pub fn new(bank_id: u8, pattern_id: u8) -> Pattern {
        Pattern {
            addr: (0x1000 * bank_id as u16) + (pattern_id as u16 * 16),
        }
    }

    pub fn row_pixels(&self, ppu: &Ppu, y: usize) -> impl Iterator<Item = u8> + '_ {
        let mut low = ppu.read_ppu_memory(self.addr + y as u16);
        let mut high = ppu.read_ppu_memory(self.addr + y as u16 + 8);

        (0..8).map(move |_| {
            let low_bit = low & 0b1000_0000 > 0;
            let high_bit = high & 0b1000_0000 > 0;
            low <<= 1;
            high <<= 1;
            (high_bit as u8) << 1 | (low_bit as u8)
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
// Registers

#[derive(Default, Encode, Decode)]
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

#[derive(PackedStruct, Encode, Decode, Debug, Default, Clone, Copy, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct ControlRegister {
    generate_nmi: bool,
    master_slave_select: bool,
    sprite_size: bool,
    background_pattern_addr: bool,
    sprite_pattern_addr: bool,
    vram_add_increment: bool,
    #[packed_field(bits = "6..=7")]
    nametable: u8,
}

#[derive(PackedStruct, Encode, Decode, Debug, Default, Clone, Copy, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct StatusRegister {
    vblank_started: bool,
    sprite_zero_hit: bool,
    sprite_overflow: bool,
}

////////////////////////////////////////////////////////////////////////////////
// Palette Lookup Table

pub static SYSTEM_PALETTE: [Color32; 64] = [
    Color32::from_rgb(0x80, 0x80, 0x80),
    Color32::from_rgb(0x00, 0x3D, 0xA6),
    Color32::from_rgb(0x00, 0x12, 0xB0),
    Color32::from_rgb(0x44, 0x00, 0x96),
    Color32::from_rgb(0xA1, 0x00, 0x5E),
    Color32::from_rgb(0xC7, 0x00, 0x28),
    Color32::from_rgb(0xBA, 0x06, 0x00),
    Color32::from_rgb(0x8C, 0x17, 0x00),
    Color32::from_rgb(0x5C, 0x2F, 0x00),
    Color32::from_rgb(0x10, 0x45, 0x00),
    Color32::from_rgb(0x05, 0x4A, 0x00),
    Color32::from_rgb(0x00, 0x47, 0x2E),
    Color32::from_rgb(0x00, 0x41, 0x66),
    Color32::from_rgb(0x00, 0x00, 0x00),
    Color32::from_rgb(0x05, 0x05, 0x05),
    Color32::from_rgb(0x05, 0x05, 0x05),
    Color32::from_rgb(0xC7, 0xC7, 0xC7),
    Color32::from_rgb(0x00, 0x77, 0xFF),
    Color32::from_rgb(0x21, 0x55, 0xFF),
    Color32::from_rgb(0x82, 0x37, 0xFA),
    Color32::from_rgb(0xEB, 0x2F, 0xB5),
    Color32::from_rgb(0xFF, 0x29, 0x50),
    Color32::from_rgb(0xFF, 0x22, 0x00),
    Color32::from_rgb(0xD6, 0x32, 0x00),
    Color32::from_rgb(0xC4, 0x62, 0x00),
    Color32::from_rgb(0x35, 0x80, 0x00),
    Color32::from_rgb(0x05, 0x8F, 0x00),
    Color32::from_rgb(0x00, 0x8A, 0x55),
    Color32::from_rgb(0x00, 0x99, 0xCC),
    Color32::from_rgb(0x21, 0x21, 0x21),
    Color32::from_rgb(0x09, 0x09, 0x09),
    Color32::from_rgb(0x09, 0x09, 0x09),
    Color32::from_rgb(0xFF, 0xFF, 0xFF),
    Color32::from_rgb(0x0F, 0xD7, 0xFF),
    Color32::from_rgb(0x69, 0xA2, 0xFF),
    Color32::from_rgb(0xD4, 0x80, 0xFF),
    Color32::from_rgb(0xFF, 0x45, 0xF3),
    Color32::from_rgb(0xFF, 0x61, 0x8B),
    Color32::from_rgb(0xFF, 0x88, 0x33),
    Color32::from_rgb(0xFF, 0x9C, 0x12),
    Color32::from_rgb(0xFA, 0xBC, 0x20),
    Color32::from_rgb(0x9F, 0xE3, 0x0E),
    Color32::from_rgb(0x2B, 0xF0, 0x35),
    Color32::from_rgb(0x0C, 0xF0, 0xA4),
    Color32::from_rgb(0x05, 0xFB, 0xFF),
    Color32::from_rgb(0x5E, 0x5E, 0x5E),
    Color32::from_rgb(0x0D, 0x0D, 0x0D),
    Color32::from_rgb(0x0D, 0x0D, 0x0D),
    Color32::from_rgb(0xFF, 0xFF, 0xFF),
    Color32::from_rgb(0xA6, 0xFC, 0xFF),
    Color32::from_rgb(0xB3, 0xEC, 0xFF),
    Color32::from_rgb(0xDA, 0xAB, 0xEB),
    Color32::from_rgb(0xFF, 0xA8, 0xF9),
    Color32::from_rgb(0xFF, 0xAB, 0xB3),
    Color32::from_rgb(0xFF, 0xD2, 0xB0),
    Color32::from_rgb(0xFF, 0xEF, 0xA6),
    Color32::from_rgb(0xFF, 0xF7, 0x9C),
    Color32::from_rgb(0xD7, 0xE8, 0x95),
    Color32::from_rgb(0xA6, 0xED, 0xAF),
    Color32::from_rgb(0xA2, 0xF2, 0xDA),
    Color32::from_rgb(0x99, 0xFF, 0xFC),
    Color32::from_rgb(0xDD, 0xDD, 0xDD),
    Color32::from_rgb(0x11, 0x11, 0x11),
    Color32::from_rgb(0x11, 0x11, 0x11),
];

////////////////////////////////////////////////////////////////////////////////
// Unit Tests

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
