#![allow(deprecated)]

#[cfg(target_os = "macos")]
pub mod macos {
    use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy};
    use cocoa::base::nil;
    use objc::runtime::Object;

    const LOG_TAG: &str = "[MACOS_APP]";

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

    pub fn set_activation_policy(policy: ActivationPolicy) -> bool {
        log::info!("{} Setting activation policy to {:?}", LOG_TAG, policy);

        unsafe {
            let app = NSApp();
            if app == nil as *mut Object {
                log::error!("{} Failed to get NSApplication", LOG_TAG);
                return false;
            }

            let ns_policy: NSApplicationActivationPolicy = policy.into();
            let result = app.setActivationPolicy_(ns_policy);

            if result {
                log::info!("{} Activation policy set successfully", LOG_TAG);
            } else {
                log::warn!("{} Failed to set activation policy", LOG_TAG);
            }

            result
        }
    }

    pub fn hide_dock_icon() -> bool {
        log::info!("{} Hiding dock icon (setting Accessory policy)", LOG_TAG);
        set_activation_policy(ActivationPolicy::Accessory)
    }

    #[allow(dead_code)]
    pub fn show_dock_icon() -> bool {
        log::info!("{} Showing dock icon (setting Regular policy)", LOG_TAG);
        set_activation_policy(ActivationPolicy::Regular)
    }

    #[allow(dead_code)]
    pub fn activate_app_ignoring_other_apps() {
        log::info!("{} Activating app (bringing to front)", LOG_TAG);

        unsafe {
            let app = NSApp();
            if app == nil as *mut Object {
                log::error!("{} Failed to get NSApplication", LOG_TAG);
                return;
            }

            app.activateIgnoringOtherApps_(true);
        }
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

    pub fn activate_app_ignoring_other_apps() {}
}
