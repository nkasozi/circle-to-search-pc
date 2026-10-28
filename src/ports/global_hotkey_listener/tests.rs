use super::*;

fn build_registrations() -> Vec<HotkeyRegistration> {
    vec![
        HotkeyRegistration {
            hotkey_id: 11,
            action: GlobalHotkeyAction::Capture,
        },
        HotkeyRegistration {
            hotkey_id: 22,
            action: GlobalHotkeyAction::ToggleRecording,
        },
    ]
}

#[test]
fn test_resolve_pressed_action_matches_registered_id() {
    let event = GlobalHotKeyEvent {
        id: 22,
        state: HotKeyState::Pressed,
    };
    let action = resolve_pressed_action(&event, &build_registrations());
    assert_eq!(action, Some(GlobalHotkeyAction::ToggleRecording));
}

#[test]
fn test_resolve_pressed_action_ignores_release_events() {
    let event = GlobalHotKeyEvent {
        id: 11,
        state: HotKeyState::Released,
    };
    assert_eq!(resolve_pressed_action(&event, &build_registrations()), None);
}

#[test]
fn test_resolve_pressed_action_ignores_unknown_ids() {
    let event = GlobalHotKeyEvent {
        id: 99,
        state: HotKeyState::Pressed,
    };
    assert_eq!(resolve_pressed_action(&event, &build_registrations()), None);
}

#[test]
fn test_default_accelerators_parse() {
    assert!(HotKey::from_str("Alt+Shift+S").is_ok());
    assert!(HotKey::from_str("Alt+Shift+R").is_ok());
}
