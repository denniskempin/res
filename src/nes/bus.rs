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
    pub fn read_u8(&self, addr: impl Into<u16>) -> u8 {
        let addr: u16 = addr.into();
        match addr {
            RamDevice::START_ADDR..=RamDevice::END_ADDR => {
                self.ram.read(addr - RamDevice::START_ADDR)
            }
            RomDevice::START_ADDR..=RomDevice::END_ADDR => {
                self.rom.read(addr - RomDevice::START_ADDR)
            }
            _ => 0,
        }
    }

    pub fn read_u16(&self, addr: impl Into<u16>) -> u16 {
        let addr: u16 = addr.into();
        u16::from_le_bytes([self.read_u8(addr), self.read_u8(addr + 1)])
    }

    pub fn write_u8(&mut self, addr: impl Into<u16>, value: u8) {
        let addr: u16 = addr.into();
        match addr {
            RamDevice::START_ADDR..=RamDevice::END_ADDR => {
                self.ram.write(addr - RamDevice::START_ADDR, value)
            }
            RomDevice::START_ADDR..=RomDevice::END_ADDR => {
                self.rom.write(addr - RomDevice::START_ADDR, value)
            }
            _ => (),
        };
    }
    pub fn write_u16(&mut self, _addr: impl Into<u16>, _value: u16) {
        todo!("")
    }
}

////////////////////////////////////////////////////////////////////////////////
// RamDevice

pub struct RamDevice {
    ram: [u8; 0x2000],
}

impl RamDevice {
    pub const START_ADDR: u16 = 0x0000;
    pub const END_ADDR: u16 = 0x1FFF;

    fn read(&self, addr: u16) -> u8 {
        self.ram[addr as usize & 0b0000_0111_1111_1111]
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.ram[addr as usize & 0b0000_0111_1111_1111] = value
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

    fn read(&self, addr: u16) -> u8 {
        self.prg[(addr as usize) % self.prg.len()]
    }

    fn write(&mut self, _: u16, _: u8) {
        panic!("Illegal write to rom device.");
    }
}

////////////////////////////////////////////////////////////////////////////////
// INES File

pub struct InesFile {}

impl InesFile {}
