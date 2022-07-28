use anyhow::anyhow;
use anyhow::Result;

////////////////////////////////////////////////////////////////////////////////
// Bus

#[derive(Default)]
pub struct Bus {
    pub ram: RamDevice,
    pub rom: RomDevice,
}

impl Bus {
    pub fn slice(&self, addr: u16, length: usize) -> &[u8] {
        match addr {
            RamDevice::START_ADDR..=RamDevice::END_ADDR => {
                self.ram.slice(addr - RamDevice::START_ADDR, length)
            }
            RomDevice::START_ADDR..=RomDevice::END_ADDR => {
                self.rom.slice(addr - RomDevice::START_ADDR, length)
            }
            _ => unimplemented!("Invalid read from {addr}"),
        }
    }

    pub fn mut_slice(&mut self, addr: u16, length: usize) -> &mut [u8] {
        match addr {
            RamDevice::START_ADDR..=RamDevice::END_ADDR => {
                self.ram.mut_slice(addr - RamDevice::START_ADDR, length)
            }
            RomDevice::START_ADDR..=RomDevice::END_ADDR => {
                self.rom.mut_slice(addr - RomDevice::START_ADDR, length)
            }
            _ => unimplemented!("Invalid write to {addr}"),
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        return self.slice(addr, 1)[0];
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        u16::from_le_bytes(self.slice(addr, 2).try_into().unwrap())
    }

    pub fn write_u8(&mut self, addr: u16, value: u8) {
        self.mut_slice(addr, 1)[0] = value;
    }
}

////////////////////////////////////////////////////////////////////////////////
// RamDevice

pub struct RamDevice {
    ram: [u8; 0x2000],
}

impl RamDevice {
    pub const START_ADDR: u16 = 0x0000;
    pub const END_ADDR: u16 = 0x3FFF;

    pub fn slice(&self, addr: u16, length: usize) -> &[u8] {
        let addr = addr as usize & 0b0000_0111_1111_1111;
        &self.ram[addr..(addr + length)]
    }

    pub fn mut_slice(&mut self, addr: u16, length: usize) -> &mut [u8] {
        let addr = addr as usize & 0b0000_0111_1111_1111;
        &mut self.ram[addr..(addr + length)]
    }
}

impl Default for RamDevice {
    fn default() -> Self {
        Self { ram: [0; 0x2000] }
    }
}

////////////////////////////////////////////////////////////////////////////////
// RomDevice

#[derive(Default)]
pub struct RomDevice {
    prg: Vec<u8>,
    chr: Vec<u8>,
}

impl RomDevice {
    pub fn load_program(&mut self, data: &[u8]) {
        self.prg = data.into();
    }

    pub fn load_ines(&mut self, raw: &[u8]) -> Result<()> {
        println!("{:?}", &raw[0..16]);

        if raw[0] != b'N' || raw[1] != b'E' || raw[2] != b'S' {
            return Err(anyhow!("Expected NES header."));
        }
        let prg_len = raw[4] as usize * 16 * 1024;
        let chr_len = raw[5] as usize * 8 * 1024;

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

        Ok(())
    }

    pub const START_ADDR: u16 = 0x8000;
    pub const END_ADDR: u16 = 0xFFFF;

    pub fn slice(&self, addr: u16, length: usize) -> &[u8] {
        let addr = addr as usize % self.prg.len();
        &self.prg[addr..(addr + length)]
    }

    pub fn mut_slice(&mut self, _: u16, _: usize) -> &mut [u8] {
        panic!("Illegal write to rom device.");
    }
}

////////////////////////////////////////////////////////////////////////////////
// Utilities

trait SliceWithLength {
    fn slice(&self, index: usize, length: usize) -> Self;
}

impl SliceWithLength for &[u8] {
    fn slice(&self, index: usize, length: usize) -> Self {
        &self[index..(index + length)]
    }
}
