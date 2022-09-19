use res::nes::trace::Trace;
use res::nes::System;
use std::fs::File;
use std::io::BufRead;
use std::io::{self};
use std::path::Path;

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
    ])
    .unwrap();
    system.cpu.program_counter = 0x8000;
    system.execute_until_halt().unwrap();
    assert_eq!(system.cpu.bus.peek(0x20_u16), 0x10);
    assert_eq!(system.cpu.bus.peek(0x21_u16), 0x12);
    assert_eq!(system.cpu.a, 0x11);
    assert_eq!(system.cpu.y, 0x13);
}

#[test]
pub fn test_nestest() {
    // Run nestest.nes and compare results against a log file collected by
    // running the same file in the accurate Nintendulator.
    let mut system = System::with_ines(Path::new("tests/cpu/nestest.nes")).unwrap();
    system.cpu.program_counter = 0xC000;
    compare_to_log(system, "tests/cpu/nestest.log", 0);
}

#[test]
pub fn test_nestest_snapshot() {
    let mut system = System::with_ines(Path::new("tests/cpu/nestest.nes")).unwrap();
    for _ in 0..1000 {
        system.cpu.execute_one().unwrap();
    }

    let snapshot = system.snapshot();
    let mut resumed_system = System::with_snapshot(&snapshot).unwrap();
    assert_eq!(system.trace().unwrap(), resumed_system.trace().unwrap());
}

pub fn compare_to_log(mut system: System, log_file: &str, goal_count: usize) {
    let log = io::BufReader::new(File::open(log_file).unwrap());
    let mut previous_actual = Trace::default();
    for (i, line) in log.lines().enumerate() {
        if goal_count > 0 && i >= goal_count {
            println!("Reached goal of {goal_count} instructions. Success.");
            break;
        }

        let expected_trace = Trace::from_log_line(&line.unwrap()).unwrap();
        println!("{i:6} Exp: {expected_trace}");
        let actual_trace = system.trace().unwrap();
        println!("{i:6} Act: {actual_trace}");

        // Cycle count mismatches happen often and are hard to read. Print
        // a nice error message to help.
        if expected_trace.cpu_cycle != actual_trace.cpu_cycle {
            let actual_delta = actual_trace.cpu_cycle - previous_actual.cpu_cycle;
            let expected_delta = expected_trace.cpu_cycle - previous_actual.cpu_cycle;
            let instr = previous_actual.opcode_str;
            println!(
                "Cpu instruction {instr} lasted {actual_delta} cycles but should be {expected_delta} cycles"
            );
        }

        assert_eq!(expected_trace, actual_trace);
        system.cpu.execute_one().unwrap();
        previous_actual = actual_trace;
    }
}
