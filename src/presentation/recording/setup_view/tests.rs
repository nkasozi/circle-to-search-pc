use super::*;

fn build_view() -> RecordingSetupView {
    let toggles = SetupToggles {
        camera: false,
        microphone: true,
        system_audio: false,
        click_highlights: true,
    };
    RecordingSetupView::build(Size::new(1000.0, 800.0), CaptureMode::Region, toggles)
}

#[test]
fn test_dragging_selects_a_normalized_region() {
    let mut view = build_view();
    view.update(&RecordingSetupMessage::PointerPressed(Point::new(
        300.0, 300.0,
    )));
    view.update(&RecordingSetupMessage::PointerMoved(Point::new(
        100.0, 150.0,
    )));
    view.update(&RecordingSetupMessage::PointerReleased);
    assert_eq!(
        view.selected_region,
        Some(Rectangle {
            x: 100.0,
            y: 150.0,
            width: 200.0,
            height: 150.0
        })
    );
    assert!(view.can_start());
}

#[test]
fn test_tiny_drag_keeps_previous_selection() {
    let mut view = build_view();
    view.selected_region = Some(Rectangle {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    });
    view.update(&RecordingSetupMessage::PointerPressed(Point::new(
        10.0, 10.0,
    )));
    view.update(&RecordingSetupMessage::PointerReleased);
    assert_eq!(view.selected_region.map(|region| region.width), Some(100.0));
}

#[test]
fn test_full_screen_mode_captures_whole_display() {
    let mut view = build_view();
    assert!(!view.can_start());
    view.update(&RecordingSetupMessage::SetMode(CaptureMode::FullScreen));
    assert_eq!(
        view.capture_rectangle(),
        Some(Rectangle::new(Point::ORIGIN, Size::new(1000.0, 800.0)))
    );
}

#[test]
fn test_dragging_switches_back_to_region_mode() {
    let mut view = build_view();
    view.mode = CaptureMode::FullScreen;
    view.update(&RecordingSetupMessage::PointerPressed(Point::new(
        10.0, 10.0,
    )));
    assert_eq!(view.mode, CaptureMode::Region);
}

#[test]
fn test_input_is_ignored_during_countdown() {
    let mut view = build_view();
    view.countdown_remaining = Some(3);
    assert!(!view.update(&RecordingSetupMessage::SetMode(CaptureMode::FullScreen)));
    assert!(!view.can_start());
}
