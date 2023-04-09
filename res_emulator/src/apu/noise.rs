use std::fmt::Display;
use std::fmt::Formatter;

use bincode::Decode;
use bincode::Encode;
use intbits::Bits;
use itertools::Itertools;
use packed_struct::prelude::PackedStruct;

#[derive(PackedStruct, Encode, Decode, Clone, Debug, Default, Copy, PartialEq, Eq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct NoiseRegister0 {
    #[packed_field(size_bits = "2")]
    _unused: u8,
    halt: bool,
    constant_volume: bool,
    #[packed_field(size_bits = "4")]
    volume: u8,
}

#[derive(PackedStruct, Encode, Decode, Clone, Debug, Default, Copy, PartialEq, Eq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct NoiseRegister2 {
    noise_loop: bool,
    #[packed_field(size_bits = "3")]
    _unused: u8,
    #[packed_field(size_bits = "4")]
    noise_period: u8,
}

#[derive(PackedStruct, Encode, Decode, Clone, Debug, Default, Copy, PartialEq, Eq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct NoiseRegister3 {
    #[packed_field(size_bits = "5")]
    length_counter_load: u8,
    #[packed_field(size_bits = "3")]
    _unused: u8,
}

#[derive(Debug, Default, Encode, Decode, Clone)]
pub struct NoiseChannel {
    register0: NoiseRegister0,
    register2: NoiseRegister2,
    register3: NoiseRegister3,

    cycle: u16,
    length_counter: u8,
    decay_level: u8,
    quarter_frames: usize,
    shift_register: u16,
}

impl Display for NoiseChannel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pulse {:02X} {:02X} {:02X}",
            self.register0.pack().unwrap()[0],
            self.register2.pack().unwrap()[0],
            self.register3.pack().unwrap()[0],
        )
    }
}

const NOTE_LENGTHS: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

const PERIODS: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

impl NoiseChannel {
    pub fn pretty_print(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!(
            "Period({:X}) {}",
            self.register2.noise_period,
            if self.register2.noise_loop {
                "Loop"
            } else {
                " "
            },
        ));
        lines.push(if self.register0.constant_volume {
            format!("Env: Const({:02X})", self.register0.volume)
        } else {
            format!(
                "Env: P({:X}) level {}",
                self.register0.volume, self.decay_level,
            )
        });
        lines.push(format!(
            "Length: {} (Ld {:02X})",
            self.length_counter, self.register3.length_counter_load,
        ));
        lines.push(format!("Reg: {:016b}", self.shift_register));
        lines.push(format!("Value: {} (Cy {})", self.value(), self.cycle));
        lines.iter().join("\n")
    }

    pub fn reset_sequencer(&mut self) {
        self.cycle = PERIODS[self.register2.noise_period as usize];
        self.shift_register = 1;
    }

    pub fn tick(&mut self, half_frame: bool, quarter_frame: bool) {
        if quarter_frame {
            self.quarter_frames += 1;
            if !self.register0.constant_volume
                && self.register0.volume > 0
                && self.quarter_frames % self.register0.volume as usize == 0
            {
                // Tick envelope
                if self.decay_level > 0 {
                    self.decay_level -= 1;
                }
            }
        }

        if half_frame && !self.register0.halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
        if self.cycle == 0 {
            self.cycle = PERIODS[self.register2.noise_period as usize];
            let feedback = if self.register2.noise_loop {
                self.shift_register.bit(6) ^ self.shift_register.bit(0)
            } else {
                self.shift_register.bit(1) ^ self.shift_register.bit(0)
            };
            self.shift_register >>= 1;
            self.shift_register.set_bit(14, feedback);
        } else {
            self.cycle -= 1;
        }
    }

    pub fn write_register(&mut self, idx: usize, value: u8) {
        match idx {
            0 => self.register0 = NoiseRegister0::unpack(&[value]).unwrap(),
            1 => (),
            2 => self.register2 = NoiseRegister2::unpack(&[value]).unwrap(),
            3 => self.register3 = NoiseRegister3::unpack(&[value]).unwrap(),
            _ => unreachable!(),
        }
        self.shift_register = 1;
        self.decay_level = 15;
        if idx == 3 {
            self.length_counter = NOTE_LENGTHS[self.register3.length_counter_load as usize];
        }
    }

    pub fn value(&self) -> f32 {
        if self.length_counter == 0 {
            return 0.0;
        }
        if self.shift_register.bit(0) {
            0.0
        } else if self.register0.constant_volume {
            self.register0.volume as f32 / 8.0
        } else {
            self.decay_level as f32 / 15.0
        }
    }
}
