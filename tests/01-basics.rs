use ners::nes::System;

#[test]
pub fn nestest() {
    let mut system = System::default();
    system.load_ines("tests/01-basics.nes").unwrap();

    loop {
        println!("{}", system.trace().unwrap());
        if !system.tick().unwrap() {
            break;
        }
    }
}
