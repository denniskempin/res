use anyhow::anyhow;
use anyhow::Result;
use bincode::Decode;
use bincode::Encode;
use intbits::Bits;
use packed_struct::prelude::*;

#[derive(Debug)]
pub enum CartridgeError {
    InvalidRead(u16),
    InvalidWrite(u16),
}

pub type CartridgeResult<T> = std::result::Result<T, CartridgeError>;


#[derive(Default, Encode, Decode, Clone)]
pub enum MirroringMode {
    #[default]
    Horizontal,
    Vertical,
}


#[derive(PackedStruct, Default, Debug, Copy, Clone)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "12")]
pub struct InesHeader {
    magic: [u8; 4],
    prg_size: u8,
    chr_size: u8,
    #[packed_field(size_bits = 4)]
    lower_mapper: u8,
    four_screen: bool,
    trainer: bool,
    has_battery_ram: bool,
    mirroring: bool,
    #[packed_field(size_bits = 4)]
    upper_mapper: u8,
    #[packed_field(size_bits = 2)]
    format: u8,
    playchoice10: bool,
    vs_unisystem: bool,
    _flags9: u8,
    _flags10: u8,
}

#[derive(Default, Encode, Decode, Clone)]
pub struct Cartridge {
    pub prg: Vec<u8>,
    pub prg_ram: Vec<u8>,
    pub chr: Vec<u8>,
    pub mirroring_mode: MirroringMode,
}

impl Cartridge {
    pub fn load_program(&mut self, data: &[u8]) {
        self.prg = data.into();
    }

    pub fn load_ines(&mut self, raw: &[u8]) -> Result<()> {
        let header = InesHeader::unpack_from_slice(&raw[0..12])?;
        if header.magic != [78, 69, 83, 26] {
            return Err(anyhow!("Expected NES header."));
        }
        let mapper = header.lower_mapper | (header.upper_mapper << 4);
        println!("Loading iNES (mapper {:}): {:?}", mapper, header);
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

        self.prg = raw[prg_start..prg_end].to_vec();
        self.chr = raw[prg_end..chr_end].to_vec();
        self.prg_ram.resize(8 * 1024, 0);

        Ok(())
    }

    pub fn cpu_bus_peek(&self, addr: u16) -> Option<u8> {
        match addr {
            0x6000..=0x7FFF => {
                if !self.prg_ram.is_empty() {
                    let ram_size = self.prg_ram.len();
                    Some(self.prg_ram[(addr as usize - 0x6000) % ram_size])
                } else {
                    None
                }
            },
            0x8000..=0xFFFF => {
                if !self.prg.is_empty() {
                    let addr = addr as usize % self.prg.len();
                    Some(self.prg[addr])
                } else {
                    None
                }
            }
            _ => None
        }
    }

    pub fn cpu_bus_read(&mut self, addr: u16) -> CartridgeResult<u8> {
        self.cpu_bus_peek(addr).ok_or(CartridgeError::InvalidRead(addr))
    }

    pub fn cpu_bus_write(&mut self, addr: u16, value: u8) -> CartridgeResult<()> {
        match addr {
            0x6000..=0x7FFF => {
                if !self.prg_ram.is_empty() {
                    let ram_size = self.prg_ram.len();
                    self.prg_ram[(addr as usize - 0x6000) % ram_size] = value;
                    Ok(())
                } else {
                    Err(CartridgeError::InvalidWrite(addr))
                }
            }
            _ => Err(CartridgeError::InvalidWrite(addr))
        }
    }
}
