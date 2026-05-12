use anyhow::Result;
use image::{imageops, RgbaImage};

use crate::core::interfaces::adapters::FrameCompositor;
use crate::core::models::recording::{
    AnnotationPoint, DrawAnnotation, RecordingFrame, WebcamPipPosition,
};

const LOG_TAG: &str = "[FRAME_COMPOSITOR]";
const PIP_PADDING_PIXELS: u32 = 16;
const PIP_BORDER_THICKNESS: u32 = 2;
const PIP_BORDER_COLOR_RGBA: [u8; 4] = [40, 40, 40, 200];
const RGBA_BYTES_PER_PIXEL: u32 = 4;

pub struct RgbaFrameCompositor;

impl RgbaFrameCompositor {
    pub fn initialize() -> Self {
        log::debug!("{} initializing rgba frame compositor", LOG_TAG);
        Self
    }

    fn resize_webcam_frame(
        webcam_frame: &RecordingFrame,
        target_width: u32,
        target_height: u32,
    ) -> Result<RgbaImage> {
        let webcam_image = RgbaImage::from_raw(
            webcam_frame.width_pixels,
            webcam_frame.height_pixels,
            webcam_frame.raw_rgba_data.clone(),
        )
        .ok_or_else(|| anyhow::anyhow!("failed to create image from webcam frame data"))?;

        let resized = imageops::resize(
            &webcam_image,
            target_width,
            target_height,
            imageops::FilterType::Triangle,
        );

        Ok(resized)
    }

    fn calculate_pip_dimensions(
        screen_width: u32,
        _screen_height: u32,
        webcam_width: u32,
        webcam_height: u32,
        pip_size_percentage: f32,
    ) -> (u32, u32) {
        let max_pip_width = (screen_width as f32 * pip_size_percentage) as u32;
        let aspect_ratio = webcam_width as f32 / webcam_height.max(1) as f32;
        let pip_width = max_pip_width;
        let pip_height = (pip_width as f32 / aspect_ratio) as u32;

        (pip_width.max(1), pip_height.max(1))
    }

    fn calculate_pip_position(
        screen_width: u32,
        screen_height: u32,
        pip_width: u32,
        pip_height: u32,
        position: &WebcamPipPosition,
    ) -> (u32, u32) {
        match position {
            WebcamPipPosition::BottomLeft => (
                PIP_PADDING_PIXELS,
                screen_height.saturating_sub(pip_height + PIP_PADDING_PIXELS),
            ),
            WebcamPipPosition::BottomRight => (
                screen_width.saturating_sub(pip_width + PIP_PADDING_PIXELS),
                screen_height.saturating_sub(pip_height + PIP_PADDING_PIXELS),
            ),
            WebcamPipPosition::TopLeft => (PIP_PADDING_PIXELS, PIP_PADDING_PIXELS),
            WebcamPipPosition::TopRight => (
                screen_width.saturating_sub(pip_width + PIP_PADDING_PIXELS),
                PIP_PADDING_PIXELS,
            ),
        }
    }

    fn overlay_image_onto_buffer(
        base_buffer: &mut Vec<u8>,
        base_width: u32,
        overlay: &RgbaImage,
        offset_x: u32,
        offset_y: u32,
    ) -> Result<bool> {
        let overlay_width = overlay.width();
        let overlay_height = overlay.height();

        for row in 0..overlay_height {
            for col in 0..overlay_width {
                let target_x = offset_x + col;
                let target_y = offset_y + row;

                if target_x >= base_width {
                    continue;
                }

                let base_index =
                    ((target_y * base_width + target_x) * RGBA_BYTES_PER_PIXEL) as usize;
                let pixel = overlay.get_pixel(col, row);

                if base_index + 3 < base_buffer.len() {
                    let source_alpha = pixel[3] as f32 / 255.0;
                    let inverse_alpha = 1.0 - source_alpha;

                    base_buffer[base_index] = (pixel[0] as f32 * source_alpha
                        + base_buffer[base_index] as f32 * inverse_alpha)
                        as u8;
                    base_buffer[base_index + 1] = (pixel[1] as f32 * source_alpha
                        + base_buffer[base_index + 1] as f32 * inverse_alpha)
                        as u8;
                    base_buffer[base_index + 2] = (pixel[2] as f32 * source_alpha
                        + base_buffer[base_index + 2] as f32 * inverse_alpha)
                        as u8;
                    base_buffer[base_index + 3] = 255;
                }
            }
        }

        Ok(true)
    }

    fn draw_border_around_region(
        buffer: &mut Vec<u8>,
        buffer_width: u32,
        region_x: u32,
        region_y: u32,
        region_width: u32,
        region_height: u32,
    ) -> Result<bool> {
        for thickness_offset in 0..PIP_BORDER_THICKNESS {
            let border_x = region_x.saturating_sub(thickness_offset + 1);
            let border_y = region_y.saturating_sub(thickness_offset + 1);
            let border_end_x = region_x + region_width + thickness_offset;
            let border_end_y = region_y + region_height + thickness_offset;

            for x_coord in border_x..=border_end_x {
                Self::set_pixel_in_buffer(buffer, buffer_width, x_coord, border_y);
                Self::set_pixel_in_buffer(buffer, buffer_width, x_coord, border_end_y);
            }

            for y_coord in border_y..=border_end_y {
                Self::set_pixel_in_buffer(buffer, buffer_width, border_x, y_coord);
                Self::set_pixel_in_buffer(buffer, buffer_width, border_end_x, y_coord);
            }
        }

        Ok(true)
    }

    fn set_pixel_in_buffer(buffer: &mut [u8], buffer_width: u32, x: u32, y: u32) {
        let index = ((y * buffer_width + x) * RGBA_BYTES_PER_PIXEL) as usize;
        if index + 3 < buffer.len() {
            buffer[index] = PIP_BORDER_COLOR_RGBA[0];
            buffer[index + 1] = PIP_BORDER_COLOR_RGBA[1];
            buffer[index + 2] = PIP_BORDER_COLOR_RGBA[2];
            buffer[index + 3] = PIP_BORDER_COLOR_RGBA[3];
        }
    }

    fn draw_line_on_buffer(
        buffer: &mut Vec<u8>,
        buffer_width: u32,
        start: &AnnotationPoint,
        end: &AnnotationPoint,
        color_rgba: [u8; 4],
        thickness: f32,
    ) {
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let distance = (dx * dx + dy * dy).sqrt();
        let steps = distance.ceil() as usize;

        if steps == 0 {
            return;
        }

        let half_thickness = (thickness / 2.0).ceil() as i32;

        for step in 0..=steps {
            let interpolation = step as f32 / steps as f32;
            let center_x = start.x + dx * interpolation;
            let center_y = start.y + dy * interpolation;

            for offset_y in -half_thickness..=half_thickness {
                for offset_x in -half_thickness..=half_thickness {
                    let pixel_x = (center_x as i32 + offset_x).max(0) as u32;
                    let pixel_y = (center_y as i32 + offset_y).max(0) as u32;

                    let index =
                        ((pixel_y * buffer_width + pixel_x) * RGBA_BYTES_PER_PIXEL) as usize;

                    if index + 3 < buffer.len() {
                        let alpha = color_rgba[3] as f32 / 255.0;
                        let inv_alpha = 1.0 - alpha;
                        buffer[index] =
                            (color_rgba[0] as f32 * alpha + buffer[index] as f32 * inv_alpha) as u8;
                        buffer[index + 1] = (color_rgba[1] as f32 * alpha
                            + buffer[index + 1] as f32 * inv_alpha)
                            as u8;
                        buffer[index + 2] = (color_rgba[2] as f32 * alpha
                            + buffer[index + 2] as f32 * inv_alpha)
                            as u8;
                        buffer[index + 3] = 255;
                    }
                }
            }
        }
    }

    fn draw_rectangle_outline_on_buffer(
        buffer: &mut Vec<u8>,
        buffer_width: u32,
        top_left: &AnnotationPoint,
        bottom_right: &AnnotationPoint,
        color_rgba: [u8; 4],
        thickness: f32,
    ) {
        let top_right = AnnotationPoint::at(bottom_right.x, top_left.y);
        let bottom_left = AnnotationPoint::at(top_left.x, bottom_right.y);

        Self::draw_line_on_buffer(
            buffer,
            buffer_width,
            top_left,
            &top_right,
            color_rgba,
            thickness,
        );
        Self::draw_line_on_buffer(
            buffer,
            buffer_width,
            &top_right,
            bottom_right,
            color_rgba,
            thickness,
        );
        Self::draw_line_on_buffer(
            buffer,
            buffer_width,
            bottom_right,
            &bottom_left,
            color_rgba,
            thickness,
        );
        Self::draw_line_on_buffer(
            buffer,
            buffer_width,
            &bottom_left,
            top_left,
            color_rgba,
            thickness,
        );
    }

    fn draw_arrowhead_on_buffer(
        buffer: &mut Vec<u8>,
        buffer_width: u32,
        start: &AnnotationPoint,
        end: &AnnotationPoint,
        color_rgba: [u8; 4],
        thickness: f32,
    ) {
        let arrow_length = 15.0_f32;
        let arrow_angle = 0.5_f32;

        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let angle = dy.atan2(dx);

        let left_tip = AnnotationPoint::at(
            end.x - arrow_length * (angle - arrow_angle).cos(),
            end.y - arrow_length * (angle - arrow_angle).sin(),
        );
        let right_tip = AnnotationPoint::at(
            end.x - arrow_length * (angle + arrow_angle).cos(),
            end.y - arrow_length * (angle + arrow_angle).sin(),
        );

        Self::draw_line_on_buffer(buffer, buffer_width, end, &left_tip, color_rgba, thickness);
        Self::draw_line_on_buffer(buffer, buffer_width, end, &right_tip, color_rgba, thickness);
    }
}

impl FrameCompositor for RgbaFrameCompositor {
    fn composite_webcam_pip(
        &self,
        screen_frame: &RecordingFrame,
        webcam_frame: &RecordingFrame,
        pip_position: &WebcamPipPosition,
        pip_size_percentage: f32,
    ) -> Result<RecordingFrame> {
        let (pip_width, pip_height) = Self::calculate_pip_dimensions(
            screen_frame.width_pixels,
            screen_frame.height_pixels,
            webcam_frame.width_pixels,
            webcam_frame.height_pixels,
            pip_size_percentage,
        );

        log::debug!(
            "{} compositing webcam pip: {}x{} at {:?}",
            LOG_TAG,
            pip_width,
            pip_height,
            pip_position,
        );

        let resized_webcam = Self::resize_webcam_frame(webcam_frame, pip_width, pip_height)?;

        let (offset_x, offset_y) = Self::calculate_pip_position(
            screen_frame.width_pixels,
            screen_frame.height_pixels,
            pip_width,
            pip_height,
            pip_position,
        );

        let mut composited_data = screen_frame.raw_rgba_data.clone();

        Self::overlay_image_onto_buffer(
            &mut composited_data,
            screen_frame.width_pixels,
            &resized_webcam,
            offset_x,
            offset_y,
        )?;

        Self::draw_border_around_region(
            &mut composited_data,
            screen_frame.width_pixels,
            offset_x,
            offset_y,
            pip_width,
            pip_height,
        )?;

        Ok(RecordingFrame::build(
            screen_frame.timestamp,
            screen_frame.width_pixels,
            screen_frame.height_pixels,
            composited_data,
        ))
    }

    fn burn_annotations(
        &self,
        frame: &RecordingFrame,
        annotations: &[DrawAnnotation],
    ) -> Result<RecordingFrame> {
        if annotations.is_empty() {
            return Ok(frame.clone());
        }

        log::debug!(
            "{} burning {} annotations onto frame",
            LOG_TAG,
            annotations.len(),
        );

        let mut annotated_data = frame.raw_rgba_data.clone();

        for annotation in annotations {
            match annotation {
                DrawAnnotation::PenStroke {
                    points,
                    color,
                    thickness,
                } => {
                    let color_bytes = color.to_rgba_bytes();
                    for window in points.windows(2) {
                        Self::draw_line_on_buffer(
                            &mut annotated_data,
                            frame.width_pixels,
                            &window[0],
                            &window[1],
                            color_bytes,
                            *thickness,
                        );
                    }
                }
                DrawAnnotation::Arrow {
                    start,
                    end,
                    color,
                    thickness,
                } => {
                    let color_bytes = color.to_rgba_bytes();
                    Self::draw_line_on_buffer(
                        &mut annotated_data,
                        frame.width_pixels,
                        start,
                        end,
                        color_bytes,
                        *thickness,
                    );
                    Self::draw_arrowhead_on_buffer(
                        &mut annotated_data,
                        frame.width_pixels,
                        start,
                        end,
                        color_bytes,
                        *thickness,
                    );
                }
                DrawAnnotation::Rectangle {
                    top_left,
                    bottom_right,
                    color,
                    thickness,
                } => {
                    let color_bytes = color.to_rgba_bytes();
                    Self::draw_rectangle_outline_on_buffer(
                        &mut annotated_data,
                        frame.width_pixels,
                        top_left,
                        bottom_right,
                        color_bytes,
                        *thickness,
                    );
                }
                DrawAnnotation::TextLabel { .. } => {
                    log::debug!("{} text annotation rendering not yet implemented", LOG_TAG,);
                }
            }
        }

        Ok(RecordingFrame::build(
            frame.timestamp,
            frame.width_pixels,
            frame.height_pixels,
            annotated_data,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::recording::AnnotationColor;
    use std::time::Duration;

    fn create_test_frame(width: u32, height: u32) -> RecordingFrame {
        let data = vec![128u8; (width * height * 4) as usize];
        RecordingFrame::build(Duration::from_secs(0), width, height, data)
    }

    fn create_test_webcam_frame(width: u32, height: u32) -> RecordingFrame {
        let data = vec![200u8; (width * height * 4) as usize];
        RecordingFrame::build(Duration::from_secs(0), width, height, data)
    }

    #[test]
    fn test_calculate_pip_dimensions_respects_aspect_ratio() {
        let (pip_width, pip_height) =
            RgbaFrameCompositor::calculate_pip_dimensions(1920, 1080, 640, 480, 0.2);

        assert_eq!(pip_width, 384);
        let expected_height = (384.0 / (640.0 / 480.0)) as u32;
        assert_eq!(pip_height, expected_height);
    }

    #[test]
    fn test_calculate_pip_position_bottom_left() {
        let (x, y) = RgbaFrameCompositor::calculate_pip_position(
            1920,
            1080,
            200,
            150,
            &WebcamPipPosition::BottomLeft,
        );

        assert_eq!(x, PIP_PADDING_PIXELS);
        assert_eq!(y, 1080 - 150 - PIP_PADDING_PIXELS);
    }

    #[test]
    fn test_calculate_pip_position_bottom_right() {
        let (x, y) = RgbaFrameCompositor::calculate_pip_position(
            1920,
            1080,
            200,
            150,
            &WebcamPipPosition::BottomRight,
        );

        assert_eq!(x, 1920 - 200 - PIP_PADDING_PIXELS);
        assert_eq!(y, 1080 - 150 - PIP_PADDING_PIXELS);
    }

    #[test]
    fn test_calculate_pip_position_top_left() {
        let (x, y) = RgbaFrameCompositor::calculate_pip_position(
            1920,
            1080,
            200,
            150,
            &WebcamPipPosition::TopLeft,
        );

        assert_eq!(x, PIP_PADDING_PIXELS);
        assert_eq!(y, PIP_PADDING_PIXELS);
    }

    #[test]
    fn test_calculate_pip_position_top_right() {
        let (x, y) = RgbaFrameCompositor::calculate_pip_position(
            1920,
            1080,
            200,
            150,
            &WebcamPipPosition::TopRight,
        );

        assert_eq!(x, 1920 - 200 - PIP_PADDING_PIXELS);
        assert_eq!(y, PIP_PADDING_PIXELS);
    }

    #[test]
    fn test_composite_webcam_pip_preserves_frame_dimensions() {
        let compositor = RgbaFrameCompositor::initialize();
        let screen = create_test_frame(400, 300);
        let webcam = create_test_webcam_frame(160, 120);

        let result =
            compositor.composite_webcam_pip(&screen, &webcam, &WebcamPipPosition::BottomLeft, 0.25);

        assert!(result.is_ok());
        let composited = result.unwrap();
        assert_eq!(composited.width_pixels, 400);
        assert_eq!(composited.height_pixels, 300);
        assert_eq!(composited.raw_rgba_data.len(), (400 * 300 * 4) as usize);
    }

    #[test]
    fn test_burn_annotations_empty_list_returns_clone() {
        let compositor = RgbaFrameCompositor::initialize();
        let frame = create_test_frame(200, 100);

        let result = compositor.burn_annotations(&frame, &[]);

        assert!(result.is_ok());
        let annotated = result.unwrap();
        assert_eq!(annotated.raw_rgba_data, frame.raw_rgba_data);
    }

    #[test]
    fn test_burn_annotations_pen_stroke_modifies_buffer() {
        let compositor = RgbaFrameCompositor::initialize();
        let frame = create_test_frame(200, 100);

        let annotations = vec![DrawAnnotation::PenStroke {
            points: vec![
                AnnotationPoint::at(10.0, 10.0),
                AnnotationPoint::at(50.0, 50.0),
            ],
            color: AnnotationColor::red(),
            thickness: 3.0,
        }];

        let result = compositor.burn_annotations(&frame, &annotations);

        assert!(result.is_ok());
        let annotated = result.unwrap();
        assert_ne!(annotated.raw_rgba_data, frame.raw_rgba_data);
    }

    #[test]
    fn test_burn_annotations_arrow_modifies_buffer() {
        let compositor = RgbaFrameCompositor::initialize();
        let frame = create_test_frame(200, 100);

        let annotations = vec![DrawAnnotation::Arrow {
            start: AnnotationPoint::at(10.0, 10.0),
            end: AnnotationPoint::at(80.0, 50.0),
            color: AnnotationColor::yellow(),
            thickness: 2.0,
        }];

        let result = compositor.burn_annotations(&frame, &annotations);

        assert!(result.is_ok());
        let annotated = result.unwrap();
        assert_ne!(annotated.raw_rgba_data, frame.raw_rgba_data);
    }

    #[test]
    fn test_burn_annotations_rectangle_modifies_buffer() {
        let compositor = RgbaFrameCompositor::initialize();
        let frame = create_test_frame(200, 100);

        let annotations = vec![DrawAnnotation::Rectangle {
            top_left: AnnotationPoint::at(20.0, 20.0),
            bottom_right: AnnotationPoint::at(80.0, 60.0),
            color: AnnotationColor::white(),
            thickness: 2.0,
        }];

        let result = compositor.burn_annotations(&frame, &annotations);

        assert!(result.is_ok());
        let annotated = result.unwrap();
        assert_ne!(annotated.raw_rgba_data, frame.raw_rgba_data);
    }

    #[test]
    fn test_burn_annotations_preserves_frame_dimensions() {
        let compositor = RgbaFrameCompositor::initialize();
        let frame = create_test_frame(300, 200);

        let annotations = vec![DrawAnnotation::PenStroke {
            points: vec![
                AnnotationPoint::at(5.0, 5.0),
                AnnotationPoint::at(50.0, 50.0),
            ],
            color: AnnotationColor::red(),
            thickness: 2.0,
        }];

        let result = compositor.burn_annotations(&frame, &annotations);

        assert!(result.is_ok());
        let annotated = result.unwrap();
        assert_eq!(annotated.width_pixels, 300);
        assert_eq!(annotated.height_pixels, 200);
        assert_eq!(annotated.timestamp, frame.timestamp);
    }
}
