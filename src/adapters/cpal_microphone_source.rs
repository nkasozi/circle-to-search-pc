use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};

use crate::core::interfaces::ports::{ActiveCapture, AudioInputSource, MediaSink};
use crate::core::models::{AudioChunk, AudioTrackKind};

mod sample_conversion;

use sample_conversion::{convert_signed_samples, convert_unsigned_samples};

const LOG_TAG: &str = "[MICROPHONE]";
const NO_MICROPHONE_ERROR: &str = "No microphone was found";
const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);
const MICROPHONE_START_TIMEOUT_ERROR: &str = "microphone did not start in time";
const MICROPHONE_THREAD_PANICKED_ERROR: &str = "microphone thread panicked";
const MICROPHONE_FORMAT_UNAVAILABLE_ERROR: &str = "microphone format unavailable";
const MICROPHONE_OPEN_FAILED_ERROR: &str = "could not open the microphone";
const MICROPHONE_START_FAILED_ERROR: &str = "could not start the microphone";
#[cfg(target_os = "macos")]
const AUDIO_MEDIA_TYPE_UNAVAILABLE_ERROR: &str = "audio media type unavailable";
const EVENT_MICROPHONE_CAPTURE_STARTED: &str = "microphone_capture_started";
const EVENT_MICROPHONE_CAPTURE_STOPPED: &str = "microphone_capture_stopped";
const EVENT_MICROPHONE_OPENING: &str = "microphone_opening";
const EVENT_MICROPHONE_STREAM_ERROR: &str = "microphone_stream_error";

pub struct CpalMicrophoneSource;

struct MicrophoneCapture {
    stop_sender: mpsc::Sender<()>,
    worker: JoinHandle<()>,
}

impl AudioInputSource for CpalMicrophoneSource {
    fn start_capture(&self, sink: Arc<dyn MediaSink>) -> Result<Box<dyn ActiveCapture>> {
        let (startup_sender, startup_receiver) = mpsc::channel::<std::result::Result<(), String>>();
        let (stop_sender, stop_receiver) = mpsc::channel::<()>();
        let worker = std::thread::spawn(move || {
            let stream_result = open_default_input_stream(sink);
            let stream = match stream_result {
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
            .context(MICROPHONE_START_TIMEOUT_ERROR)?
            .map_err(|message| anyhow!(message))?;
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": EVENT_MICROPHONE_CAPTURE_STARTED})
        );
        Ok(Box::new(MicrophoneCapture {
            stop_sender,
            worker,
        }))
    }
}

impl ActiveCapture for MicrophoneCapture {
    fn stop(self: Box<Self>) -> Result<()> {
        let _ = self.stop_sender.send(());
        self.worker
            .join()
            .map_err(|_| anyhow!(MICROPHONE_THREAD_PANICKED_ERROR))?;
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": EVENT_MICROPHONE_CAPTURE_STOPPED})
        );
        Ok(())
    }
}

fn open_default_input_stream(sink: Arc<dyn MediaSink>) -> Result<Stream> {
    ensure_microphone_access()?;
    let device = cpal::default_host()
        .default_input_device()
        .context(NO_MICROPHONE_ERROR)?;
    let supported_config = device
        .default_input_config()
        .context(MICROPHONE_FORMAT_UNAVAILABLE_ERROR)?;
    let sample_format = supported_config.sample_format();
    let stream_config: StreamConfig = supported_config.config();
    let _ = log_microphone_opening(&stream_config, sample_format);
    let deliver = build_chunk_deliverer(sink, &stream_config);
    let stream = build_microphone_stream(&device, &stream_config, sample_format, deliver)?;
    stream.play().context(MICROPHONE_START_FAILED_ERROR)?;
    Ok(stream)
}

fn log_microphone_opening(stream_config: &StreamConfig, sample_format: SampleFormat) -> bool {
    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": EVENT_MICROPHONE_OPENING, "sample_rate": stream_config.sample_rate, "channels": stream_config.channels, "format": format!("{:?}", sample_format)})
    );
    true
}

fn build_chunk_deliverer(
    sink: Arc<dyn MediaSink>,
    stream_config: &StreamConfig,
) -> impl Fn(Vec<f32>) -> bool + Send + 'static {
    let sample_rate = stream_config.sample_rate;
    let channel_count = stream_config.channels;
    move |samples: Vec<f32>| {
        sink.accept_audio_chunk(
            AudioTrackKind::Microphone,
            AudioChunk {
                sample_rate,
                channel_count,
                interleaved_samples: samples,
            },
        )
    }
}

fn build_microphone_stream(
    device: &cpal::Device,
    stream_config: &StreamConfig,
    sample_format: SampleFormat,
    deliver: impl Fn(Vec<f32>) -> bool + Send + 'static,
) -> Result<Stream> {
    let on_error = |stream_error: cpal::StreamError| {
        log::error!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": EVENT_MICROPHONE_STREAM_ERROR, "error": stream_error.to_string()})
        );
    };
    match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            stream_config,
            move |data: &[f32], _| {
                deliver(data.to_vec());
            },
            on_error,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            stream_config,
            move |data: &[i16], _| {
                deliver(convert_signed_samples(data));
            },
            on_error,
            None,
        ),
        SampleFormat::U16 => device.build_input_stream(
            stream_config,
            move |data: &[u16], _| {
                deliver(convert_unsigned_samples(data));
            },
            on_error,
            None,
        ),
        unsupported => {
            return Err(anyhow!(
                "unsupported microphone sample format {:?}",
                unsupported
            ))
        }
    }
    .context(MICROPHONE_OPEN_FAILED_ERROR)
}

#[cfg(target_os = "macos")]
fn ensure_microphone_access() -> Result<bool> {
    use crate::adapters::macos_media::{ensure_capture_access, CaptureDeviceKind};
    let audio_type = unsafe { objc2_av_foundation::AVMediaTypeAudio }
        .context(AUDIO_MEDIA_TYPE_UNAVAILABLE_ERROR)?;
    ensure_capture_access(audio_type, CaptureDeviceKind::Microphone)
}

#[cfg(not(target_os = "macos"))]
fn ensure_microphone_access() -> Result<bool> {
    Ok(true)
}
