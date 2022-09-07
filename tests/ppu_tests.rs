use image::RgbImage;

use ners::nes::System;

use std::path::Path;
use std::path::PathBuf;

#[test]
pub fn test_render_tile_bank() {
    let mut system = System::with_ines(Path::new("tests/ppu/alter_ego.nes")).unwrap();
    let ppu = &mut system.cpu.bus.ppu;
    // Write example palette 0.
    ppu.write_ppu_memory(0x3F00, 0x0F);
    ppu.write_ppu_memory(0x3F01, 0x01);
    ppu.write_ppu_memory(0x3F02, 0x11);
    ppu.write_ppu_memory(0x3F03, 0x21);
    compare_to_golden(ppu.render_chr(0).unwrap(), "alter_ego_tiles");
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
