use std::path::PathBuf;

use image::RgbaImage;
use res::nes::System;

#[test]
pub fn test_nestest() {
    // Tests basic rendering with an open source ROM.
    test_playback("nestest", &[60, 90]);
}

#[test]
pub fn test_alter_ego() {
    // Tests gameplay with an open source ROM.
    test_playback("alter_ego", &[100, 600, 1000, 1500]);
}

#[test]
pub fn test_donkey_kong() {
    // Basic gameplay with a real ROM.
    test_playback("donkey_kong", &[100, 1000, 2000]);
}

#[test]
pub fn test_super_mario_bros() {
    // Vertical mirroring and scrolling.
    test_playback(
        "super_mario_bros",
        &[
            // Title and intro
            50, 150, // Split scroll glitches
            374, 389, 395,
        ],
    );
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
        while system.ppu().frame != *frame_number {
            system.update_buttons([false; 8]);
            system.execute_one_frame().unwrap();
        }
        compare_to_golden(
            &system.ppu().framebuffer.as_rgba_image(),
            &format!("{name}-{frame_number}"),
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
            panic!(
                "Image {} does not match golden. See {:?}",
                name, actual_path
            );
        }
    } else {
        image.save(golden_path).unwrap();
    }
}
