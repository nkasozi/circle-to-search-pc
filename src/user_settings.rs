use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

use crate::global_constants;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThemeMode {
    Dark,
    Light,
}

impl fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThemeMode::Dark => write!(f, "Dark"),
            ThemeMode::Light => write!(f, "Light"),
        }
    }
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::Dark
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub image_search_url_template: String,
    pub capture_hotkey: String,
    pub theme_mode: ThemeMode,
    #[serde(default)]
    pub run_in_system_tray: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            image_search_url_template: global_constants::DEFAULT_IMAGE_SEARCH_URL.to_string(),
            capture_hotkey: global_constants::DEFAULT_CAPTURE_HOTKEY.to_string(),
            theme_mode: ThemeMode::default(),
            run_in_system_tray: false,
        }
    }
}

impl UserSettings {
    pub fn load() -> anyhow::Result<Self> {
        let settings_path = Self::get_settings_file_path()?;

        if !settings_path.exists() {
            log::info!("[SETTINGS] No settings file found, using defaults");
            let default_settings = Self::default();
            default_settings.save()?;
            return Ok(default_settings);
        }

        let contents = std::fs::read_to_string(&settings_path)?;
        let settings: UserSettings = serde_json::from_str(&contents)?;

        log::info!("[SETTINGS] Loaded settings from {:?}", settings_path);
        log::debug!(
            "[SETTINGS] Image search URL: {}",
            settings.image_search_url_template
        );
        log::debug!("[SETTINGS] Capture hotkey: {}", settings.capture_hotkey);

        Ok(settings)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let settings_path = Self::get_settings_file_path()?;

        if let Some(parent) = settings_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(&settings_path, contents)?;

        log::info!("[SETTINGS] Saved settings to {:?}", settings_path);
        Ok(())
    }

    fn get_settings_file_path() -> anyhow::Result<PathBuf> {
        let config_dir = if cfg!(target_os = "macos") {
            dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
                .join("circle-to-search-pc")
        } else if cfg!(target_os = "windows") {
            dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
                .join("circle-to-search-pc")
        } else {
            dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
                .join("circle-to-search-pc")
        };

        Ok(config_dir.join(global_constants::SETTINGS_FILE_NAME))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_mode_default_is_dark() {
        let default_theme = ThemeMode::default();
        assert_eq!(default_theme, ThemeMode::Dark);
    }

    #[test]
    fn test_theme_mode_display_dark() {
        let theme = ThemeMode::Dark;
        assert_eq!(format!("{}", theme), "Dark");
    }

    #[test]
    fn test_theme_mode_display_light() {
        let theme = ThemeMode::Light;
        assert_eq!(format!("{}", theme), "Light");
    }

    #[test]
    fn test_theme_mode_serialization() {
        let theme = ThemeMode::Dark;
        let serialized = serde_json::to_string(&theme).unwrap();
        assert_eq!(serialized, "\"Dark\"");
    }

    #[test]
    fn test_theme_mode_deserialization() {
        let json = "\"Light\"";
        let theme: ThemeMode = serde_json::from_str(json).unwrap();
        assert_eq!(theme, ThemeMode::Light);
    }

    #[test]
    fn test_user_settings_default_values() {
        let settings = UserSettings::default();

        assert_eq!(
            settings.image_search_url_template,
            global_constants::DEFAULT_IMAGE_SEARCH_URL
        );
        assert_eq!(
            settings.capture_hotkey,
            global_constants::DEFAULT_CAPTURE_HOTKEY
        );
        assert_eq!(settings.theme_mode, ThemeMode::Dark);
        assert!(!settings.run_in_system_tray);
    }

    #[test]
    fn test_user_settings_serialization() {
        let settings = UserSettings {
            image_search_url_template: "https://example.com/{IMAGE_URL}".to_string(),
            capture_hotkey: "ctrl+shift+a".to_string(),
            theme_mode: ThemeMode::Light,
            run_in_system_tray: true,
        };

        let serialized = serde_json::to_string(&settings).unwrap();
        let deserialized: UserSettings = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.image_search_url_template, settings.image_search_url_template);
        assert_eq!(deserialized.capture_hotkey, settings.capture_hotkey);
        assert_eq!(deserialized.theme_mode, settings.theme_mode);
        assert_eq!(deserialized.run_in_system_tray, settings.run_in_system_tray);
    }

    #[test]
    fn test_user_settings_deserialization_with_missing_run_in_system_tray() {
        let json = r#"{
            "image_search_url_template": "https://example.com",
            "capture_hotkey": "ctrl+a",
            "theme_mode": "Dark"
        }"#;

        let settings: UserSettings = serde_json::from_str(json).unwrap();
        assert!(!settings.run_in_system_tray);
    }

    #[test]
    fn test_user_settings_save_and_load_roundtrip() {
        let temp_dir = std::env::temp_dir().join("circle-to-search-test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let original_settings = UserSettings {
            image_search_url_template: "https://test.com/{IMAGE_URL}".to_string(),
            capture_hotkey: "ctrl+shift+t".to_string(),
            theme_mode: ThemeMode::Light,
            run_in_system_tray: true,
        };

        let test_file = temp_dir.join("test_settings.json");
        let contents = serde_json::to_string_pretty(&original_settings).unwrap();
        std::fs::write(&test_file, contents).unwrap();

        let loaded_contents = std::fs::read_to_string(&test_file).unwrap();
        let loaded_settings: UserSettings = serde_json::from_str(&loaded_contents).unwrap();

        assert_eq!(loaded_settings.image_search_url_template, original_settings.image_search_url_template);
        assert_eq!(loaded_settings.capture_hotkey, original_settings.capture_hotkey);
        assert_eq!(loaded_settings.theme_mode, original_settings.theme_mode);
        assert_eq!(loaded_settings.run_in_system_tray, original_settings.run_in_system_tray);

        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
