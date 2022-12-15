use bincode::Decode;
use bincode::Encode;
use intbits::Bits;

use super::CartridgeError;
use super::CartridgeResult;
use super::Mapper;

const PRG_BANK_SIZE: usize = 16 * 1024;

#[derive(Encode, Decode, Clone)]
pub struct Mmc1Mapper {
    pub prg: Vec<u8>,
    pub chr: Vec<u8>,
    pub ram: Vec<u8>,
    pub shift_register: u8,
    pub control_register: u8,
    pub chr_bank0_register: u8,
    pub chr_bank1_register: u8,
    pub prg_bank_register: u8,
}

impl Mmc1Mapper {
    const RAM_SIZE: usize = 8 * 1024;

    pub fn new(prg: &[u8], chr: &[u8]) -> Mmc1Mapper {
        Mmc1Mapper {
            prg: prg.to_vec(),
            chr: if chr.is_empty() {
                vec![0; Mmc1Mapper::RAM_SIZE]
            } else {
                chr.to_vec()
            },
            ram: vec![0; Mmc1Mapper::RAM_SIZE],
            shift_register: 0b100000,
            control_register: 0,
            chr_bank0_register: 0,
            chr_bank1_register: 0,
            prg_bank_register: 0,
        }
    }

    fn get_prg_data(&self, addr: u16) -> u8 {
        match self.control_register.bits(2..=3) {
            0..=1 => {
                // 32kB bank switching mode (ignores 1st bit of bank register).
                let bank = (self.prg_bank_register & 0xFE) as usize * PRG_BANK_SIZE;
                self.prg[bank + addr as usize]
            }
            2 => {
                if addr < 0x4000 {
                    self.prg[addr as usize]
                } else {
                    let bank = self.prg_bank_register as usize * PRG_BANK_SIZE;
                    self.prg[bank + (addr - 0x4000) as usize]
                }
            }
            3 => {
                if addr < 0x4000 {
                    let bank = self.prg_bank_register as usize * PRG_BANK_SIZE;
                    self.prg[bank + addr as usize]
                } else {
                    let bank = self.prg.len() - PRG_BANK_SIZE;
                    self.prg[bank + (addr - 0x4000) as usize]
                }
            }
            _ => panic!("This should not be possible"),
        }
    }

    fn get_chr_index(&self, addr: u16) -> usize {
        let chr_8kb_mode = self.control_register.bit(5);
        let bank_size = if chr_8kb_mode { 8 * 1024 } else { 4 * 1024 };
        let bank = self.chr_bank0_register as usize * bank_size;
        bank + addr as usize
    }

    fn write_shift_register(&mut self, addr: u16, value: u8) {
        if value.bit(7) {
            // Write with bit 7 set will reset the register.
            self.shift_register = 0b100000;
        } else {
            // Shift new bit into the register
            self.shift_register >>= 1;
            self.shift_register.set_bit(5, value.bit(0));
            // Once the 1 reaches bit 0, we have added 5 bits. Write the
            if self.shift_register.bit(0) {
                let register_value = self.shift_register.bits(1..=5);
                match addr {
                    0x8000..=0x9FFF => self.control_register = register_value,
                    0xA000..=0xBFFF => self.chr_bank0_register = register_value,
                    0xC000..=0xDFFF => self.chr_bank1_register = register_value,
                    0xE000..=0xFFFF => self.prg_bank_register = register_value,
                    _ => panic!("Invalid shift register address: 0x{addr:04X}"),
                };
                self.shift_register = 0b100000;
            }
        }
    }
}

impl Default for Mmc1Mapper {
    fn default() -> Self {
        Self::new(&[], &[])
    }
}

impl Mapper for Mmc1Mapper {
    fn cpu_bus_peek(&self, addr: u16) -> Option<u8> {
        match addr {
            0x6000..=0x7FFF => Some(self.ram[(addr as usize - 0x6000) % Mmc1Mapper::RAM_SIZE]),
            0x8000..=0xFFFF => Some(self.get_prg_data(addr - 0x8000)),
            _ => None,
        }
    }

    fn cpu_bus_read(&mut self, addr: u16) -> CartridgeResult<u8> {
        Ok(self.cpu_bus_peek(addr).unwrap_or_default())
    }

    fn cpu_bus_write(&mut self, addr: u16, value: u8) -> CartridgeResult<()> {
        match addr {
            0x6000..=0x7FFF => {
                self.ram[(addr as usize - 0x6000) % Mmc1Mapper::RAM_SIZE] = value;
                Ok(())
            }
            0x8000..=0xFFFF => {
                self.write_shift_register(addr, value);
                Ok(())
            }
            _ => Err(CartridgeError::InvalidWrite(addr)),
        }
    }

    fn ppu_bus_peek(&self, addr: u16) -> Option<u8> {
        match addr {
            0x0000..=0x3FFF => Some(self.chr[self.get_chr_index(addr)]),
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
                let idx = self.get_chr_index(addr);
                self.chr[idx] = value;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Mmc1Mapper;
    use super::PRG_BANK_SIZE;
    use crate::nes::cartridge::Mapper;

    fn test_mapper() -> Mmc1Mapper {
        // Create a mapper with the bank number in the first byte of each bank to allow us to
        // identify them after mapping.
        let mut prg = vec![0x00_u8; 4 * PRG_BANK_SIZE];
        prg[0] = 0x01;
        prg[PRG_BANK_SIZE] = 0x02;
        prg[2 * PRG_BANK_SIZE] = 0x03;
        prg[3 * PRG_BANK_SIZE] = 0x04;
        Mmc1Mapper::new(prg.as_slice(), &[])
    }

    fn serial_write_u5(mapper: &mut Mmc1Mapper, addr: u16, value: u8) {
        mapper.cpu_bus_write(addr, value).unwrap();
        mapper.cpu_bus_write(addr, value >> 1).unwrap();
        mapper.cpu_bus_write(addr, value >> 2).unwrap();
        mapper.cpu_bus_write(addr, value >> 3).unwrap();
        mapper.cpu_bus_write(addr, value >> 4).unwrap();
    }

    #[test]
    pub fn test_control_register_write() {
        let mut mapper = test_mapper();
        serial_write_u5(&mut mapper, 0x8000, 0b10011);
        assert_eq!(mapper.control_register, 0b10011);
    }

    #[test]
    pub fn test_32k_prg_mapping() {
        let mut mapper = test_mapper();
        // Set prg bank to 32kB mode
        serial_write_u5(&mut mapper, 0x8000, 0b00000);

        // Select prg bank 0 (Bank 1 and 2 should be accessible)
        serial_write_u5(&mut mapper, 0xE000, 0b00000);
        assert_eq!(mapper.cpu_bus_peek(0x8000), Some(0x01));
        assert_eq!(mapper.cpu_bus_peek(0xC000), Some(0x02));

        // Select prg bank 2 (Bank 3 and 4 should be accessible)
        serial_write_u5(&mut mapper, 0xE000, 0b00010);
        assert_eq!(mapper.cpu_bus_peek(0x8000), Some(0x03));
        assert_eq!(mapper.cpu_bus_peek(0xC000), Some(0x04));

        // Select prg bank 1. Lowest bit is ignores, so same as prg bank 0.
        serial_write_u5(&mut mapper, 0xE000, 0b00001);
        assert_eq!(mapper.cpu_bus_peek(0x8000), Some(0x01));
        assert_eq!(mapper.cpu_bus_peek(0xC000), Some(0x02));
    }

    #[test]
    pub fn test_prg_mapping_mode_2() {
        let mut mapper = test_mapper();
        serial_write_u5(&mut mapper, 0x8000, 0b01000);

        // The first memory block will alwys be fixed to bank 1.
        // The second memory block will be switched by 0xE000.

        serial_write_u5(&mut mapper, 0xE000, 0);
        assert_eq!(mapper.cpu_bus_peek(0x8000), Some(0x01));
        assert_eq!(mapper.cpu_bus_peek(0xC000), Some(0x01));

        serial_write_u5(&mut mapper, 0xE000, 1);
        assert_eq!(mapper.cpu_bus_peek(0x8000), Some(0x01));
        assert_eq!(mapper.cpu_bus_peek(0xC000), Some(0x02));

        serial_write_u5(&mut mapper, 0xE000, 2);
        assert_eq!(mapper.cpu_bus_peek(0x8000), Some(0x01));
        assert_eq!(mapper.cpu_bus_peek(0xC000), Some(0x03));
    }

    #[test]
    pub fn test_prg_mapping_mode_3() {
        let mut mapper = test_mapper();
        serial_write_u5(&mut mapper, 0x8000, 0b01100);

        // The first memory block will be switched by 0xE000.
        // The second memory block will alwys be fixed to bank 4 (the last one).

        serial_write_u5(&mut mapper, 0xE000, 0);
        assert_eq!(mapper.cpu_bus_peek(0x8000), Some(0x01));
        assert_eq!(mapper.cpu_bus_peek(0xC000), Some(0x04));

        serial_write_u5(&mut mapper, 0xE000, 1);
        assert_eq!(mapper.cpu_bus_peek(0x8000), Some(0x02));
        assert_eq!(mapper.cpu_bus_peek(0xC000), Some(0x04));

        serial_write_u5(&mut mapper, 0xE000, 2);
        assert_eq!(mapper.cpu_bus_peek(0x8000), Some(0x03));
        assert_eq!(mapper.cpu_bus_peek(0xC000), Some(0x04));
    }
}
