use std::time::Duration;

pub const LOG_TAG: &str = "[APP]";
pub const EVENT_APP_INITIALIZING: &str = "app_initializing";
pub const EVENT_PERMISSION_CHECK: &str = "permission_check";
pub const EVENT_SETTINGS_LOAD_FAILED: &str = "settings_load_failed_using_defaults";
pub const EVENT_SYSTEM_TRAY_INITIALIZED: &str = "system_tray_initialized";
pub const EVENT_SYSTEM_TRAY_INITIALIZATION_FAILED: &str = "system_tray_initialization_failed";
pub const EVENT_OCR_SERVICE_INITIALIZED: &str = "tesseract_ocr_service_initialized";
pub const EVENT_OCR_SERVICE_INITIALIZATION_FAILED: &str =
    "tesseract_ocr_service_initialization_failed";
pub const EVENT_SHOWING_ONBOARDING: &str = "launch_showing_onboarding";
pub const EVENT_SHOWING_MAIN_WINDOW: &str = "launch_showing_main_window";
pub const EVENT_LAUNCHING_HIDDEN: &str = "launch_staying_hidden";
pub const OCR_SERVICE_NOT_INITIALIZED_ERROR: &str = "OCR service not initialized yet";
pub const SUBSCRIPTION_CHANNEL_CAPACITY: usize = 10;
pub const TRAY_POLL_INTERVAL: Duration = Duration::from_millis(100);
pub const SPINNER_TICK_INTERVAL: Duration = Duration::from_millis(80);
pub const ONBOARDING_PERMISSION_POLL_INTERVAL: Duration = Duration::from_millis(1_500);
