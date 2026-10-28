use super::*;

#[test]
fn test_get_filtered_windows_returns_all_when_empty_query() {
    let windows = vec![
        create_test_window(1, "Chrome", "Google"),
        create_test_window(2, "Firefox", "Mozilla"),
    ];
    let view = WindowPickerView::build(windows);

    let filtered = view.get_filtered_windows();

    assert_eq!(filtered.len(), 2);
}

#[test]
fn test_get_filtered_windows_filters_by_app_name() {
    let windows = vec![
        create_test_window(1, "Chrome", "Google"),
        create_test_window(2, "Firefox", "Mozilla"),
        create_test_window(3, "Chrome", "GitHub"),
    ];
    let mut view = WindowPickerView::build(windows);
    view.filter_query = "chrome".to_string();

    let filtered = view.get_filtered_windows();

    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|w| w.app_name == "Chrome"));
}

#[test]
fn test_get_filtered_windows_filters_by_title() {
    let windows = vec![
        create_test_window(1, "Chrome", "Google Search"),
        create_test_window(2, "Firefox", "Google Maps"),
        create_test_window(3, "Safari", "Apple"),
    ];
    let mut view = WindowPickerView::build(windows);
    view.filter_query = "google".to_string();

    let filtered = view.get_filtered_windows();

    assert_eq!(filtered.len(), 2);
}

#[test]
fn test_get_filtered_windows_case_insensitive() {
    let windows = vec![
        create_test_window(1, "CHROME", "Google"),
        create_test_window(2, "chrome", "Test"),
        create_test_window(3, "Firefox", "Chrome Page"),
    ];
    let mut view = WindowPickerView::build(windows);
    view.filter_query = "CHROME".to_string();

    let filtered = view.get_filtered_windows();

    assert_eq!(filtered.len(), 3);
}

#[test]
fn test_get_filtered_windows_returns_empty_when_no_match() {
    let windows = vec![
        create_test_window(1, "Chrome", "Google"),
        create_test_window(2, "Firefox", "Mozilla"),
    ];
    let mut view = WindowPickerView::build(windows);
    view.filter_query = "safari".to_string();

    let filtered = view.get_filtered_windows();

    assert!(filtered.is_empty());
}

#[test]
fn test_get_selected_window_id_returns_none_initially() {
    let view = WindowPickerView::build(vec![]);

    assert!(view.get_selected_window_id().is_none());
}

#[test]
fn test_get_selected_window_id_returns_selected() {
    let windows = vec![create_test_window(99, "App", "Window")];
    let mut view = WindowPickerView::build(windows);
    view.update(WindowPickerMessage::WindowSelected(99));

    assert_eq!(view.get_selected_window_id(), Some(99));
}

#[test]
fn test_get_selected_window_info_returns_none_when_not_selected() {
    let windows = vec![create_test_window(1, "App", "Window")];
    let view = WindowPickerView::build(windows);

    assert!(view.get_selected_window_info().is_none());
}

#[test]
fn test_get_selected_window_info_returns_window() {
    let windows = vec![
        create_test_window(1, "Chrome", "Google"),
        create_test_window(2, "Firefox", "Mozilla"),
    ];
    let mut view = WindowPickerView::build(windows);
    view.update(WindowPickerMessage::WindowSelected(2));

    let info = view.get_selected_window_info();

    assert!(info.is_some());
    assert_eq!(info.unwrap().app_name, "Firefox");
}

#[test]
fn test_get_selected_window_info_returns_none_for_invalid_id() {
    let windows = vec![create_test_window(1, "App", "Window")];
    let mut view = WindowPickerView::build(windows);
    view.selected_window_id = Some(999);

    assert!(view.get_selected_window_info().is_none());
}
