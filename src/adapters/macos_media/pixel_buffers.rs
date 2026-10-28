use std::ptr::NonNull;

use anyhow::{anyhow, bail, Result};
use objc2_core_foundation::CFRetained;
use objc2_core_video::{
    kCVPixelFormatType_32BGRA, kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange, kCVReturnSuccess,
    CVPixelBuffer, CVPixelBufferGetBaseAddressOfPlane, CVPixelBufferGetBytesPerRowOfPlane,
    CVPixelBufferLockBaseAddress, CVPixelBufferLockFlags, CVPixelBufferPool,
    CVPixelBufferUnlockBaseAddress,
};

use crate::core::models::{PixelLayout, VideoFrame};

const UNSUPPORTED_LAYOUT_ERROR: &str = "pixel layout is not supported by the native encoder";
const TRUNCATED_FRAME_ERROR: &str = "frame data is truncated";
const RGBA_BYTES_PER_PIXEL: usize = 4;

pub fn core_video_pixel_format(pixel_layout: PixelLayout) -> u32 {
    match pixel_layout {
        PixelLayout::Nv12 => kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange,
        PixelLayout::Rgba => kCVPixelFormatType_32BGRA,
    }
}

pub fn create_pooled_buffer(pool: &CVPixelBufferPool) -> Result<CFRetained<CVPixelBuffer>> {
    let mut raw_buffer: *mut CVPixelBuffer = std::ptr::null_mut();
    let status = unsafe {
        CVPixelBufferPool::create_pixel_buffer(None, pool, NonNull::from(&mut raw_buffer))
    };
    let buffer_pointer = NonNull::new(raw_buffer).filter(|_| status == kCVReturnSuccess);
    let buffer_pointer =
        buffer_pointer.ok_or_else(|| anyhow!("pixel buffer pool exhausted ({})", status))?;
    Ok(unsafe { CFRetained::from_raw(buffer_pointer) })
}

pub fn fill_buffer_from_frame(buffer: &CVPixelBuffer, frame: &VideoFrame) -> Result<usize> {
    let expected_length = frame
        .pixel_layout
        .expected_byte_length(frame.width, frame.height);
    if frame.data.len() < expected_length {
        bail!(TRUNCATED_FRAME_ERROR);
    }
    unsafe { CVPixelBufferLockBaseAddress(buffer, CVPixelBufferLockFlags(0)) };
    let copy_result = match frame.pixel_layout {
        PixelLayout::Nv12 => copy_nv12_planes(buffer, frame),
        PixelLayout::Rgba => copy_rgba_as_bgra(buffer, frame),
    };
    unsafe { CVPixelBufferUnlockBaseAddress(buffer, CVPixelBufferLockFlags(0)) };
    copy_result
}

fn copy_nv12_planes(buffer: &CVPixelBuffer, frame: &VideoFrame) -> Result<usize> {
    let width = frame.width as usize;
    let height = frame.height as usize;
    let luma_length = width * height;
    let luma_rows = copy_plane_rows(buffer, 0, &frame.data[..luma_length], width, height)?;
    let chroma_rows = copy_plane_rows(buffer, 1, &frame.data[luma_length..], width, height / 2)?;
    Ok(luma_rows + chroma_rows)
}

fn copy_plane_rows(
    buffer: &CVPixelBuffer,
    plane_index: usize,
    source: &[u8],
    row_length: usize,
    row_count: usize,
) -> Result<usize> {
    let destination = CVPixelBufferGetBaseAddressOfPlane(buffer, plane_index) as *mut u8;
    let destination_stride = CVPixelBufferGetBytesPerRowOfPlane(buffer, plane_index);
    if destination.is_null() || destination_stride < row_length {
        bail!(UNSUPPORTED_LAYOUT_ERROR);
    }
    for (row_index, source_row) in source.chunks_exact(row_length).take(row_count).enumerate() {
        unsafe {
            std::ptr::copy_nonoverlapping(
                source_row.as_ptr(),
                destination.add(row_index * destination_stride),
                row_length,
            );
        }
    }
    Ok(row_count)
}

fn copy_rgba_as_bgra(buffer: &CVPixelBuffer, frame: &VideoFrame) -> Result<usize> {
    let row_length = frame.width as usize * RGBA_BYTES_PER_PIXEL;
    let row_count = frame.height as usize;
    let swapped: Vec<u8> = frame
        .data
        .chunks_exact(RGBA_BYTES_PER_PIXEL)
        .flat_map(|pixel| [pixel[2], pixel[1], pixel[0], pixel[3]])
        .collect();
    copy_plane_rows(buffer, 0, &swapped, row_length, row_count)
}
