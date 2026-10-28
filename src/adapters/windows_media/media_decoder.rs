use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use windows::core::PROPVARIANT;
use windows::Win32::Media::MediaFoundation::{
    IMFSample, IMFSourceReader, MFAudioFormat_Float, MFCreateSourceReaderFromURL,
    MFMediaType_Audio, MF_MT_AUDIO_NUM_CHANNELS, MF_MT_AUDIO_SAMPLES_PER_SECOND, MF_MT_MAJOR_TYPE,
    MF_MT_SUBTYPE, MF_PD_DURATION, MF_SOURCE_READERF_ENDOFSTREAM,
    MF_SOURCE_READER_FIRST_AUDIO_STREAM, MF_SOURCE_READER_MEDIASOURCE,
};

use super::frame_reader::{hundred_nanoseconds_to_duration, VideoFrameReader};
use super::runtime::{as_pcwstr, create_media_type, ensure_media_foundation, wide_path};
use crate::core::interfaces::ports::{
    AudioTrackDecoder, MediaDecoderFactory, PlaybackRequest, TimedVideoFrame, VideoTrackDecoder,
};
use crate::core::models::AudioChunk;
use crate::core::playback::frame_scaling::downscale_rgba_to_width;

const OPEN_RECORDING_FAILED_ERROR: &str = "could not open the recording";
const SEEK_RECORDING_FAILED_ERROR: &str = "could not seek the recording";
const DURATION_UNAVAILABLE_ERROR: &str = "recording duration is unavailable";

pub struct MediaFoundationDecoderFactory;

struct ReaderVideoDecoder {
    reader: VideoFrameReader,
    maximum_width: u32,
}

struct ReaderAudioDecoder {
    reader: IMFSourceReader,
    sample_rate: u32,
    channel_count: u16,
}

unsafe impl Send for ReaderAudioDecoder {}

fn audio_stream() -> u32 {
    MF_SOURCE_READER_FIRST_AUDIO_STREAM.0 as u32
}

fn open_reader(path: &Path) -> Result<IMFSourceReader> {
    ensure_media_foundation()?;
    let wide = wide_path(path);
    unsafe { MFCreateSourceReaderFromURL(as_pcwstr(&wide), None) }
        .context(OPEN_RECORDING_FAILED_ERROR)
}

fn seek_reader(reader: &IMFSourceReader, position: Duration) -> Result<()> {
    let variant = PROPVARIANT::from((position.as_nanos() / 100) as i64);
    unsafe { reader.SetCurrentPosition(&windows::core::GUID::zeroed(), &variant) }
        .context(SEEK_RECORDING_FAILED_ERROR)
}

impl MediaDecoderFactory for MediaFoundationDecoderFactory {
    fn media_duration(&self, request: &PlaybackRequest) -> Result<Duration> {
        let reader = open_reader(&request.file_path)?;
        let value = unsafe {
            reader.GetPresentationAttribute(MF_SOURCE_READER_MEDIASOURCE.0 as u32, &MF_PD_DURATION)
        }?;
        let hundred_nanoseconds = u64::try_from(&value).context(DURATION_UNAVAILABLE_ERROR)?;
        Ok(hundred_nanoseconds_to_duration(hundred_nanoseconds as i64))
    }

    fn open_video(
        &self,
        request: &PlaybackRequest,
        start: Duration,
    ) -> Result<Box<dyn VideoTrackDecoder>> {
        let mut reader = VideoFrameReader::open(&request.file_path)?;
        reader.seek(start)?;
        Ok(Box::new(ReaderVideoDecoder {
            reader,
            maximum_width: request.maximum_frame_width,
        }))
    }

    fn open_audio(
        &self,
        request: &PlaybackRequest,
        start: Duration,
    ) -> Result<Option<Box<dyn AudioTrackDecoder>>> {
        let reader = open_reader(&request.file_path)?;
        let output_type = create_media_type()?;
        unsafe {
            output_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)?;
            output_type.SetGUID(&MF_MT_SUBTYPE, &MFAudioFormat_Float)?;
        }
        if unsafe { reader.SetCurrentMediaType(audio_stream(), None, &output_type) }.is_err() {
            return Ok(None);
        }
        let current_type = unsafe { reader.GetCurrentMediaType(audio_stream()) }?;
        let sample_rate = unsafe { current_type.GetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND) }?;
        let channel_count = unsafe { current_type.GetUINT32(&MF_MT_AUDIO_NUM_CHANNELS) }? as u16;
        seek_reader(&reader, start)?;
        Ok(Some(Box::new(ReaderAudioDecoder {
            reader,
            sample_rate,
            channel_count,
        })))
    }
}

impl VideoTrackDecoder for ReaderVideoDecoder {
    fn next_frame(&mut self) -> Result<Option<TimedVideoFrame>> {
        let decoded = self.reader.read_next()?;
        Ok(decoded.map(|decoded| TimedVideoFrame {
            timestamp: decoded.timestamp,
            frame: downscale_rgba_to_width(decoded.frame, self.maximum_width),
        }))
    }
}

impl AudioTrackDecoder for ReaderAudioDecoder {
    fn next_chunk(&mut self) -> Result<Option<AudioChunk>> {
        loop {
            let mut stream_flags = 0u32;
            let mut sample: Option<IMFSample> = None;
            unsafe {
                self.reader.ReadSample(
                    audio_stream(),
                    0,
                    None,
                    Some(&mut stream_flags),
                    None,
                    Some(&mut sample),
                )
            }?;
            if stream_flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                return Ok(None);
            }
            let Some(audio_sample) = sample else {
                continue;
            };
            let buffer = unsafe { audio_sample.ConvertToContiguousBuffer() }?;
            let mut source: *mut u8 = std::ptr::null_mut();
            let mut current_length = 0u32;
            unsafe { buffer.Lock(&mut source, None, Some(&mut current_length)) }?;
            let bytes = unsafe { std::slice::from_raw_parts(source, current_length as usize) };
            let samples: Vec<f32> = bytes
                .chunks_exact(4)
                .map(|value| f32::from_le_bytes([value[0], value[1], value[2], value[3]]))
                .collect();
            unsafe { buffer.Unlock() }?;
            return Ok(Some(AudioChunk {
                sample_rate: self.sample_rate,
                channel_count: self.channel_count,
                interleaved_samples: samples,
            }));
        }
    }
}
