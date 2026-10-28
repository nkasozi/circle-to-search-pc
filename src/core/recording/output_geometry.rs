use crate::core::models::{LogicalRect, OutputDimensions, RecordingQuality};

pub const MAXIMUM_OUTPUT_WIDTH: u32 = 3_840;
const MINIMUM_OUTPUT_EDGE: u32 = 64;

pub fn round_down_to_even(value: u32) -> u32 {
    value - (value % 2)
}

pub fn compute_output_dimensions(
    area: &LogicalRect,
    display_scale_factor: f64,
    quality: RecordingQuality,
) -> OutputDimensions {
    let pixel_scale = match quality {
        RecordingQuality::Standard => 1.0,
        RecordingQuality::Crisp => display_scale_factor.max(1.0),
    };
    let scaled_width = area.width * pixel_scale;
    let scaled_height = area.height * pixel_scale;
    let width_limit_ratio = (MAXIMUM_OUTPUT_WIDTH as f64 / scaled_width).min(1.0);
    let width = (scaled_width * width_limit_ratio).round() as u32;
    let height = (scaled_height * width_limit_ratio).round() as u32;
    OutputDimensions {
        width: round_down_to_even(width.max(MINIMUM_OUTPUT_EDGE)),
        height: round_down_to_even(height.max(MINIMUM_OUTPUT_EDGE)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(width: f64, height: f64) -> LogicalRect {
        LogicalRect {
            x: 0.0,
            y: 0.0,
            width,
            height,
        }
    }

    #[test]
    fn test_crisp_quality_uses_display_scale() {
        let dimensions =
            compute_output_dimensions(&rect(1512.0, 982.0), 2.0, RecordingQuality::Crisp);
        assert_eq!(
            dimensions,
            OutputDimensions {
                width: 3024,
                height: 1964
            }
        );
    }

    #[test]
    fn test_standard_quality_uses_logical_size_and_even_edges() {
        let dimensions =
            compute_output_dimensions(&rect(801.0, 451.0), 2.0, RecordingQuality::Standard);
        assert_eq!(
            dimensions,
            OutputDimensions {
                width: 800,
                height: 450
            }
        );
    }

    #[test]
    fn test_output_width_is_capped() {
        let dimensions =
            compute_output_dimensions(&rect(2560.0, 1440.0), 2.0, RecordingQuality::Crisp);
        assert_eq!(
            dimensions,
            OutputDimensions {
                width: 3840,
                height: 2160
            }
        );
    }

    #[test]
    fn test_tiny_regions_get_minimum_size() {
        let dimensions = compute_output_dimensions(&rect(3.0, 3.0), 1.0, RecordingQuality::Crisp);
        assert_eq!(
            dimensions,
            OutputDimensions {
                width: 64,
                height: 64
            }
        );
    }
}
