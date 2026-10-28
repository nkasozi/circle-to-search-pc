use std::sync::Arc;

use crate::core::models::{PixelLayout, VideoFrame};

pub fn downscale_rgba_to_width(frame: VideoFrame, maximum_width: u32) -> VideoFrame {
    if frame.pixel_layout != PixelLayout::Rgba || frame.width <= maximum_width || maximum_width == 0
    {
        return frame;
    }
    let target_width = maximum_width;
    let target_height =
        ((frame.height as u64 * target_width as u64) / frame.width as u64).max(1) as u32;
    let source_width = frame.width as usize;
    let mut output = Vec::with_capacity(target_width as usize * target_height as usize * 4);
    for target_row in 0..target_height as usize {
        let source_row = target_row * frame.height as usize / target_height as usize;
        for target_column in 0..target_width as usize {
            let source_column = target_column * source_width / target_width as usize;
            let offset = (source_row * source_width + source_column) * 4;
            output.extend_from_slice(&frame.data[offset..offset + 4]);
        }
    }
    VideoFrame {
        width: target_width,
        height: target_height,
        pixel_layout: PixelLayout::Rgba,
        data: Arc::new(output),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(width: u32, height: u32) -> VideoFrame {
        let data: Vec<u8> = (0..width * height)
            .flat_map(|index| [index as u8, 0, 0, 255])
            .collect();
        VideoFrame {
            width,
            height,
            pixel_layout: PixelLayout::Rgba,
            data: Arc::new(data),
        }
    }

    #[test]
    fn test_small_frames_are_unchanged() {
        let original = frame(4, 2);
        assert_eq!(downscale_rgba_to_width(original.clone(), 8), original);
    }

    #[test]
    fn test_wide_frames_are_halved_with_nearest_sampling() {
        let scaled = downscale_rgba_to_width(frame(4, 2), 2);
        assert_eq!((scaled.width, scaled.height), (2, 1));
        assert_eq!(scaled.data[0], 0);
        assert_eq!(scaled.data[4], 2);
    }
}
