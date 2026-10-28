use super::*;

fn create_test_buffer_with_pattern(width: u32, height: u32) -> CaptureBuffer {
    let mut raw_data = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let red = (x % 256) as u8;
            let green = (y % 256) as u8;
            let blue = ((x + y) % 256) as u8;
            let alpha = 255u8;

            raw_data.extend_from_slice(&[red, green, blue, alpha]);
        }
    }

    CaptureBuffer::build_from_raw_data(1.0, width, height, raw_data)
}

#[test]
fn test_build_from_raw_data_creates_buffer_with_correct_dimensions() {
    let width = 100;
    let height = 50;
    let raw_data = vec![0u8; (width * height * 4) as usize];

    let buffer = CaptureBuffer::build_from_raw_data(1.0, width, height, raw_data.clone());

    assert_eq!(buffer.width, width);
    assert_eq!(buffer.height, height);
    assert_eq!(buffer.raw_data.len(), raw_data.len());
    assert_eq!(buffer._scale_factor, 1.0);
}

#[test]
fn test_crop_region_with_valid_dimensions_returns_cropped_buffer() {
    let buffer = create_test_buffer_with_pattern(100, 100);

    let result = buffer.crop_region(10, 20, 30, 40);

    assert!(result.is_ok());
    let cropped = result.unwrap();
    assert_eq!(cropped.width, 30);
    assert_eq!(cropped.height, 40);
    assert_eq!(cropped.raw_data.len(), (30 * 40 * 4) as usize);
}

#[test]
fn test_crop_region_with_zero_width_returns_error() {
    let buffer = create_test_buffer_with_pattern(100, 100);

    let result = buffer.crop_region(10, 10, 0, 50);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("must be greater than zero"));
}

#[test]
fn test_crop_region_with_zero_height_returns_error() {
    let buffer = create_test_buffer_with_pattern(100, 100);

    let result = buffer.crop_region(10, 10, 50, 0);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("must be greater than zero"));
}

#[test]
fn test_crop_region_clamps_coordinates_to_buffer_bounds() {
    let buffer = create_test_buffer_with_pattern(100, 100);

    let result = buffer.crop_region(95, 95, 20, 20);

    assert!(result.is_ok());
    let cropped = result.unwrap();
    assert_eq!(cropped.width, 5);
    assert_eq!(cropped.height, 5);
}

#[test]
fn test_crop_region_at_origin_returns_correct_subset() {
    let buffer = create_test_buffer_with_pattern(50, 50);

    let result = buffer.crop_region(0, 0, 10, 10);

    assert!(result.is_ok());
    let cropped = result.unwrap();
    assert_eq!(cropped.width, 10);
    assert_eq!(cropped.height, 10);
}

#[test]
fn test_crop_region_preserves_scale_factor() {
    let raw_data = vec![0u8; (100 * 100 * 4) as usize];
    let buffer = CaptureBuffer::build_from_raw_data(2.5, 100, 100, raw_data);

    let result = buffer.crop_region(10, 10, 20, 20);

    assert!(result.is_ok());
    let cropped = result.unwrap();
    assert_eq!(cropped._scale_factor, 2.5);
}

#[test]
fn test_crop_full_image_returns_identical_dimensions() {
    let buffer = create_test_buffer_with_pattern(50, 50);

    let result = buffer.crop_region(0, 0, 50, 50);

    assert!(result.is_ok());
    let cropped = result.unwrap();
    assert_eq!(cropped.width, buffer.width);
    assert_eq!(cropped.height, buffer.height);
}
