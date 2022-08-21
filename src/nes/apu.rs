use super::cpu::CpuMemoryMap;

#[derive(Default)]
pub struct Apu {}

impl CpuMemoryMap for Apu {
    fn read(&mut self, _addr: u16) -> u8 {
        0
    }

    fn write(&mut self, _addr: u16, _: u8) {}
}
