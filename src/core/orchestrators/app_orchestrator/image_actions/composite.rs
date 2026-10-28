use super::super::*;
use super::constants::*;
use crate::presentation::DrawStroke;

const LOG_TAG: &str = "[ORCHESTRATOR]";

type ConvertedStroke = (Vec<(f32, f32)>, (f32, f32, f32, f32), f32);

impl AppOrchestrator {
    pub(in crate::core::orchestrators::app_orchestrator) fn build_clipboard_image_data(
        buffer: &CaptureBuffer,
        draw_strokes: &[DrawStroke],
    ) -> Vec<u8> {
        let rgba_data = buffer.raw_data.clone();

        if draw_strokes.is_empty() {
            return rgba_data;
        }

        let converted_strokes = convert_draw_strokes(draw_strokes);
        composite_strokes_onto_image(buffer, rgba_data, &converted_strokes)
    }
}

fn convert_draw_strokes(draw_strokes: &[DrawStroke]) -> Vec<ConvertedStroke> {
    draw_strokes
        .iter()
        .map(|stroke| {
            let points: Vec<(f32, f32)> = stroke
                .points
                .iter()
                .map(|point| (point.x, point.y))
                .collect();
            let color = (
                stroke.color.r,
                stroke.color.g,
                stroke.color.b,
                stroke.color.a,
            );
            (points, color, stroke.width)
        })
        .collect()
}

fn composite_strokes_onto_image(
    buffer: &CaptureBuffer,
    rgba_data: Vec<u8>,
    converted_strokes: &[ConvertedStroke],
) -> Vec<u8> {
    match crate::infrastructure::utils::composite_drawings_on_image(
        &rgba_data,
        buffer.width,
        buffer.height,
        converted_strokes,
    ) {
        Ok(composited_data) => composited_data,
        Err(composite_error) => {
            log::warn!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({
                    "event": EVENT_DRAWINGS_COMPOSITE_FAILED,
                    "error": composite_error.to_string(),
                })
            );
            rgba_data
        }
    }
}
