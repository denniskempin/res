use libretro_rs::*;
use res_emulator::joypad::JoypadButton;
use res_emulator::ppu::Framebuffer;
use res_emulator::System;

struct ResCore {
    pixels: Vec<u8>,
    emulator: Option<System>,
}

libretro_core!(ResCore);

impl RetroCore for ResCore {
    fn init(_env: &RetroEnvironment) -> Self {
        Self {
            pixels: vec![0; Framebuffer::SIZE[0] * Framebuffer::SIZE[1] * 4],
            emulator: None,
        }
    }

    fn get_system_info() -> RetroSystemInfo {
        RetroSystemInfo::new("Rust Entertainment System", env!("CARGO_PKG_VERSION"))
    }

    fn load_game(&mut self, _env: &RetroEnvironment, game: RetroGame) -> RetroLoadGameResult {
        if let RetroGame::Data { data, meta: _ } = game {
            self.emulator = Some(System::with_ines_bytes(data, None).unwrap());
        }
        RetroLoadGameResult::Success {
            audio: RetroAudioInfo::new(44_1000.0),
            video: RetroVideoInfo::new(
                60.0,
                Framebuffer::SIZE[0] as u32,
                Framebuffer::SIZE[1] as u32,
            )
            .with_pixel_format(RetroPixelFormat::XRGB8888),
        }
    }

    fn reset(&mut self, _env: &RetroEnvironment) {}

    fn run(&mut self, _env: &RetroEnvironment, runtime: &RetroRuntime) {
        if let Some(emulator) = self.emulator.as_mut() {
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
            self.pixels = emulator.ppu().framebuffer.as_rgba_image().into_raw();
        }

        runtime.upload_video_frame(
            &self.pixels,
            Framebuffer::SIZE[0] as u32,
            Framebuffer::SIZE[1] as u32,
            Framebuffer::SIZE[0] * 4,
        );
    }
}
