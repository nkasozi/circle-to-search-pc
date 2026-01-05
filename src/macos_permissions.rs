#![allow(unexpected_cfgs)]
#![allow(deprecated)]

#[cfg(target_os = "macos")]
pub mod macos {
    use core_foundation::base::TCFType;
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::CFString;
    use std::process::Command;

    const LOG_TAG_PERMISSIONS: &str = "[PERMISSIONS]";

    pub fn request_screen_recording_permission() -> bool {
        log::info!(
            "{} Checking screen recording permission",
            LOG_TAG_PERMISSIONS
        );

        let has_permission = check_screen_recording_permission();

        if !has_permission {
            log::warn!(
                "{} Screen recording permission not granted",
                LOG_TAG_PERMISSIONS
            );
            log::warn!(
                "{} Please grant Screen Recording permission in System Settings > Privacy & Security > Screen Recording",
                LOG_TAG_PERMISSIONS
            );
            show_permission_notification(
                "Screen Recording",
                "Circle to Search needs screen recording permission. Opening System Settings..."
            );
            open_system_preferences("Screen Recording");
            return false;
        }

        log::info!(
            "{} Screen recording permission granted",
            LOG_TAG_PERMISSIONS
        );
        true
    }

    fn check_screen_recording_permission() -> bool {
        use std::ptr;

        unsafe {
            let framework_path = std::ffi::CString::new(
                "/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics",
            )
            .unwrap();

            let lib = libc::dlopen(framework_path.as_ptr(), libc::RTLD_LAZY);

            if lib.is_null() {
                log::warn!(
                    "{} Could not load CoreGraphics framework",
                    LOG_TAG_PERMISSIONS
                );
                return true;
            }

            type CreateStreamFn = unsafe extern "C" fn(
                u32,
                usize,
                usize,
                u32,
                *const std::ffi::c_void,
                *const std::ffi::c_void,
                *const std::ffi::c_void,
            ) -> *const std::ffi::c_void;

            let func_name = std::ffi::CString::new("CGDisplayStreamCreateWithDispatchQueue").unwrap();
            let func_ptr = libc::dlsym(lib, func_name.as_ptr());

            if func_ptr.is_null() {
                libc::dlclose(lib);
                log::warn!(
                    "{} Could not find CGDisplayStreamCreateWithDispatchQueue",
                    LOG_TAG_PERMISSIONS
                );
                return true;
            }

            let func: CreateStreamFn = std::mem::transmute(func_ptr);
            let stream = func(0, 1, 1, 0, ptr::null(), ptr::null(), ptr::null());

            libc::dlclose(lib);

            !stream.is_null()
        }
    }

    pub fn request_accessibility_permission() -> bool {
        log::info!("{} Checking accessibility permission", LOG_TAG_PERMISSIONS);

        let has_permission = check_accessibility_permission(false);

        if !has_permission {
            log::warn!(
                "{} Accessibility permission not granted, triggering prompt",
                LOG_TAG_PERMISSIONS
            );

            check_accessibility_permission(true);

            log::warn!(
                "{} Please grant Accessibility permission in System Settings > Privacy & Security > Accessibility",
                LOG_TAG_PERMISSIONS
            );
            show_permission_notification(
                "Accessibility",
                "Circle to Search needs accessibility permission. Opening System Settings..."
            );
            open_system_preferences("Accessibility");
            return false;
        }

        log::info!("{} Accessibility permission granted", LOG_TAG_PERMISSIONS);
        true
    }

    fn check_accessibility_permission(prompt: bool) -> bool {
        use std::ffi::CString;
        use std::ptr;

        unsafe {
            let framework_path = CString::new(
                "/System/Library/Frameworks/ApplicationServices.framework/ApplicationServices",
            )
            .unwrap();

            let lib = libc::dlopen(framework_path.as_ptr(), libc::RTLD_LAZY);

            if lib.is_null() {
                log::warn!(
                    "{} Could not load ApplicationServices framework",
                    LOG_TAG_PERMISSIONS
                );
                return true;
            }

            type AXIsProcessTrustedWithOptionsFn = unsafe extern "C" fn(*const libc::c_void) -> bool;

            let func_name = CString::new("AXIsProcessTrustedWithOptions").unwrap();
            let func_ptr = libc::dlsym(lib, func_name.as_ptr());

            if func_ptr.is_null() {
                libc::dlclose(lib);
                log::warn!(
                    "{} Could not find AXIsProcessTrustedWithOptions",
                    LOG_TAG_PERMISSIONS
                );
                return true;
            }

            let ax_is_process_trusted: AXIsProcessTrustedWithOptionsFn =
                std::mem::transmute(func_ptr);

            let result = if prompt {
                let key = CFString::from_static_string("AXTrustedCheckOptionPrompt");
                let value = CFBoolean::true_value();
                let options = CFDictionary::from_CFType_pairs(&[(key, value.as_CFType())]);
                ax_is_process_trusted(options.as_concrete_TypeRef() as *const libc::c_void)
            } else {
                ax_is_process_trusted(ptr::null())
            };

            libc::dlclose(lib);

            result
        }
    }

    fn show_permission_notification(permission_name: &str, _message: &str) {
        log::info!(
            "{} Permission request for: {}",
            LOG_TAG_PERMISSIONS,
            permission_name
        );
    }

    fn open_system_preferences(permission_type: &str) {
        let pane = match permission_type {
            "Screen Recording" => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture"
            }
            "Accessibility" => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
            }
            _ => "x-apple.systempreferences:com.apple.preference.security",
        };

        let result = Command::new("open").arg(pane).status();

        match result {
            Ok(status) if status.success() => {
                log::info!(
                    "{} Opened system preferences for {}",
                    LOG_TAG_PERMISSIONS,
                    permission_type
                );
            }
            Ok(status) => {
                log::error!(
                    "{} Failed to open system preferences: exit code {:?}",
                    LOG_TAG_PERMISSIONS,
                    status.code()
                );
            }
            Err(error) => {
                log::error!(
                    "{} Failed to execute open command: {}",
                    LOG_TAG_PERMISSIONS,
                    error
                );
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub mod macos {
    pub fn request_screen_recording_permission() -> bool {
        true
    }

    pub fn request_accessibility_permission() -> bool {
        true
    }
}
