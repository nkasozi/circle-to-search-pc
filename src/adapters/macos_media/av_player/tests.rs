use std::time::Duration;

use objc2_core_media::CMTime;

use super::frame_conversion::swap_blue_and_red;
use super::*;

#[test]
fn test_scaled_output_size_caps_width_and_keeps_even_edges() {
    assert_eq!(scaled_output_size(3024.0, 1964.0, 1280), (1280, 830));
    assert_eq!(scaled_output_size(801.0, 451.0, 1280), (800, 450));
}

#[test]
fn test_swap_blue_and_red_converts_bgra_to_rgba() {
    let mut pixels = vec![1, 2, 3, 255, 10, 20, 30, 255];
    assert_eq!(swap_blue_and_red(&mut pixels), 2);
    assert_eq!(pixels, vec![3, 2, 1, 255, 30, 20, 10, 255]);
}

#[test]
fn test_cm_time_conversion_handles_invalid_times() {
    let half_second = unsafe { CMTime::new(300, 600) };
    assert_eq!(cm_time_to_duration(half_second), Duration::from_millis(500));
    let invalid = unsafe { CMTime::new(10, 0) };
    assert_eq!(cm_time_to_duration(invalid), Duration::ZERO);
}
