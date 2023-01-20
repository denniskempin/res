use std::sync::Arc;
use std::sync::Mutex;

use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use cpal::traits::StreamTrait;
use cpal::Stream;
use tracing::instrument;

pub struct AudioEngine {
    output_stream: Stream,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
}

#[instrument(skip_all)]
fn write_buffer<T>(output: &mut [T], channels: usize, audio_buffer: Arc<Mutex<Vec<f32>>>)
where
    T: cpal::Sample,
{
    let mut buffer = audio_buffer.lock().unwrap();

    let requested_size = output.len() / channels;
    let buffer_size = buffer.len();

    if requested_size > buffer_size {
        return;
    }

    for (frame, sample) in output.chunks_mut(channels).zip(buffer.drain(0..512)) {
        let value: T = cpal::Sample::from::<f32>(&sample);
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

fn play_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
) -> Stream
where
    T: cpal::Sample,
{
    let channels = config.channels as usize;
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _| write_buffer(data, channels, audio_buffer.clone()),
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
        let audio_buffer = Arc::new(Mutex::new(Vec::<f32>::with_capacity(1024 * 1024)));
        let output_stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                play_stream::<f32>(&device, &config.into(), audio_buffer.clone())
            }
            cpal::SampleFormat::I16 => {
                play_stream::<i16>(&device, &config.into(), audio_buffer.clone())
            }
            cpal::SampleFormat::U16 => {
                play_stream::<u16>(&device, &config.into(), audio_buffer.clone())
            }
        };

        AudioEngine {
            output_stream,
            audio_buffer,
        }
    }

    pub fn append_samples(&mut self, samples: &mut Vec<f32>) {
        self.audio_buffer.lock().unwrap().append(samples);
    }

    pub fn start(&mut self) {
        self.output_stream.play().unwrap();
    }
}
