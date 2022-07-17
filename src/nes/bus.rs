////////////////////////////////////////////////////////////////////////////////
// Bus

#[derive(Default)]
pub struct Bus {
    pub ram: RamDevice,
    pub rom: RomDevice,
}

impl Bus {
    pub fn read_u8(&mut self, addr: impl Into<u16>) -> u8 {
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

    pub fn read_u16(&mut self, addr: u16) -> u16 {
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
    rom: Vec<u8>,
}

impl RomDevice {
    pub fn load(&mut self, data: &[u8]) {
        self.rom = data.into();
    }
    pub const START_ADDR: u16 = 0x8000;
    pub const END_ADDR: u16 = 0xFFFF;

    fn read(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }

    fn write(&mut self, _: u16, _: u8) {
        panic!("Illegal write to rom device.");
    }
}
