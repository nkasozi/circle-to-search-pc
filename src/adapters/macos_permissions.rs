#![allow(unexpected_cfgs)]
#![allow(deprecated)]

#[cfg(target_os = "macos")]
pub mod macos {
    use core_foundation::base::TCFType;
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::CFString;
    use std::ffi::CString;
    use std::process::Command;

    const LOG_TAG_PERMISSIONS: &str = "[PERMISSIONS]";
    const SCREEN_RECORDING_PERMISSION: &str = "Screen Recording";
    const ACCESSIBILITY_PERMISSION: &str = "Accessibility";
    const INPUT_MONITORING_PERMISSION: &str = "Input Monitoring";
    const CORE_GRAPHICS_FRAMEWORK_PATH: &str =
        "/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics";
    const IOKIT_FRAMEWORK_PATH: &str = "/System/Library/Frameworks/IOKit.framework/IOKit";
    const APPLICATION_SERVICES_FRAMEWORK_PATH: &str =
        "/System/Library/Frameworks/ApplicationServices.framework/ApplicationServices";
    const CG_PREFLIGHT_ACCESS_SYMBOL: &str = "CGPreflightScreenCaptureAccess";
    const CG_DISPLAY_STREAM_SYMBOL: &str = "CGDisplayStreamCreateWithDispatchQueue";
    const IOHID_CHECK_ACCESS_SYMBOL: &str = "IOHIDCheckAccess";
    const AX_TRUSTED_OPTIONS_SYMBOL: &str = "AXIsProcessTrustedWithOptions";
    const AX_TRUSTED_CHECK_OPTION_PROMPT: &str = "AXTrustedCheckOptionPrompt";
    const SETTINGS_PANE_SCREEN_RECORDING: &str =
        "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture";
    const SETTINGS_PANE_ACCESSIBILITY: &str =
        "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";
    const SETTINGS_PANE_INPUT_MONITORING: &str =
        "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent";
    const SETTINGS_PANE_SECURITY: &str = "x-apple.systempreferences:com.apple.preference.security";
    const SYSTEM_OPEN_COMMAND: &str = "open";

    struct DynamicLibraryHandle {
        raw_handle: *mut libc::c_void,
    }

    impl DynamicLibraryHandle {
        fn open(path: &'static str) -> Option<Self> {
            let framework_path = build_c_string(path)?;
            let raw_handle = unsafe { libc::dlopen(framework_path.as_ptr(), libc::RTLD_LAZY) };

            if raw_handle.is_null() {
                return None;
            }

            Some(Self { raw_handle })
        }

        fn find_symbol(&self, symbol_name: &'static str) -> Option<*mut libc::c_void> {
            let function_name = build_c_string(symbol_name)?;
            let symbol_pointer = unsafe { libc::dlsym(self.raw_handle, function_name.as_ptr()) };

            if symbol_pointer.is_null() {
                return None;
            }

            Some(symbol_pointer)
        }
    }

    impl Drop for DynamicLibraryHandle {
        fn drop(&mut self) {
            if self.raw_handle.is_null() {
                return;
            }

            unsafe {
                libc::dlclose(self.raw_handle);
            }
        }
    }

    pub fn check_screen_recording_permission() -> bool {
        log::info!(
            "{} Checking screen recording permission",
            LOG_TAG_PERMISSIONS
        );

        let has_permission = check_screen_recording_permission_internal();

        if has_permission {
            log::info!(
                "{} Screen recording permission granted",
                LOG_TAG_PERMISSIONS
            );
        } else {
            log::warn!(
                "{} Screen recording permission not granted",
                LOG_TAG_PERMISSIONS
            );
        }

        has_permission
    }

    #[allow(dead_code)]
    pub fn check_accessibility_permission() -> bool {
        log::info!("{} Checking accessibility permission", LOG_TAG_PERMISSIONS);

        let has_permission = check_accessibility_permission_internal(false);

        if has_permission {
            log::info!("{} Accessibility permission granted", LOG_TAG_PERMISSIONS);
        } else {
            log::warn!(
                "{} Accessibility permission not granted",
                LOG_TAG_PERMISSIONS
            );
        }

        has_permission
    }

    pub fn open_screen_recording_settings() -> bool {
        log::info!("{} Opening screen recording settings", LOG_TAG_PERMISSIONS);
        open_system_preferences(SCREEN_RECORDING_PERMISSION)
    }

    #[allow(dead_code)]
    pub fn open_accessibility_settings() -> bool {
        log::info!("{} Opening accessibility settings", LOG_TAG_PERMISSIONS);
        open_system_preferences(ACCESSIBILITY_PERMISSION)
    }

    pub fn check_input_monitoring_permission() -> bool {
        log::info!(
            "{} Checking input monitoring permission",
            LOG_TAG_PERMISSIONS
        );
        let has_permission = check_input_monitoring_permission_internal();

        if has_permission {
            log::info!(
                "{} Input monitoring permission granted",
                LOG_TAG_PERMISSIONS
            );
        } else {
            log::warn!(
                "{} Input monitoring permission not granted",
                LOG_TAG_PERMISSIONS
            );
        }

        has_permission
    }

    pub fn open_input_monitoring_settings() -> bool {
        log::info!("{} Opening input monitoring settings", LOG_TAG_PERMISSIONS);
        open_system_preferences(INPUT_MONITORING_PERMISSION)
    }

    fn build_c_string(value: &'static str) -> Option<CString> {
        match CString::new(value) {
            Ok(c_string) => Some(c_string),
            Err(error) => {
                log::error!("{} Failed to build CString: {}", LOG_TAG_PERMISSIONS, error);
                None
            }
        }
    }

    fn check_screen_recording_permission_internal() -> bool {
        let Some(core_graphics_library) = DynamicLibraryHandle::open(CORE_GRAPHICS_FRAMEWORK_PATH)
        else {
            log::warn!(
                "{} Could not load CoreGraphics framework",
                LOG_TAG_PERMISSIONS
            );
            return false;
        };

        type CGPreflightScreenCaptureAccessFn = unsafe extern "C" fn() -> bool;

        let Some(symbol_pointer) = core_graphics_library.find_symbol(CG_PREFLIGHT_ACCESS_SYMBOL)
        else {
            log::warn!(
                "{} Could not find CGPreflightScreenCaptureAccess, falling back to stream check",
                LOG_TAG_PERMISSIONS
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
                "{} Could not find CGDisplayStreamCreateWithDispatchQueue",
                LOG_TAG_PERMISSIONS
            );
            return false;
        };

        let create_stream: CreateStreamFn = unsafe { std::mem::transmute(symbol_pointer) };
        let stream = unsafe { create_stream(0, 1, 1, 0, ptr::null(), ptr::null(), ptr::null()) };

        !stream.is_null()
    }

    fn check_input_monitoring_permission_internal() -> bool {
        let Some(iokit_library) = DynamicLibraryHandle::open(IOKIT_FRAMEWORK_PATH) else {
            log::warn!("{} Could not load IOKit framework", LOG_TAG_PERMISSIONS);
            return check_accessibility_permission_internal(false);
        };

        type IOHIDCheckAccessFn = unsafe extern "C" fn(u32) -> u32;

        let Some(symbol_pointer) = iokit_library.find_symbol(IOHID_CHECK_ACCESS_SYMBOL) else {
            log::warn!(
                "{} Could not find IOHIDCheckAccess, falling back to accessibility check",
                LOG_TAG_PERMISSIONS
            );
            return check_accessibility_permission_internal(false);
        };

        let check_access_function: IOHIDCheckAccessFn =
            unsafe { std::mem::transmute(symbol_pointer) };

        let k_iohid_request_type_listen_event: u32 = 1;
        let k_iohid_access_type_granted: u32 = 0;

        let result = unsafe { check_access_function(k_iohid_request_type_listen_event) };

        log::debug!(
            "{} IOHIDCheckAccess returned: {} (0=granted, 1=denied, 2=unknown)",
            LOG_TAG_PERMISSIONS,
            result
        );

        result == k_iohid_access_type_granted
    }

    fn check_accessibility_permission_internal(prompt: bool) -> bool {
        use std::ptr;

        let Some(application_services_library) =
            DynamicLibraryHandle::open(APPLICATION_SERVICES_FRAMEWORK_PATH)
        else {
            log::warn!(
                "{} Could not load ApplicationServices framework",
                LOG_TAG_PERMISSIONS
            );
            return true;
        };

        type AXIsProcessTrustedWithOptionsFn = unsafe extern "C" fn(*const libc::c_void) -> bool;

        let Some(symbol_pointer) =
            application_services_library.find_symbol(AX_TRUSTED_OPTIONS_SYMBOL)
        else {
            log::warn!(
                "{} Could not find AXIsProcessTrustedWithOptions",
                LOG_TAG_PERMISSIONS
            );
            return true;
        };

        let ax_is_process_trusted: AXIsProcessTrustedWithOptionsFn =
            unsafe { std::mem::transmute(symbol_pointer) };

        if prompt {
            let key = CFString::from_static_string(AX_TRUSTED_CHECK_OPTION_PROMPT);
            let value = CFBoolean::true_value();
            let options = CFDictionary::from_CFType_pairs(&[(key, value.as_CFType())]);
            return unsafe {
                ax_is_process_trusted(options.as_concrete_TypeRef() as *const libc::c_void)
            };
        }

        unsafe { ax_is_process_trusted(ptr::null()) }
    }

    fn open_system_preferences(permission_type: &str) -> bool {
        let pane = match permission_type {
            SCREEN_RECORDING_PERMISSION => SETTINGS_PANE_SCREEN_RECORDING,
            ACCESSIBILITY_PERMISSION => SETTINGS_PANE_ACCESSIBILITY,
            INPUT_MONITORING_PERMISSION => SETTINGS_PANE_INPUT_MONITORING,
            _ => SETTINGS_PANE_SECURITY,
        };

        let result = Command::new(SYSTEM_OPEN_COMMAND).arg(pane).status();

        match result {
            Ok(status) if status.success() => {
                log::info!(
                    "{} Opened system preferences for {}",
                    LOG_TAG_PERMISSIONS,
                    permission_type
                );
                true
            }
            Ok(status) => {
                log::error!(
                    "{} Failed to open system preferences: exit code {:?}",
                    LOG_TAG_PERMISSIONS,
                    status.code()
                );
                false
            }
            Err(error) => {
                log::error!(
                    "{} Failed to execute open command: {}",
                    LOG_TAG_PERMISSIONS,
                    error
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

    pub fn check_accessibility_permission() -> bool {
        true
    }

    pub fn check_input_monitoring_permission() -> bool {
        true
    }

    pub fn open_screen_recording_settings() -> bool {
        false
    }

    pub fn open_accessibility_settings() -> bool {
        false
    }

    pub fn open_input_monitoring_settings() -> bool {
        false
    }
}
