use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub struct SystemTray {
    _tray_icon: TrayIcon,
    _menu: Menu,
    _show_window_item: MenuItem,
    _settings_item: MenuItem,
    _quit_item: MenuItem,
}

#[derive(Debug, Clone)]
pub enum TrayEvent {
    ShowWindow,
    OpenSettings,
    Quit,
}

impl SystemTray {
    pub fn build() -> anyhow::Result<Self> {
        log::info!("[SYSTEM_TRAY] Initializing system tray");

        let icon_bytes = include_bytes!("../assets/tray_icon.png");
        let icon_image = image::load_from_memory(icon_bytes)?;
        let icon_rgba = icon_image.to_rgba8();
        let (width, height) = icon_rgba.dimensions();

        let icon = Icon::from_rgba(icon_rgba.into_raw(), width, height)?;

        let menu = Menu::new();
        let show_window_item = MenuItem::new("Show Window", true, None);
        let settings_item = MenuItem::new("Settings", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        menu.append(&show_window_item)?;
        menu.append(&settings_item)?;
        menu.append(&quit_item)?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_tooltip("Circle to Search")
            .with_icon(icon)
            .build()?;

        log::info!("[SYSTEM_TRAY] System tray initialized successfully");

        Ok(Self {
            _tray_icon: tray_icon,
            _menu: menu,
            _show_window_item: show_window_item,
            _settings_item: settings_item,
            _quit_item: quit_item,
        })
    }

    pub fn poll_events() -> Option<TrayEvent> {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            log::debug!("[SYSTEM_TRAY] Received menu event: {:?}", event.id);
            return Some(TrayEvent::from_menu_event(&event));
        }
        None
    }
}

impl TrayEvent {
    fn from_menu_event(event: &MenuEvent) -> Self {
        let event_id: &str = event.id.0.as_ref();
        log::debug!("[SYSTEM_TRAY] Processing menu event ID: {}", event_id);

        if event_id.contains("Show Window") {
            TrayEvent::ShowWindow
        } else if event_id.contains("Settings") {
            TrayEvent::OpenSettings
        } else if event_id.contains("Quit") {
            TrayEvent::Quit
        } else {
            log::warn!("[SYSTEM_TRAY] Unknown menu event: {}", event_id);
            TrayEvent::ShowWindow
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TrayEvent;

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
