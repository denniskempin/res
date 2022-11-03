mod nrom;

use std::fmt::Formatter;

use anyhow::anyhow;
use anyhow::Result;
use bincode::Decode;
use bincode::Encode;
use nrom::NromMapper;
use packed_struct::prelude::*;
use thiserror::Error;

#[derive(Error)]
pub enum CartridgeError {
    #[error("Invalid read from 0x{0:04X}")]
    InvalidRead(u16),
    #[error("Invalid write to 0x{0:04X}")]
    InvalidWrite(u16),
}

impl std::fmt::Debug for CartridgeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self)
    }
}

pub type CartridgeResult<T> = std::result::Result<T, CartridgeError>;

trait Mapper: Encode + Decode + Clone + Default {
    fn cpu_bus_peek(&self, addr: u16) -> Option<u8>;
    fn cpu_bus_read(&mut self, addr: u16) -> CartridgeResult<u8>;
    fn cpu_bus_write(&mut self, addr: u16, value: u8) -> CartridgeResult<()>;

    fn ppu_bus_peek(&self, addr: u16) -> Option<u8>;
    fn ppu_bus_read(&mut self, addr: u16) -> CartridgeResult<u8>;
    fn ppu_bus_write(&mut self, addr: u16, value: u8) -> CartridgeResult<()>;
}

#[derive(Default, Encode, Decode, Clone)]
pub enum MirroringMode {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(PackedStruct, Default, Debug, Copy, Clone)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "11")]
pub struct InesHeader {
    magic: [u8; 4],
    prg_size: u8,
    chr_size: u8,
    #[packed_field(size_bits = "4")]
    lower_mapper: u8,
    four_screen: bool,
    trainer: bool,
    has_battery_ram: bool,
    mirroring: bool,
    #[packed_field(size_bits = "4")]
    upper_mapper: u8,
    #[packed_field(size_bits = "2")]
    format: u8,
    playchoice10: bool,
    vs_unisystem: bool,
    _flags8: u8,
    _flags9: u8,
    _flags10: u8,
}

/// Enum of all supported Mappers.
/// This is used in place of Box<Mapper> since Encode/Decode do not support trait objects.
/// TODO: Consider using serde and https://github.com/dtolnay/typetag
#[derive(Encode, Decode, Clone)]
enum MapperEnum {
    NromMapper(NromMapper),
}

impl Default for MapperEnum {
    fn default() -> Self {
        MapperEnum::NromMapper(NromMapper::default())
    }
}

#[derive(Default, Encode, Decode, Clone)]
pub struct Cartridge {
    mapper: MapperEnum,
    pub mirroring_mode: MirroringMode,
}

impl Cartridge {
    pub fn load_data(&mut self, prg: &[u8], chr: &[u8]) {
        self.mapper = MapperEnum::NromMapper(NromMapper::new(prg, chr));
    }

    pub fn load_ines(&mut self, raw: &[u8]) -> Result<()> {
        let header = InesHeader::unpack_from_slice(&raw[0..11])?;
        if header.magic != [78, 69, 83, 26] {
            return Err(anyhow!("Expected NES header."));
        }
        let mapper_id = header.lower_mapper | (header.upper_mapper << 4);
        println!("Loading iNES (mapper {:}): {:?}", mapper_id, header);
        let prg_len = header.prg_size as usize * 16 * 1024;
        let chr_len = header.chr_size as usize * 8 * 1024;
        self.mirroring_mode = if header.mirroring {
            MirroringMode::Vertical
        } else {
            MirroringMode::Horizontal
        };
        let prg_start = 16;
        let prg_end = prg_start + prg_len;
        let chr_end = prg_end + chr_len;

        if chr_end != raw.len() {
            return Err(anyhow!(
                "Expected rom size to be {}, but it is {}",
                chr_end,
                raw.len()
            ));
        }

        match mapper_id {
            0 => {
                self.mapper = MapperEnum::NromMapper(NromMapper::new(
                    &raw[prg_start..prg_end],
                    &raw[prg_end..chr_end],
                ))
            }
            _ => return Err(anyhow!("Unsupported mapper {mapper_id}")),
        };
        Ok(())
    }

    pub fn cpu_bus_peek(&self, addr: u16) -> Option<u8> {
        match &self.mapper {
            MapperEnum::NromMapper(mapper) => mapper.cpu_bus_peek(addr),
        }
    }

    pub fn cpu_bus_read(&mut self, addr: u16) -> CartridgeResult<u8> {
        match &mut self.mapper {
            MapperEnum::NromMapper(mapper) => mapper.cpu_bus_read(addr),
        }
    }

    pub fn cpu_bus_write(&mut self, addr: u16, value: u8) -> CartridgeResult<()> {
        match &mut self.mapper {
            MapperEnum::NromMapper(mapper) => mapper.cpu_bus_write(addr, value),
        }
    }

    pub fn ppu_bus_peek(&self, addr: u16) -> Option<u8> {
        match &self.mapper {
            MapperEnum::NromMapper(mapper) => mapper.ppu_bus_peek(addr),
        }
    }

    pub fn ppu_bus_read(&mut self, addr: u16) -> CartridgeResult<u8> {
        match &mut self.mapper {
            MapperEnum::NromMapper(mapper) => mapper.ppu_bus_read(addr),
        }
    }

    pub fn ppu_bus_write(&mut self, addr: u16, value: u8) -> CartridgeResult<()> {
        match &mut self.mapper {
            MapperEnum::NromMapper(mapper) => mapper.ppu_bus_write(addr, value),
        }
    }
}
