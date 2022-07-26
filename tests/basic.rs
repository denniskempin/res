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
