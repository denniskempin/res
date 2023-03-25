mod frame_counter;
mod pulse;

use anyhow::Result;
use bincode::Decode;
use bincode::Encode;
use packed_struct::prelude::PackedStruct;

use self::frame_counter::FrameCounter;
use self::pulse::PulseChannel;

#[derive(Default, bincode::Encode, bincode::Decode, Clone)]
pub struct Apu {
    cycle: u64,
    pub audio_buffer: Vec<f32>,
    pub audio_sample_rate: usize,
    pub cycles_since_last_sample: f64,
    pub frame_counter: FrameCounter,
    pub status: StatusRegister,
    pub pulse0: PulseChannel,
    pub pulse1: PulseChannel,
}

#[derive(PackedStruct, Encode, Decode, Clone, Debug, Default, Copy, PartialEq, Eq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct StatusRegister {
    pub _unused: [bool; 3],
    pub dmc_enable: bool,
    pub noise_enable: bool,
    pub triangle_enable: bool,
    pub pulse1_enable: bool,
    pub pulse0_enable: bool,
}

impl Apu {
    pub fn new() -> Apu {
        Apu {
            cycle: 0,
            audio_buffer: Vec::with_capacity(1024 * 1024),
            audio_sample_rate: 0,
            cycles_since_last_sample: 0.0,
            frame_counter: FrameCounter::default(),
            status: StatusRegister::default(),
            pulse0: PulseChannel::default(),
            pulse1: PulseChannel::default(),
        }
    }

    pub fn advance_clock(&mut self, cycles: usize) -> Result<()> {
        let samples_per_frame = self.audio_sample_rate as f64 / 60.0;
        let cycles_per_frame = 29268.67105 + 512.0;
        let cycles_per_sample = cycles_per_frame / samples_per_frame as f64;

        for _ in 0..cycles {
            self.frame_counter.tick();
            self.cycle += 1;
            if self.cycle % 2 == 0 {
                self.pulse0.tick(
                    self.frame_counter.half_frame,
                    self.frame_counter.quarter_frame,
                );
                self.pulse1.tick(
                    self.frame_counter.half_frame,
                    self.frame_counter.quarter_frame,
                );
            }
            self.cycles_since_last_sample += 1.0;

            if self.cycles_since_last_sample > cycles_per_sample {
                self.cycles_since_last_sample -= cycles_per_sample;
                self.audio_buffer.push(self.sample());
            }
        }

        Ok(())
    }

    pub fn sample(&self) -> f32 {
        let mut value = 0.0;
        if self.status.pulse0_enable {
            value += self.pulse0.value() * 0.5
        }
        if self.status.pulse1_enable {
            value += self.pulse1.value() * 0.5
        }
        value
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

    pub fn cpu_bus_write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4000..=0x4003 => self.pulse0.write_register((addr - 0x4000) as usize, value),
            0x4004..=0x4007 => self.pulse1.write_register((addr - 0x4004) as usize, value),
            0x4015 => self.status = StatusRegister::unpack(&[value]).unwrap(),
            0x4017 => self.frame_counter.write_register(value),
            _ => {}
        }
    }
}
