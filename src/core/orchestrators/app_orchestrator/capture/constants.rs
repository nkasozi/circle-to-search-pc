pub(super) const EVENT_CAPTURE_SCREEN_STARTED: &str = "capture_screen_started";
pub(super) const EVENT_CAPTURE_SCREEN_MINIMIZING_WINDOW: &str = "capture_screen_minimizing_window";
pub(super) const EVENT_SCREEN_CAPTURE_PERFORMING: &str = "screen_capture_performing";
pub(super) const EVENT_MOUSE_POSITION_RESOLVED: &str = "mouse_position_resolved";
pub(super) const EVENT_MOUSE_POSITION_FAILED: &str = "mouse_position_failed";
pub(super) const EVENT_SCREEN_CAPTURED: &str = "screen_captured";
pub(super) const EVENT_SCREEN_CAPTURE_FAILED: &str = "screen_capture_failed";
pub(super) const EVENT_CAPTURE_ERROR_RECEIVED: &str = "capture_error_received";
pub(super) const EVENT_CAPTURE_OVERLAY_OPENING: &str = "capture_overlay_opening";
pub(super) const EVENT_CAPTURE_OVERLAY_MONITOR_FOUND: &str = "capture_overlay_monitor_found";
pub(super) const EVENT_CAPTURE_OVERLAY_CREATED: &str = "capture_overlay_created";
pub(super) const EVENT_CAPTURE_OVERLAY_MONITOR_FAILED: &str = "capture_overlay_monitor_failed";
pub(super) const EVENT_ESCAPE_PRESSED: &str = "escape_pressed";
pub(super) const EVENT_GLOBAL_HOTKEY_PRESSED: &str = "global_hotkey_pressed";
pub(super) const EVENT_CAPTURE_OVERLAY_MESSAGE_RECEIVED: &str = "capture_overlay_message_received";
pub(super) const EVENT_CAPTURE_OVERLAY_SWITCHING_TO_RECORDING: &str =
    "capture_overlay_switching_to_recording";
pub(super) const EVENT_WINDOW_SELECTION_REQUESTED_FROM_OVERLAY: &str =
    "window_selection_requested_from_overlay";
pub(super) const EVENT_CAPTURE_OVERLAY_WINDOW_NOT_FOUND: &str = "capture_overlay_window_not_found";
pub(super) const EVENT_SELECTION_CONFIRMING: &str = "selection_confirming";
pub(super) const EVENT_CONFIRM_SELECTION_OVERLAY_NOT_FOUND: &str =
    "confirm_selection_overlay_not_found";
pub(super) const EVENT_CONFIRM_SELECTION_NO_REGION: &str = "confirm_selection_no_region";
pub(super) const EVENT_SELECTION_CONFIRMED: &str = "selection_confirmed";
pub(super) const EVENT_CROPPED_IMAGE_SHOWING: &str = "cropped_image_showing";
pub(super) const EVENT_IMAGE_CROPPED: &str = "image_cropped";
pub(super) const EVENT_IMAGE_CROP_FAILED: &str = "image_crop_failed";

pub(super) const MOUSE_POSITION_FALLBACK_DESCRIPTION: &str = "0,0";
pub(super) const MOUSE_POSITION_FALLBACK: (i32, i32) = (0, 0);
pub(super) const CAPTURE_DELAY_AFTER_MINIMIZE_MILLISECONDS: u64 = 200;
pub(super) const MONITOR_FALLBACK_ORIGIN: i32 = 0;
pub(super) const MONITOR_FALLBACK_WIDTH: u32 = 1920;
pub(super) const MONITOR_FALLBACK_HEIGHT: u32 = 1080;
pub(super) const CROPPED_IMAGE_WINDOW_MAX_WIDTH: f32 = 1200.0;
pub(super) const CROPPED_IMAGE_WINDOW_MAX_HEIGHT: f32 = 800.0;
