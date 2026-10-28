use std::str::FromStr;

use tray_icon::menu::accelerator::Accelerator;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::global_constants;

const SEARCH_ITEM_ID: &str = "tray-search";
const RECORD_ITEM_ID: &str = "tray-record";
const HOME_ITEM_ID: &str = "tray-home";
const SETTINGS_ITEM_ID: &str = "tray-settings";
const QUIT_ITEM_ID: &str = "tray-quit";
const SEARCH_ITEM_LABEL: &str = "Search Screen";
const RECORD_ITEM_LABEL: &str = "Record Screen";
const STOP_RECORDING_ITEM_LABEL: &str = "Stop Recording";
const HOME_ITEM_LABEL: &str = "Open Circle to Search";
const SETTINGS_ITEM_LABEL: &str = "Settings…";
const QUIT_ITEM_LABEL: &str = "Quit Circle to Search";
const LOG_TAG: &str = "[SYSTEM_TRAY]";
const EVENT_SYSTEM_TRAY_CREATED: &str = "system_tray_created";
const EVENT_TRAY_MENU_SELECTED: &str = "tray_menu_selected";

pub struct SystemTray {
    _tray_icon: TrayIcon,
    _menu: Menu,
    record_item: MenuItem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    SearchScreen,
    ToggleRecording,
    ShowWindow,
    OpenSettings,
    Quit,
}

pub struct TrayShortcuts<'a> {
    pub capture_hotkey: &'a str,
    pub recording_hotkey: &'a str,
}

impl SystemTray {
    pub fn build(shortcuts: TrayShortcuts<'_>) -> anyhow::Result<Self> {
        let icon = load_tray_icon()?;
        let (menu, record_item) = build_tray_menu(&shortcuts)?;
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_tooltip(global_constants::APPLICATION_TITLE)
            .with_icon(icon)
            .with_icon_as_template(true)
            .build()?;
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": EVENT_SYSTEM_TRAY_CREATED})
        );
        Ok(Self {
            _tray_icon: tray_icon,
            _menu: menu,
            record_item,
        })
    }

    pub fn show_recording_state(&self, is_recording: bool) -> bool {
        let label = match is_recording {
            true => STOP_RECORDING_ITEM_LABEL,
            false => RECORD_ITEM_LABEL,
        };
        self.record_item.set_text(label);
        is_recording
    }

    pub fn poll_events() -> Option<TrayEvent> {
        let event = MenuEvent::receiver().try_recv().ok()?;
        let tray_event = resolve_tray_event(event.id.as_ref());
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": EVENT_TRAY_MENU_SELECTED, "id": event.id.as_ref(), "resolved": format!("{:?}", tray_event)})
        );
        tray_event
    }
}

fn load_tray_icon() -> anyhow::Result<Icon> {
    let icon_rgba = image::load_from_memory(include_bytes!("../assets/tray_icon.png"))?.to_rgba8();
    let (width, height) = icon_rgba.dimensions();
    Ok(Icon::from_rgba(icon_rgba.into_raw(), width, height)?)
}

fn build_tray_menu(shortcuts: &TrayShortcuts<'_>) -> anyhow::Result<(Menu, MenuItem)> {
    let search_item = MenuItem::with_id(
        SEARCH_ITEM_ID,
        SEARCH_ITEM_LABEL,
        true,
        parse_accelerator(shortcuts.capture_hotkey),
    );
    let record_item = MenuItem::with_id(
        RECORD_ITEM_ID,
        RECORD_ITEM_LABEL,
        true,
        parse_accelerator(shortcuts.recording_hotkey),
    );
    let home_item = MenuItem::with_id(HOME_ITEM_ID, HOME_ITEM_LABEL, true, None);
    let settings_item = MenuItem::with_id(SETTINGS_ITEM_ID, SETTINGS_ITEM_LABEL, true, None);
    let quit_item = MenuItem::with_id(QUIT_ITEM_ID, QUIT_ITEM_LABEL, true, None);
    let menu = Menu::new();
    menu.append_items(&[
        &search_item,
        &record_item,
        &PredefinedMenuItem::separator(),
        &home_item,
        &settings_item,
        &PredefinedMenuItem::separator(),
        &quit_item,
    ])?;
    Ok((menu, record_item))
}

fn parse_accelerator(accelerator: &str) -> Option<Accelerator> {
    Accelerator::from_str(accelerator).ok()
}

pub fn resolve_tray_event(menu_item_id: &str) -> Option<TrayEvent> {
    match menu_item_id {
        SEARCH_ITEM_ID => Some(TrayEvent::SearchScreen),
        RECORD_ITEM_ID => Some(TrayEvent::ToggleRecording),
        HOME_ITEM_ID => Some(TrayEvent::ShowWindow),
        SETTINGS_ITEM_ID => Some(TrayEvent::OpenSettings),
        QUIT_ITEM_ID => Some(TrayEvent::Quit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_ids_resolve_to_events() {
        assert_eq!(
            resolve_tray_event(SEARCH_ITEM_ID),
            Some(TrayEvent::SearchScreen)
        );
        assert_eq!(
            resolve_tray_event(RECORD_ITEM_ID),
            Some(TrayEvent::ToggleRecording)
        );
        assert_eq!(resolve_tray_event(QUIT_ITEM_ID), Some(TrayEvent::Quit));
        assert_eq!(resolve_tray_event("unknown"), None);
    }

    #[test]
    fn test_default_hotkeys_parse_as_menu_accelerators() {
        assert!(parse_accelerator("Alt+Shift+S").is_some());
        assert!(parse_accelerator("Alt+Shift+R").is_some());
    }
}
