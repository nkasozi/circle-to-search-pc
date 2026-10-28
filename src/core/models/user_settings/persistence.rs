use std::path::{Path, PathBuf};

use super::install_identity::{get_or_create_install_id, is_new_installation};
use super::UserSettings;
use crate::global_constants;

const CONFIG_DIRECTORY_MISSING_ERROR: &str = "Could not find config directory";
const APPLICATION_CONFIG_FOLDER_NAME: &str = "circle-to-search-pc";
const LOG_TAG: &str = "[SETTINGS]";
const SETTINGS_FILE_MISSING_EVENT: &str = "settings_file_missing";
const SETTINGS_RESET_FOR_NEW_INSTALLATION_EVENT: &str = "settings_reset_for_new_installation";
const SETTINGS_SAVED_EVENT: &str = "settings_saved";
const SETTINGS_LOADED_EVENT: &str = "settings_loaded";
const SETTINGS_DETAILS_EVENT: &str = "settings_details";

impl UserSettings {
    pub fn load() -> anyhow::Result<Self> {
        let current_install_id = get_or_create_install_id();
        let settings_path = settings_file_path()?;
        if !settings_path.exists() {
            log::info!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": SETTINGS_FILE_MISSING_EVENT, "settings_path": settings_path.display().to_string()})
            );
            return save_fresh_defaults(current_install_id);
        }
        let stored_settings = read_settings_file(&settings_path)?;
        if is_new_installation(&stored_settings.install_id, &current_install_id) {
            log::info!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": SETTINGS_RESET_FOR_NEW_INSTALLATION_EVENT})
            );
            return save_fresh_defaults(current_install_id);
        }
        Ok(adopt_loaded_settings(
            stored_settings,
            current_install_id,
            &settings_path,
        ))
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let settings_path = settings_file_path()?;
        if let Some(parent) = settings_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(&settings_path, contents)?;
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": SETTINGS_SAVED_EVENT, "settings_path": settings_path.display().to_string()})
        );
        Ok(())
    }
}

fn settings_file_path() -> anyhow::Result<PathBuf> {
    let config_directory =
        dirs::config_dir().ok_or_else(|| anyhow::anyhow!(CONFIG_DIRECTORY_MISSING_ERROR))?;
    Ok(config_directory
        .join(APPLICATION_CONFIG_FOLDER_NAME)
        .join(global_constants::SETTINGS_FILE_NAME))
}

fn save_fresh_defaults(install_id: Option<String>) -> anyhow::Result<UserSettings> {
    let default_settings = UserSettings {
        install_id,
        ..UserSettings::default()
    };
    default_settings.save()?;
    Ok(default_settings)
}

fn read_settings_file(settings_path: &Path) -> anyhow::Result<UserSettings> {
    let contents = std::fs::read_to_string(settings_path)?;
    let settings: UserSettings = serde_json::from_str(&contents)?;
    Ok(settings)
}

fn adopt_loaded_settings(
    stored_settings: UserSettings,
    current_install_id: Option<String>,
    settings_path: &Path,
) -> UserSettings {
    let settings = UserSettings {
        install_id: current_install_id,
        ..stored_settings
    };
    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": SETTINGS_LOADED_EVENT, "settings_path": settings_path.display().to_string()})
    );
    log::debug!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": SETTINGS_DETAILS_EVENT, "image_search_url_template": settings.image_search_url_template, "capture_hotkey": settings.capture_hotkey})
    );
    settings
}
