mod mmc1;
mod nrom;
mod uxrom;

use std::fmt::Formatter;

use anyhow::anyhow;
use anyhow::Result;
use bincode::Decode;
use bincode::Encode;
use nrom::NromMapper;
use packed_struct::prelude::*;
use thiserror::Error;

use self::mmc1::Mmc1Mapper;
use self::uxrom::UxRomMapper;

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
    fn get_mirroring_mode(&self) -> MirroringMode;
    fn persistent_data(&self) -> Vec<u8>;

    fn cpu_bus_peek(&self, addr: u16) -> Option<u8>;
    fn cpu_bus_read(&mut self, addr: u16) -> CartridgeResult<u8>;
    fn cpu_bus_write(&mut self, addr: u16, value: u8) -> CartridgeResult<()>;

    fn ppu_bus_peek(&self, addr: u16) -> Option<u8>;
    fn ppu_bus_read(&mut self, addr: u16) -> CartridgeResult<u8>;
    fn ppu_bus_write(&mut self, addr: u16, value: u8) -> CartridgeResult<()>;
}

#[derive(Default, Encode, Decode, Clone, Copy)]
pub enum MirroringMode {
    #[default]
    Horizontal,
    Vertical,
    FourScreen,
    SingleLower,
    SingleUpper,
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
    Nrom(NromMapper),
    Mmc1(Mmc1Mapper),
    UxRom(UxRomMapper),
}

#[derive(Encode, Decode, Clone)]
pub struct Cartridge {
    mapper: MapperEnum,
    pub has_persistent_data: bool,
}

impl Cartridge {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            mapper: MapperEnum::Nrom(NromMapper::default()),
            has_persistent_data: false,
        }
    }

    pub fn load_nrom_with_data(&mut self, prg: &[u8], chr: &[u8]) {
        self.mapper = MapperEnum::Nrom(NromMapper::new(prg, chr, MirroringMode::Horizontal, None));
    }

    pub fn load_ines(&mut self, raw: &[u8], persistent_data: Option<&[u8]>) -> Result<()> {
        let header = InesHeader::unpack_from_slice(&raw[0..11])?;
        if header.magic != [78, 69, 83, 26] {
            return Err(anyhow!("Expected NES header."));
        }
        let mapper_id = header.lower_mapper | (header.upper_mapper << 4);
        //println!("Loading iNES (mapper {:}): {:#?}", mapper_id, header);
        let prg_len = header.prg_size as usize * 16 * 1024;
        let chr_len = header.chr_size as usize * 8 * 1024;
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

        self.has_persistent_data = header.has_battery_ram;

        let mirroring_mode = if header.four_screen {
            MirroringMode::FourScreen
        } else if header.mirroring {
            MirroringMode::Vertical
        } else {
            MirroringMode::Horizontal
        };

        match mapper_id {
            0 => {
                self.mapper = MapperEnum::Nrom(NromMapper::new(
                    &raw[prg_start..prg_end],
                    &raw[prg_end..chr_end],
                    mirroring_mode,
                    persistent_data,
                ))
            }
            1 => {
                self.mapper = MapperEnum::Mmc1(Mmc1Mapper::new(
                    &raw[prg_start..prg_end],
                    &raw[prg_end..chr_end],
                    persistent_data,
                ))
            }
            2 => {
                self.mapper = MapperEnum::UxRom(UxRomMapper::new(
                    &raw[prg_start..prg_end],
                    &raw[prg_end..chr_end],
                    mirroring_mode,
                    persistent_data,
                ))
            }
            _ => return Err(anyhow!("Unsupported mapper {mapper_id}")),
        };
        Ok(())
    }

    pub fn persistent_data(&self) -> Vec<u8> {
        match &self.mapper {
            MapperEnum::Nrom(mapper) => mapper.persistent_data(),
            MapperEnum::Mmc1(mapper) => mapper.persistent_data(),
            MapperEnum::UxRom(mapper) => mapper.persistent_data(),
        }
    }

    pub fn get_mirroring_mode(&self) -> MirroringMode {
        match &self.mapper {
            MapperEnum::Nrom(mapper) => mapper.get_mirroring_mode(),
            MapperEnum::Mmc1(mapper) => mapper.get_mirroring_mode(),
            MapperEnum::UxRom(mapper) => mapper.get_mirroring_mode(),
        }
    }

    pub fn cpu_bus_peek(&self, addr: u16) -> Option<u8> {
        match &self.mapper {
            MapperEnum::Nrom(mapper) => mapper.cpu_bus_peek(addr),
            MapperEnum::Mmc1(mapper) => mapper.cpu_bus_peek(addr),
            MapperEnum::UxRom(mapper) => mapper.cpu_bus_peek(addr),
        }
    }

    pub fn cpu_bus_read(&mut self, addr: u16) -> CartridgeResult<u8> {
        match &mut self.mapper {
            MapperEnum::Nrom(mapper) => mapper.cpu_bus_read(addr),
            MapperEnum::Mmc1(mapper) => mapper.cpu_bus_read(addr),
            MapperEnum::UxRom(mapper) => mapper.cpu_bus_read(addr),
        }
    }

    pub fn cpu_bus_write(&mut self, addr: u16, value: u8) -> CartridgeResult<()> {
        match &mut self.mapper {
            MapperEnum::Nrom(mapper) => mapper.cpu_bus_write(addr, value),
            MapperEnum::Mmc1(mapper) => mapper.cpu_bus_write(addr, value),
            MapperEnum::UxRom(mapper) => mapper.cpu_bus_write(addr, value),
        }
    }

    pub fn ppu_bus_peek(&self, addr: u16) -> Option<u8> {
        match &self.mapper {
            MapperEnum::Nrom(mapper) => mapper.ppu_bus_peek(addr),
            MapperEnum::Mmc1(mapper) => mapper.ppu_bus_peek(addr),
            MapperEnum::UxRom(mapper) => mapper.ppu_bus_peek(addr),
        }
    }

    pub fn ppu_bus_read(&mut self, addr: u16) -> CartridgeResult<u8> {
        match &mut self.mapper {
            MapperEnum::Nrom(mapper) => mapper.ppu_bus_read(addr),
            MapperEnum::Mmc1(mapper) => mapper.ppu_bus_read(addr),
            MapperEnum::UxRom(mapper) => mapper.ppu_bus_read(addr),
        }
    }

    pub fn ppu_bus_write(&mut self, addr: u16, value: u8) -> CartridgeResult<()> {
        match &mut self.mapper {
            MapperEnum::Nrom(mapper) => mapper.ppu_bus_write(addr, value),
            MapperEnum::Mmc1(mapper) => mapper.ppu_bus_write(addr, value),
            MapperEnum::UxRom(mapper) => mapper.ppu_bus_write(addr, value),
        }
    }
}
