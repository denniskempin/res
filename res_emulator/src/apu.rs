use anyhow::Result;

#[derive(Default, bincode::Encode, bincode::Decode, Clone)]
pub struct Apu {
    cycle: u64,
    pub audio_buffer: Vec<f32>,
    pub audio_sample_rate: usize,
    pub cycles_since_last_sample: f64,
}

impl Apu {
    pub fn new() -> Apu {
        Apu {
            cycle: 0,
            audio_buffer: Vec::with_capacity(1024 * 1024),
            audio_sample_rate: 0,
            cycles_since_last_sample: 0.0,
        }
    }

    pub fn advance_clock(&mut self, cycles: usize) -> Result<()> {
        let samples_per_frame = self.audio_sample_rate as f64 / 60.0;
        let cycles_per_frame = 29268.67105 + 512.0;
        let cycles_per_sample = cycles_per_frame / samples_per_frame as f64;

        for _ in 0..cycles {
            self.cycle += 1;
            self.cycles_since_last_sample += 1.0;

            if self.cycles_since_last_sample > cycles_per_sample {
                self.cycles_since_last_sample -= cycles_per_sample;
                self.audio_buffer.push(self.sample());
            }
        }

        Ok(())
    }

    pub fn sample(&self) -> f32 {
        f32::sin(self.cycle as f32 * 0.001)
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
