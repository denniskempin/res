use std::fs::File;
use std::io::BufRead;
use std::io::{self};
use std::path::Path;

use ners::nes::trace::Trace;
use ners::nes::System;

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
    assert_eq!(system.cpu.a, 0x11);
    assert_eq!(system.cpu.y, 0x13);
}

#[test]
pub fn test_01_basic() {
    let system = System::with_ines(Path::new("tests/cpu/01-basics.nes")).unwrap();
    compare_to_log(system, "tests/cpu/01-basics.log", 4);
}

#[test]
pub fn test_nestest() {
    let mut system = System::with_ines(Path::new("tests/cpu/nestest.nes")).unwrap();
    system.cpu.program_counter = 0xC000;
    compare_to_log(system, "tests/cpu/nestest.log", 0);
}

pub fn compare_to_log(mut system: System, log_file: &str, goal_count: usize) {
    let log = io::BufReader::new(File::open(log_file).unwrap());

    for (i, line) in log.lines().enumerate() {
        if goal_count > 0 && i >= goal_count {
            println!("Reached goal of {goal_count} instructions. Success.");
            break;
        }

        let expected_trace = Trace::from_log_line(&line.unwrap()).unwrap();
        println!("{i:6} Exp: {expected_trace}");
        let actual_trace = system.trace().unwrap();
        println!("{i:6} Act: {actual_trace}");
        assert_eq!(expected_trace, actual_trace);
        system.tick().unwrap();
    }
}
