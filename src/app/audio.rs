use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use cpal::traits::StreamTrait;
use cpal::Stream;

pub struct AudioEngine {
    output_stream: Stream,
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

fn play_stream<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Stream
where
    T: cpal::Sample,
{
    let channels = config.channels as usize;

    // Generate noise samples
    use fundsp::hacker::*;
    let mut noise = white();
    let mut next_value = move || noise.get_mono() as f32;

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _| write_data(data, channels, &mut next_value),
            err_fn,
        )
        .unwrap();
    stream.play().unwrap();
    stream
}

impl AudioEngine {
    pub fn new() -> AudioEngine {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");
        let config = device.default_output_config().unwrap();
        let output_stream = match config.sample_format() {
            cpal::SampleFormat::F32 => play_stream::<f32>(&device, &config.into()),
            cpal::SampleFormat::I16 => play_stream::<i16>(&device, &config.into()),
            cpal::SampleFormat::U16 => play_stream::<u16>(&device, &config.into()),
        };

        AudioEngine { output_stream }
    }

    pub fn start(&mut self) {
        self.output_stream.play().unwrap();
    }
}
