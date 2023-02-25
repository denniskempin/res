use anyhow::Result;

#[derive(Default, bincode::Encode, bincode::Decode, Clone)]
pub struct Apu {
    cycle: u64,
}

impl Apu {
    pub fn new() -> Apu {
        Apu { cycle: 0 }
    }

    pub fn advance_clock(&mut self, cycles: usize) -> Result<()> {
        self.cycle += cycles as u64;
        Ok(())
    }

    pub fn sample(&self) -> f32 {
        f32::sin((self.cycle as f32) * 0.00001)
    }

    pub fn tick(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn cpu_bus_peek(&self, _addr: u16) -> u8 {
        0
    }

    pub fn cpu_bus_read(&mut self, addr: u16) -> u8 {
        self.cpu_bus_peek(addr)
    }

    pub fn cpu_bus_write(&mut self, _addr: u16, _: u8) {}
}
