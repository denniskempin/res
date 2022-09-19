#[derive(Default, bincode::Encode, bincode::Decode)]
pub struct Apu {}

impl Apu {
    pub fn tick(&mut self, _clock: u64) {}

    pub fn cpu_bus_peek(&self, _addr: u16) -> u8 {
        0
    }

    pub fn cpu_bus_read(&mut self, addr: u16) -> u8 {
        self.cpu_bus_peek(addr)
    }

    pub fn cpu_bus_write(&mut self, _addr: u16, _: u8) {}
}
