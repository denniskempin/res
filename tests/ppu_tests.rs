use image::GenericImage;
use image::ImageBuffer;
use image::RgbImage;

use ners::nes::System;

use std::path::Path;
use std::path::PathBuf;

#[test]
pub fn test_nestest_pattern_table() {
    let mut system = System::with_ines(Path::new("tests/cpu/nestest.nes")).unwrap();
    system.execute_frames(60).unwrap();
    compare_to_golden(
        system.cpu.bus.ppu.render_pattern_table(0).unwrap(),
        "test_nestest_pattern_table",
    );
}

#[test]
pub fn test_nestest_title_screen() {
    let mut system = System::with_ines(Path::new("tests/cpu/nestest.nes")).unwrap();
    system.execute_frames(60).unwrap();
    compare_to_golden(render_nametable(&mut system), "test_nestest_title_screen");
}

#[test]
pub fn test_alter_ego_intro() {
    let mut system = System::with_ines(Path::new("tests/ppu/alter_ego.nes")).unwrap();

    system.execute_frames(60).unwrap();
    compare_to_golden(render_nametable(&mut system), "test_alter_ego_intro_0");

    system.execute_frames(240).unwrap();
    compare_to_golden(render_nametable(&mut system), "test_alter_ego_intro_1");
}

#[test]
pub fn test_alter_ego_pattern_table() {
    let mut system = System::with_ines(Path::new("tests/ppu/alter_ego.nes")).unwrap();
    system.execute_frames(60).unwrap();
    compare_to_golden(
        system.cpu.bus.ppu.render_pattern_table(0).unwrap(),
        "test_alter_ego_pattern_table",
    );
}

pub fn compare_to_golden(image: RgbImage, name: &str) {
    let path_prefix = PathBuf::from("tests/ppu").join(name);
    let golden_path = path_prefix.with_extension("png");
    if golden_path.exists() {
        let golden: RgbImage = image::open(golden_path).unwrap().into_rgb8();
        assert_eq!(image, golden);
    } else {
        image.save(golden_path).unwrap();
    }
}

fn render_nametable(system: &mut System) -> ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    let mut actual: RgbImage = ImageBuffer::new(33 * 8, 30 * 8);
    system
        .cpu
        .bus
        .ppu
        .render_nametable(&mut actual.sub_image(0, 0, 32 * 8, 30 * 8));
    actual
}
