use anyhow::Result;
use bincode::{Decode, Encode};
use intbits::Bits;
use itertools::Itertools;
use packed_struct::prelude::PackedStruct;
use std::fmt::{Display, Formatter};

macro_rules! field_from_bits {
    ($idx: literal, $get: ident, $set: ident, $range: expr) => {
        pub fn $get(&self) -> u8 {
            self.value[$idx].bits($range)
        }

        pub fn $set(&mut self, value: u8) {
            self.value[$idx].set_bits($range, value);
        }
    };
}

macro_rules! field_from_bit {
    ($idx: literal, $get: ident, $set: ident, $bit: literal) => {
        pub fn $get(&self) -> bool {
            self.value[$idx].bit($bit)
        }

        pub fn $set(&mut self, value: bool) {
            self.value[$idx].set_bit($bit, value);
        }
    };
}

#[derive(Debug, Default, Encode, Decode, Clone)]
pub struct PulseRegisters {
    value: [u8; 4],
    counter: u8,
    cycle: u16,
    length_counter: u8,
    decay_level: u8,
    quarter_frames: usize,
}

impl Display for PulseRegisters {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pulse {:02X} {:02X} {:02X} {:02X}",
            self.value[0], self.value[1], self.value[2], self.value[3]
        )
    }
}

const NOTE_LENGTHS: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

const WAVEFORMS: [[f32; 8]; 4] = [
    [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0],
];

impl PulseRegisters {
    // 0x4000 / 0x4004 byte
    field_from_bits!(0, duty, set_duty, 6..=7);
    field_from_bit!(0, length_counter_halt, set_length_counter_halt, 5);
    field_from_bit!(0, constant_volume, set_constant_volume, 4);
    field_from_bits!(0, volume, set_volume, 0..=3);

    // 0x4001 / 0x4005 byte
    field_from_bit!(1, sweep_enabled, set_sweep_enabled, 7);
    field_from_bits!(1, sweep_period, set_sweep_period, 4..=6);
    field_from_bit!(1, sweep_negate, set_sweep_negate, 3);
    field_from_bits!(1, sweep_shift, set_sweep_shift, 0..=2);

    // 0x4003 / 0x4007 byte
    field_from_bits!(3, length_counter_load, set_length_counter_load, 3..=7);

    // The timer value is split across byte 2 (low) and 3 (high).
    fn timer(&self) -> u16 {
        let timer_low = self.value[2] as u16;
        let timer_high = self.value[3].bits(0..=2) as u16;
        timer_low + (timer_high << 8)
    }

    fn set_timer(&mut self, timer: u16) {
        let timer_low = timer.bits(0..=7);
        let timer_high = timer.bits(8..=10);
        self.value[2] = timer_low as u8;
        self.value[3].set_bits(0..=2, timer_high as u8);
    }

    pub fn pretty_print(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!(
            "Duty({:X}) Timer({:04X}) Length({:X}{})",
            self.duty(),
            self.timer(),
            self.length_counter_load(),
            if self.length_counter_halt() { "H" } else { " " },
        ));
        lines.push(if self.constant_volume() {
            format!("Env: Const({:02X})", self.volume())
        } else if self.sweep_enabled() {
            "Env: Disabled".to_owned()
        } else {
            format!(
                "Env: P({:X}) S({:X}) {} level {}",
                self.volume(),
                self.sweep_shift(),
                if self.sweep_negate() { "N" } else { " " },
                self.decay_level,
            )
        });
        lines.push(format!(
            "Length: {} (Ld {:02X})",
            self.length_counter,
            self.length_counter_load(),
        ));
        lines.push(format!(
            "Value: {} (Cy {}, Ct {})",
            self.value(),
            self.cycle,
            self.counter
        ));
        lines.iter().join("\n")
    }

    pub fn reset_sequencer(&mut self) {
        self.cycle = self.timer();
        self.counter = 0;
    }

    pub fn tick(&mut self, half_frame: bool, quarter_frame: bool) {
        if quarter_frame {
            self.quarter_frames += 1;
            if !self.constant_volume()
                && self.volume() > 0
                && self.quarter_frames % self.volume() as usize == 0
            {
                // Tick envelope
                if self.decay_level > 0 {
                    self.decay_level -= 1;
                }
            }
        }

        if half_frame && !self.length_counter_halt() {
            if self.length_counter == 0 {
                self.set_timer(0);
            } else {
                self.length_counter -= 1;
            }
        }
        if self.cycle == 0 {
            self.cycle = self.timer();
            self.counter = (self.counter + 1) % 8;
        } else {
            self.cycle -= 1;
        }
    }

    pub fn write_register(&mut self, idx: usize, value: u8) {
        self.value[idx] = value;
        self.decay_level = 15;
        if idx == 3 {
            self.length_counter = NOTE_LENGTHS[self.length_counter_load() as usize];
        }
    }

    pub fn value(&self) -> f32 {
        if self.timer() < 8 {
            return 0.0;
        }
        let waveform = WAVEFORMS[self.duty() as usize];
        let volume = if self.constant_volume() {
            self.volume() as f32 / 8.0
        } else {
            self.decay_level as f32 / 15.0
        };
        waveform[self.counter as usize] * volume
    }
}

#[derive(Default, bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct FrameCounter {
    value: [u8; 1],
    cycle: usize,
    cpu_cycles: usize,

    half_frame: bool,
    quarter_frame: bool,
    irq_frame: bool,
}

impl FrameCounter {
    field_from_bit!(0, mode, set_mode, 0);
    field_from_bit!(0, inhibit_irq, set_inhibit_irq, 1);

    pub fn tick(&mut self) {
        self.cpu_cycles += 1;
        if self.cpu_cycles > 7457 {
            self.cpu_cycles -= 7457;

            if self.mode() {
                self.cycle = (self.cycle + 1) % 5;
            } else {
                self.cycle = (self.cycle + 1) % 4;
            }

            let last_frame = if self.mode() { 4 } else { 3 };
            self.irq_frame = !self.mode() && self.cycle == last_frame;
            self.half_frame = self.cycle == 1 || self.cycle == last_frame;
            self.quarter_frame =
                self.cycle == 0 || self.cycle == 1 || self.cycle == 2 || self.cycle == last_frame;
        } else {
            self.half_frame = false;
            self.quarter_frame = false;
            self.irq_frame = false;
        }
    }
}

#[derive(Default, bincode::Encode, bincode::Decode, Clone)]
pub struct Apu {
    cycle: u64,
    pub audio_buffer: Vec<f32>,
    pub audio_sample_rate: usize,
    pub cycles_since_last_sample: f64,
    pub frame_counter: FrameCounter,
    pub status: StatusRegister,
    pub pulse0: PulseRegisters,
    pub pulse1: PulseRegisters,
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
            pulse0: PulseRegisters::default(),
            pulse1: PulseRegisters::default(),
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
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PulseRegisters;

    #[test]
    fn pulse_registers_test() {
        let mut register = PulseRegisters::default();
        let expected_value = 0x07FF_u16;
        register.set_timer(expected_value);
        println!("Register: {:}", register);
        assert_eq!(register.value, [0x00, 0x00, 0xFF, 0x07]);
        assert_eq!(register.timer(), expected_value);
    }
}
