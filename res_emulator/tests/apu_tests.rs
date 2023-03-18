use std::fs::File;
use std::path::{Path, PathBuf};
use wav::{self, BitDepth};

use res_emulator::apu::Apu;

static SAMPLE_RATE: usize = 44100;

#[test]
pub fn test_dk_intro() {
    apu_audio_test("dk_intro", include!("apu/dk_intro.log"));
}

#[test]
pub fn test_smb_intro() {
    apu_audio_test("smb_intro", include!("apu/smb_intro.log"));
}

fn apu_audio_test(test_name: &str, data: &[(usize, u16, u8)]) {
    let mut apu = Apu::default();
    apu.audio_sample_rate = SAMPLE_RATE;
    let mut output = Vec::new();
    let mut current_cycle = data[0].0;
    for (cycle, addr, value) in data {
        apu.advance_clock(cycle - current_cycle).unwrap();
        apu.cpu_bus_write(*addr, *value);
        output.append(&mut apu.audio_buffer);
        current_cycle = *cycle;
    }
    compare_to_golden(test_name, output);
}

fn compare_to_golden(test_name: &str, output: Vec<f32>) {
    let golden_path = PathBuf::from(format!("tests/apu/{test_name}.golden.wav"));
    let actual_path = PathBuf::from(format!("tests/apu/{test_name}.actual.wav"));

    if !golden_path.exists() {
        write_wav(&golden_path, output);
    } else {
        let golden = read_wav(&golden_path);
        if !compare_signals(&output, &golden) {
            write_wav(&actual_path, output);
            panic!("Output of test {test_name} does not match golden");
        }
    }
}

fn compare_signals(a: &[f32], b: &[f32]) -> bool {
    // TODO: Implement cross-corellation for more stable comparison.
    if a.len() != b.len() {
        println!("Signal lengths do not match");
        return false;
    }
    for i in 0..a.len() {
        if (a[i] - b[i]).abs() > 0.01 {
            return false;
        }
    }
    true
}

fn write_wav(filename: &Path, data: Vec<f32>) {
    let mut file = File::create(filename).unwrap();
    let header = wav::header::Header::new(
        wav::header::WAV_FORMAT_IEEE_FLOAT,
        1,
        SAMPLE_RATE as u32,
        32,
    );
    wav::write(header, &data.into(), &mut file).unwrap();
}

fn read_wav(filename: &Path) -> Vec<f32> {
    let mut file = File::open(filename).unwrap();
    let (_, data) = wav::read(&mut file).unwrap();
    if let BitDepth::ThirtyTwoFloat(data) = data {
        data
    } else {
        panic!("Invalid wav format");
    }
}
