use std::sync::Arc;

use crate::core::models::{PixelLayout, VideoFrame};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(target_os = "macos", allow(dead_code))]
pub enum SourceChannelOrder {
    Bgra,
    Rgba,
    Rgb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CameraImageGeometry {
    pub width: usize,
    pub height: usize,
    pub bytes_per_row: usize,
    pub channel_order: SourceChannelOrder,
}

impl SourceChannelOrder {
    fn bytes_per_pixel(self) -> usize {
        match self {
            SourceChannelOrder::Bgra | SourceChannelOrder::Rgba => 4,
            SourceChannelOrder::Rgb => 3,
        }
    }

    fn to_rgba(self, pixel: &[u8]) -> [u8; 4] {
        match self {
            SourceChannelOrder::Bgra => [pixel[2], pixel[1], pixel[0], 255],
            SourceChannelOrder::Rgba => [pixel[0], pixel[1], pixel[2], 255],
            SourceChannelOrder::Rgb => [pixel[0], pixel[1], pixel[2], 255],
        }
    }
}

pub fn build_mirrored_square_preview(
    source: &[u8],
    geometry: CameraImageGeometry,
    target_edge: usize,
) -> Option<VideoFrame> {
    let square_edge = geometry.width.min(geometry.height);
    let bytes_per_pixel = geometry.channel_order.bytes_per_pixel();
    let minimum_length = geometry.bytes_per_row * geometry.height.saturating_sub(1)
        + geometry.width * bytes_per_pixel;
    if square_edge == 0 || target_edge == 0 || source.len() < minimum_length {
        return None;
    }
    let horizontal_offset = (geometry.width - square_edge) / 2;
    let vertical_offset = (geometry.height - square_edge) / 2;
    let mut output = Vec::with_capacity(target_edge * target_edge * 4);
    for output_row in 0..target_edge {
        let source_row = vertical_offset + output_row * square_edge / target_edge;
        for output_column in 0..target_edge {
            let mirrored_column = target_edge - 1 - output_column;
            let source_column = horizontal_offset + mirrored_column * square_edge / target_edge;
            let pixel_start = source_row * geometry.bytes_per_row + source_column * bytes_per_pixel;
            output.extend_from_slice(
                &geometry
                    .channel_order
                    .to_rgba(&source[pixel_start..pixel_start + bytes_per_pixel]),
            );
        }
    }
    Some(VideoFrame {
        width: target_edge as u32,
        height: target_edge as u32,
        pixel_layout: PixelLayout::Rgba,
        data: Arc::new(output),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_is_mirrored_and_converted_to_rgba() {
        let source = vec![10, 20, 30, 255, 40, 50, 60, 255];
        let geometry = CameraImageGeometry {
            width: 2,
            height: 1,
            bytes_per_row: 8,
            channel_order: SourceChannelOrder::Bgra,
        };
        let frame = build_mirrored_square_preview(&source, geometry, 1).expect("frame");
        assert_eq!(frame.data.as_slice(), &[30, 20, 10, 255]);
    }

    #[test]
    fn test_preview_center_crops_wide_images() {
        let mut source = Vec::new();
        for column in 0..4u8 {
            source.extend_from_slice(&[column, column, column]);
        }
        let geometry = CameraImageGeometry {
            width: 4,
            height: 2,
            bytes_per_row: 12,
            channel_order: SourceChannelOrder::Rgb,
        };
        let source_two_rows = [source.clone(), source].concat();
        let frame = build_mirrored_square_preview(&source_two_rows, geometry, 2).expect("frame");
        assert_eq!(frame.width, 2);
        assert_eq!(&frame.data[0..4], &[2, 2, 2, 255]);
        assert_eq!(&frame.data[4..8], &[1, 1, 1, 255]);
    }

    #[test]
    fn test_preview_rejects_truncated_input() {
        let geometry = CameraImageGeometry {
            width: 4,
            height: 4,
            bytes_per_row: 16,
            channel_order: SourceChannelOrder::Rgba,
        };
        assert!(build_mirrored_square_preview(&[0u8; 10], geometry, 2).is_none());
    }
}
