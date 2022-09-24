use anyhow::Result;
use bincode::Decode;
use bincode::Encode;
use image::GenericImage;

use image::ImageBuffer;
use image::Rgba;
use image::RgbaImage;
use image::SubImage;
use packed_struct::prelude::*;
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

type RgbaSubImage<'a> = SubImage<&'a mut RgbaImage>;

#[derive(Encode, Decode)]
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

    pub framebuffer: BincodeImage,
}

pub struct BincodeImage {
    pub image: RgbaImage,
}

impl std::ops::Deref for BincodeImage {
    type Target = RgbaImage;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}

impl Encode for BincodeImage {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(&self.image.width(), encoder)?;
        bincode::Encode::encode(&self.image.height(), encoder)?;
        bincode::Encode::encode(&self.image.as_raw(), encoder)?;
        Ok(())
    }
}

impl Decode for BincodeImage {
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(BincodeImage {
            image: RgbaImage::from_raw(
                Decode::decode(decoder)?,
                Decode::decode(decoder)?,
                Decode::decode(decoder)?,
            )
            .unwrap(),
        })
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new(Rc::new(RefCell::new(Cartridge::default())))
    }
}

static REVERSE_U8_TABLE: [u8; 256] = [
    0x00, 0x80, 0x40, 0xc0, 0x20, 0xa0, 0x60, 0xe0, 0x10, 0x90, 0x50, 0xd0, 0x30, 0xb0, 0x70, 0xf0,
    0x08, 0x88, 0x48, 0xc8, 0x28, 0xa8, 0x68, 0xe8, 0x18, 0x98, 0x58, 0xd8, 0x38, 0xb8, 0x78, 0xf8,
    0x04, 0x84, 0x44, 0xc4, 0x24, 0xa4, 0x64, 0xe4, 0x14, 0x94, 0x54, 0xd4, 0x34, 0xb4, 0x74, 0xf4,
    0x0c, 0x8c, 0x4c, 0xcc, 0x2c, 0xac, 0x6c, 0xec, 0x1c, 0x9c, 0x5c, 0xdc, 0x3c, 0xbc, 0x7c, 0xfc,
    0x02, 0x82, 0x42, 0xc2, 0x22, 0xa2, 0x62, 0xe2, 0x12, 0x92, 0x52, 0xd2, 0x32, 0xb2, 0x72, 0xf2,
    0x0a, 0x8a, 0x4a, 0xca, 0x2a, 0xaa, 0x6a, 0xea, 0x1a, 0x9a, 0x5a, 0xda, 0x3a, 0xba, 0x7a, 0xfa,
    0x06, 0x86, 0x46, 0xc6, 0x26, 0xa6, 0x66, 0xe6, 0x16, 0x96, 0x56, 0xd6, 0x36, 0xb6, 0x76, 0xf6,
    0x0e, 0x8e, 0x4e, 0xce, 0x2e, 0xae, 0x6e, 0xee, 0x1e, 0x9e, 0x5e, 0xde, 0x3e, 0xbe, 0x7e, 0xfe,
    0x01, 0x81, 0x41, 0xc1, 0x21, 0xa1, 0x61, 0xe1, 0x11, 0x91, 0x51, 0xd1, 0x31, 0xb1, 0x71, 0xf1,
    0x09, 0x89, 0x49, 0xc9, 0x29, 0xa9, 0x69, 0xe9, 0x19, 0x99, 0x59, 0xd9, 0x39, 0xb9, 0x79, 0xf9,
    0x05, 0x85, 0x45, 0xc5, 0x25, 0xa5, 0x65, 0xe5, 0x15, 0x95, 0x55, 0xd5, 0x35, 0xb5, 0x75, 0xf5,
    0x0d, 0x8d, 0x4d, 0xcd, 0x2d, 0xad, 0x6d, 0xed, 0x1d, 0x9d, 0x5d, 0xdd, 0x3d, 0xbd, 0x7d, 0xfd,
    0x03, 0x83, 0x43, 0xc3, 0x23, 0xa3, 0x63, 0xe3, 0x13, 0x93, 0x53, 0xd3, 0x33, 0xb3, 0x73, 0xf3,
    0x0b, 0x8b, 0x4b, 0xcb, 0x2b, 0xab, 0x6b, 0xeb, 0x1b, 0x9b, 0x5b, 0xdb, 0x3b, 0xbb, 0x7b, 0xfb,
    0x07, 0x87, 0x47, 0xc7, 0x27, 0xa7, 0x67, 0xe7, 0x17, 0x97, 0x57, 0xd7, 0x37, 0xb7, 0x77, 0xf7,
    0x0f, 0x8f, 0x4f, 0xcf, 0x2f, 0xaf, 0x6f, 0xef, 0x1f, 0x9f, 0x5f, 0xdf, 0x3f, 0xbf, 0x7f, 0xff,
];

pub fn reverse_u8(b: u8) -> u8 {
    REVERSE_U8_TABLE[b as usize]
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

            framebuffer: BincodeImage {
                image: RgbaImage::new(32 * 8, 30 * 8),
            },
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
                self.vblank = false;
            }

            if self.scanline >= 262 {
                self.scanline = 0;
            }

            if self.scanline < 240 {
                self.render_scanline();
            }
        }
    }

    fn collect_sprites_on_scanline(&self, scanline: usize) -> Vec<OamSprite> {
        (0..64)
            .filter_map(|i| {
                let oam_addr = i * 4;
                let sprite =
                    OamSprite::unpack_from_slice(&self.oam_data[oam_addr..oam_addr + 4]).unwrap();
                let delta_y = scanline as i32 - sprite.y as i32;
                if (0..8).contains(&delta_y) {
                    Some(sprite)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_nametable_entry(&self, coarse_x: usize, coarse_y: usize) -> usize {
        let addr = 0x2000 + coarse_y * 0x20 + coarse_x;
        self.read_ppu_memory(addr as u16) as usize
    }

    pub fn get_tile_row(
        &self,
        bank_id: usize,
        tile_id: usize,
        mut fine_y: usize,
        flip_h: bool,
        flip_v: bool,
    ) -> TileRow {
        let bank_addr = (0x1000 * bank_id) as u16;
        let tile_addr = bank_addr + (tile_id * 16) as u16;
        if flip_v {
            fine_y = 7 - fine_y;
        }
        let line_addr = tile_addr + fine_y as u16;

        let mut low = self.read_ppu_memory(line_addr);
        let mut high = self.read_ppu_memory(line_addr + 8);

        if flip_h {
            low = reverse_u8(low);
            high = reverse_u8(high);
        }
        TileRow { low, high }
    }

    pub fn render_scanline(&mut self) {
        let y = self.scanline;
        let coarse_y = y / 8;
        let fine_y = y % 8;

        let bg_bank = self.control_register.background_pattern_addr as usize;
        let _sprite_bank = self.control_register.sprite_pattern_addr as usize;

        let mut sprites = self.collect_sprites_on_scanline(y);
        sprites.sort_by(|a, b| b.x.cmp(&a.x));

        let mut sprite_pixels = TileRow::default();
        let mut sprite_palette_id: u8 = 0;

        for coarse_x in 0..32 {
            let bg_tile_id = self.get_nametable_entry(coarse_x, coarse_y);
            let bg_palette_id = self.get_nametable_attribute(coarse_x, coarse_y);

            for (fine_x, bg_color_id) in &mut self
                .get_tile_row(bg_bank, bg_tile_id, fine_y, false, false)
                .pixels()
            {
                let x = coarse_x * 8 + fine_x;

                if let Some(next_sprite) = sprites.last() {
                    if next_sprite.x as usize == x {
                        let sprite_y_offset = y - next_sprite.y as usize;

                        sprite_pixels = self.get_tile_row(
                            self.control_register.sprite_pattern_addr as usize,
                            next_sprite.index as usize,
                            sprite_y_offset as usize,
                            next_sprite.attributes.flip_h,
                            next_sprite.attributes.flip_v,
                        );
                        sprite_palette_id = next_sprite.attributes.palette_id;
                        sprites.pop();
                    }
                }

                let sprite_color_id = sprite_pixels.next_pixel();
                let (final_color_id, final_palette_id) = if sprite_color_id != 0 {
                    (sprite_color_id, sprite_palette_id + 4)
                } else {
                    (bg_color_id, bg_palette_id)
                };

                let rgb =
                    self.get_palette_entry(final_palette_id as usize, final_color_id as usize);
                self.framebuffer.image.put_pixel(x as u32, y as u32, rgb);
            }
        }
    }

    pub fn get_palette_entry(&self, palette_id: usize, entry: usize) -> Rgba<u8> {
        if entry == 0 {
            SYSTEM_PALLETE[self.read_ppu_memory(0x3F00) as usize]
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
        // Map memory addresses
        let addr = match addr {
            0x3F10 => 0x3F00,
            addr => addr,
        };

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

    pub fn get_nametable_attribute(&self, x: usize, y: usize) -> u8 {
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

    pub fn debug_render_sprites(&self, target: &mut RgbaSubImage) {
        for sprite_num in 0..64 {
            let oam_addr = sprite_num * 4;
            let sprite =
                OamSprite::unpack_from_slice(&self.oam_data[oam_addr..oam_addr + 4]).unwrap();
            if sprite.y > 0xEF {
                continue;
            }
            if sprite.x < 31 * 8 && sprite.y < 29 * 8 {
                self.debug_render_tile(
                    self.control_register.sprite_pattern_addr as usize,
                    sprite.index as usize,
                    sprite.attributes.palette_id as usize + 4,
                    &mut target.sub_image(sprite.x.into(), sprite.y.into(), 8, 8),
                    true,
                    sprite.attributes.flip_h,
                    sprite.attributes.flip_v,
                );
            }
        }
    }
    pub fn debug_render_nametable(&mut self, target: &mut RgbaSubImage) {
        let bank = self.control_register.background_pattern_addr as usize;

        for y in 0..30_u32 {
            for x in 0..32 {
                let addr = 0x2000 + y * 0x20 + x;
                let tile_num: usize = self.read_ppu_memory(addr as u16).into();
                self.debug_render_tile(
                    bank,
                    tile_num,
                    self.get_nametable_attribute(x as usize, y as usize).into(),
                    &mut target.sub_image(x * 8, y * 8, 8, 8),
                    false,
                    false,
                    false,
                )
            }
        }
    }

    pub fn debug_render_pattern_table(&self, palette_id: usize) -> Result<RgbaImage> {
        let mut rendered: RgbaImage = ImageBuffer::new(32 * 8, 16 * 8);
        self.debug_render_tile_bank(0, palette_id, &mut rendered.sub_image(0, 0, 16 * 8, 16 * 8));
        self.debug_render_tile_bank(
            1,
            palette_id,
            &mut rendered.sub_image(16 * 8, 0, 16 * 8, 16 * 8),
        );
        Ok(rendered)
    }

    pub fn debug_render_tile_bank(
        &self,
        bank: usize,
        palette_id: usize,
        target: &mut RgbaSubImage,
    ) {
        for y in 0..16 {
            for x in 0..16 {
                let tile_num = (y * 16) + x;
                self.debug_render_tile(
                    bank,
                    tile_num,
                    palette_id,
                    &mut target.sub_image((x * 8) as u32, (y * 8) as u32, 8, 8),
                    false,
                    false,
                    false,
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn debug_render_tile(
        &self,
        bank: usize,
        tile_num: usize,
        palette_id: usize,
        target: &mut RgbaSubImage,
        is_sprite: bool,
        flip_h: bool,
        flip_v: bool,
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
                    let pixel_x = if flip_h { 8 - x } else { x };
                    let pixel_y = if flip_v { 8 - y } else { y };
                    target.put_pixel(pixel_x as u32, pixel_y as u32, rgb);
                }
                upper >>= 1;
                lower >>= 1;
            }
        }
    }
}

#[derive(Default)]
pub struct TileRow {
    pub low: u8,
    pub high: u8,
}

impl TileRow {
    pub fn next_pixel(&mut self) -> u8 {
        let low_bit = self.low & 0b1000_0000 != 0;
        let high_bit = self.high & 0b1000_0000 != 0;
        self.low <<= 1;
        self.high <<= 1;
        (high_bit as u8) << 1 | (low_bit as u8)
    }

    pub fn pixels(&mut self) -> impl Iterator<Item = (usize, u8)> + '_ {
        (0..8).map(move |fine_x| (fine_x, self.next_pixel()))
    }
}

#[derive(PackedStruct, Default, Debug, Copy, Clone)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "4")]
pub struct OamSprite {
    y: u8,
    index: u8,
    #[packed_field(size_bytes = "1")]
    attributes: OamSpriteAttributes,
    x: u8,
}

#[derive(PackedStruct, Default, Debug, Copy, Clone)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct OamSpriteAttributes {
    flip_v: bool,
    flip_h: bool,
    priority: bool,
    #[packed_field(bits = "6..=7")]
    palette_id: u8,
}

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
