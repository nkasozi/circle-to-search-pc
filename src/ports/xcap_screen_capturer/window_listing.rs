use iced::widget::image::Handle;

use crate::core::models::WindowInfo;
use crate::global_constants::LOG_TAG_CAPTURE;

const THUMBNAIL_MAX_SIZE: u32 = 120;
const MINIMUM_WINDOW_EDGE: u32 = 50;
const MAXIMUM_THUMBNAIL_SCALE: f32 = 1.0;
const CAPTURABLE_WINDOW_FOUND_EVENT: &str = "capturable_window_found";

pub(super) fn build_window_info(window: &xcap::Window) -> Option<WindowInfo> {
    let window_id = window.id().ok()?;
    let app_name = window.app_name().unwrap_or_default();
    let title = window.title().unwrap_or_default();
    let width = window.width().unwrap_or(0);
    let height = window.height().unwrap_or(0);
    let is_minimized = window.is_minimized().unwrap_or(false);

    if app_name.is_empty() && title.is_empty() {
        return None;
    }

    if width < MINIMUM_WINDOW_EDGE || height < MINIMUM_WINDOW_EDGE {
        return None;
    }

    let thumbnail = match is_minimized {
        true => None,
        false => capture_window_thumbnail(window),
    };

    let window_info = WindowInfo::build(
        window_id,
        app_name,
        title,
        width,
        height,
        is_minimized,
        thumbnail,
    );

    log::debug!(
        "{} {}",
        LOG_TAG_CAPTURE,
        serde_json::json!({"event": CAPTURABLE_WINDOW_FOUND_EVENT, "window_id": window_id, "display_name": window_info.display_name()})
    );

    Some(window_info)
}

fn capture_window_thumbnail(window: &xcap::Window) -> Option<Handle> {
    let captured_image = window.capture_image().ok()?;
    create_thumbnail_from_image(&captured_image)
}

fn create_thumbnail_from_image(image: &xcap::image::RgbaImage) -> Option<Handle> {
    let width = image.width();
    let height = image.height();

    if width == 0 || height == 0 {
        return None;
    }

    let scale = (THUMBNAIL_MAX_SIZE as f32 / width.max(height) as f32).min(MAXIMUM_THUMBNAIL_SCALE);
    let new_width = (width as f32 * scale) as u32;
    let new_height = (height as f32 * scale) as u32;

    if new_width == 0 || new_height == 0 {
        return None;
    }

    let resized = xcap::image::imageops::resize(
        image,
        new_width,
        new_height,
        xcap::image::imageops::FilterType::Triangle,
    );

    Some(Handle::from_rgba(new_width, new_height, resized.into_raw()))
}
