#![allow(deprecated)]

#[cfg(target_os = "macos")]
pub mod macos {
    use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy};
    use cocoa::base::nil;
    use objc::runtime::Object;

    const LOG_TAG: &str = "[MACOS_APP]";
    const ACTIVATION_POLICY_UPDATE_REQUESTED_EVENT: &str = "activation_policy_update_requested";
    const NS_APPLICATION_UNAVAILABLE_EVENT: &str = "ns_application_unavailable";
    const ACTIVATION_POLICY_UPDATED_EVENT: &str = "activation_policy_updated";
    const ACTIVATION_POLICY_UPDATE_FAILED_EVENT: &str = "activation_policy_update_failed";
    const DOCK_ICON_HIDING_EVENT: &str = "dock_icon_hiding";
    const DOCK_ICON_SHOWING_EVENT: &str = "dock_icon_showing";
    const APP_ACTIVATION_REQUESTED_EVENT: &str = "app_activation_requested";

    #[repr(u64)]
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum ActivationPolicy {
        Regular = 0,
        Accessory = 1,
        Prohibited = 2,
    }

    impl From<ActivationPolicy> for NSApplicationActivationPolicy {
        fn from(policy: ActivationPolicy) -> Self {
            match policy {
                ActivationPolicy::Regular => {
                    NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular
                }
                ActivationPolicy::Accessory => {
                    NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory
                }
                ActivationPolicy::Prohibited => {
                    NSApplicationActivationPolicy::NSApplicationActivationPolicyProhibited
                }
            }
        }
    }

    fn get_ns_application() -> Option<*mut Object> {
        let application = unsafe { NSApp() };
        if application == nil as *mut Object {
            return None;
        }
        Some(application)
    }

    pub fn set_activation_policy(policy: ActivationPolicy) -> bool {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": ACTIVATION_POLICY_UPDATE_REQUESTED_EVENT, "activation_policy": format!("{:?}", policy)})
        );

        let Some(application) = get_ns_application() else {
            log::error!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": NS_APPLICATION_UNAVAILABLE_EVENT})
            );
            return false;
        };

        let native_policy: NSApplicationActivationPolicy = policy.into();
        let result = unsafe { application.setActivationPolicy_(native_policy) };

        match result {
            true => log::info!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": ACTIVATION_POLICY_UPDATED_EVENT, "activation_policy": format!("{:?}", policy)})
            ),
            false => log::warn!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": ACTIVATION_POLICY_UPDATE_FAILED_EVENT, "activation_policy": format!("{:?}", policy)})
            ),
        }

        result
    }

    pub fn hide_dock_icon() -> bool {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": DOCK_ICON_HIDING_EVENT})
        );
        set_activation_policy(ActivationPolicy::Accessory)
    }

    #[allow(dead_code)]
    pub fn show_dock_icon() -> bool {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": DOCK_ICON_SHOWING_EVENT})
        );
        set_activation_policy(ActivationPolicy::Regular)
    }

    #[allow(dead_code)]
    pub fn activate_app_ignoring_other_apps() -> bool {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": APP_ACTIVATION_REQUESTED_EVENT})
        );

        let Some(application) = get_ns_application() else {
            log::error!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": NS_APPLICATION_UNAVAILABLE_EVENT})
            );
            return false;
        };

        unsafe {
            application.activateIgnoringOtherApps_(true);
        }
        true
    }
}

#[cfg(not(target_os = "macos"))]
pub mod macos {
    #[repr(u64)]
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum ActivationPolicy {
        Regular = 0,
        Accessory = 1,
        Prohibited = 2,
    }

    pub fn set_activation_policy(_policy: ActivationPolicy) -> bool {
        true
    }

    pub fn hide_dock_icon() -> bool {
        true
    }

    pub fn show_dock_icon() -> bool {
        true
    }

    pub fn activate_app_ignoring_other_apps() -> bool {
        false
    }
}
