use std::ptr::NonNull;
use std::sync::Arc;

use objc2_core_audio_types::{
    kAudioFormatFlagIsFloat, kAudioFormatFlagIsNonInterleaved, AudioBuffer, AudioBufferList,
};
use objc2_core_foundation::CFRetained;
use objc2_core_media::{
    CMAudioFormatDescriptionGetStreamBasicDescription, CMBlockBuffer, CMSampleBuffer,
};
use objc2_core_video::{
    kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange, CVPixelBuffer,
    CVPixelBufferGetBaseAddressOfPlane, CVPixelBufferGetBytesPerRowOfPlane, CVPixelBufferGetHeight,
    CVPixelBufferGetPixelFormatType, CVPixelBufferGetWidth, CVPixelBufferLockBaseAddress,
    CVPixelBufferLockFlags, CVPixelBufferUnlockBaseAddress,
};

use crate::core::models::{AudioChunk, PixelLayout, VideoFrame};

const AUDIO_BUFFER_LIST_ALIGNMENT_FLAG: u32 = 1 << 0;

pub fn convert_screen_sample(sample_buffer: &CMSampleBuffer) -> Option<VideoFrame> {
    let image_buffer = unsafe { sample_buffer.image_buffer() }?;
    let pixel_buffer: &CVPixelBuffer = &image_buffer;
    if CVPixelBufferGetPixelFormatType(pixel_buffer)
        != kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange
    {
        return None;
    }
    unsafe { CVPixelBufferLockBaseAddress(pixel_buffer, CVPixelBufferLockFlags::ReadOnly) };
    let frame = copy_nv12_frame(pixel_buffer);
    unsafe { CVPixelBufferUnlockBaseAddress(pixel_buffer, CVPixelBufferLockFlags::ReadOnly) };
    frame
}

fn copy_nv12_frame(pixel_buffer: &CVPixelBuffer) -> Option<VideoFrame> {
    let width = CVPixelBufferGetWidth(pixel_buffer);
    let height = CVPixelBufferGetHeight(pixel_buffer);
    let even_width = width - (width % 2);
    let even_height = height - (height % 2);
    let mut data = Vec::with_capacity(
        PixelLayout::Nv12.expected_byte_length(even_width as u32, even_height as u32),
    );
    append_plane_rows(pixel_buffer, 0, even_width, even_height, &mut data)?;
    append_plane_rows(pixel_buffer, 1, even_width, even_height / 2, &mut data)?;
    Some(VideoFrame {
        width: even_width as u32,
        height: even_height as u32,
        pixel_layout: PixelLayout::Nv12,
        data: Arc::new(data),
    })
}

fn append_plane_rows(
    pixel_buffer: &CVPixelBuffer,
    plane_index: usize,
    row_length: usize,
    row_count: usize,
    destination: &mut Vec<u8>,
) -> Option<usize> {
    let base_address = CVPixelBufferGetBaseAddressOfPlane(pixel_buffer, plane_index) as *const u8;
    let stride = CVPixelBufferGetBytesPerRowOfPlane(pixel_buffer, plane_index);
    if base_address.is_null() || stride < row_length {
        return None;
    }
    for row_index in 0..row_count {
        let row =
            unsafe { std::slice::from_raw_parts(base_address.add(row_index * stride), row_length) };
        destination.extend_from_slice(row);
    }
    Some(row_count)
}

pub fn convert_audio_sample(sample_buffer: &CMSampleBuffer) -> Option<AudioChunk> {
    let format_description = unsafe { sample_buffer.format_description() }?;
    let stream_description =
        unsafe { CMAudioFormatDescriptionGetStreamBasicDescription(&format_description).as_ref() }?;
    if stream_description.mFormatFlags & kAudioFormatFlagIsFloat == 0 {
        return None;
    }
    let is_non_interleaved =
        stream_description.mFormatFlags & kAudioFormatFlagIsNonInterleaved != 0;
    let channel_count = stream_description.mChannelsPerFrame.max(1) as u16;
    let samples = read_float_samples(sample_buffer, is_non_interleaved, channel_count as usize)?;
    Some(AudioChunk {
        sample_rate: stream_description.mSampleRate as u32,
        channel_count,
        interleaved_samples: samples,
    })
}

fn read_float_samples(
    sample_buffer: &CMSampleBuffer,
    is_non_interleaved: bool,
    channel_count: usize,
) -> Option<Vec<f32>> {
    let mut required_size: usize = 0;
    unsafe {
        sample_buffer.audio_buffer_list_with_retained_block_buffer(
            &mut required_size,
            std::ptr::null_mut(),
            0,
            None,
            None,
            AUDIO_BUFFER_LIST_ALIGNMENT_FLAG,
            std::ptr::null_mut(),
        )
    };
    if required_size == 0 {
        return None;
    }
    let mut storage = vec![0u64; required_size.div_ceil(8)];
    let buffer_list = storage.as_mut_ptr() as *mut AudioBufferList;
    let mut block_buffer: *mut CMBlockBuffer = std::ptr::null_mut();
    let status = unsafe {
        sample_buffer.audio_buffer_list_with_retained_block_buffer(
            std::ptr::null_mut(),
            buffer_list,
            required_size,
            None,
            None,
            AUDIO_BUFFER_LIST_ALIGNMENT_FLAG,
            &mut block_buffer,
        )
    };
    let _block_guard =
        NonNull::new(block_buffer).map(|pointer| unsafe { CFRetained::from_raw(pointer) });
    if status != 0 {
        return None;
    }
    let buffers = unsafe { audio_buffers(buffer_list) };
    Some(interleave(buffers, is_non_interleaved, channel_count))
}

unsafe fn audio_buffers<'a>(buffer_list: *const AudioBufferList) -> Vec<&'a [f32]> {
    let buffer_count = (*buffer_list).mNumberBuffers as usize;
    let first_buffer = std::ptr::addr_of!((*buffer_list).mBuffers) as *const AudioBuffer;
    (0..buffer_count)
        .map(|index| &*first_buffer.add(index))
        .filter(|buffer| !buffer.mData.is_null())
        .map(|buffer| {
            std::slice::from_raw_parts(
                buffer.mData as *const f32,
                buffer.mDataByteSize as usize / 4,
            )
        })
        .collect()
}

pub fn interleave(
    channel_buffers: Vec<&[f32]>,
    is_non_interleaved: bool,
    channel_count: usize,
) -> Vec<f32> {
    if !is_non_interleaved || channel_buffers.len() < 2 {
        return channel_buffers.concat();
    }
    let frame_count = channel_buffers
        .iter()
        .map(|buffer| buffer.len())
        .min()
        .unwrap_or(0);
    (0..frame_count)
        .flat_map(|frame_index| {
            channel_buffers
                .iter()
                .take(channel_count)
                .map(move |buffer| buffer[frame_index])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interleave_combines_planar_channels() {
        let left = [1.0f32, 2.0, 3.0];
        let right = [-1.0f32, -2.0, -3.0];
        assert_eq!(
            interleave(vec![&left, &right], true, 2),
            vec![1.0, -1.0, 2.0, -2.0, 3.0, -3.0]
        );
    }

    #[test]
    fn test_interleave_passes_through_interleaved_audio() {
        let samples = [0.5f32, -0.5];
        assert_eq!(interleave(vec![&samples], false, 2), vec![0.5, -0.5]);
    }

    #[test]
    fn test_interleave_truncates_to_shortest_channel() {
        let left = [1.0f32, 2.0];
        let right = [9.0f32];
        assert_eq!(interleave(vec![&left, &right], true, 2), vec![1.0, 9.0]);
    }
}
