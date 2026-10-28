use super::*;

mod filtering;

fn create_test_window(id: u32, app_name: &str, title: &str) -> WindowInfo {
    WindowInfo {
        id,
        app_name: app_name.to_string(),
        title: title.to_string(),
        width: 800,
        height: 600,
        is_minimized: false,
        thumbnail: None,
    }
}

#[test]
fn test_build_creates_view_with_windows() {
    let windows = vec![
        create_test_window(1, "Chrome", "Google"),
        create_test_window(2, "Firefox", "Mozilla"),
    ];
    let view = WindowPickerView::build(windows);

    assert_eq!(view.windows.len(), 2);
    assert!(view.selected_window_id.is_none());
    assert!(!view.is_loading);
    assert!(view.filter_query.is_empty());
}

#[test]
fn test_build_creates_empty_view() {
    let view = WindowPickerView::build(vec![]);

    assert!(view.windows.is_empty());
}

#[test]
fn test_set_windows_updates_list() {
    let mut view = WindowPickerView::build(vec![]);
    let windows = vec![create_test_window(1, "App", "Window")];

    view.set_windows(windows);

    assert_eq!(view.windows.len(), 1);
    assert!(!view.is_loading);
}

#[test]
fn test_set_loading_updates_state() {
    let mut view = WindowPickerView::build(vec![]);

    view.set_loading(true);
    assert!(view.is_loading);

    view.set_loading(false);
    assert!(!view.is_loading);
}

#[test]
fn test_update_window_selected_sets_id() {
    let windows = vec![create_test_window(42, "App", "Window")];
    let mut view = WindowPickerView::build(windows);

    view.update(WindowPickerMessage::WindowSelected(42));

    assert_eq!(view.selected_window_id, Some(42));
}

#[test]
fn test_update_refresh_windows_sets_loading() {
    let mut view = WindowPickerView::build(vec![]);
    view.filter_query = "test".to_string();

    view.update(WindowPickerMessage::RefreshWindows);

    assert!(view.is_loading);
    assert_eq!(view.spinner_frame, 0);
    assert!(view.filter_query.is_empty());
}

#[test]
fn test_update_spinner_tick_increments_frame() {
    let mut view = WindowPickerView::build(vec![]);
    view.is_loading = true;

    view.update(WindowPickerMessage::SpinnerTick);
    assert_eq!(view.spinner_frame, 1);

    view.update(WindowPickerMessage::SpinnerTick);
    assert_eq!(view.spinner_frame, 2);
}

#[test]
fn test_update_spinner_tick_wraps_around() {
    let mut view = WindowPickerView::build(vec![]);
    view.is_loading = true;
    view.spinner_frame = 7;

    view.update(WindowPickerMessage::SpinnerTick);

    assert_eq!(view.spinner_frame, 0);
}

#[test]
fn test_update_spinner_tick_does_nothing_when_not_loading() {
    let mut view = WindowPickerView::build(vec![]);
    view.is_loading = false;
    view.spinner_frame = 3;

    view.update(WindowPickerMessage::SpinnerTick);

    assert_eq!(view.spinner_frame, 3);
}

#[test]
fn test_update_filter_changed_updates_query() {
    let mut view = WindowPickerView::build(vec![]);

    view.update(WindowPickerMessage::FilterChanged("chrome".to_string()));

    assert_eq!(view.filter_query, "chrome");
}
