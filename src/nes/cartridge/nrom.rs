use bincode::Decode;
use bincode::Encode;

use super::CartridgeError;
use super::CartridgeResult;
use super::Mapper;
use super::MirroringMode;

#[derive(Encode, Decode, Clone)]
pub struct NromMapper {
    pub prg: Vec<u8>,
    pub chr: Vec<u8>,
    pub ram: Vec<u8>,
    pub mirroring_mode: MirroringMode,
}

impl NromMapper {
    const RAM_SIZE: usize = 8 * 1024;

    pub fn new(prg: &[u8], chr: &[u8], mirroring_mode: MirroringMode) -> NromMapper {
        let mut m = NromMapper {
            prg: prg.to_vec(),
            chr: chr.to_vec(),
            ram: vec![0; NromMapper::RAM_SIZE],
            mirroring_mode,
        };
        m.chr.resize(8 * 1024, 0);
        m
    }
}

impl Default for NromMapper {
    fn default() -> Self {
        Self::new(&[], &[], MirroringMode::Horizontal)
    }
}

impl Mapper for NromMapper {
    fn cpu_bus_peek(&self, addr: u16) -> Option<u8> {
        match addr {
            0x6000..=0x7FFF => Some(self.ram[(addr as usize - 0x6000) % NromMapper::RAM_SIZE]),
            0x8000..=0xFFFF => {
                if !self.prg.is_empty() {
                    let addr = addr as usize % self.prg.len();
                    Some(self.prg[addr])
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn cpu_bus_read(&mut self, addr: u16) -> CartridgeResult<u8> {
        Ok(self.cpu_bus_peek(addr).unwrap_or_default())
    }

    fn cpu_bus_write(&mut self, addr: u16, value: u8) -> CartridgeResult<()> {
        match addr {
            0x6000..=0x7FFF => {
                self.ram[(addr as usize - 0x6000) % NromMapper::RAM_SIZE] = value;
                Ok(())
            }
            _ => Err(CartridgeError::InvalidWrite(addr)),
        }
    }

    fn ppu_bus_peek(&self, addr: u16) -> Option<u8> {
        if (addr as usize) < self.chr.len() {
            Some(self.chr[addr as usize])
        } else {
            None
        }
    }

    fn ppu_bus_read(&mut self, addr: u16) -> CartridgeResult<u8> {
        self.ppu_bus_peek(addr)
            .ok_or(CartridgeError::InvalidRead(addr))
    }

    fn ppu_bus_write(&mut self, _addr: u16, _value: u8) -> CartridgeResult<()> {
        // Some games will try to write to character ROM and expect it to NOOP.
        Ok(())
    }

    fn get_mirroring_mode(&self) -> MirroringMode {
        self.mirroring_mode
    }
}
