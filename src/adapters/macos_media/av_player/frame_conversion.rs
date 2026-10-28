use objc2_core_video::{
    CVPixelBuffer, CVPixelBufferGetBaseAddress, CVPixelBufferGetBytesPerRow,
    CVPixelBufferGetHeight, CVPixelBufferGetWidth, CVPixelBufferLockBaseAddress,
    CVPixelBufferLockFlags, CVPixelBufferUnlockBaseAddress,
};

use crate::core::models::{PixelLayout, VideoFrame};

const BYTES_PER_PIXEL: usize = 4;

pub fn convert_bgra_buffer(pixel_buffer: &CVPixelBuffer) -> Option<VideoFrame> {
    unsafe { CVPixelBufferLockBaseAddress(pixel_buffer, CVPixelBufferLockFlags::ReadOnly) };
    let base_address = CVPixelBufferGetBaseAddress(pixel_buffer) as *const u8;
    let width = CVPixelBufferGetWidth(pixel_buffer);
    let height = CVPixelBufferGetHeight(pixel_buffer);
    let stride = CVPixelBufferGetBytesPerRow(pixel_buffer);
    let frame = (!base_address.is_null()).then(|| {
        let row_length = width * BYTES_PER_PIXEL;
        let mut rgba = vec![0u8; row_length * height];
        for (row, destination_row) in rgba.chunks_exact_mut(row_length).enumerate() {
            let source_row =
                unsafe { std::slice::from_raw_parts(base_address.add(row * stride), row_length) };
            destination_row.copy_from_slice(source_row);
            swap_blue_and_red(destination_row);
        }
        VideoFrame {
            width: width as u32,
            height: height as u32,
            pixel_layout: PixelLayout::Rgba,
            data: std::sync::Arc::new(rgba),
        }
    });
    unsafe { CVPixelBufferUnlockBaseAddress(pixel_buffer, CVPixelBufferLockFlags::ReadOnly) };
    frame
}

pub fn swap_blue_and_red(pixels: &mut [u8]) -> usize {
    pixels
        .chunks_exact_mut(BYTES_PER_PIXEL)
        .for_each(|pixel| pixel.swap(0, 2));
    pixels.len() / BYTES_PER_PIXEL
}
