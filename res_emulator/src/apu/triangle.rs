use std::fmt::Display;
use std::fmt::Formatter;

use bincode::Decode;
use bincode::Encode;
use intbits::Bits;
use itertools::Itertools;
use packed_struct::prelude::PackedStruct;

#[derive(PackedStruct, Encode, Decode, Clone, Debug, Default, Copy, PartialEq, Eq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct TriangleRegister0 {
    control: bool,
    #[packed_field(size_bits = "7")]
    linear_counter_reload: u8,
}

type TriangleRegister2 = u8;

#[derive(PackedStruct, Encode, Decode, Clone, Debug, Default, Copy, PartialEq, Eq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct TriangleRegister3 {
    #[packed_field(size_bits = "5")]
    length_counter_load: u8,
    #[packed_field(size_bits = "3")]
    timer_high: u8,
}

#[derive(Debug, Default, Encode, Decode, Clone)]
pub struct TriangleChannel {
    register0: TriangleRegister0,
    register2: TriangleRegister2,
    register3: TriangleRegister3,

    counter: u8,
    cycle: u16,
    length_counter: u8,
    linear_counter: u8,
    linear_counter_reload: bool,
}

impl Display for TriangleChannel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pulse {:02X} 00 {:02X} {:02X}",
            self.register0.pack().unwrap()[0],
            self.register2,
            self.register3.pack().unwrap()[0],
        )
    }
}

const NOTE_LENGTHS: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

const WAVEFORM: [f32; 32] = [
    15.0, 14.0, 13.0, 12.0, 11.0, 10.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0, 0.0, 0.0, 1.0,
    2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
];

impl TriangleChannel {
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
            "Timer({:04X}) Length({:X}{})",
            self.timer(),
            self.register3.length_counter_load,
            if self.register0.control { " " } else { "H" },
        ));
        lines.push(format!(
            "Length: {} (Ld {:02X})",
            self.length_counter, self.register3.length_counter_load,
        ));
        lines.push(format!(
            "Linear: {} (Ld {:02X})",
            self.linear_counter, self.register0.linear_counter_reload,
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
        if !self.register0.control {
            if quarter_frame {
                if self.linear_counter_reload {
                    self.linear_counter = self.register0.linear_counter_reload;
                } else if self.linear_counter == 0 {
                    self.set_timer(0);
                } else {
                    self.linear_counter -= 1;
                }
                self.linear_counter_reload = false;
            }
            if half_frame {
                if self.length_counter == 0 {
                    self.set_timer(0);
                } else {
                    self.length_counter -= 1;
                }
            }
        }

        if self.cycle == 0 {
            self.cycle = self.timer();
            self.counter = (self.counter + 1) % 32;
        } else {
            self.cycle -= 1;
        }
    }

    pub fn write_register(&mut self, idx: usize, value: u8) {
        match idx {
            0 => self.register0 = TriangleRegister0::unpack(&[value]).unwrap(),
            1 => (),
            2 => self.register2 = TriangleRegister2::unpack(&[value]).unwrap(),
            3 => self.register3 = TriangleRegister3::unpack(&[value]).unwrap(),
            _ => unreachable!(),
        }
        if idx == 3 {
            self.length_counter = NOTE_LENGTHS[self.register3.length_counter_load as usize] / 2;
            self.linear_counter_reload = true;
        }
    }

    pub fn value(&self) -> f32 {
        if self.timer() < 2 {
            return 0.0;
        }
        WAVEFORM[self.counter as usize] / 15.0
    }
}
