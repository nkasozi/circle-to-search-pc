use std::ffi::CString;

const LOG_TAG_DYNAMIC_LIBRARY: &str = "[PERMISSIONS]";
const C_STRING_BUILD_FAILED_EVENT: &str = "c_string_build_failed";

pub(super) struct DynamicLibraryHandle {
    raw_handle: *mut libc::c_void,
}

impl DynamicLibraryHandle {
    pub(super) fn open(path: &'static str) -> Option<Self> {
        let framework_path = build_c_string(path)?;
        let raw_handle = unsafe { libc::dlopen(framework_path.as_ptr(), libc::RTLD_LAZY) };

        if raw_handle.is_null() {
            return None;
        }

        Some(Self { raw_handle })
    }

    pub(super) fn find_symbol(&self, symbol_name: &'static str) -> Option<*mut libc::c_void> {
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

fn build_c_string(value: &'static str) -> Option<CString> {
    match CString::new(value) {
        Ok(c_string) => Some(c_string),
        Err(error) => {
            log::error!(
                "{} {}",
                LOG_TAG_DYNAMIC_LIBRARY,
                serde_json::json!({"event": C_STRING_BUILD_FAILED_EVENT, "value": value, "error": error.to_string()})
            );
            None
        }
    }
}
