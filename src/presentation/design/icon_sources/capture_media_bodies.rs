pub const SEARCH_BODY: &str = r#"<circle cx="11" cy="11" r="7"/><path d="M20 20l-4-4"/>"#;
pub const SCAN_TEXT_BODY: &str = r#"<path d="M4 8V6a2 2 0 0 1 2-2h2M16 4h2a2 2 0 0 1 2 2v2M20 16v2a2 2 0 0 1-2 2h-2M8 20H6a2 2 0 0 1-2-2v-2"/><path d="M8 10h8M8 14h5"/>"#;
pub const LASSO_BODY: &str = r#"<path d="M12 5c5 0 9 2.7 9 6s-4 6-9 6-9-2.7-9-6 4-6 9-6z"/><path d="M6.5 15.5c-1.2 1.6-1 3.6 1.2 4.5"/>"#;
pub const REGION_BODY: &str = r#"<path d="M4 7V5a1 1 0 0 1 1-1h2M11 4h2M17 4h2a1 1 0 0 1 1 1v2M20 11v2M20 17v2a1 1 0 0 1-1 1h-2M13 20h-2M7 20H5a1 1 0 0 1-1-1v-2M4 13v-2"/>"#;
pub const CROP_BODY: &str =
    r#"<path d="M6 2v14a2 2 0 0 0 2 2h14"/><path d="M18 22V8a2 2 0 0 0-2-2H2"/>"#;
pub const RECORD_BODY: &str = r#"<circle cx="12" cy="12" r="6.5" fill="black" stroke="none"/>"#;
pub const STOP_BODY: &str =
    r#"<rect x="6" y="6" width="12" height="12" rx="2.5" fill="black" stroke="none"/>"#;
pub const PAUSE_BODY: &str = r#"<rect x="6.5" y="5" width="3.6" height="14" rx="1.2" fill="black" stroke="none"/><rect x="13.9" y="5" width="3.6" height="14" rx="1.2" fill="black" stroke="none"/>"#;
pub const PLAY_BODY: &str = r#"<path d="M8 5.5v13a1 1 0 0 0 1.5.86l10.4-6.5a1 1 0 0 0 0-1.72L9.5 4.64A1 1 0 0 0 8 5.5z" fill="black" stroke="none"/>"#;
pub const CAMERA_BODY: &str =
    r#"<rect x="3" y="6" width="13" height="12" rx="2.5"/><path d="M16 10.5l5-3v9l-5-3"/>"#;
pub const CAMERA_OFF_BODY: &str = r#"<rect x="3" y="6" width="13" height="12" rx="2.5"/><path d="M16 10.5l5-3v9l-5-3"/><path d="M3 3l18 18"/>"#;
pub const MICROPHONE_BODY: &str = r#"<rect x="9" y="3" width="6" height="11" rx="3"/><path d="M5 11a7 7 0 0 0 14 0M12 18v3M9 21h6"/>"#;
pub const MICROPHONE_OFF_BODY: &str = r#"<rect x="9" y="3" width="6" height="11" rx="3"/><path d="M5 11a7 7 0 0 0 14 0M12 18v3M9 21h6"/><path d="M3 3l18 18"/>"#;
pub const SPEAKER_BODY: &str =
    r#"<path d="M4 9v6h4l5 4V5L8 9z"/><path d="M16.5 8.5a5 5 0 0 1 0 7M19 6a8.5 8.5 0 0 1 0 12"/>"#;
pub const SPEAKER_OFF_BODY: &str =
    r#"<path d="M4 9v6h4l5 4V5L8 9z"/><path d="M17 9l5 6M22 9l-5 6"/>"#;
pub const MONITOR_BODY: &str =
    r#"<rect x="3" y="4" width="18" height="12" rx="2"/><path d="M8 20h8M12 16v4"/>"#;
pub const APP_WINDOW_BODY: &str =
    r#"<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 9h18"/>"#;
pub const FILM_BODY: &str = r#"<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M7 4v16M17 4v16M3 9h4M3 15h4M17 9h4M17 15h4"/>"#;
pub const TIMER_BODY: &str = r#"<circle cx="12" cy="13" r="8"/><path d="M12 9v4l2.5 2.5M9 2h6"/>"#;
pub const SPOTLIGHT_BODY: &str =
    r#"<circle cx="12" cy="12" r="3"/><circle cx="12" cy="12" r="8.5" stroke-dasharray="2.5 3"/>"#;
