use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};

use crate::core::interfaces::adapters::VideoEncoder;
use crate::core::models::recording::{AudioSample, EncodingConfig, RecordingFrame};
use crate::global_constants::{
    FFMPEG_AUDIO_CODEC, FFMPEG_AUDIO_FORMAT, FFMPEG_BINARY_NAME, FFMPEG_LOG_LEVEL,
    FFMPEG_OUTPUT_FORMAT, FFMPEG_PIXEL_FORMAT, FFMPEG_VIDEO_CODEC, FFMPEG_VIDEO_FORMAT,
    FFMPEG_VIDEO_PRESET, LOG_TAG_RECORDING,
};

const AUDIO_PCM_BYTES_PER_SAMPLE: usize = 2;

enum EncoderProcessState {
    Idle,
    Encoding {
        video_process: Child,
        output_file_path: PathBuf,
    },
}

pub struct FfmpegVideoEncoder {
    process_state: EncoderProcessState,
}

impl FfmpegVideoEncoder {
    pub fn initialize() -> Self {
        log::debug!("{} initializing ffmpeg video encoder", LOG_TAG_RECORDING);
        Self {
            process_state: EncoderProcessState::Idle,
        }
    }

    pub fn detect_ffmpeg_availability() -> Result<bool> {
        let detection_result = std::process::Command::new(FFMPEG_BINARY_NAME)
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        match detection_result {
            Ok(exit_status) => {
                log::info!(
                    "{} ffmpeg detected, exit_status={}",
                    LOG_TAG_RECORDING,
                    exit_status
                );
                Ok(true)
            }
            Err(detection_error) => {
                log::warn!(
                    "{} ffmpeg not found: {}",
                    LOG_TAG_RECORDING,
                    detection_error
                );
                Ok(false)
            }
        }
    }

    fn build_ffmpeg_arguments(config: &EncodingConfig) -> Vec<String> {
        let video_size = format!("{}x{}", config.width_pixels, config.height_pixels);
        let framerate = config.frames_per_second.to_string();
        let sample_rate = config.audio_sample_rate.to_string();
        let channel_count = config.audio_channel_count.to_string();
        let output_path = config.output_file_path.to_string_lossy().to_string();

        vec![
            "-y".to_string(),
            "-loglevel".to_string(),
            FFMPEG_LOG_LEVEL.to_string(),
            "-f".to_string(),
            FFMPEG_VIDEO_FORMAT.to_string(),
            "-pix_fmt".to_string(),
            FFMPEG_PIXEL_FORMAT.to_string(),
            "-s".to_string(),
            video_size,
            "-r".to_string(),
            framerate,
            "-i".to_string(),
            "pipe:0".to_string(),
            "-f".to_string(),
            FFMPEG_AUDIO_FORMAT.to_string(),
            "-ar".to_string(),
            sample_rate,
            "-ac".to_string(),
            channel_count,
            "-i".to_string(),
            "pipe:3".to_string(),
            "-c:v".to_string(),
            FFMPEG_VIDEO_CODEC.to_string(),
            "-preset".to_string(),
            FFMPEG_VIDEO_PRESET.to_string(),
            "-c:a".to_string(),
            FFMPEG_AUDIO_CODEC.to_string(),
            "-f".to_string(),
            FFMPEG_OUTPUT_FORMAT.to_string(),
            output_path,
        ]
    }

    fn build_video_only_ffmpeg_arguments(config: &EncodingConfig) -> Vec<String> {
        let video_size = format!("{}x{}", config.width_pixels, config.height_pixels);
        let framerate = config.frames_per_second.to_string();
        let output_path = config.output_file_path.to_string_lossy().to_string();

        vec![
            "-y".to_string(),
            "-loglevel".to_string(),
            FFMPEG_LOG_LEVEL.to_string(),
            "-f".to_string(),
            FFMPEG_VIDEO_FORMAT.to_string(),
            "-pix_fmt".to_string(),
            FFMPEG_PIXEL_FORMAT.to_string(),
            "-s".to_string(),
            video_size,
            "-r".to_string(),
            framerate,
            "-i".to_string(),
            "pipe:0".to_string(),
            "-c:v".to_string(),
            FFMPEG_VIDEO_CODEC.to_string(),
            "-preset".to_string(),
            FFMPEG_VIDEO_PRESET.to_string(),
            "-an".to_string(),
            "-f".to_string(),
            FFMPEG_OUTPUT_FORMAT.to_string(),
            output_path,
        ]
    }

    fn convert_pcm_f32_to_s16le(pcm_float_data: &[f32]) -> Vec<u8> {
        let mut output_bytes =
            Vec::with_capacity(pcm_float_data.len() * AUDIO_PCM_BYTES_PER_SAMPLE);

        for &sample_value in pcm_float_data {
            let clamped = sample_value.clamp(-1.0, 1.0);
            let integer_sample = (clamped * i16::MAX as f32) as i16;
            output_bytes.extend_from_slice(&integer_sample.to_le_bytes());
        }

        output_bytes
    }

    fn spawn_ffmpeg_process(arguments: &[String]) -> Result<Child> {
        log::info!(
            "{} spawning ffmpeg with {} arguments",
            LOG_TAG_RECORDING,
            arguments.len()
        );

        Command::new(FFMPEG_BINARY_NAME)
            .args(arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .context("failed to spawn ffmpeg process")
    }
}

#[async_trait]
impl VideoEncoder for FfmpegVideoEncoder {
    async fn start_encoding(&mut self, config: EncodingConfig) -> Result<bool> {
        if matches!(self.process_state, EncoderProcessState::Encoding { .. }) {
            log::warn!(
                "{} encoding already in progress, ignoring start request",
                LOG_TAG_RECORDING
            );
            return Ok(false);
        }

        let has_audio = config.audio_sample_rate > 0 && config.audio_channel_count > 0;

        let arguments = match has_audio {
            true => Self::build_ffmpeg_arguments(&config),
            false => Self::build_video_only_ffmpeg_arguments(&config),
        };

        log::info!(
            "{} starting encoding: {}x{} @ {}fps, audio={}, output={}",
            LOG_TAG_RECORDING,
            config.width_pixels,
            config.height_pixels,
            config.frames_per_second,
            has_audio,
            config.output_file_path.display()
        );

        let process = Self::spawn_ffmpeg_process(&arguments)?;

        self.process_state = EncoderProcessState::Encoding {
            video_process: process,
            output_file_path: config.output_file_path,
        };

        Ok(true)
    }

    async fn write_video_frame(&mut self, frame: &RecordingFrame) -> Result<bool> {
        let EncoderProcessState::Encoding {
            ref mut video_process,
            ..
        } = self.process_state
        else {
            log::warn!(
                "{} cannot write frame: encoder not started",
                LOG_TAG_RECORDING
            );
            return Ok(false);
        };

        let stdin: &mut tokio::process::ChildStdin = video_process
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("ffmpeg stdin not available"))?;

        stdin
            .write_all(&frame.raw_rgba_data)
            .await
            .context("failed to write video frame to ffmpeg stdin")?;

        Ok(true)
    }

    async fn write_audio_samples(&mut self, sample: &AudioSample) -> Result<bool> {
        let EncoderProcessState::Encoding {
            ref mut video_process,
            ..
        } = self.process_state
        else {
            log::warn!(
                "{} cannot write audio: encoder not started",
                LOG_TAG_RECORDING
            );
            return Ok(false);
        };

        let stdin: &mut tokio::process::ChildStdin = video_process
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("ffmpeg stdin not available for audio"))?;

        let pcm_bytes = Self::convert_pcm_f32_to_s16le(&sample.pcm_data);

        stdin
            .write_all(&pcm_bytes)
            .await
            .context("failed to write audio samples to ffmpeg")?;

        Ok(true)
    }

    async fn finish_encoding(&mut self) -> Result<PathBuf> {
        let current_state = std::mem::replace(&mut self.process_state, EncoderProcessState::Idle);

        let EncoderProcessState::Encoding {
            mut video_process,
            output_file_path,
        } = current_state
        else {
            return Err(anyhow::anyhow!(
                "cannot finish encoding: encoder not started"
            ));
        };

        log::info!(
            "{} finishing encoding, closing ffmpeg stdin",
            LOG_TAG_RECORDING
        );

        drop(video_process.stdin.take());

        let process_output = video_process
            .wait_with_output()
            .await
            .context("failed to wait for ffmpeg process")?;

        if !process_output.status.success() {
            let stderr_text = String::from_utf8_lossy(&process_output.stderr);
            log::error!(
                "{} ffmpeg exited with error: {}",
                LOG_TAG_RECORDING,
                stderr_text
            );
            return Err(anyhow::anyhow!(
                "ffmpeg encoding failed with exit code: {:?}, stderr: {}",
                process_output.status.code(),
                stderr_text
            ));
        }

        log::info!(
            "{} encoding completed successfully: {}",
            LOG_TAG_RECORDING,
            output_file_path.display()
        );

        Ok(output_file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_video_only_arguments_contains_expected_flags() {
        let config = EncodingConfig::build(1920, 1080, 30, 0, 0, PathBuf::from("/tmp/test.mp4"));

        let arguments = FfmpegVideoEncoder::build_video_only_ffmpeg_arguments(&config);

        assert!(arguments.contains(&"-y".to_string()));
        assert!(arguments.contains(&FFMPEG_VIDEO_FORMAT.to_string()));
        assert!(arguments.contains(&FFMPEG_PIXEL_FORMAT.to_string()));
        assert!(arguments.contains(&"1920x1080".to_string()));
        assert!(arguments.contains(&"30".to_string()));
        assert!(arguments.contains(&"pipe:0".to_string()));
        assert!(arguments.contains(&FFMPEG_VIDEO_CODEC.to_string()));
        assert!(arguments.contains(&FFMPEG_VIDEO_PRESET.to_string()));
        assert!(arguments.contains(&"-an".to_string()));
        assert!(arguments.contains(&"/tmp/test.mp4".to_string()));
    }

    #[test]
    fn test_build_full_arguments_includes_audio_params() {
        let config =
            EncodingConfig::build(1280, 720, 24, 44100, 2, PathBuf::from("/tmp/output.mp4"));

        let arguments = FfmpegVideoEncoder::build_ffmpeg_arguments(&config);

        assert!(arguments.contains(&FFMPEG_AUDIO_FORMAT.to_string()));
        assert!(arguments.contains(&"44100".to_string()));
        assert!(arguments.contains(&"2".to_string()));
        assert!(arguments.contains(&"pipe:3".to_string()));
        assert!(arguments.contains(&FFMPEG_AUDIO_CODEC.to_string()));
        assert!(!arguments.contains(&"-an".to_string()));
    }

    #[test]
    fn test_convert_pcm_f32_to_s16le_handles_full_range() {
        let pcm_samples = vec![0.0_f32, 1.0, -1.0, 0.5, -0.5];

        let output_bytes = FfmpegVideoEncoder::convert_pcm_f32_to_s16le(&pcm_samples);

        assert_eq!(
            output_bytes.len(),
            pcm_samples.len() * AUDIO_PCM_BYTES_PER_SAMPLE
        );

        let sample_zero = i16::from_le_bytes([output_bytes[0], output_bytes[1]]);
        assert_eq!(sample_zero, 0);

        let sample_max = i16::from_le_bytes([output_bytes[2], output_bytes[3]]);
        assert_eq!(sample_max, i16::MAX);

        let sample_min = i16::from_le_bytes([output_bytes[4], output_bytes[5]]);
        assert_eq!(sample_min, -i16::MAX);
    }

    #[test]
    fn test_convert_pcm_f32_to_s16le_clamps_out_of_range_values() {
        let pcm_samples = vec![2.0_f32, -3.0];

        let output_bytes = FfmpegVideoEncoder::convert_pcm_f32_to_s16le(&pcm_samples);

        let sample_clamped_high = i16::from_le_bytes([output_bytes[0], output_bytes[1]]);
        assert_eq!(sample_clamped_high, i16::MAX);

        let sample_clamped_low = i16::from_le_bytes([output_bytes[2], output_bytes[3]]);
        assert_eq!(sample_clamped_low, -i16::MAX);
    }

    #[test]
    fn test_convert_pcm_f32_to_s16le_empty_input_returns_empty() {
        let pcm_samples: Vec<f32> = vec![];

        let output_bytes = FfmpegVideoEncoder::convert_pcm_f32_to_s16le(&pcm_samples);

        assert!(output_bytes.is_empty());
    }

    #[test]
    fn test_initialize_creates_idle_encoder() {
        let encoder = FfmpegVideoEncoder::initialize();
        assert!(matches!(encoder.process_state, EncoderProcessState::Idle));
    }

    #[test]
    fn test_video_only_arguments_do_not_contain_audio_input() {
        let config = EncodingConfig::build(640, 480, 15, 0, 0, PathBuf::from("/tmp/no_audio.mp4"));

        let arguments = FfmpegVideoEncoder::build_video_only_ffmpeg_arguments(&config);

        assert!(!arguments.contains(&"pipe:3".to_string()));
        assert!(!arguments.contains(&FFMPEG_AUDIO_FORMAT.to_string()));
        assert!(!arguments.contains(&FFMPEG_AUDIO_CODEC.to_string()));
    }

    #[test]
    fn test_full_arguments_preserve_correct_order_for_video_input() {
        let config =
            EncodingConfig::build(1920, 1080, 60, 48000, 1, PathBuf::from("/tmp/ordered.mp4"));

        let arguments = FfmpegVideoEncoder::build_ffmpeg_arguments(&config);

        let rawvideo_position = arguments.iter().position(|a| a == FFMPEG_VIDEO_FORMAT);
        let pipe_zero_position = arguments.iter().position(|a| a == "pipe:0");
        let audio_format_position = arguments.iter().position(|a| a == FFMPEG_AUDIO_FORMAT);
        let pipe_three_position = arguments.iter().position(|a| a == "pipe:3");

        assert!(rawvideo_position < pipe_zero_position);
        assert!(pipe_zero_position < audio_format_position);
        assert!(audio_format_position < pipe_three_position);
    }
}
