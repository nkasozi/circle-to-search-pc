#![allow(dead_code)]

pub const APPLICATION_NAME: &str = "Circle to Search - Desktop";
pub const APPLICATION_TITLE: &str = "Circle to Search";

pub const HOTKEY_CAPTURE: &str = "Alt+Shift+S";
pub const HOTKEY_CANCEL: &str = "Escape";
pub const HOTKEY_EXIT: &str = "Ctrl+C";

pub const LOG_TAG_APP: &str = "[APP]";
pub const LOG_TAG_CAPTURE: &str = "[CAPTURE]";
pub const LOG_TAG_KEYBOARD: &str = "[KEYBOARD]";
pub const LOG_TAG_WINDOW: &str = "[WINDOW]";

pub const MESSAGE_STARTING_APP: &str = "starting circle-to-search-pc";
pub const MESSAGE_INITIALIZING_APP: &str = "initializing app";
pub const MESSAGE_OPENING_CAPTURE: &str = "opening capture window";
pub const MESSAGE_CAPTURE_CREATED: &str = "capture window created successfully";
pub const MESSAGE_CAPTURE_FAILED: &str = "failed to create capture";
pub const MESSAGE_WINDOW_CLOSED: &str = "window closed";
pub const MESSAGE_EXITING_APP: &str = "exiting application";
pub const MESSAGE_CANCEL_REQUESTED: &str = "cancel requested";
pub const MESSAGE_MONITOR_FOUND: &str = "found monitor at";
pub const MESSAGE_MONITOR_FAILED: &str = "failed to get monitor";
pub const MESSAGE_MOUSE_POSITION_FAILED: &str = "failed to get mouse position, using (0,0)";
pub const MESSAGE_CREATING_CAPTURE: &str = "creating new capture";
pub const MESSAGE_CANCEL_RECEIVED: &str = "cancel message received";

pub const MESSAGE_KEYBOARD_ALT_PRESSED: &str = "Alt pressed";
pub const MESSAGE_KEYBOARD_ALT_RELEASED: &str = "Alt released";
pub const MESSAGE_KEYBOARD_SHIFT_PRESSED: &str = "Shift pressed";
pub const MESSAGE_KEYBOARD_SHIFT_RELEASED: &str = "Shift released";
pub const MESSAGE_KEYBOARD_HOTKEY_DETECTED: &str = "Alt+Shift+S detected - opening capture";
pub const MESSAGE_KEYBOARD_ESCAPE_PRESSED: &str = "Escape pressed - canceling";

pub const USER_MESSAGE_INFO_OPENING: &str = "[INFO] Opening capture window...";
pub const USER_MESSAGE_SUCCESS_OPENED: &str =
    "[SUCCESS] Capture window opened! Press Escape to cancel.";
pub const USER_MESSAGE_INFO_CLOSED: &str = "[INFO] Capture window closed. Ready for next capture.";

pub const ERROR_CONTEXT_SCALE_FACTOR: &str = "Unable to get scale factor";
pub const ERROR_CONTEXT_CAPTURE_MONITOR: &str = "Unable to capture Monitor";

pub const CAPTURE_FORMAT_DIMENSIONS: &str = "captured {}x{} screenshot, scale_factor={}";

pub const DEFAULT_MOUSE_POSITION_X: i32 = 0;
pub const DEFAULT_MOUSE_POSITION_Y: i32 = 0;

pub const OVERLAY_BACKGROUND_RGBA: (f32, f32, f32, f32) = (0.0, 0.0, 0.0, 0.3);

pub const IMGBB_API_URL: &str = "https://api.imgbb.com/1/upload";
pub const IMGBB_API_KEY: &str = "851a50a8320bc2c59b0f564f0a1343de";
pub const IMGBB_EXPIRATION_SECONDS: &str = "900";

pub const DEFAULT_IMAGE_SEARCH_URL: &str = "https://lens.google.com/uploadbyurl?url={}";

pub const DEFAULT_CAPTURE_HOTKEY: &str = "Alt+Shift+S";

pub const SETTINGS_FILE_NAME: &str = "settings.json";

pub const STARTUP_BANNER: &str = r#"
╔════════════════════════════════════════════════════════╗
║  Circle to Search - Desktop                            ║
║                                                        ║
║  App is running in the background!                     ║
║                                                        ║
║  Press Alt+Shift+S to capture screen                   ║
║  Press Escape to cancel capture                        ║
║  Press Ctrl+C to exit                                  ║
║                                                        ║
╚════════════════════════════════════════════════════════╝
"#;
