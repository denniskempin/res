use std::fmt::Display;
use std::fmt::Formatter;

use bincode::Decode;
use bincode::Encode;
use intbits::Bits;
use itertools::Itertools;
use packed_struct::prelude::PackedStruct;

#[derive(PackedStruct, Encode, Decode, Clone, Debug, Default, Copy, PartialEq, Eq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct PulseRegister0 {
    #[packed_field(size_bits = "2")]
    duty: u8,
    halt: bool,
    constant_volume: bool,
    #[packed_field(size_bits = "4")]
    volume: u8,
}

#[derive(PackedStruct, Encode, Decode, Clone, Debug, Default, Copy, PartialEq, Eq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct PulseRegister1 {
    sweep_enabled: bool,
    #[packed_field(size_bits = "3")]
    sweep_period: u8,
    sweep_negate: bool,
    #[packed_field(size_bits = "3")]
    sweep_shift: u8,
}

type PulseRegister2 = u8;

#[derive(PackedStruct, Encode, Decode, Clone, Debug, Default, Copy, PartialEq, Eq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct PulseRegister3 {
    #[packed_field(size_bits = "5")]
    length_counter_load: u8,
    #[packed_field(size_bits = "3")]
    timer_high: u8,
}

#[derive(Debug, Default, Encode, Decode, Clone)]
pub struct PulseChannel {
    register0: PulseRegister0,
    register1: PulseRegister1,
    register2: PulseRegister2,
    register3: PulseRegister3,

    counter: u8,
    cycle: u16,
    length_counter: u8,
    decay_level: u8,
    quarter_frames: usize,
}

impl Display for PulseChannel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pulse {:02X} {:02X} {:02X} {:02X}",
            self.register0.pack().unwrap()[0],
            self.register1.pack().unwrap()[0],
            self.register2,
            self.register3.pack().unwrap()[0],
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

impl PulseChannel {
    // The timer value is split across byte 2 (low) and 3 (high).
    fn timer(&self) -> u16 {
        (self.register2 as u16) + ((self.register3.timer_high as u16) << 8)
    }

    fn set_timer(&mut self, timer: u16) {
        self.register2 = timer.bits(0..=7) as u8;
        self.register3.timer_high = timer.bits(8..=10) as u8;
    }

    pub fn pretty_print(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!(
            "Duty({:X}) Timer({:04X}) Length({:X}{})",
            self.register0.duty,
            self.timer(),
            self.register3.length_counter_load,
            if self.register0.halt { "H" } else { " " },
        ));
        lines.push(if self.register0.constant_volume {
            format!("Env: Const({:02X})", self.register0.volume)
        } else if self.register1.sweep_enabled {
            "Env: Disabled".to_owned()
        } else {
            format!(
                "Env: P({:X}) S({:X}) {} level {}",
                self.register0.volume,
                self.register1.sweep_shift,
                if self.register1.sweep_negate {
                    "N"
                } else {
                    " "
                },
                self.decay_level,
            )
        });
        lines.push(format!(
            "Length: {} (Ld {:02X})",
            self.length_counter, self.register3.length_counter_load,
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

        if half_frame && !self.register0.halt {
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
        match idx {
            0 => self.register0 = PulseRegister0::unpack(&[value]).unwrap(),
            1 => self.register1 = PulseRegister1::unpack(&[value]).unwrap(),
            2 => self.register2 = PulseRegister2::unpack(&[value]).unwrap(),
            3 => self.register3 = PulseRegister3::unpack(&[value]).unwrap(),
            _ => unreachable!(),
        }
        self.decay_level = 15;
        if idx == 3 {
            self.length_counter = NOTE_LENGTHS[self.register3.length_counter_load as usize];
        }
    }

    pub fn value(&self) -> f32 {
        if self.timer() < 8 {
            return 0.0;
        }
        let waveform = WAVEFORMS[self.register0.duty as usize];
        let volume = if self.register0.constant_volume {
            self.register0.volume as f32 / 8.0
        } else {
            self.decay_level as f32 / 15.0
        };
        waveform[self.counter as usize] * volume
    }
}
