use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

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
    #[serde(default)]
    pub onboarding_complete: bool,
    #[serde(default)]
    pub launch_at_login: bool,
    #[serde(default)]
    pub install_id: Option<String>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            image_search_url_template: global_constants::DEFAULT_IMAGE_SEARCH_URL.to_string(),
            capture_hotkey: global_constants::DEFAULT_CAPTURE_HOTKEY.to_string(),
            theme_mode: ThemeMode::default(),
            run_in_system_tray: true,
            onboarding_complete: false,
            launch_at_login: false,
            install_id: None,
        }
    }
}

impl UserSettings {
    pub fn load() -> anyhow::Result<Self> {
        let current_install_id = Self::get_or_create_install_id();
        let settings_path = Self::get_settings_file_path()?;

        if !settings_path.exists() {
            log::info!("[SETTINGS] No settings file found, using defaults");
            let mut default_settings = Self::default();
            default_settings.install_id = current_install_id.clone();
            default_settings.save()?;
            return Ok(default_settings);
        }

        let contents = std::fs::read_to_string(&settings_path)?;
        let mut settings: UserSettings = serde_json::from_str(&contents)?;

        if Self::is_new_installation(&settings.install_id, &current_install_id) {
            log::info!("[SETTINGS] New installation detected, resetting settings");
            let mut default_settings = Self::default();
            default_settings.install_id = current_install_id;
            default_settings.save()?;
            return Ok(default_settings);
        }

        settings.install_id = current_install_id;

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

    fn get_or_create_install_id() -> Option<String> {
        let install_id_path = match Self::get_install_id_path() {
            Some(path) => path,
            None => {
                log::debug!("[SETTINGS] Could not determine install ID path (not in app bundle)");
                return None;
            }
        };

        if install_id_path.exists() {
            match fs::read_to_string(&install_id_path) {
                Ok(id) => {
                    let id = id.trim().to_string();
                    log::debug!("[SETTINGS] Found existing install ID: {}", id);
                    return Some(id);
                }
                Err(error) => {
                    log::warn!("[SETTINGS] Failed to read install ID: {}", error);
                }
            }
        }

        let new_id = Uuid::new_v4().to_string();
        log::info!("[SETTINGS] Generated new install ID: {}", new_id);

        if let Some(parent) = install_id_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        match fs::write(&install_id_path, &new_id) {
            Ok(()) => {
                log::info!("[SETTINGS] Saved install ID to {:?}", install_id_path);
            }
            Err(error) => {
                log::warn!("[SETTINGS] Failed to save install ID: {}", error);
            }
        }

        Some(new_id)
    }

    #[cfg(target_os = "macos")]
    fn get_install_id_path() -> Option<PathBuf> {
        let exe_path = std::env::current_exe().ok()?;
        let exe_str = exe_path.to_str()?;

        if exe_str.contains(".app/Contents/MacOS/") {
            let parts: Vec<&str> = exe_str.split(".app/Contents/MacOS/").collect();
            if !parts.is_empty() {
                let app_bundle = format!("{}.app", parts[0]);
                return Some(PathBuf::from(app_bundle).join("Contents/Resources/.install_id"));
            }
        }

        None
    }

    #[cfg(not(target_os = "macos"))]
    fn get_install_id_path() -> Option<PathBuf> {
        let exe_path = std::env::current_exe().ok()?;
        let exe_dir = exe_path.parent()?;
        Some(exe_dir.join(".install_id"))
    }

    fn is_new_installation(saved_id: &Option<String>, current_id: &Option<String>) -> bool {
        match (saved_id, current_id) {
            (Some(saved), Some(current)) => {
                let is_different = saved != current;
                if is_different {
                    log::info!(
                        "[SETTINGS] Install ID mismatch: saved={}, current={}",
                        saved,
                        current
                    );
                }
                is_different
            }
            (Some(_), None) => false,
            (None, Some(_)) => false,
            (None, None) => false,
        }
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
        assert!(settings.run_in_system_tray);
        assert!(!settings.onboarding_complete);
        assert!(!settings.launch_at_login);
    }

    #[test]
    fn test_user_settings_serialization() {
        let settings = UserSettings {
            image_search_url_template: "https://example.com/{IMAGE_URL}".to_string(),
            capture_hotkey: "ctrl+shift+a".to_string(),
            theme_mode: ThemeMode::Light,
            run_in_system_tray: true,
            onboarding_complete: true,
            launch_at_login: true,
            install_id: Some("test-id".to_string()),
        };

        let serialized = serde_json::to_string(&settings).unwrap();
        let deserialized: UserSettings = serde_json::from_str(&serialized).unwrap();

        assert_eq!(
            deserialized.image_search_url_template,
            settings.image_search_url_template
        );
        assert_eq!(deserialized.capture_hotkey, settings.capture_hotkey);
        assert_eq!(deserialized.theme_mode, settings.theme_mode);
        assert_eq!(deserialized.run_in_system_tray, settings.run_in_system_tray);
        assert_eq!(
            deserialized.onboarding_complete,
            settings.onboarding_complete
        );
        assert_eq!(deserialized.launch_at_login, settings.launch_at_login);
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
            onboarding_complete: true,
            launch_at_login: true,
            install_id: Some("test-roundtrip-id".to_string()),
        };

        let test_file = temp_dir.join("test_settings.json");
        let contents = serde_json::to_string_pretty(&original_settings).unwrap();
        std::fs::write(&test_file, contents).unwrap();

        let loaded_contents = std::fs::read_to_string(&test_file).unwrap();
        let loaded_settings: UserSettings = serde_json::from_str(&loaded_contents).unwrap();

        assert_eq!(
            loaded_settings.image_search_url_template,
            original_settings.image_search_url_template
        );
        assert_eq!(
            loaded_settings.capture_hotkey,
            original_settings.capture_hotkey
        );
        assert_eq!(loaded_settings.theme_mode, original_settings.theme_mode);
        assert_eq!(
            loaded_settings.run_in_system_tray,
            original_settings.run_in_system_tray
        );
        assert_eq!(
            loaded_settings.onboarding_complete,
            original_settings.onboarding_complete
        );
        assert_eq!(
            loaded_settings.launch_at_login,
            original_settings.launch_at_login
        );

        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
