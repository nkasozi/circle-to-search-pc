use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};

use crate::core::interfaces::ports::{ActiveAudioOutput, AudioOutput, PlaybackAudioSource};

const LOG_TAG: &str = "[AUDIO_OUTPUT]";
const NO_SPEAKERS_ERROR: &str = "No audio output device was found";
const STARTUP_TIMEOUT: Duration = Duration::from_secs(5);
const AUDIO_OUTPUT_START_TIMEOUT_ERROR: &str = "audio output did not start";
const AUDIO_OUTPUT_THREAD_PANICKED_ERROR: &str = "audio output thread panicked";
const SPEAKER_FORMAT_UNAVAILABLE_ERROR: &str = "speaker format unavailable";
const SPEAKER_OPEN_FAILED_ERROR: &str = "could not open the speakers";
const SPEAKER_START_FAILED_ERROR: &str = "could not start the speakers";
const EVENT_AUDIO_OUTPUT_STARTED: &str = "audio_output_started";
const EVENT_AUDIO_OUTPUT_ERROR: &str = "audio_output_error";

pub struct CpalAudioOutput;

struct CpalActiveOutput {
    stop_sender: mpsc::Sender<()>,
    worker: JoinHandle<()>,
}

impl AudioOutput for CpalAudioOutput {
    fn start(&self, source: Arc<dyn PlaybackAudioSource>) -> Result<Box<dyn ActiveAudioOutput>> {
        let (startup_sender, startup_receiver) = mpsc::channel::<std::result::Result<(), String>>();
        let (stop_sender, stop_receiver) = mpsc::channel::<()>();
        let worker = std::thread::spawn(move || {
            let stream = match open_output_stream(source) {
                Ok(stream) => stream,
                Err(open_error) => {
                    let _ = startup_sender.send(Err(open_error.to_string()));
                    return;
                }
            };
            let _ = startup_sender.send(Ok(()));
            let _ = stop_receiver.recv();
            drop(stream);
        });
        startup_receiver
            .recv_timeout(STARTUP_TIMEOUT)
            .context(AUDIO_OUTPUT_START_TIMEOUT_ERROR)?
            .map_err(|message| anyhow!(message))?;
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": EVENT_AUDIO_OUTPUT_STARTED})
        );
        Ok(Box::new(CpalActiveOutput {
            stop_sender,
            worker,
        }))
    }
}

impl ActiveAudioOutput for CpalActiveOutput {
    fn stop(self: Box<Self>) -> Result<()> {
        let _ = self.stop_sender.send(());
        self.worker
            .join()
            .map_err(|_| anyhow!(AUDIO_OUTPUT_THREAD_PANICKED_ERROR))
    }
}

fn open_output_stream(source: Arc<dyn PlaybackAudioSource>) -> Result<Stream> {
    let device = cpal::default_host()
        .default_output_device()
        .context(NO_SPEAKERS_ERROR)?;
    let supported_config = device
        .default_output_config()
        .context(SPEAKER_FORMAT_UNAVAILABLE_ERROR)?;
    let sample_format = supported_config.sample_format();
    let config: StreamConfig = supported_config.config();
    let stream = build_speaker_stream(&device, &config, sample_format, source)?;
    stream.play().context(SPEAKER_START_FAILED_ERROR)?;
    Ok(stream)
}

fn build_speaker_stream(
    device: &cpal::Device,
    config: &StreamConfig,
    sample_format: SampleFormat,
    source: Arc<dyn PlaybackAudioSource>,
) -> Result<Stream> {
    let (sample_rate, channel_count) = (config.sample_rate, config.channels);
    let on_error = |stream_error: cpal::StreamError| {
        log::error!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": EVENT_AUDIO_OUTPUT_ERROR, "error": stream_error.to_string()})
        );
    };
    match sample_format {
        SampleFormat::F32 => device.build_output_stream(
            config,
            move |data: &mut [f32], _| {
                source.fill(data, sample_rate, channel_count);
            },
            on_error,
            None,
        ),
        SampleFormat::I16 => device.build_output_stream(
            config,
            move |data: &mut [i16], _| {
                let mut floats = vec![0.0f32; data.len()];
                source.fill(&mut floats, sample_rate, channel_count);
                for (slot, sample) in data.iter_mut().zip(floats) {
                    *slot = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                }
            },
            on_error,
            None,
        ),
        unsupported => {
            return Err(anyhow!(
                "unsupported speaker sample format {:?}",
                unsupported
            ))
        }
    }
    .context(SPEAKER_OPEN_FAILED_ERROR)
}
