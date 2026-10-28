use crate::global_constants;

const LINUX_PERMISSION_KEYWORDS: [&str; 5] = [
    global_constants::CAPTURE_ERROR_KEYWORD_PERMISSION,
    global_constants::CAPTURE_ERROR_KEYWORD_ACCESS,
    global_constants::CAPTURE_ERROR_KEYWORD_DENIED,
    global_constants::CAPTURE_ERROR_KEYWORD_PIPEWIRE,
    global_constants::CAPTURE_ERROR_KEYWORD_PORTAL,
];

const MACOS_PERMISSION_KEYWORDS: [&str; 3] = [
    global_constants::CAPTURE_ERROR_KEYWORD_PERMISSION,
    global_constants::CAPTURE_ERROR_KEYWORD_ACCESS,
    global_constants::CAPTURE_ERROR_KEYWORD_DENIED,
];

const LINUX_PERMISSION_AFFIXES: (&str, &str) = (
    global_constants::CAPTURE_ERROR_LINUX_PERMISSION_PREFIX,
    global_constants::CAPTURE_ERROR_LINUX_PERMISSION_SUFFIX,
);

const MACOS_PERMISSION_AFFIXES: (&str, &str) = (
    global_constants::CAPTURE_ERROR_MACOS_PERMISSION_PREFIX,
    global_constants::CAPTURE_ERROR_MACOS_PERMISSION_SUFFIX,
);

const GENERIC_AFFIXES: (&str, &str) = (
    global_constants::CAPTURE_ERROR_GENERIC_PREFIX,
    global_constants::CAPTURE_ERROR_GENERIC_SUFFIX,
);

struct PermissionErrorRule {
    keywords: &'static [&'static str],
    affixes: (&'static str, &'static str),
}

pub(super) fn build_capture_error_message(error_message: &str) -> String {
    #[cfg(target_os = "linux")]
    let platform = global_constants::CAPTURE_PLATFORM_LINUX;
    #[cfg(target_os = "macos")]
    let platform = global_constants::CAPTURE_PLATFORM_MACOS;
    #[cfg(target_os = "windows")]
    let platform = global_constants::CAPTURE_PLATFORM_WINDOWS;
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let platform = global_constants::CAPTURE_PLATFORM_UNKNOWN;

    build_capture_error_message_for_platform(error_message, platform)
}

pub(super) fn build_capture_error_message_for_platform(
    error_message: &str,
    platform: &str,
) -> String {
    let lowercase_error_message = error_message.to_lowercase();
    let permission_rule = select_permission_error_rule(platform);
    let (prefix, suffix) = select_error_affixes(&lowercase_error_message, permission_rule);
    format!("{}{}{}", prefix, error_message, suffix)
}

fn select_permission_error_rule(platform: &str) -> Option<PermissionErrorRule> {
    match platform {
        global_constants::CAPTURE_PLATFORM_LINUX => Some(PermissionErrorRule {
            keywords: &LINUX_PERMISSION_KEYWORDS,
            affixes: LINUX_PERMISSION_AFFIXES,
        }),
        global_constants::CAPTURE_PLATFORM_MACOS => Some(PermissionErrorRule {
            keywords: &MACOS_PERMISSION_KEYWORDS,
            affixes: MACOS_PERMISSION_AFFIXES,
        }),
        _ => None,
    }
}

fn select_error_affixes(
    lowercase_error_message: &str,
    permission_rule: Option<PermissionErrorRule>,
) -> (&'static str, &'static str) {
    match permission_rule {
        Some(rule) if contains_any_keyword(lowercase_error_message, rule.keywords) => rule.affixes,
        _ => GENERIC_AFFIXES,
    }
}

fn contains_any_keyword(lowercase_error_message: &str, keywords: &[&str]) -> bool {
    keywords
        .iter()
        .any(|keyword| lowercase_error_message.contains(keyword))
}
