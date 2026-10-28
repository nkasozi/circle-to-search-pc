use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use windows::Win32::Media::MediaFoundation::{
    IMFSample, IMFSourceReader, MFCreateSourceReaderFromURL, MFMediaType_Video,
    MFVideoFormat_RGB32, MF_MT_FRAME_SIZE, MF_MT_MAJOR_TYPE, MF_MT_SUBTYPE,
    MF_SOURCE_READERF_ENDOFSTREAM, MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING,
    MF_SOURCE_READER_FIRST_VIDEO_STREAM,
};

use super::runtime::{
    as_pcwstr, create_attributes, create_media_type, ensure_media_foundation, wide_path,
    HUNDRED_NANOSECONDS_PER_SECOND,
};
use crate::core::models::{PixelLayout, VideoFrame};

const OPEN_VIDEO_FAILED_ERROR: &str = "could not open the video";
const SEEK_VIDEO_FAILED_ERROR: &str = "could not seek the video";
const BYTES_PER_PIXEL: usize = 4;
const OPAQUE_ALPHA: u8 = 255;
const NANOSECONDS_PER_HUNDRED_NANOSECONDS: i64 = 100;
const NANOSECONDS_PER_SECOND: u64 = 1_000_000_000;
const FRAME_HEIGHT_MASK: u64 = 0xFFFF_FFFF;

pub struct DecodedVideoFrame {
    pub frame: VideoFrame,
    pub timestamp: Duration,
}

pub struct VideoFrameReader {
    reader: IMFSourceReader,
    width: u32,
    height: u32,
}

unsafe impl Send for VideoFrameReader {}

fn video_stream() -> u32 {
    MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32
}

impl VideoFrameReader {
    pub fn open(path: &Path) -> Result<Self> {
        ensure_media_foundation()?;
        let attributes = create_attributes(1)?;
        unsafe { attributes.SetUINT32(&MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING, 1) }?;
        let wide = wide_path(path);
        let reader = unsafe { MFCreateSourceReaderFromURL(as_pcwstr(&wide), &attributes) }
            .context(OPEN_VIDEO_FAILED_ERROR)?;
        let output_type = create_media_type()?;
        unsafe {
            output_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
            output_type.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32)?;
            reader.SetCurrentMediaType(video_stream(), None, &output_type)?;
        }
        let current_type = unsafe { reader.GetCurrentMediaType(video_stream()) }?;
        let packed_size = unsafe { current_type.GetUINT64(&MF_MT_FRAME_SIZE) }?;
        Ok(Self {
            reader,
            width: (packed_size >> 32) as u32,
            height: (packed_size & FRAME_HEIGHT_MASK) as u32,
        })
    }

    pub fn read_next(&mut self) -> Result<Option<DecodedVideoFrame>> {
        loop {
            let mut stream_flags = 0u32;
            let mut timestamp = 0i64;
            let mut sample: Option<IMFSample> = None;
            unsafe {
                self.reader.ReadSample(
                    video_stream(),
                    0,
                    None,
                    Some(&mut stream_flags),
                    Some(&mut timestamp),
                    Some(&mut sample),
                )
            }?;
            if stream_flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                return Ok(None);
            }
            let Some(video_sample) = sample else {
                continue;
            };
            let frame = self.convert_sample(&video_sample)?;
            let timestamp = Duration::from_nanos(
                (timestamp.max(0) * NANOSECONDS_PER_HUNDRED_NANOSECONDS) as u64,
            );
            return Ok(Some(DecodedVideoFrame { frame, timestamp }));
        }
    }

    pub fn seek(&mut self, position: Duration) -> Result<Duration> {
        let hundred_nanoseconds =
            (position.as_nanos() / NANOSECONDS_PER_HUNDRED_NANOSECONDS as u128) as i64;
        let variant = windows::core::PROPVARIANT::from(hundred_nanoseconds);
        unsafe {
            self.reader
                .SetCurrentPosition(&windows::core::GUID::zeroed(), &variant)
        }
        .context(SEEK_VIDEO_FAILED_ERROR)?;
        Ok(position)
    }

    fn convert_sample(&self, sample: &IMFSample) -> Result<VideoFrame> {
        let buffer = unsafe { sample.ConvertToContiguousBuffer() }?;
        let mut source: *mut u8 = std::ptr::null_mut();
        let mut current_length = 0u32;
        unsafe { buffer.Lock(&mut source, None, Some(&mut current_length)) }?;
        let bytes = unsafe { std::slice::from_raw_parts(source, current_length as usize) };
        let rgba: Vec<u8> = bytes
            .chunks_exact(BYTES_PER_PIXEL)
            .flat_map(|pixel| [pixel[2], pixel[1], pixel[0], OPAQUE_ALPHA])
            .collect();
        unsafe { buffer.Unlock() }?;
        Ok(VideoFrame {
            width: self.width,
            height: self.height,
            pixel_layout: PixelLayout::Rgba,
            data: Arc::new(rgba),
        })
    }
}

pub fn hundred_nanoseconds_to_duration(value: i64) -> Duration {
    Duration::from_nanos(
        (value.max(0) as u64) * (NANOSECONDS_PER_SECOND / HUNDRED_NANOSECONDS_PER_SECOND as u64),
    )
}
