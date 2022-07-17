pub mod bus;
pub mod cpu;

use self::bus::Bus;
use self::bus::RomDevice;
use self::cpu::Cpu;
use anyhow::Ok;
use anyhow::Result;

////////////////////////////////////////////////////////////////////////////////
// System

#[derive(Default)]
pub struct System {
    bus: Bus,
    cpu: Cpu,
}

impl System {
    pub fn execute_until_halt(&mut self) -> Result<()> {
        while self.cpu.execute_one(&mut self.bus)? {}
        Ok(())
    }

    pub fn with_program(program: &[u8]) -> System {
        let mut system = System::default();
        system.load_program(program);
        system
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.bus.rom.load(program);
        self.cpu.program_counter = RomDevice::START_ADDR;
    }
}

////////////////////////////////////////////////////////////////////////////////
// test

#[cfg(test)]
mod test {
    use super::System;

    #[test]
    pub fn test_basic_program() {
        let mut system = System::with_program(&[
            0xa9, 0x10, // LDA #$10     -> A = #$10
            0x85, 0x20, // STA $20      -> $20 = #$10
            0xa9, 0x01, // LDA #$1      -> A = #$1
            0x65, 0x20, // ADC $20      -> A = #$11
            0x85, 0x21, // STA $21      -> $21=#$11
            0xe6, 0x21, // INC $21      -> $21=#$12
            0xa4, 0x21, // LDY $21      -> Y=#$12
            0xc8, // INY          -> Y=#$13
            0x00, // BRK
        ]);
        system.execute_until_halt().unwrap();
        assert_eq!(system.bus.read_u8(0x20_u16), 0x10);
        assert_eq!(system.bus.read_u8(0x21_u16), 0x12);
        assert_eq!(system.cpu.registers.a, 0x11);
        assert_eq!(system.cpu.registers.y, 0x13);
    }
}
