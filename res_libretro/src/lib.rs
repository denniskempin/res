use std::time::Instant;

use libretro_rs::*;
use res_emulator::joypad::JoypadButton;
use res_emulator::ppu::Framebuffer;
use res_emulator::System;

struct ResCore {
    pixels: Vec<u8>,
    emulator: Option<System>,
    counter: f32,
    audio_buffer: Vec<i16>,
    last_frame_time: Instant,
}

libretro_core!(ResCore);

const SAMPLE_RATE: f64 = 44_1000.0;
const FRAME_RATE: f64 = 120.0;
const SAMPLES_PER_FRAME: usize = (SAMPLE_RATE / FRAME_RATE) as usize + 1;

impl RetroCore for ResCore {
    fn init(_env: &RetroEnvironment) -> Self {
        Self {
            pixels: vec![0; Framebuffer::SIZE[0] * Framebuffer::SIZE[1] * 4],
            emulator: None,
            counter: 0.0,
            audio_buffer: vec![0; SAMPLES_PER_FRAME * 2],
            last_frame_time: Instant::now(),
        }
    }

    fn get_system_info() -> RetroSystemInfo {
        RetroSystemInfo::new("Rust Entertainment System", env!("CARGO_PKG_VERSION"))
            .with_valid_extensions(&["nes", "srm"])
    }

    fn load_game(&mut self, _env: &RetroEnvironment, game: RetroGame) -> RetroLoadGameResult {
        if let RetroGame::Data { data, meta: _ } = game {
            self.emulator = Some(System::with_ines_bytes(data, None).unwrap());
        }
        RetroLoadGameResult::Success {
            audio: RetroAudioInfo::new(SAMPLE_RATE),
            video: RetroVideoInfo::new(
                FRAME_RATE,
                Framebuffer::SIZE[0] as u32,
                Framebuffer::SIZE[1] as u32,
            )
            .with_pixel_format(RetroPixelFormat::XRGB8888),
        }
    }

    fn reset(&mut self, _env: &RetroEnvironment) {}

    fn run(&mut self, _env: &RetroEnvironment, runtime: &RetroRuntime) {
        if let Some(emulator) = self.emulator.as_mut() {
            let emu_start_time = Instant::now();

            let mut joypad0 = [false; 8];
            joypad0[JoypadButton::Right as usize] =
                runtime.is_joypad_button_pressed(0, RetroJoypadButton::Right);
            joypad0[JoypadButton::Left as usize] =
                runtime.is_joypad_button_pressed(0, RetroJoypadButton::Left);
            joypad0[JoypadButton::Up as usize] =
                runtime.is_joypad_button_pressed(0, RetroJoypadButton::Up);
            joypad0[JoypadButton::Down as usize] =
                runtime.is_joypad_button_pressed(0, RetroJoypadButton::Down);
            joypad0[JoypadButton::Start as usize] =
                runtime.is_joypad_button_pressed(0, RetroJoypadButton::Start);
            joypad0[JoypadButton::Select as usize] =
                runtime.is_joypad_button_pressed(0, RetroJoypadButton::Select);
            joypad0[JoypadButton::ButtonA as usize] =
                runtime.is_joypad_button_pressed(0, RetroJoypadButton::A);
            joypad0[JoypadButton::ButtonB as usize] =
                runtime.is_joypad_button_pressed(0, RetroJoypadButton::B);

            emulator.update_buttons(joypad0);
            emulator.execute_one_frame().unwrap();
            // TODO: Verify if this should be rgba on little endian architectures.
            self.pixels = emulator.ppu().framebuffer.as_raw_bgra();

            for i in 0..SAMPLES_PER_FRAME {
                let sample = (f32::sin(self.counter) * 4096.0) as i16;
                self.audio_buffer[i * 2] = sample;
                self.audio_buffer[i * 2 + 1] = sample;
                self.counter += 0.01;
            }

            let audio_start_time = Instant::now();
            runtime.upload_audio_frame(&self.audio_buffer);

            let video_start_time = Instant::now();
            runtime.upload_video_frame(
                &self.pixels,
                Framebuffer::SIZE[0] as u32,
                Framebuffer::SIZE[1] as u32,
                Framebuffer::SIZE[0] * 4,
            );

            let emu_time = audio_start_time - emu_start_time;
            let audio_time = video_start_time - audio_start_time;
            let video_time = Instant::now() - video_start_time;
            let frame_delta = Instant::now() - self.last_frame_time;
            self.last_frame_time = Instant::now();
            println!(
                "delta: {} (emu: {}, audio: {}, video: {})",
                frame_delta.as_millis(),
                emu_time.as_millis(),
                audio_time.as_millis(),
                video_time.as_millis(),
            );
        }
    }
}
