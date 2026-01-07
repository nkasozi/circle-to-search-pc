use std::sync::OnceLock;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

static SHOW_WINDOW_ID: OnceLock<MenuId> = OnceLock::new();
static SETTINGS_ID: OnceLock<MenuId> = OnceLock::new();
static QUIT_ID: OnceLock<MenuId> = OnceLock::new();

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

        let _ = SHOW_WINDOW_ID.set(show_window_item.id().clone());
        let _ = SETTINGS_ID.set(settings_item.id().clone());
        let _ = QUIT_ID.set(quit_item.id().clone());

        log::info!(
            "[SYSTEM_TRAY] Menu item IDs - Show: {:?}, Settings: {:?}, Quit: {:?}",
            show_window_item.id(),
            settings_item.id(),
            quit_item.id()
        );

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
            log::info!("[SYSTEM_TRAY] Received menu event: {:?}", event.id);
            return TrayEvent::from_menu_event(&event);
        }
        None
    }
}

impl TrayEvent {
    fn from_menu_event(event: &MenuEvent) -> Option<Self> {
        let event_id = &event.id;

        if SHOW_WINDOW_ID
            .get()
            .map(|id| id == event_id)
            .unwrap_or(false)
        {
            log::info!("[SYSTEM_TRAY] Show Window clicked");
            return Some(TrayEvent::ShowWindow);
        }

        if SETTINGS_ID.get().map(|id| id == event_id).unwrap_or(false) {
            log::info!("[SYSTEM_TRAY] Settings clicked");
            return Some(TrayEvent::OpenSettings);
        }

        if QUIT_ID.get().map(|id| id == event_id).unwrap_or(false) {
            log::info!("[SYSTEM_TRAY] Quit clicked");
            return Some(TrayEvent::Quit);
        }

        log::warn!("[SYSTEM_TRAY] Unknown menu event ID: {:?}", event_id);
        None
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
