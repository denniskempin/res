use image::RgbImage;

use ners::nes::System;

use std::path::Path;
use std::path::PathBuf;

#[test]
pub fn test_render_tile_bank() {
    let system = System::with_ines(Path::new("tests/ppu/alter_ego.nes")).unwrap();
    let ppu = &system.cpu.bus.ppu;
    compare_to_golden(ppu.render_tile_bank(0).unwrap(), "alter_ego_bank0");
    compare_to_golden(ppu.render_tile_bank(1).unwrap(), "alter_ego_bank1");
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
