use std::path::PathBuf;

use image::RgbaImage;
use res::nes::System;


#[test]
pub fn test_nestest() {
    test_playback("nestest", &[60, 90]);
}

#[test]
pub fn test_donkey_kong() {
    test_playback("donkey_kong", &[100, 1000, 2000, 3000]);
}

fn test_playback(name: &str, frame_numbers: &[usize]) {
    let rom_path = PathBuf::from(&format!("tests/e2e/{name}.nes"));
    let recording_path = PathBuf::from(&format!("tests/e2e/{name}.recording.json"));
    if !rom_path.exists() {
        return;
    }
    let mut system = System::with_ines(&rom_path).unwrap();
    system.playback_from_file(&recording_path);
    execute_and_compare_screenshots(name, &mut system, frame_numbers);
}

fn execute_and_compare_screenshots(name: &str, system: &mut System, frame_numbers: &[usize]) {
    for frame_number in frame_numbers {
        // Don't render while skipping frames to make test run faster.
        system.cpu.bus.ppu.skip_rendering = true;
        while system.ppu().frame < *frame_number - 1 {
            system.update_buttons([false; 8]);
            system.execute_one_frame().unwrap();
        }
        system.cpu.bus.ppu.skip_rendering = false;
        system.update_buttons([false; 8]);
        system.execute_one_frame().unwrap();
        compare_to_golden(
            &system.ppu().framebuffer.as_rgba_image(),
            &format!("{name}-{frame_number}")
        );
    } 
}

fn compare_to_golden(image: &RgbaImage, name: &str) {
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
