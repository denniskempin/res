use std::path::Path;
use std::path::PathBuf;

use image::RgbaImage;
use res::nes::ppu::Framebuffer;
use res::nes::System;

#[test]
pub fn test_nestest_title_screen() {
    let mut system = System::with_ines(Path::new("tests/cpu/nestest.nes")).unwrap();
    system.execute_frames(60).unwrap();
    compare_to_golden(&system.cpu.bus.ppu.framebuffer, "test_nestest_title_screen");
}

#[test]
pub fn test_alter_ego_intro() {
    let mut system = System::with_ines(Path::new("tests/ppu/alter_ego.nes")).unwrap();

    system.execute_frames(60).unwrap();
    compare_to_golden(&system.cpu.bus.ppu.framebuffer, "test_alter_ego_intro_0");

    system.execute_frames(240).unwrap();
    compare_to_golden(&system.cpu.bus.ppu.framebuffer, "test_alter_ego_intro_1");
}

#[test]
#[ignore] // Requires donkey_kong.nes rom
pub fn test_donkey_kong_intro() {
    let mut system = System::with_ines(Path::new("tests/ppu/donkey_kong.nes")).unwrap();

    system.execute_frames(60).unwrap();
    compare_to_golden(&system.cpu.bus.ppu.framebuffer, "test_donkey_kong_intro");
}

pub fn compare_to_golden(image: &Framebuffer, name: &str) {
    let rgba_image = image.as_rgba_image();

    let path_prefix = PathBuf::from("tests/ppu").join(name);
    let golden_path = path_prefix.with_extension("png");
    if golden_path.exists() {
        let golden: RgbaImage = image::open(golden_path).unwrap().into_rgba8();
        assert_eq!(rgba_image, golden);
    } else {
        rgba_image.save(golden_path).unwrap();
    }
}
