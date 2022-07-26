use std::fs::File;
use std::io;
use std::io::BufRead;

use ners::nes::trace::Trace;
use ners::nes::System;

#[test]
pub fn nestest() {
    let log = io::BufReader::new(File::open("tests/nestest.log").unwrap());

    let mut system = System::default();
    system.load_ines("tests/nestest.nes").unwrap();
    // Jump directly to start of the test where the log begins.
    system.cpu.program_counter = 0xC000;

    for line in log.lines() {
        println!();
        let expected_trace = Trace::from_log_line(&line.unwrap()).unwrap();
        println!("Expected: {expected_trace}");
        let actual_trace = system.trace().unwrap();
        println!("Actual:   {actual_trace}");
        assert_eq!(expected_trace, actual_trace);
        system.tick().unwrap();
    }
}
