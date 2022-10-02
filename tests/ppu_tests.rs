use std::path::Path;
use std::path::PathBuf;

use egui::ColorImage;
use image::RgbaImage;
use res::nes::ppu::Framebuffer;
use res::nes::System;

#[test]
pub fn test_nestest_title_screen() {
    let mut system = System::with_ines(Path::new("tests/cpu/nestest.nes")).unwrap();
    system.execute_frames(60).unwrap();
    compare_to_golden(
        &system.ppu().framebuffer.as_rgba_image(),
        "test_nestest_title_screen",
    );
}

#[test]
pub fn test_alter_ego_intro() {
    let mut system = System::with_ines(Path::new("tests/ppu/alter_ego.nes")).unwrap();

    system.execute_frames(60).unwrap();
    compare_to_golden(
        &system.ppu().framebuffer.as_rgba_image(),
        "test_alter_ego_intro_0",
    );

    system.execute_frames(240).unwrap();
    compare_to_golden(
        &system.ppu().framebuffer.as_rgba_image(),
        "test_alter_ego_intro_1",
    );
}

#[test]
pub fn test_alter_ego_nametable() {
    let mut system = System::with_ines(Path::new("tests/ppu/alter_ego.nes")).unwrap();

    system.execute_frames(60).unwrap();
    compare_to_golden(
        &into_rgba(&system.ppu().debug_render_nametable()),
        "test_alter_ego_nametable_0",
    );
}

#[test]
#[ignore] // Requires donkey_kong.nes rom
pub fn test_donkey_kong_intro() {
    let mut system = System::with_ines(Path::new("tests/ppu/donkey_kong.nes")).unwrap();

    system.execute_frames(60).unwrap();
    compare_to_golden(
        &system.ppu().framebuffer.as_rgba_image(),
        "test_donkey_kong_intro",
    );
}

pub fn into_rgba(image: &ColorImage) -> RgbaImage {
    RgbaImage::from_vec(
        image.width() as u32,
        image.height() as u32,
        image
            .pixels
            .iter()
            .flat_map(|c| [c.r(), c.g(), c.b(), c.a()])
            .collect(),
    )
    .unwrap()
}

pub fn compare_to_golden(image: &RgbaImage, name: &str) {
    let path_prefix = PathBuf::from("tests/ppu").join(name);
    let golden_path = path_prefix.with_extension("png");
    if golden_path.exists() {
        let golden: RgbaImage = image::open(golden_path).unwrap().into_rgba8();
        assert_eq!(*image, golden);
    } else {
        image.save(golden_path).unwrap();
    }
}
