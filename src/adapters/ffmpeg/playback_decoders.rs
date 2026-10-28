use std::io::Read;
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};

use super::locator::{locate_bundled_ffmpeg, BUNDLED_ENCODER_MISSING_MESSAGE};
use super::media_probe::{parse_media_probe, MediaProbe};
use super::playback_arguments::{
    build_audio_decode_arguments, build_probe_arguments, build_video_decode_arguments,
    scaled_even_size, PLAYBACK_CHANNEL_COUNT, PLAYBACK_SAMPLE_RATE,
};
use crate::core::interfaces::ports::{
    AudioTrackDecoder, MediaDecoderFactory, PlaybackRequest, TimedVideoFrame, VideoTrackDecoder,
};
use crate::core::models::{AudioChunk, PixelLayout, VideoFrame};

const AUDIO_CHUNK_FRAMES: usize = 4_800;
const UNREADABLE_RECORDING_ERROR: &str = "could not read this recording";
const PLAYBACK_START_FAILED_ERROR: &str = "could not start playback";
const PLAYBACK_OUTPUT_UNAVAILABLE_ERROR: &str = "playback output unavailable";
const INSPECT_RECORDING_FAILED_ERROR: &str = "could not inspect the recording";
const FLOAT_SAMPLE_BYTES: usize = 4;

pub struct FfmpegDecoderFactory;

struct ProcessOutput {
    child: Child,
    stdout: ChildStdout,
}

impl ProcessOutput {
    fn spawn(arguments: &[String]) -> Result<Self> {
        let executable = locate_bundled_ffmpeg().context(BUNDLED_ENCODER_MISSING_MESSAGE)?;
        let mut child = Command::new(executable)
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context(PLAYBACK_START_FAILED_ERROR)?;
        let stdout = child
            .stdout
            .take()
            .context(PLAYBACK_OUTPUT_UNAVAILABLE_ERROR)?;
        Ok(Self { child, stdout })
    }

    fn read_up_to(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let mut filled = 0usize;
        while filled < buffer.len() {
            let read = self.stdout.read(&mut buffer[filled..])?;
            if read == 0 {
                break;
            }
            filled += read;
        }
        Ok(filled)
    }
}

impl Drop for ProcessOutput {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn probe(file_path: &std::path::Path) -> Result<MediaProbe> {
    let executable = locate_bundled_ffmpeg().context(BUNDLED_ENCODER_MISSING_MESSAGE)?;
    let output = Command::new(executable)
        .args(build_probe_arguments(file_path))
        .stdin(Stdio::null())
        .output()
        .context(INSPECT_RECORDING_FAILED_ERROR)?;
    parse_media_probe(&String::from_utf8_lossy(&output.stderr)).context(UNREADABLE_RECORDING_ERROR)
}

struct FfmpegVideoDecoder {
    process: ProcessOutput,
    width: u32,
    height: u32,
    start: Duration,
    frames_per_second: f64,
    frames_read: u64,
}

struct FfmpegAudioDecoder {
    process: ProcessOutput,
}

impl MediaDecoderFactory for FfmpegDecoderFactory {
    fn media_duration(&self, request: &PlaybackRequest) -> Result<Duration> {
        Ok(probe(&request.file_path)?.duration)
    }

    fn open_video(
        &self,
        request: &PlaybackRequest,
        start: Duration,
    ) -> Result<Box<dyn VideoTrackDecoder>> {
        let media = probe(&request.file_path)?;
        let (width, height) =
            scaled_even_size(media.width, media.height, request.maximum_frame_width);
        let process = ProcessOutput::spawn(&build_video_decode_arguments(
            &request.file_path,
            start,
            width,
            height,
        ))?;
        Ok(Box::new(FfmpegVideoDecoder {
            process,
            width,
            height,
            start,
            frames_per_second: media.frames_per_second,
            frames_read: 0,
        }))
    }

    fn open_audio(
        &self,
        request: &PlaybackRequest,
        start: Duration,
    ) -> Result<Option<Box<dyn AudioTrackDecoder>>> {
        if !probe(&request.file_path)?.has_audio {
            return Ok(None);
        }
        let process =
            ProcessOutput::spawn(&build_audio_decode_arguments(&request.file_path, start))?;
        Ok(Some(Box::new(FfmpegAudioDecoder { process })))
    }
}

impl VideoTrackDecoder for FfmpegVideoDecoder {
    fn next_frame(&mut self) -> Result<Option<TimedVideoFrame>> {
        let mut buffer = vec![0u8; PixelLayout::Rgba.expected_byte_length(self.width, self.height)];
        if self.process.read_up_to(&mut buffer)? < buffer.len() {
            return Ok(None);
        }
        let timestamp = self.start
            + Duration::from_secs_f64(self.frames_read as f64 / self.frames_per_second.max(1.0));
        self.frames_read += 1;
        let frame = VideoFrame {
            width: self.width,
            height: self.height,
            pixel_layout: PixelLayout::Rgba,
            data: Arc::new(buffer),
        };
        Ok(Some(TimedVideoFrame { timestamp, frame }))
    }
}

impl AudioTrackDecoder for FfmpegAudioDecoder {
    fn next_chunk(&mut self) -> Result<Option<AudioChunk>> {
        let mut buffer =
            vec![0u8; AUDIO_CHUNK_FRAMES * PLAYBACK_CHANNEL_COUNT as usize * FLOAT_SAMPLE_BYTES];
        let filled = self.process.read_up_to(&mut buffer)?;
        if filled == 0 {
            return Ok(None);
        }
        let samples = buffer[..filled - filled % FLOAT_SAMPLE_BYTES]
            .chunks_exact(FLOAT_SAMPLE_BYTES)
            .map(|value| f32::from_le_bytes([value[0], value[1], value[2], value[3]]))
            .collect();
        Ok(Some(AudioChunk {
            sample_rate: PLAYBACK_SAMPLE_RATE,
            channel_count: PLAYBACK_CHANNEL_COUNT,
            interleaved_samples: samples,
        }))
    }
}
