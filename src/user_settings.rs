use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::global_constants;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub image_search_url_template: String,
    pub capture_hotkey: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            image_search_url_template: global_constants::DEFAULT_IMAGE_SEARCH_URL.to_string(),
            capture_hotkey: global_constants::DEFAULT_CAPTURE_HOTKEY.to_string(),
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
        log::debug!("[SETTINGS] Image search URL: {}", settings.image_search_url_template);
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
