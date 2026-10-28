use image::{Rgba, RgbaImage};

const COLOR_CHANNEL_MAXIMUM: f32 = 255.0;
const MINIMUM_LINE_LENGTH: f32 = 0.001;
const LINE_SAMPLES_PER_PIXEL: f32 = 2.0;
const MINIMUM_CIRCLE_RADIUS: i32 = 1;
const MINIMUM_STROKE_POINTS: usize = 2;

type StrokePoint = (f32, f32);
type StrokeColor = (f32, f32, f32, f32);

pub fn composite_drawings_on_image(
    rgba_data: &[u8],
    width: u32,
    height: u32,
    draw_strokes: &[(Vec<(f32, f32)>, (f32, f32, f32, f32), f32)],
) -> Result<Vec<u8>, String> {
    let mut canvas = RgbaImage::from_raw(width, height, rgba_data.to_vec())
        .ok_or_else(|| crate::global_constants::OCR_RAW_IMAGE_CREATION_FAILED.to_string())?;

    for (points, stroke_color, stroke_width) in draw_strokes {
        let _ = draw_stroke(&mut canvas, points, *stroke_color, *stroke_width);
    }

    Ok(canvas.into_raw())
}

fn draw_stroke(
    canvas: &mut RgbaImage,
    points: &[StrokePoint],
    stroke_color: StrokeColor,
    stroke_width: f32,
) -> bool {
    if points.len() < MINIMUM_STROKE_POINTS {
        return false;
    }

    let color = convert_stroke_color(stroke_color);
    for segment in points.windows(MINIMUM_STROKE_POINTS) {
        let _ = draw_thick_line(canvas, segment[0], segment[1], stroke_width, color);
    }
    true
}

fn convert_stroke_color((red, green, blue, alpha): StrokeColor) -> Rgba<u8> {
    Rgba([
        (red * COLOR_CHANNEL_MAXIMUM) as u8,
        (green * COLOR_CHANNEL_MAXIMUM) as u8,
        (blue * COLOR_CHANNEL_MAXIMUM) as u8,
        (alpha * COLOR_CHANNEL_MAXIMUM) as u8,
    ])
}

fn draw_thick_line(
    canvas: &mut RgbaImage,
    start: StrokePoint,
    end: StrokePoint,
    thickness: f32,
    color: Rgba<u8>,
) -> bool {
    let (start_x, start_y) = start;
    let delta_x = end.0 - start_x;
    let delta_y = end.1 - start_y;
    let length = (delta_x * delta_x + delta_y * delta_y).sqrt();
    let radius = (thickness / 2.0) as i32;

    if length < MINIMUM_LINE_LENGTH {
        return draw_filled_circle(canvas, start_x as i32, start_y as i32, radius, color);
    }

    let steps = (length * LINE_SAMPLES_PER_PIXEL) as i32;
    for step in 0..=steps {
        let progress = step as f32 / steps as f32;
        let center_x = start_x + progress * delta_x;
        let center_y = start_y + progress * delta_y;
        let _ = draw_filled_circle(canvas, center_x as i32, center_y as i32, radius, color);
    }
    true
}

fn draw_filled_circle(
    canvas: &mut RgbaImage,
    center_x: i32,
    center_y: i32,
    radius: i32,
    color: Rgba<u8>,
) -> bool {
    let (width, height) = canvas.dimensions();
    let radius = radius.max(MINIMUM_CIRCLE_RADIUS);

    for offset_y in -radius..=radius {
        for offset_x in -radius..=radius {
            let is_inside_circle = offset_x * offset_x + offset_y * offset_y <= radius * radius;
            let pixel_x = center_x + offset_x;
            let pixel_y = center_y + offset_y;
            let is_inside_canvas = pixel_x >= 0
                && pixel_y >= 0
                && (pixel_x as u32) < width
                && (pixel_y as u32) < height;
            if is_inside_circle && is_inside_canvas {
                canvas.put_pixel(pixel_x as u32, pixel_y as u32, color);
            }
        }
    }
    true
}
