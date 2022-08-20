#[derive(Default)]
pub struct Apu {}

impl Apu {
    pub const START_ADDR: u16 = 0x4000;
    pub const END_ADDR: u16 = 0x4017;

    pub fn read(&self, _addr: u16) -> u8 {
        0
    }

    pub fn write(&mut self, _addr: u16, _: u8) {}
}
