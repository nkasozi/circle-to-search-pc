use super::*;
use crate::core::models::{LogicalRect, OutputDimensions};

fn request(capture_area: CaptureArea) -> ScreenCaptureRequest {
    ScreenCaptureRequest {
        display_origin_x: 100,
        display_origin_y: 0,
        capture_area,
        output: OutputDimensions {
            width: 2,
            height: 2,
        },
        frames_per_second: 30,
        shows_cursor: true,
        captures_system_audio: false,
        visible_own_window_titles: vec![],
    }
}

#[test]
fn test_full_display_uses_whole_monitor() {
    let region = resolve_physical_region(&request(CaptureArea::FullDisplay), 3840, 2160, 2.0);
    assert_eq!(
        region,
        PhysicalRegion {
            x: 0,
            y: 0,
            width: 3840,
            height: 2160
        }
    );
}

#[test]
fn test_region_is_scaled_relative_to_display_and_clamped() {
    let area = CaptureArea::Region(LogicalRect {
        x: 150.0,
        y: 10.0,
        width: 2000.0,
        height: 101.0,
    });
    let region = resolve_physical_region(&request(area), 3840, 2160, 2.0);
    assert_eq!(
        region,
        PhysicalRegion {
            x: 100,
            y: 20,
            width: 3740,
            height: 202
        }
    );
}
