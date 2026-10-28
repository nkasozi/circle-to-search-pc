use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use windows::Win32::Media::MediaFoundation::{
    IMFSinkWriter, MFCreateMemoryBuffer, MFCreateSample, MFCreateSinkWriterFromURL,
    MFMediaType_Video, MFVideoFormat_H264, MFVideoFormat_RGB32, MFVideoInterlace_Progressive,
    MF_MT_AVG_BITRATE, MF_MT_DEFAULT_STRIDE, MF_MT_FRAME_RATE, MF_MT_FRAME_SIZE,
    MF_MT_INTERLACE_MODE, MF_MT_MAJOR_TYPE, MF_MT_PIXEL_ASPECT_RATIO, MF_MT_SUBTYPE,
    MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, MF_SINK_WRITER_DISABLE_THROTTLING,
};

use super::runtime::{
    as_pcwstr, create_attributes, create_media_type, ensure_media_foundation, pack_ratio,
    wide_path, HUNDRED_NANOSECONDS_PER_SECOND,
};
use crate::core::interfaces::ports::VideoEncoderSession;
use crate::core::models::{PixelLayout, VideoEncodingRequest, VideoFrame};

const LOG_TAG: &str = "[MEDIA_FOUNDATION]";
const BITS_PER_PIXEL_FRAME: f64 = 0.12;
const MINIMUM_BITRATE: f64 = 3_000_000.0;
const MAXIMUM_BITRATE: f64 = 40_000_000.0;
const BYTES_PER_PIXEL: u32 = 4;
const SINK_WRITER_STARTED_EVENT: &str = "sink_writer_started";
const SINK_WRITER_FINISHED_EVENT: &str = "sink_writer_finished";
const RGBA_FRAMES_REQUIRED_ERROR: &str = "the Windows encoder expects RGBA frames";
const CREATE_VIDEO_FILE_FAILED_ERROR: &str = "could not create the video file";
const START_ENCODER_FAILED_ERROR: &str = "could not start the video encoder";
const FRAME_SIZE_CHANGED_ERROR: &str = "frame size changed during recording";
const VIDEO_FRAME_REJECTED_ERROR: &str = "a video frame was rejected";
const FINISH_VIDEO_FILE_FAILED_ERROR: &str = "could not finish the video file";

pub struct SinkWriterSession {
    writer: IMFSinkWriter,
    stream_index: u32,
    request: VideoEncodingRequest,
    frames_written: u64,
}

unsafe impl Send for SinkWriterSession {}

pub fn compute_bitrate(request: &VideoEncodingRequest) -> u32 {
    let pixel_rate = request.dimensions.width as f64
        * request.dimensions.height as f64
        * request.frames_per_second as f64;
    (pixel_rate * BITS_PER_PIXEL_FRAME).clamp(MINIMUM_BITRATE, MAXIMUM_BITRATE) as u32
}

pub fn convert_rgba_to_bgra(rgba: &[u8]) -> Vec<u8> {
    rgba.chunks_exact(4)
        .flat_map(|pixel| [pixel[2], pixel[1], pixel[0], pixel[3]])
        .collect()
}

fn configure_streams(writer: &IMFSinkWriter, request: &VideoEncodingRequest) -> Result<u32> {
    let frame_size = pack_ratio(request.dimensions.width, request.dimensions.height);
    let frame_rate = pack_ratio(request.frames_per_second, 1);
    let output_type = create_media_type()?;
    let input_type = create_media_type()?;
    unsafe {
        output_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        output_type.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_H264)?;
        output_type.SetUINT32(&MF_MT_AVG_BITRATE, compute_bitrate(request))?;
        output_type.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
        output_type.SetUINT64(&MF_MT_FRAME_SIZE, frame_size)?;
        output_type.SetUINT64(&MF_MT_FRAME_RATE, frame_rate)?;
        output_type.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, pack_ratio(1, 1))?;
        let stream_index = writer.AddStream(&output_type)?;
        input_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        input_type.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32)?;
        input_type.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
        input_type.SetUINT32(
            &MF_MT_DEFAULT_STRIDE,
            request.dimensions.width * BYTES_PER_PIXEL,
        )?;
        input_type.SetUINT64(&MF_MT_FRAME_SIZE, frame_size)?;
        input_type.SetUINT64(&MF_MT_FRAME_RATE, frame_rate)?;
        input_type.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, pack_ratio(1, 1))?;
        writer.SetInputMediaType(stream_index, &input_type, None)?;
        Ok(stream_index)
    }
}

impl SinkWriterSession {
    pub fn start(request: &VideoEncodingRequest) -> Result<Self> {
        ensure_media_foundation()?;
        if request.pixel_layout != PixelLayout::Rgba {
            bail!(RGBA_FRAMES_REQUIRED_ERROR);
        }
        let _ = std::fs::remove_file(&request.output_file_path);
        let attributes = create_attributes(2)?;
        unsafe {
            attributes.SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, 1)?;
            attributes.SetUINT32(&MF_SINK_WRITER_DISABLE_THROTTLING, 1)?;
        }
        let output_path = wide_path(&request.output_file_path);
        let writer =
            unsafe { MFCreateSinkWriterFromURL(as_pcwstr(&output_path), None, &attributes) }
                .context(CREATE_VIDEO_FILE_FAILED_ERROR)?;
        let stream_index = configure_streams(&writer, request)?;
        unsafe { writer.BeginWriting() }.context(START_ENCODER_FAILED_ERROR)?;
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": SINK_WRITER_STARTED_EVENT, "width": request.dimensions.width, "height": request.dimensions.height})
        );
        Ok(Self {
            writer,
            stream_index,
            request: request.clone(),
            frames_written: 0,
        })
    }
}

impl VideoEncoderSession for SinkWriterSession {
    fn write_frame(&mut self, frame: &VideoFrame) -> Result<u64> {
        let matches_request = frame.width == self.request.dimensions.width
            && frame.height == self.request.dimensions.height;
        if !matches_request {
            bail!(FRAME_SIZE_CHANGED_ERROR);
        }
        let bgra = convert_rgba_to_bgra(&frame.data);
        let frames_per_second = self.request.frames_per_second.max(1) as i64;
        unsafe {
            let buffer = MFCreateMemoryBuffer(bgra.len() as u32)?;
            let mut destination: *mut u8 = std::ptr::null_mut();
            buffer.Lock(&mut destination, None, None)?;
            std::ptr::copy_nonoverlapping(bgra.as_ptr(), destination, bgra.len());
            buffer.Unlock()?;
            buffer.SetCurrentLength(bgra.len() as u32)?;
            let sample = MFCreateSample()?;
            sample.AddBuffer(&buffer)?;
            sample.SetSampleTime(
                self.frames_written as i64 * HUNDRED_NANOSECONDS_PER_SECOND / frames_per_second,
            )?;
            sample.SetSampleDuration(HUNDRED_NANOSECONDS_PER_SECOND / frames_per_second)?;
            self.writer
                .WriteSample(self.stream_index, &sample)
                .context(VIDEO_FRAME_REJECTED_ERROR)?;
        }
        self.frames_written += 1;
        Ok(self.frames_written)
    }

    fn finish(self: Box<Self>) -> Result<PathBuf> {
        ensure_media_foundation()?;
        unsafe { self.writer.Finalize() }.context(FINISH_VIDEO_FILE_FAILED_ERROR)?;
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": SINK_WRITER_FINISHED_EVENT, "frames_written": self.frames_written})
        );
        Ok(self.request.output_file_path.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgba_to_bgra_swaps_red_and_blue() {
        assert_eq!(
            convert_rgba_to_bgra(&[1, 2, 3, 4, 5, 6, 7, 8]),
            vec![3, 2, 1, 4, 7, 6, 5, 8]
        );
    }
}
