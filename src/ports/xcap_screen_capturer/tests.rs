use super::*;

#[test]
fn test_initialize_creates_capturer() {
    let capturer = XcapScreenCapturer::initialize();

    assert!(std::mem::size_of_val(&capturer) == 0);
}

#[test]
fn test_convert_image_to_capture_buffer_creates_buffer_with_correct_dimensions() {
    let capturer = XcapScreenCapturer::initialize();
    let width = 100u32;
    let height = 50u32;
    let raw_data = vec![255u8; (width * height * 4) as usize];
    let image = xcap::image::RgbaImage::from_raw(width, height, raw_data).unwrap();

    let buffer = capturer.convert_image_to_capture_buffer(image, 2.0);

    assert_eq!(buffer.width, width);
    assert_eq!(buffer.height, height);
    assert_eq!(buffer._scale_factor, 2.0);
}

#[test]
fn test_convert_image_to_capture_buffer_preserves_scale_factor() {
    let capturer = XcapScreenCapturer::initialize();
    let width = 100u32;
    let height = 100u32;
    let raw_data = vec![0u8; (width * height * 4) as usize];
    let image = xcap::image::RgbaImage::from_raw(width, height, raw_data).unwrap();

    let buffer = capturer.convert_image_to_capture_buffer(image, 1.5);

    assert_eq!(buffer._scale_factor, 1.5);
}
