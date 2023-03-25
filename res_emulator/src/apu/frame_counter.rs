use bincode::Decode;
use bincode::Encode;
use packed_struct::prelude::PackedStruct;

#[derive(PackedStruct, Encode, Decode, Clone, Debug, Default, Copy, PartialEq, Eq)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "1")]
pub struct FrameCounterRegister {
    pub mode: bool,
    pub irq_inhibit: bool,
}

#[derive(Default, bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct FrameCounter {
    register: FrameCounterRegister,
    cycle: usize,
    cpu_cycles: usize,

    pub half_frame: bool,
    pub quarter_frame: bool,
    pub irq_frame: bool,
}

impl FrameCounter {
    pub fn tick(&mut self) {
        self.cpu_cycles += 1;
        if self.cpu_cycles > 7457 {
            self.cpu_cycles -= 7457;

            if self.register.mode {
                self.cycle = (self.cycle + 1) % 5;
            } else {
                self.cycle = (self.cycle + 1) % 4;
            }

            let last_frame = if self.register.mode { 4 } else { 3 };
            self.irq_frame = !self.register.mode && self.cycle == last_frame;
            self.half_frame = self.cycle == 1 || self.cycle == last_frame;
            self.quarter_frame =
                self.cycle == 0 || self.cycle == 1 || self.cycle == 2 || self.cycle == last_frame;
        } else {
            self.half_frame = false;
            self.quarter_frame = false;
            self.irq_frame = false;
        }
    }

    pub fn write_register(&mut self, value: u8) {
        self.register = FrameCounterRegister::unpack(&[value]).unwrap();
    }
}
