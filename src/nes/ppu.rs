use super::memory_map::MemoryMap;

#[derive(Default)]
pub struct Ppu {}

impl Ppu {
    pub const START_ADDR: u16 = 0x2000;
    pub const END_ADDR: u16 = 0x2007;
}

impl MemoryMap for Ppu {
    fn read(&self, _addr: u16) -> u8 {
        0
    }

    fn write(&mut self, _addr: u16, _: u8) {}
}
