#[cfg(test)]
mod tests {
    use crate::system_tray::TrayEvent;

    #[test]
    fn test_tray_event_debug_implements() {
        let event = TrayEvent::ShowWindow;
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("ShowWindow"));
    }

    #[test]
    fn test_tray_event_clone() {
        let event = TrayEvent::ShowWindow;
        let cloned = event.clone();

        matches!(cloned, TrayEvent::ShowWindow);
    }

    #[test]
    fn test_all_tray_event_variants_are_cloneable() {
        let show_window = TrayEvent::ShowWindow;
        let settings = TrayEvent::OpenSettings;
        let quit = TrayEvent::Quit;

        let _cloned1 = show_window.clone();
        let _cloned2 = settings.clone();
        let _cloned3 = quit.clone();
    }
}
