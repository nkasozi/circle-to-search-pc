#![allow(unexpected_cfgs)]
#![allow(deprecated)]

#[cfg(target_os = "macos")]
mod dynamic_library;
#[cfg(target_os = "macos")]
mod log_events;

#[cfg(target_os = "macos")]
pub mod macos {
    use super::dynamic_library::DynamicLibraryHandle;
    use super::log_events::*;
    use std::process::Command;

    const SCREEN_RECORDING_PERMISSION: &str = "Screen Recording";
    const CORE_GRAPHICS_FRAMEWORK_PATH: &str =
        "/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics";
    const CG_PREFLIGHT_ACCESS_SYMBOL: &str = "CGPreflightScreenCaptureAccess";
    const CG_REQUEST_ACCESS_SYMBOL: &str = "CGRequestScreenCaptureAccess";
    const CG_DISPLAY_STREAM_SYMBOL: &str = "CGDisplayStreamCreateWithDispatchQueue";
    const SETTINGS_PANE_SCREEN_RECORDING: &str =
        "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture";
    const SETTINGS_PANE_SECURITY: &str = "x-apple.systempreferences:com.apple.preference.security";
    const SYSTEM_OPEN_COMMAND: &str = "open";

    pub fn check_screen_recording_permission() -> bool {
        log::info!(
            "{} {}",
            LOG_TAG_PERMISSIONS,
            serde_json::json!({"event": EVENT_SCREEN_RECORDING_PERMISSION_CHECKING})
        );

        let has_permission = check_screen_recording_permission_internal();

        match has_permission {
            true => log::info!(
                "{} {}",
                LOG_TAG_PERMISSIONS,
                serde_json::json!({"event": EVENT_SCREEN_RECORDING_PERMISSION_GRANTED})
            ),
            false => log::warn!(
                "{} {}",
                LOG_TAG_PERMISSIONS,
                serde_json::json!({"event": EVENT_SCREEN_RECORDING_PERMISSION_NOT_GRANTED})
            ),
        }

        has_permission
    }

    pub fn request_screen_recording_permission() -> bool {
        let Some(core_graphics_library) = DynamicLibraryHandle::open(CORE_GRAPHICS_FRAMEWORK_PATH)
        else {
            return false;
        };
        let Some(symbol_pointer) = core_graphics_library.find_symbol(CG_REQUEST_ACCESS_SYMBOL)
        else {
            log::warn!(
                "{} {}",
                LOG_TAG_PERMISSIONS,
                serde_json::json!({"event": EVENT_SCREEN_ACCESS_REQUEST_UNAVAILABLE})
            );
            return false;
        };
        type CGRequestScreenCaptureAccessFn = unsafe extern "C" fn() -> bool;
        let request_function: CGRequestScreenCaptureAccessFn =
            unsafe { std::mem::transmute(symbol_pointer) };
        let granted = unsafe { request_function() };
        log::info!(
            "{} {}",
            LOG_TAG_PERMISSIONS,
            serde_json::json!({"event": EVENT_SCREEN_ACCESS_REQUESTED, "granted": granted})
        );
        granted
    }

    pub fn open_screen_recording_settings() -> bool {
        log::info!(
            "{} {}",
            LOG_TAG_PERMISSIONS,
            serde_json::json!({"event": EVENT_SCREEN_RECORDING_SETTINGS_OPENING})
        );
        open_system_preferences(SCREEN_RECORDING_PERMISSION)
    }

    fn check_screen_recording_permission_internal() -> bool {
        let Some(core_graphics_library) = DynamicLibraryHandle::open(CORE_GRAPHICS_FRAMEWORK_PATH)
        else {
            log::warn!(
                "{} {}",
                LOG_TAG_PERMISSIONS,
                serde_json::json!({"event": EVENT_CORE_GRAPHICS_LOAD_FAILED, "framework_path": CORE_GRAPHICS_FRAMEWORK_PATH})
            );
            return false;
        };

        type CGPreflightScreenCaptureAccessFn = unsafe extern "C" fn() -> bool;

        let Some(symbol_pointer) = core_graphics_library.find_symbol(CG_PREFLIGHT_ACCESS_SYMBOL)
        else {
            log::warn!(
                "{} {}",
                LOG_TAG_PERMISSIONS,
                serde_json::json!({"event": EVENT_PREFLIGHT_SYMBOL_MISSING, "symbol_name": CG_PREFLIGHT_ACCESS_SYMBOL})
            );
            return check_screen_recording_via_stream(&core_graphics_library);
        };

        let preflight_function: CGPreflightScreenCaptureAccessFn =
            unsafe { std::mem::transmute(symbol_pointer) };
        unsafe { preflight_function() }
    }

    fn check_screen_recording_via_stream(core_graphics_library: &DynamicLibraryHandle) -> bool {
        use std::ptr;

        type CreateStreamFn = unsafe extern "C" fn(
            u32,
            usize,
            usize,
            u32,
            *const std::ffi::c_void,
            *const std::ffi::c_void,
            *const std::ffi::c_void,
        ) -> *const std::ffi::c_void;

        let Some(symbol_pointer) = core_graphics_library.find_symbol(CG_DISPLAY_STREAM_SYMBOL)
        else {
            log::warn!(
                "{} {}",
                LOG_TAG_PERMISSIONS,
                serde_json::json!({"event": EVENT_DISPLAY_STREAM_SYMBOL_MISSING, "symbol_name": CG_DISPLAY_STREAM_SYMBOL})
            );
            return false;
        };

        let create_stream: CreateStreamFn = unsafe { std::mem::transmute(symbol_pointer) };
        let stream = unsafe { create_stream(0, 1, 1, 0, ptr::null(), ptr::null(), ptr::null()) };

        !stream.is_null()
    }

    fn open_system_preferences(permission_type: &str) -> bool {
        let pane = match permission_type {
            SCREEN_RECORDING_PERMISSION => SETTINGS_PANE_SCREEN_RECORDING,
            _ => SETTINGS_PANE_SECURITY,
        };

        let result = Command::new(SYSTEM_OPEN_COMMAND).arg(pane).status();

        match result {
            Ok(status) if status.success() => {
                log::info!(
                    "{} {}",
                    LOG_TAG_PERMISSIONS,
                    serde_json::json!({"event": EVENT_SYSTEM_PREFERENCES_OPENED, "permission_type": permission_type})
                );
                true
            }
            Ok(status) => {
                log::error!(
                    "{} {}",
                    LOG_TAG_PERMISSIONS,
                    serde_json::json!({"event": EVENT_SYSTEM_PREFERENCES_OPEN_FAILED, "exit_code": status.code()})
                );
                false
            }
            Err(error) => {
                log::error!(
                    "{} {}",
                    LOG_TAG_PERMISSIONS,
                    serde_json::json!({"event": EVENT_OPEN_COMMAND_EXECUTION_FAILED, "error": error.to_string()})
                );
                false
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub mod macos {
    pub fn check_screen_recording_permission() -> bool {
        true
    }

    pub fn request_screen_recording_permission() -> bool {
        true
    }

    pub fn open_screen_recording_settings() -> bool {
        false
    }
}
