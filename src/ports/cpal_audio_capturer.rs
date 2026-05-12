use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::{self, Receiver};

use crate::core::interfaces::ports::AudioCapturer;
use crate::core::models::recording::AudioSample;

const LOG_TAG: &str = "[AUDIO_CAPTURER]";
const AUDIO_CHANNEL_BUFFER_SIZE: usize = 30;
const AUDIO_CHUNK_DURATION_MILLIS: u64 = 20;
const ERROR_NO_INPUT_DEVICE: &str = "no audio input device available";
const ERROR_NO_SUPPORTED_CONFIG: &str = "no supported audio input config found";
const ERROR_FAILED_TO_BUILD_STREAM: &str = "failed to build audio input stream";
const ERROR_FAILED_TO_START_STREAM: &str = "failed to start audio input stream";

pub struct CpalAudioCapturer {
    is_capturing: Arc<AtomicBool>,
}

impl CpalAudioCapturer {
    pub fn initialize() -> Self {
        log::debug!("{} initializing cpal audio capturer", LOG_TAG);
        Self {
            is_capturing: Arc::new(AtomicBool::new(false)),
        }
    }

    fn get_default_input_device() -> Result<cpal::Device> {
        let host = cpal::default_host();

        log::debug!("{} using audio host: {:?}", LOG_TAG, host.id(),);

        host.default_input_device()
            .ok_or_else(|| anyhow::anyhow!(ERROR_NO_INPUT_DEVICE))
    }

    fn get_input_config(
        device: &cpal::Device,
        preferred_sample_rate: Option<u32>,
    ) -> Result<cpal::SupportedStreamConfig> {
        let supported_configs = device
            .supported_input_configs()
            .with_context(|| ERROR_NO_SUPPORTED_CONFIG)?;

        let mut best_config: Option<cpal::SupportedStreamConfigRange> = None;

        for config_range in supported_configs {
            match &best_config {
                None => {
                    best_config = Some(config_range);
                }
                Some(current_best) => {
                    if config_range.channels() <= 2
                        && config_range.channels() >= current_best.channels()
                    {
                        best_config = Some(config_range);
                    }
                }
            }
        }

        let config_range = best_config.ok_or_else(|| anyhow::anyhow!(ERROR_NO_SUPPORTED_CONFIG))?;

        let chosen_config = match preferred_sample_rate {
            Some(rate) => {
                let clamped_rate = rate
                    .max(config_range.min_sample_rate())
                    .min(config_range.max_sample_rate());
                config_range.with_sample_rate(clamped_rate)
            }
            None => config_range.with_max_sample_rate(),
        };

        log::info!(
            "{} selected audio config: rate={}, channels={}, format={:?}",
            LOG_TAG,
            chosen_config.sample_rate(),
            chosen_config.channels(),
            chosen_config.sample_format(),
        );

        Ok(chosen_config)
    }
}

impl AudioCapturer for CpalAudioCapturer {
    fn start_audio_capture(
        &self,
        preferred_sample_rate: Option<u32>,
    ) -> Result<Receiver<AudioSample>> {
        log::info!(
            "{} starting audio capture, preferred_rate={:?}",
            LOG_TAG,
            preferred_sample_rate,
        );

        let device = Self::get_default_input_device()?;

        let device_name = device
            .description()
            .map(|desc| desc.name().to_string())
            .unwrap_or_else(|_| String::from("unknown"));
        log::info!("{} using input device: {}", LOG_TAG, device_name);

        let supported_config = Self::get_input_config(&device, preferred_sample_rate)?;
        let sample_rate = supported_config.sample_rate();
        let channel_count = supported_config.channels();
        let sample_format = supported_config.sample_format();

        self.is_capturing.store(true, Ordering::SeqCst);

        let (sample_sender, sample_receiver) =
            mpsc::channel::<AudioSample>(AUDIO_CHANNEL_BUFFER_SIZE);

        let is_capturing_flag = self.is_capturing.clone();
        let recording_start_time = Instant::now();

        let samples_per_chunk = (sample_rate as u64 * AUDIO_CHUNK_DURATION_MILLIS / 1000) as usize
            * channel_count as usize;

        let accumulated_samples: Arc<std::sync::Mutex<Vec<f32>>> =
            Arc::new(std::sync::Mutex::new(Vec::with_capacity(samples_per_chunk)));

        let accumulated_for_callback = accumulated_samples.clone();
        let sender_for_callback = sample_sender.clone();
        let flag_for_callback = is_capturing_flag.clone();

        let data_callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            if !flag_for_callback.load(Ordering::SeqCst) {
                return;
            }

            let mut buffer = match accumulated_for_callback.lock() {
                Ok(guard) => guard,
                Err(_) => return,
            };

            buffer.extend_from_slice(data);

            if buffer.len() >= samples_per_chunk {
                let chunk: Vec<f32> = buffer.drain(..).collect();
                let timestamp = recording_start_time.elapsed();

                let audio_sample = AudioSample::build(timestamp, chunk, sample_rate, channel_count);

                let _ = sender_for_callback.try_send(audio_sample);
            }
        };

        let error_callback = move |err: cpal::StreamError| {
            log::error!("{} audio stream error: {}", LOG_TAG, err);
        };

        let stream_config: cpal::StreamConfig = supported_config.into();

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device
                .build_input_stream(&stream_config, data_callback, error_callback, None)
                .with_context(|| ERROR_FAILED_TO_BUILD_STREAM)?,
            _ => {
                let converted_callback = move |data: &[f32], info: &cpal::InputCallbackInfo| {
                    let _ = (data, info);
                };
                device
                    .build_input_stream(&stream_config, converted_callback, error_callback, None)
                    .with_context(|| ERROR_FAILED_TO_BUILD_STREAM)?
            }
        };

        stream
            .play()
            .with_context(|| ERROR_FAILED_TO_START_STREAM)?;

        let stop_flag = self.is_capturing.clone();

        std::thread::spawn(move || {
            let _stream_guard = stream;

            while stop_flag.load(Ordering::SeqCst) {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            log::info!("{} audio capture thread ending", LOG_TAG);
        });

        log::info!(
            "{} audio capture started: rate={}, channels={}",
            LOG_TAG,
            sample_rate,
            channel_count,
        );

        Ok(sample_receiver)
    }

    fn stop_audio_capture(&self) -> Result<bool> {
        log::info!("{} stopping audio capture", LOG_TAG);
        self.is_capturing.store(false, Ordering::SeqCst);
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_creates_instance_with_capturing_false() {
        let capturer = CpalAudioCapturer::initialize();
        assert!(!capturer.is_capturing.load(Ordering::SeqCst));
    }

    #[test]
    fn test_stop_audio_capture_sets_flag_to_false() {
        let capturer = CpalAudioCapturer::initialize();
        capturer.is_capturing.store(true, Ordering::SeqCst);

        let result = capturer.stop_audio_capture();

        assert!(result.is_ok());
        assert!(result.unwrap());
        assert!(!capturer.is_capturing.load(Ordering::SeqCst));
    }

    #[test]
    fn test_stop_audio_capture_is_idempotent() {
        let capturer = CpalAudioCapturer::initialize();

        let first_result = capturer.stop_audio_capture();
        let second_result = capturer.stop_audio_capture();

        assert!(first_result.is_ok());
        assert!(second_result.is_ok());
        assert!(!capturer.is_capturing.load(Ordering::SeqCst));
    }
}
