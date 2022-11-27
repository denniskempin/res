use std::cell::RefCell;
use std::fmt::Display;
use std::fmt::Formatter;
use std::rc::Rc;

use bincode::Decode;
use bincode::Encode;
use egui::Color32;
use egui::ColorImage;
use image::RgbaImage;
use intbits::Bits;
use itertools::Itertools;
use packed_struct::prelude::*;
use thiserror::Error;

use super::cartridge::Cartridge;
use super::cartridge::CartridgeError;

#[derive(Error)]
pub enum PpuError {
    #[error("Invalid read from 0x{0:04X}")]
    InvalidBusRead(u16),
    #[error("Invalid write to 0x{0:04X}")]
    InvalidBusWrite(u16),
    #[error("Invalid peek from 0x{0:04X}")]
    InvalidBusPeek(u16),
    #[error(transparent)]
    CartridgeError(#[from] CartridgeError),
}

impl std::fmt::Debug for PpuError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self)
    }
}

pub type PpuResult<T> = std::result::Result<T, PpuError>;

const CONTROL_REGISTER_ADDR: u16 = 0x2000;
const MASK_REGISTER_ADDR: u16 = 0x2001;
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

#[derive(Encode, Decode, Clone)]
pub struct Ppu {
    pub cartridge: Rc<RefCell<Cartridge>>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 0x2000],
    pub oam_data: [u8; 256],
    pub internal_data_buffer: u8,
    pub cycle: usize,
    pub scanline: usize,
    pub frame: usize,
    pub oam_addr: u8,

    pub control_register: ControlRegister,
    pub mask_register: MaskRegister,
    pub status_register: StatusRegister,

    pub v_register: VramAddress,
    pub t_register: VramAddress,
    pub fine_scroll_x: u8,
    pub register_latch: bool,

    pub nmi_interrupt: bool,
    pub vblank: bool,

    pub framebuffer: Framebuffer,
}

impl Ppu {
    pub fn new(cartridge: Rc<RefCell<Cartridge>>) -> Self {
        Self {
            cartridge,
            vram: [0; 0x2000],
            oam_data: [0; 256],
            palette_table: [0; 32],
            internal_data_buffer: 0,
            cycle: 0,
            scanline: 0,
            frame: 0,
            oam_addr: 0,

            control_register: ControlRegister::default(),
            mask_register: MaskRegister::default(),
            status_register: StatusRegister::default(),

            v_register: VramAddress::default(),
            t_register: VramAddress::default(),
            fine_scroll_x: 0,
            register_latch: false,

            nmi_interrupt: false,
            vblank: false,

            framebuffer: Framebuffer::default(),
        }
    }

    pub fn advance_clock(&mut self, cycles: usize) -> PpuResult<()> {
        for _ in 0..cycles {
            self.tick()?;
        }
        Ok(())
    }

    fn tick(&mut self) -> PpuResult<()> {
        self.cycle += 1;
        if self.cycle == 341 {
            self.cycle = 0;
            self.scanline += 1;
        }
        if self.scanline == 262 {
            self.scanline = 0;
            self.frame += 1;
        }

        match self.scanline {
            // Visible scanlines
            0..=239 => {
                match self.cycle {
                    2..=255 | 320.. => {
                        // Increment x every 8 cycles during visible or pre-render cycles
                        if self.cycle % 8 == 0 && self.mask_register.show_background {
                            self.v_register.increment_x();
                        }
                    }
                    256 => {
                        // Increment y at end of visible cycles
                        if self.mask_register.show_background {
                            self.v_register.increment_y();
                        }
                    }
                    257 => {
                        // Reset x at end of visible cycles
                        if self.mask_register.show_background {
                            self.v_register.set_coarse_x(self.t_register.coarse_x());
                            self.v_register
                                .set_nametable_x(self.t_register.nametable_x());
                        }
                    }
                    _ => (),
                }
            }
            // Start of vblank
            241 => {
                if self.cycle == 1 {
                    self.status_register.vblank_started = true;
                    self.vblank = true;
                    if self.control_register.generate_nmi {
                        self.nmi_interrupt = true;
                    }
                }
            }
            // Start of pre-render
            261 => {
                match self.cycle {
                    1 => {
                        self.status_register.vblank_started = false;
                        self.status_register.sprite_zero_hit = false;
                        self.vblank = false;
                    }
                    2..=255 | 320.. => {
                        // Increment x every 8 cycles
                        if self.cycle % 8 == 0 && self.mask_register.show_background {
                            self.v_register.increment_x();
                        }
                    }
                    256 => {
                        // Increment y at end of visible cycles
                        if self.mask_register.show_background {
                            self.v_register.increment_y();
                        }
                    }
                    257 => {
                        // Reset x at end of visible cycles
                        if self.mask_register.show_background {
                            self.v_register.set_coarse_x(self.t_register.coarse_x());
                            self.v_register
                                .set_nametable_x(self.t_register.nametable_x());
                        }
                    }
                    280..=304 => {
                        if self.mask_register.show_background {
                            self.v_register = self.t_register.clone();
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }

        // Shortcut: Render the whole scanline at once at cycle 255.
        if self.scanline < 240 && self.cycle == 255 {
            let sprite_0_hit = self.render_scanline()?;
            if sprite_0_hit {
                self.status_register.sprite_zero_hit = true;
            }
        }

        Ok(())
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

    pub fn get_nametable_entry(&self, coarse_x: usize, coarse_y: usize) -> PpuResult<usize> {
        let addr = 0x2000 + coarse_y * 0x20 + coarse_x;
        Ok(self.read_ppu_memory(addr as u16)? as usize)
    }

    pub fn render_scanline(&mut self) -> PpuResult<bool> {
        let screen_y = self.scanline as usize;
        let mut sprite_0_hit = false;

        // Temporary buffer of pixels as (color, palette_id) pairs.
        let mut pixels = [(0_u8, 0_u8); 32 * 8];

        // Write background pixels to buffer
        if self.mask_register.show_background {
            // Create a temporary copy of the v_register since we are drawing a whole scanline.
            // Make sure to reset the x location to the beginning of the scanline.
            let mut addr = self.v_register.clone();
            addr.set_coarse_x(self.t_register.coarse_x());
            addr.set_nametable_x(self.t_register.nametable_x());

            for coarse_x in 0..33 {
                let background = NametableEntry::new(self, &addr)?;
                for (fine_x, pixel) in background
                    .pattern
                    .row_pixels(self, addr.fine_y() as usize)?
                    .enumerate()
                {
                    let screen_x = coarse_x * 8 + fine_x as usize;
                    if screen_x >= self.fine_scroll_x as usize
                        && screen_x - (self.fine_scroll_x as usize) < 256
                    {
                        pixels[screen_x as usize - self.fine_scroll_x as usize] =
                            (pixel, background.palette_id);
                    }
                }
                addr.increment_x();
            }
        }

        // Add sprite pixels
        if self.mask_register.show_sprites {
            for sprite in self.collect_sprites_on_scanline(self.scanline) {
                let sprite_row = screen_y - sprite.data.y as usize;
                for (fine_x, pixel) in sprite.row_pixels(self, sprite_row)?.enumerate() {
                    let screen_x = sprite.data.x as usize + fine_x as usize;
                    if screen_x >= 32 * 8 {
                        break;
                    }
                    let (bg_pixel, _) = pixels[screen_x as usize];
                    if bg_pixel == 0 || (pixel > 0 && !sprite.data.attr.priority) {
                        pixels[screen_x as usize] = (pixel, sprite.data.attr.palette_id + 4);
                    }
                    if sprite.id == 0 && pixel > 0 {
                        sprite_0_hit = true;
                    }
                }
            }
        }

        // Convert into RGBA and write into framebuffer
        for (screen_x, (color, palette)) in pixels.into_iter().enumerate() {
            self.framebuffer[(screen_x, screen_y)] =
                self.get_palette_entry(palette as usize, color as usize)?;
        }
        Ok(sprite_0_hit)
    }

    pub fn get_palette_entry(&self, palette_id: usize, entry: usize) -> PpuResult<u8> {
        if entry == 0 {
            self.read_ppu_memory(0x3F00)
        } else {
            let addr = 0x3F00 + (palette_id as u16 * 4) + entry as u16;
            self.read_ppu_memory(addr)
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    // PPU Bus

    pub fn vram_addr_to_idx(&self, addr: u16) -> usize {
        addr as usize - 0x2000
    }

    pub fn peek_slice(&self, addr: u16, length: u16) -> impl Iterator<Item = Option<u8>> + '_ {
        (addr..(addr + length)).map(|addr| self.peek_ppu_memory(addr))
    }

    pub fn peek_ppu_memory(&self, addr: u16) -> Option<u8> {
        match addr {
            0..=0x1FFF => self.cartridge.borrow_mut().ppu_bus_peek(addr),
            0x2000..=0x3FFF => Some(self.vram[self.vram_addr_to_idx(addr)]),
            _ => None,
        }
    }

    pub fn read_ppu_memory(&self, addr: u16) -> PpuResult<u8> {
        self.peek_ppu_memory(addr)
            .ok_or(PpuError::InvalidBusRead(addr))
    }

    pub fn write_ppu_memory(&mut self, addr: u16, value: u8) -> PpuResult<()> {
        // Map memory addresses
        let addr = match addr {
            0x3F10 => 0x3F00,
            addr => addr,
        };
        match addr {
            0..=0x1FFF => {
                self.cartridge.borrow_mut().ppu_bus_write(addr, value)?;
                Ok(())
            }
            0x2000..=0x3FFF => {
                self.vram[self.vram_addr_to_idx(addr)] = value;
                Ok(())
            }
            _ => Err(PpuError::InvalidBusWrite(addr)),
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Registers exposed to CPU bus

    fn increment_address_register(&mut self) -> u16 {
        let addr = self.v_register.value;
        let inc = if self.control_register.vram_add_increment {
            32
        } else {
            1
        };
        self.v_register.value = self.v_register.value.wrapping_add(inc);
        addr.bits(0..14)
    }

    pub fn read_data_register(&mut self) -> PpuResult<u8> {
        let addr = self.increment_address_register();
        let buffer = self.internal_data_buffer;
        self.internal_data_buffer = self.read_ppu_memory(addr)?;
        Ok(buffer)
    }

    pub fn write_data_register(&mut self, value: u8) -> PpuResult<()> {
        let addr = self.increment_address_register();
        self.write_ppu_memory(addr, value)
    }

    pub fn read_status_register(&mut self) -> PpuResult<u8> {
        let status = self.status_register.pack().unwrap()[0];
        self.status_register.vblank_started = false;
        self.register_latch = false;
        Ok(status)
    }

    pub fn cpu_bus_peek(&self, addr: u16) -> Option<u8> {
        match addr {
            OAM_ADDR => Some(self.oam_addr),
            OAM_DATA => Some(self.oam_data[self.oam_addr as usize]),
            PPU_SCROLL => Some(0),
            CONTROL_REGISTER_ADDR => Some(self.control_register.pack().unwrap()[0]),
            MASK_REGISTER_ADDR => Some(self.mask_register.pack().unwrap()[0]),
            STATUS_REGISTER_ADDR => Some(self.status_register.pack().unwrap()[0]),
            _ => None,
        }
    }

    pub fn cpu_bus_read(&mut self, addr: u16) -> PpuResult<u8> {
        match addr {
            OAM_DATA => {
                let value = self.oam_data[self.oam_addr as usize];
                self.oam_addr = self.oam_addr.wrapping_add(1);
                Ok(value)
            }
            DATA_REGISTER_ADDR => self.read_data_register(),
            STATUS_REGISTER_ADDR => self.read_status_register(),
            _ => Ok(self.cpu_bus_peek(addr).unwrap_or_default()),
        }
    }

    pub fn cpu_bus_write(&mut self, addr: u16, value: u8) -> PpuResult<()> {
        match addr {
            OAM_ADDR => {
                self.oam_addr = value;
                Ok(())
            }
            OAM_DATA => {
                self.oam_data[self.oam_addr as usize] = value;
                Ok(())
            }
            PPU_SCROLL => {
                if !self.register_latch {
                    self.t_register.set_coarse_x(value.bits(3..=7) as u16);
                    self.fine_scroll_x = value.bits(0..=2);
                } else {
                    self.t_register.set_coarse_y(value.bits(3..=7) as u16);
                    self.t_register.set_fine_y(value.bits(0..=2) as u16);
                }
                self.register_latch = !self.register_latch;
                Ok(())
            }
            CONTROL_REGISTER_ADDR => {
                self.control_register = ControlRegister::unpack(&[value]).unwrap();
                self.t_register
                    .set_nametable(self.control_register.nametable as u16);
                Ok(())
            }
            MASK_REGISTER_ADDR => {
                self.mask_register = MaskRegister::unpack(&[value]).unwrap();
                Ok(())
            }
            ADDRESS_REGISTER_ADDR => {
                if self.register_latch {
                    self.t_register.set_low_byte(value as u16);
                    self.v_register.value = self.t_register.value;
                } else {
                    self.t_register.set_high_byte(value.bits(0..=5) as u16);
                }
                self.register_latch = !self.register_latch;
                Ok(())
            }
            DATA_REGISTER_ADDR => self.write_data_register(value),
            _ => Err(PpuError::InvalidBusWrite(addr)),
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
        let mut image = ColorImage::new([64 * 8, 60 * 8], Color32::TRANSPARENT);
        let mut addr = VramAddress { value: 0 };
        for scanline in 0..(60 * 8) {
            addr.set_coarse_x(0);
            for coarse_x in 0..64 {
                if let Ok(background) = NametableEntry::new(self, &addr) {
                    if let Ok(pixels) = background.pattern.row_pixels(self, addr.fine_y() as usize)
                    {
                        for (fine_x, pixel) in pixels.enumerate() {
                            if let Ok(color) = self
                                .get_palette_entry(background.palette_id as usize, pixel as usize)
                            {
                                let rgb = if color < 64 {
                                    SYSTEM_PALETTE[color as usize]
                                } else {
                                    Color32::RED
                                };
                                image[(coarse_x * 8 + fine_x, scanline)] = rgb;
                            }
                        }
                    }
                }
                addr.increment_x();
            }
            addr.increment_y();
        }
        image
    }

    pub fn debug_render_pattern_table(&self) -> ColorImage {
        let mut image = ColorImage::new([32 * 8, 16 * 8], Color32::TRANSPARENT);
        for bank in 0..=1_usize {
            for coarse_x in 0..16_usize {
                for coarse_y in 0..16_usize {
                    for fine_y in 0..8_usize {
                        let img_y = coarse_y * 8 + fine_y;
                        let pattern_id = coarse_y * 8 + coarse_x;
                        if let Ok(row) =
                            Pattern::new(bank as u8, pattern_id as u8).row_pixels(self, fine_y)
                        {
                            for (fine_x, pixel) in row.enumerate() {
                                let img_x = bank * (16 * 8) + coarse_x * 8 + fine_x;
                                image[(img_x, img_y)] = PATTERN_PALETTE[pixel as usize];
                            }
                        }
                    }
                }
            }
        }
        image
    }
}

////////////////////////////////////////////////////////////////////////////////
// Framebuffer

#[derive(Decode, Encode, Clone)]
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
                .map(|color| {
                    if *color < 64 {
                        SYSTEM_PALETTE[*color as usize]
                    } else {
                        Color32::RED
                    }
                })
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
    pub fn new(ppu: &Ppu, addr: &VramAddress) -> PpuResult<NametableEntry> {
        let nametable_value = ppu.read_ppu_memory(addr.tile_addr())?;
        let attr_byte = ppu.read_ppu_memory(addr.attribute_addr())?;
        let attribute = match (addr.coarse_x() % 4 / 2, addr.coarse_y() % 4 / 2) {
            (0, 0) => attr_byte & 0b11,
            (1, 0) => (attr_byte >> 2) & 0b11,
            (0, 1) => (attr_byte >> 4) & 0b11,
            (1, 1) => (attr_byte >> 6) & 0b11,
            (_, _) => panic!("should not happen"),
        };

        Ok(NametableEntry {
            pattern: Pattern::new(
                ppu.control_register.background_pattern_addr as u8,
                nametable_value,
            ),
            palette_id: attribute,
        })
    }

    pub fn from_coarse_x_y(
        ppu: &Ppu,
        mut coarse_x: usize,
        mut coarse_y: usize,
    ) -> PpuResult<NametableEntry> {
        let bank_x = coarse_x / 32;
        let bank_y = coarse_y / 30;
        let bank = bank_x + bank_y * 2 + ppu.control_register.nametable as usize;
        coarse_x %= 32;
        coarse_y %= 30;
        let base_addr = 0x2000 + (0x0400 * bank);

        let addr = base_addr + coarse_y * 0x20 + coarse_x;
        let nametable_value = ppu.read_ppu_memory(addr as u16)?;

        let attr_table_idx = coarse_y / 4 * 8 + coarse_x / 4;
        let attr_byte = ppu.read_ppu_memory((base_addr + 0x03C0 + attr_table_idx) as u16)?;
        let attribute = match (coarse_x % 4 / 2, coarse_y % 4 / 2) {
            (0, 0) => attr_byte & 0b11,
            (1, 0) => (attr_byte >> 2) & 0b11,
            (0, 1) => (attr_byte >> 4) & 0b11,
            (1, 1) => (attr_byte >> 6) & 0b11,
            (_, _) => panic!("should not happen"),
        };

        Ok(NametableEntry {
            pattern: Pattern::new(
                ppu.control_register.background_pattern_addr as u8,
                nametable_value,
            ),
            palette_id: attribute,
        })
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

    pub fn row_pixels(&self, ppu: &Ppu, mut y: usize) -> PpuResult<impl Iterator<Item = u8>> {
        if self.data.attr.flip_v {
            y = 7 - y;
        }
        let mut row: Vec<u8> = self.pattern.row_pixels(ppu, y)?.collect();
        if self.data.attr.flip_h {
            row.reverse();
        }
        Ok(row.into_iter())
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

    pub fn row_pixels(&self, ppu: &Ppu, y: usize) -> PpuResult<impl Iterator<Item = u8> + '_> {
        let mut low = ppu.read_ppu_memory(self.addr + y as u16)?;
        let mut high = ppu.read_ppu_memory(self.addr + y as u16 + 8)?;

        Ok((0..8).map(move |_| {
            let low_bit = low & 0b1000_0000 > 0;
            let high_bit = high & 0b1000_0000 > 0;
            low <<= 1;
            high <<= 1;
            (high_bit as u8) << 1 | (low_bit as u8)
        }))
    }
}

////////////////////////////////////////////////////////////////////////////////
// VRAM Address

macro_rules! field_from_bits {
    ($get: ident, $set: ident, $range: expr) => {
        pub fn $get(&self) -> u16 {
            self.value.bits($range)
        }

        pub fn $set(&mut self, value: u16) {
            self.value.set_bits($range, value);
        }
    };
}

macro_rules! field_from_bit {
    ($get: ident, $set: ident, $bit: literal) => {
        pub fn $get(&self) -> bool {
            self.value.bit($bit)
        }

        pub fn $set(&mut self, value: bool) {
            self.value.set_bit($bit, value);
        }
    };
}
/// yyy NN YYYYY XXXXX
/// ||| || ||||| +++++-- coarse X scroll
/// ||| || +++++-------- coarse Y scroll
/// ||| ++-------------- nametable select
/// +++----------------- fine Y scroll
///
/// Explore using macros to reduce boilerplate
#[derive(Debug, Default, Encode, Decode, Clone)]
pub struct VramAddress {
    value: u16,
}

impl Display for VramAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:04X} (fy:{:X} n:{:X} y:{:02X} x:{:02X})",
            self.value,
            self.fine_y(),
            self.nametable(),
            self.coarse_y(),
            self.coarse_x()
        )
    }
}

impl VramAddress {
    field_from_bits!(coarse_x, set_coarse_x, 0..=4);
    field_from_bits!(coarse_y, set_coarse_y, 5..=9);
    field_from_bit!(nametable_x, set_nametable_x, 10);
    field_from_bit!(nametable_y, set_nametable_y, 11);
    field_from_bits!(nametable, set_nametable, 10..=11);
    field_from_bits!(fine_y, set_fine_y, 12..=14);
    field_from_bits!(low_byte, set_low_byte, 0..=7);
    field_from_bits!(high_byte, set_high_byte, 8..=15);

    pub fn increment_x(&mut self) {
        if self.coarse_x() >= 31 {
            self.set_coarse_x(0);
            self.set_nametable_x(!self.nametable_x());
        } else {
            self.set_coarse_x(self.coarse_x() + 1);
        }
    }
    pub fn increment_y(&mut self) {
        if self.fine_y() < 7 {
            self.set_fine_y(self.fine_y() + 1);
        } else {
            self.set_fine_y(0);
            if self.coarse_y() >= 29 {
                self.set_coarse_y(0);
                self.set_nametable_y(!self.nametable_y());
            } else {
                self.set_coarse_y(self.coarse_y() + 1);
            }
        }
    }

    pub fn tile_addr(&self) -> u16 {
        0x2000 | self.value.bits(0..=11)
    }

    pub fn attribute_addr(&self) -> u16 {
        0x23C0 | (self.value & 0x0C00) | ((self.value >> 4) & 0x38) | ((self.value >> 2) & 0x07)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Registers

#[derive(PackedStruct, Encode, Decode, Clone, Debug, Default, Copy, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct ControlRegister {
    pub generate_nmi: bool,
    pub master_slave_select: bool,
    pub sprite_size: bool,
    pub background_pattern_addr: bool,
    pub sprite_pattern_addr: bool,
    pub vram_add_increment: bool,
    #[packed_field(bits = "6..=7")]
    pub nametable: u8,
}

#[derive(PackedStruct, Encode, Decode, Clone, Debug, Default, Copy, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct StatusRegister {
    pub vblank_started: bool,
    pub sprite_zero_hit: bool,
    pub sprite_overflow: bool,
}

impl StatusRegister {
    pub fn pretty_print(&self) -> String {
        let mut chars: Vec<&str> = Vec::new();
        chars.push(if self.sprite_overflow { "Ov" } else { ".." });
        chars.push(if self.sprite_zero_hit { "Sz" } else { ".." });
        chars.push(if self.vblank_started { "Vs" } else { ".." });
        chars.iter().join("")
    }
}

#[derive(PackedStruct, Encode, Decode, Clone, Debug, Default, Copy, PartialEq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct MaskRegister {
    pub emphasize_blue: bool,
    pub emphasize_green: bool,
    pub emphasize_red: bool,
    pub show_sprites: bool,
    pub show_background: bool,
    pub mask_sprites: bool,
    pub mask_background: bool,
    pub grayscale: bool,
}

impl MaskRegister {
    pub fn pretty_print(&self) -> String {
        let mut chars: Vec<&str> = Vec::new();
        chars.push(if self.grayscale { "Gr" } else { ".." });
        chars.push(if self.mask_background { "Mb" } else { ".." });
        chars.push(if self.mask_sprites { "Ms" } else { ".." });
        chars.push(if self.show_background { "Sb" } else { ".." });
        chars.push(if self.show_sprites { "Ss" } else { ".." });
        chars.push(if self.emphasize_red { "Er" } else { ".." });
        chars.push(if self.emphasize_green { "Eg" } else { ".." });
        chars.push(if self.emphasize_blue { "Eb" } else { ".." });
        chars.iter().join("")
    }
}

////////////////////////////////////////////////////////////////////////////////
// Palette Lookup Table

pub static PATTERN_PALETTE: [Color32; 4] = [
    Color32::from_rgb(0x00, 0x00, 0x00),
    Color32::from_rgb(0xFF, 0x00, 0x00),
    Color32::from_rgb(0x00, 0xFF, 0x00),
    Color32::from_rgb(0x00, 0x00, 0xFF),
];

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

#[allow(clippy::unusual_byte_groupings)]
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_ppu() -> Ppu {
        Ppu::new(Rc::new(RefCell::new(Cartridge::new())))
    }

    #[test]
    pub fn test_data_register() {
        let mut ppu = create_test_ppu();
        let mut chr = vec![0; 0x2000];
        chr[0x1000] = 0x12;
        chr[0x1001] = 0x34;
        ppu.cartridge.borrow_mut().load_data(&[], &chr);

        ppu.cpu_bus_write(ADDRESS_REGISTER_ADDR, 0x10).unwrap();
        ppu.cpu_bus_write(ADDRESS_REGISTER_ADDR, 0x00).unwrap();
        assert_eq!(ppu.v_register.value, 0x1000);
        assert_eq!(ppu.cpu_bus_read(DATA_REGISTER_ADDR).unwrap(), 0x00);
        assert_eq!(ppu.v_register.value, 0x1001);
        assert_eq!(ppu.cpu_bus_read(DATA_REGISTER_ADDR).unwrap(), 0x12);
        assert_eq!(ppu.v_register.value, 0x1002);
        assert_eq!(ppu.cpu_bus_read(DATA_REGISTER_ADDR).unwrap(), 0x34);
    }

    #[test]
    pub fn test_addr_register_clipping() {
        let mut ppu = create_test_ppu();
        ppu.cpu_bus_write(ADDRESS_REGISTER_ADDR, 0xFF).unwrap();
        ppu.cpu_bus_write(ADDRESS_REGISTER_ADDR, 0xFF).unwrap();
        println!("{:016b}", ppu.v_register.value);
        println!("{:016b}", 0x3FFF);
        assert_eq!(ppu.v_register.value, 0x3FFF);
    }

    #[test]
    pub fn test_v_and_t_register() {
        // Following the example in https://www.nesdev.org/wiki/PPU_scrolling#Summary
        let mut ppu = create_test_ppu();

        // Verify setting nametable via control register.
        ppu.cpu_bus_write(0x2000, 0b0000_0011).unwrap();
        assert_eq!(ppu.t_register.value, 0b000_11_00000_00000);

        // Verify register latch is reset via status register reads.
        ppu.register_latch = true;
        ppu.cpu_bus_read(0x2002).unwrap();
        assert!(!ppu.register_latch);

        // Verify first scroll write, setting coarse and fine x.
        ppu.cpu_bus_write(0x2005, 0b01111101).unwrap();
        assert_eq!(ppu.t_register.value, 0b000_11_00000_01111);
        assert_eq!(ppu.fine_scroll_x, 0b101);
        assert!(ppu.register_latch);

        // Verify second scroll write, setting coarse and fine y.
        ppu.cpu_bus_write(0x2005, 0b01011110).unwrap();
        assert_eq!(ppu.t_register.value, 0b110_11_01011_01111);
        assert!(!ppu.register_latch);

        // Verify first address register write. Writing high byte (except bit 14, which is set to 0).
        ppu.cpu_bus_write(0x2006, 0b11111101).unwrap();
        assert_eq!(ppu.t_register.value, 0b011_11_01011_01111);
        assert!(ppu.register_latch);

        // Verify second address register write. Writing low byte. Then copying from t to v.
        ppu.cpu_bus_write(0x2006, 0b11110000).unwrap();
        assert_eq!(ppu.t_register.value, 0b011_11_01111_10000);
        assert_eq!(ppu.v_register.value, ppu.t_register.value);
        assert!(!ppu.register_latch);
    }
}
