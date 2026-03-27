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
pub const IMGBB_PUBLIC_KEY: &str = "851a50a8320bc2c59b0f564f0a1343de";
pub const IMGBB_EXPIRATION_SECONDS: &str = "900";
pub const IMGBB_PUBLIC_KEY_QUERY_NAME: &str = "key";
pub const IMGBB_PUBLIC_KEY_ENV_VAR_NAME: &str = "IMGBB_API_KEY";
pub const IMGBB_HTTP_METHOD: &str = "POST";
pub const IMGBB_IMAGE_FIELD_NAME: &str = "image";
pub const IMAGE_HOSTING_VALIDATION_URL_EMPTY: &str = "Image hosting URL cannot be empty";
pub const IMAGE_HOSTING_VALIDATION_URL_INVALID: &str =
    "Image hosting URL must be a valid absolute URL";
pub const IMAGE_HOSTING_VALIDATION_KEY_NAME_EMPTY: &str = "Public key name cannot be empty";
pub const IMAGE_HOSTING_VALIDATION_KEY_EMPTY: &str = "Public key cannot be empty";
pub const IMAGE_HOSTING_VALIDATION_EXPIRATION_EMPTY: &str = "Expiration seconds cannot be empty";
pub const IMAGE_HOSTING_VALIDATION_EXPIRATION_INVALID: &str =
    "Expiration seconds must be a positive integer";
pub const IMAGE_HOSTING_SETTINGS_TIP: &str =
    "Tip: If uploads fail, replace Provider URL, Auth Mode, and public key with your own account values.";

pub const DEFAULT_IMAGE_SEARCH_URL: &str = "https://lens.google.com/uploadbyurl?url={}";

pub const DEFAULT_CAPTURE_HOTKEY: &str = "Alt+Shift+S";

pub const SETTINGS_FILE_NAME: &str = "settings.json";

pub const STATUS_INITIALIZING: &str = "Initializing OCR service...";
pub const STATUS_PREPARING_CAPTURE: &str = "Preparing to capture...";
pub const STATUS_CAPTURING_SCREEN: &str = "Capturing screen...";
pub const STATUS_OVERLAY_READY: &str = "Overlay ready!";
pub const STATUS_PROCESSING_SELECTION: &str = "Processing selection...";
pub const STATUS_READY_SIMPLE: &str = "Ready";
pub const STATUS_OCR_COMPLETE: &str = "OCR complete";
pub const STATUS_READY: &str = "Ready - Press Alt+Shift+S to capture";
pub const STATUS_PROCESSING_OCR: &str = "Processing OCR...";
pub const STATUS_SETTINGS_SAVED: &str = "Settings saved";
pub const STATUS_SETTINGS_EDITOR_NOT_ACTIVE: &str = "Settings editor is not active";
pub const STATUS_SETTINGS_SAVE_FAILED_PREFIX: &str = "Failed to save settings: ";
pub const STATUS_ONBOARDING_SCREEN_RECORDING_FAILED: &str =
    "Failed to open Screen Recording settings";
pub const STATUS_ONBOARDING_INPUT_MONITORING_FAILED: &str =
    "Failed to open Input Monitoring settings";
pub const STATUS_KEYWORD_READY: &str = "Ready";
pub const STATUS_KEYWORD_LOADING: &str = "Loading";
pub const STATUS_KEYWORD_INITIALIZING: &str = "Initializing";
pub const STATUS_KEYWORD_ERROR: &str = "Error";
pub const STATUS_KEYWORD_FAILED: &str = "Failed";

pub const CAPTURE_ERROR_MONITOR_PREFIX: &str = "Monitor error: ";
pub const CAPTURE_ERROR_CROP_PREFIX: &str = "Error cropping image: ";
pub const CAPTURE_ERROR_GENERIC_PREFIX: &str = "Capture failed: ";
pub const CAPTURE_ERROR_GENERIC_SUFFIX: &str = ". Try closing other instances.";
pub const CAPTURE_ERROR_LINUX_PERMISSION_PREFIX: &str = "Screen capture failed: ";
pub const CAPTURE_ERROR_LINUX_PERMISSION_SUFFIX: &str = "\n\nOn Linux (Wayland), screen capture requires:\n• PipeWire and xdg-desktop-portal installed\n• A portal dialog will appear - click 'Share' to allow\n\nTry: sudo apt install pipewire xdg-desktop-portal";
pub const CAPTURE_ERROR_MACOS_PERMISSION_PREFIX: &str = "Screen capture failed: ";
pub const CAPTURE_ERROR_MACOS_PERMISSION_SUFFIX: &str = "\n\nPlease grant Screen Recording permission:\nSystem Settings -> Privacy & Security -> Screen Recording";
pub const CAPTURE_PLATFORM_LINUX: &str = "linux";
pub const CAPTURE_PLATFORM_MACOS: &str = "macos";
pub const CAPTURE_PLATFORM_WINDOWS: &str = "windows";
pub const CAPTURE_PLATFORM_UNKNOWN: &str = "unknown";
pub const CAPTURE_ERROR_KEYWORD_PERMISSION: &str = "permission";
pub const CAPTURE_ERROR_KEYWORD_ACCESS: &str = "access";
pub const CAPTURE_ERROR_KEYWORD_DENIED: &str = "denied";
pub const CAPTURE_ERROR_KEYWORD_PIPEWIRE: &str = "pipewire";
pub const CAPTURE_ERROR_KEYWORD_PORTAL: &str = "portal";

pub const IMAGE_SEARCH_FAILURE_SUFFIX: &str =
    " - Update Settings > Image Hosting or use Copy Image to clipboard";
pub const IMAGE_SEARCH_TIMEOUT_SECONDS: u64 = 30;
pub const IMAGE_SEARCH_TIMEOUT_MESSAGE: &str =
    "Search timed out after 30 seconds - Update Settings > Image Hosting or use Copy Image to clipboard";
pub const OCR_RAW_IMAGE_CREATION_FAILED: &str = "Failed to create image from raw data";
pub const OCR_INITIALIZATION_FAILED_PREFIX: &str = "OCR initialization failed: ";

pub const UI_GENERIC_LOADING: &str = "Loading...";

pub const MAIN_WINDOW_ICON_SEARCH: &str = "🔍";
pub const MAIN_WINDOW_SUBTITLE: &str = "Search anything on your screen instantly";
pub const MAIN_WINDOW_ICON_CAPTURE: &str = "📸";
pub const MAIN_WINDOW_CAPTURE_BUTTON_LABEL: &str = "Capture Screen";
pub const MAIN_WINDOW_HOTKEY_HINT_TEMPLATE: &str = "Press {hotkey} anywhere";
pub const MAIN_WINDOW_OR_TEXT: &str = "OR";
pub const MAIN_WINDOW_KEEP_RUNNING_LABEL: &str = "Keep running in background";
pub const MAIN_WINDOW_ICON_SETTINGS: &str = "⚙";
pub const MAIN_WINDOW_SETTINGS_BUTTON_LABEL: &str = "Settings";
pub const MAIN_WINDOW_HOTKEY_TEMPLATE_TOKEN: &str = "{hotkey}";
pub const MAIN_WINDOW_STATUS_ICON_FILLED: &str = "●";
pub const MAIN_WINDOW_STATUS_ICON_EMPTY: &str = "○";

pub const SETTINGS_WINDOW_ICON: &str = "⚙";
pub const SETTINGS_WINDOW_TITLE: &str = "Settings";
pub const SETTINGS_SECTION_SEARCH_TITLE: &str = "Search";
pub const SETTINGS_SECTION_SEARCH_ICON: &str = "🔍";
pub const SETTINGS_LABEL_IMAGE_SEARCH_URL: &str = "Image Search URL";
pub const SETTINGS_DESCRIPTION_IMAGE_SEARCH_URL: &str = "Template URL for reverse image search";
pub const SETTINGS_SECTION_IMAGE_HOSTING_TITLE: &str = "Image Hosting";
pub const SETTINGS_SECTION_IMAGE_HOSTING_ICON: &str = "🖼";
pub const SETTINGS_LABEL_PROVIDER_URL: &str = "Provider URL";
pub const SETTINGS_DESCRIPTION_PROVIDER_URL: &str = "Upload endpoint URL";
pub const SETTINGS_LABEL_AUTH_MODE: &str = "Auth Mode";
pub const SETTINGS_DESCRIPTION_AUTH_MODE: &str = "How the public key is sent";
pub const SETTINGS_LABEL_PUBLIC_KEY_NAME: &str = "Public Key Name";
pub const SETTINGS_DESCRIPTION_PUBLIC_KEY_NAME: &str = "Query parameter or header name";
pub const SETTINGS_LABEL_PUBLIC_KEY: &str = "Public Key";
pub const SETTINGS_DESCRIPTION_PUBLIC_KEY: &str = "Public key used for image hosting uploads";
pub const SETTINGS_LABEL_EXPIRATION_SECONDS: &str = "Expiration Seconds";
pub const SETTINGS_DESCRIPTION_EXPIRATION_SECONDS: &str = "Upload expiry lifetime in seconds";
pub const SETTINGS_PUBLIC_KEY_PLACEHOLDER: &str = "Enter public key";
pub const SETTINGS_LABEL_HTTP_METHOD: &str = "HTTP Method";
pub const SETTINGS_DESCRIPTION_HTTP_METHOD: &str = "HTTP method used for image upload requests";
pub const SETTINGS_HTTP_METHOD_PLACEHOLDER: &str = "POST";
pub const SETTINGS_LABEL_IMAGE_FIELD_NAME: &str = "Image Field Name";
pub const SETTINGS_DESCRIPTION_IMAGE_FIELD_NAME: &str =
    "Multipart form field name for the base64 image";
pub const SETTINGS_IMAGE_FIELD_NAME_PLACEHOLDER: &str = "image";
pub const SETTINGS_RESTART_REQUIRED_WARNING: &str = "Requires app restart to take effect";
pub const SETTINGS_SECTION_KEYBOARD_TITLE: &str = "Keyboard";
pub const SETTINGS_SECTION_KEYBOARD_ICON: &str = "⌨";
pub const SETTINGS_LABEL_CAPTURE_HOTKEY: &str = "Capture Hotkey";
pub const SETTINGS_DESCRIPTION_CAPTURE_HOTKEY: &str = "Global shortcut to start capture";
pub const SETTINGS_SECTION_APPEARANCE_TITLE: &str = "Appearance";
pub const SETTINGS_SECTION_APPEARANCE_ICON: &str = "🎨";
pub const SETTINGS_LABEL_THEME: &str = "Theme";
pub const SETTINGS_DESCRIPTION_THEME: &str = "Choose light or dark mode";
pub const SETTINGS_ICON_SAVE: &str = "💾";
pub const SETTINGS_SAVE_CHANGES_LABEL: &str = "Save Changes";

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
