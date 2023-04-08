use bincode::Decode;
use bincode::Encode;
use intbits::Bits;

use super::CartridgeError;
use super::CartridgeResult;
use super::Mapper;
use super::MirroringMode;

const PRG_BANK_SIZE: usize = 16 * 1024;

#[derive(Encode, Decode, Clone)]
pub struct UxRomMapper {
    pub prg: Vec<u8>,
    pub chr: Vec<u8>,
    pub ram: Vec<u8>,
    pub control_register: u8,
    pub last_bank: usize,
    pub mirroring_mode: MirroringMode,
}

impl UxRomMapper {
    const RAM_SIZE: usize = 8 * 1024;

    pub fn new(
        prg: &[u8],
        chr: &[u8],
        mirroring_mode: MirroringMode,
        persistent_data: Option<&[u8]>,
    ) -> UxRomMapper {
        UxRomMapper {
            prg: prg.to_vec(),
            chr: if chr.is_empty() {
                vec![0; UxRomMapper::RAM_SIZE]
            } else {
                chr.to_vec()
            },
            ram: persistent_data
                .unwrap_or(&[0; UxRomMapper::RAM_SIZE])
                .to_vec(),
            control_register: 0,
            mirroring_mode,
            last_bank: (prg.len() / PRG_BANK_SIZE) - 1,
        }
    }
}

impl Default for UxRomMapper {
    fn default() -> Self {
        Self::new(&[], &[], MirroringMode::Horizontal, None)
    }
}

impl Mapper for UxRomMapper {
    fn cpu_bus_peek(&self, addr: u16) -> Option<u8> {
        match addr {
            0x6000..=0x7FFF => Some(self.ram[(addr as usize - 0x6000) % UxRomMapper::RAM_SIZE]),
            0x8000..=0xBFFF => Some(
                self.prg[self.control_register as usize * PRG_BANK_SIZE + (addr as usize - 0x8000)],
            ),
            0xC000..=0xFFFF => {
                Some(self.prg[self.last_bank * PRG_BANK_SIZE + (addr as usize - 0xC000)])
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
                self.ram[(addr as usize - 0x6000) % UxRomMapper::RAM_SIZE] = value;
                Ok(())
            }
            0x8000..=0xFFFF => {
                self.control_register = value.bits(0..4);
                Ok(())
            }
            _ => Err(CartridgeError::InvalidWrite(addr)),
        }
    }

    fn ppu_bus_peek(&self, addr: u16) -> Option<u8> {
        match addr {
            0x0000..=0x3FFF => Some(self.chr[addr as usize]),
            _ => None,
        }
    }

    fn ppu_bus_read(&mut self, addr: u16) -> CartridgeResult<u8> {
        self.ppu_bus_peek(addr)
            .ok_or(CartridgeError::InvalidRead(addr))
    }

    fn ppu_bus_write(&mut self, addr: u16, value: u8) -> CartridgeResult<()> {
        match addr {
            0x0000..=0x3FFF => {
                self.chr[addr as usize] = value;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn get_mirroring_mode(&self) -> super::MirroringMode {
        self.mirroring_mode
    }

    fn persistent_data(&self) -> Vec<u8> {
        self.ram.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::UxRomMapper;
    use super::PRG_BANK_SIZE;
    use crate::cartridge::Mapper;
    use crate::cartridge::MirroringMode;

    fn test_mapper() -> UxRomMapper {
        // Create a mapper with the bank number in the first byte of each bank to allow us to
        // identify them after mapping.
        let mut prg = vec![0x00_u8; 4 * PRG_BANK_SIZE];
        prg[0] = 0x01;
        prg[PRG_BANK_SIZE] = 0x02;
        prg[2 * PRG_BANK_SIZE] = 0x03;
        prg[3 * PRG_BANK_SIZE] = 0x04;
        UxRomMapper::new(prg.as_slice(), &[], MirroringMode::Horizontal, None)
    }

    #[test]
    pub fn test_prg_mapping() {
        let mut mapper = test_mapper();

        // The first memory block will be switched by the control register
        // The second memory block will always be fixed to the last bank

        mapper.cpu_bus_write(0xE000, 0).unwrap();
        assert_eq!(mapper.cpu_bus_peek(0x8000), Some(0x01));
        assert_eq!(mapper.cpu_bus_peek(0xC000), Some(0x04));

        mapper.cpu_bus_write(0xE000, 1).unwrap();
        assert_eq!(mapper.cpu_bus_peek(0x8000), Some(0x02));
        assert_eq!(mapper.cpu_bus_peek(0xC000), Some(0x04));

        mapper.cpu_bus_write(0xE000, 2).unwrap();
        assert_eq!(mapper.cpu_bus_peek(0x8000), Some(0x03));
        assert_eq!(mapper.cpu_bus_peek(0xC000), Some(0x04));
    }
}
