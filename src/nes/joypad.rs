use bincode::Decode;
use bincode::Encode;

pub enum JoypadButton {
    ButtonA = 0,
    ButtonB = 1,
    Select = 2,
    Start = 3,
    Up = 4,
    Down = 5,
    Left = 6,
    Right = 7,
}

#[derive(Default, Encode, Decode)]
pub struct Joypad {
    strobe: bool,
    index: usize,
    button_states: [bool; 8],
}

impl Joypad {
    pub fn set_button(&mut self, button: JoypadButton, state: bool) {
        self.button_states[button as usize] = state;
    }

    pub fn cpu_bus_peek(&self) -> u8 {
        0
    }

    pub fn cpu_bus_write(&mut self, data: u8) {
        self.strobe = data & 1 == 1;
        if self.strobe {
            self.index = 0
        }
    }

    pub fn cpu_bus_read(&mut self) -> u8 {
        let pressed = self.button_states[self.index];
        if !self.strobe {
            self.index = (self.index + 1) % 8;
        }
        if pressed {
            1
        } else {
            0
        }
    }
}
