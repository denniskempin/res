use super::memory_map::MemoryMap;

#[derive(Default)]
pub struct Apu {}

impl Apu {
    pub const START_ADDR: u16 = 0x4000;
    pub const END_ADDR: u16 = 0x4017;
}

impl MemoryMap for Apu {
    fn read(&self, _addr: u16) -> u8 {
        0
    }

    fn write(&mut self, _addr: u16, _: u8) {}
}
