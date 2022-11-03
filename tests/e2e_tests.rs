use std::path::Path;
use std::path::PathBuf;

use image::RgbaImage;
use res::nes::System;


#[test]
pub fn test_nestest() {
    let mut system = System::with_ines(Path::new("tests/e2e/nestest.nes")).unwrap();
    system.playback_from_file(Path::new("tests/e2e/nestest.recording.json"));
    execute_and_compare_screenshots("nestest", &mut system, &[60, 90]);
}

pub fn execute_and_compare_screenshots(name: &str, system: &mut System, frame_numbers: &[usize]) {
    for frame_number in frame_numbers {
        while system.ppu().frame != *frame_number {
            system.update_buttons([false; 8]);
            system.execute_one_frame().unwrap();
        }
        compare_to_golden(
            &system.ppu().framebuffer.as_rgba_image(),
            &format!("{name}-{frame_number}")
        );
    } 
}

pub fn compare_to_golden(image: &RgbaImage, name: &str) {
    let path_prefix = PathBuf::from("tests/e2e").join(name);
    let golden_path = path_prefix.with_extension("png");
    if golden_path.exists() {
        let golden: RgbaImage = image::open(&golden_path).unwrap().into_rgba8();
        if golden != *image {
            let actual_path = golden_path.with_extension("actual.png");
            image.save(&actual_path).unwrap();
            panic!("Image {} does not match golden. See {:?}", name, actual_path);
        }
    } else {
        image.save(golden_path).unwrap();
    }
}
