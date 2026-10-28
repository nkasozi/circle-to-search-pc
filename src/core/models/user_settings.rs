use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::core::models::RecordingPreferences;
use crate::global_constants;

mod display_enums;
mod install_identity;
mod persistence;
#[cfg(test)]
mod tests;

pub use display_enums::{ImageHostingAuthMode, ImageUploadHttpMethod, ThemeMode};

const CURRENT_DIRECTORY: &str = ".";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub image_search_url_template: String,
    #[serde(default = "UserSettings::default_image_hosting_provider_url")]
    pub image_hosting_provider_url: String,
    #[serde(default)]
    pub image_hosting_auth_mode: ImageHostingAuthMode,
    #[serde(
        rename = "image_hosting_api_key_name",
        default = "UserSettings::default_image_hosting_public_key_name"
    )]
    pub image_hosting_public_key_name: String,
    #[serde(
        rename = "image_hosting_api_key_value",
        default = "UserSettings::default_image_hosting_public_key_value"
    )]
    pub image_hosting_public_key_value: String,
    #[serde(default = "UserSettings::default_image_hosting_expiration_seconds")]
    pub image_hosting_expiration_seconds: String,
    #[serde(default)]
    pub image_hosting_http_method: ImageUploadHttpMethod,
    #[serde(default = "UserSettings::default_image_hosting_image_field_name")]
    pub image_hosting_image_field_name: String,
    pub capture_hotkey: String,
    pub theme_mode: ThemeMode,
    #[serde(default = "UserSettings::default_show_home_on_launch")]
    pub show_home_on_launch: bool,
    #[serde(default)]
    pub onboarding_complete: bool,
    #[serde(default)]
    pub launch_at_login: bool,
    #[serde(default)]
    pub install_id: Option<String>,
    #[serde(default = "UserSettings::default_screenshot_save_location")]
    pub screenshot_save_location: String,
    #[serde(default)]
    pub recording_preferences: RecordingPreferences,
    #[serde(default = "UserSettings::default_recording_save_location")]
    pub recording_save_location: String,
    #[serde(default = "UserSettings::default_recording_hotkey")]
    pub recording_hotkey: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            image_search_url_template: global_constants::DEFAULT_IMAGE_SEARCH_URL.to_string(),
            image_hosting_provider_url: Self::default_image_hosting_provider_url(),
            image_hosting_auth_mode: ImageHostingAuthMode::default(),
            image_hosting_public_key_name: Self::default_image_hosting_public_key_name(),
            image_hosting_public_key_value: Self::default_image_hosting_public_key_value(),
            image_hosting_expiration_seconds: Self::default_image_hosting_expiration_seconds(),
            image_hosting_http_method: ImageUploadHttpMethod::default(),
            image_hosting_image_field_name: Self::default_image_hosting_image_field_name(),
            capture_hotkey: global_constants::DEFAULT_CAPTURE_HOTKEY.to_string(),
            theme_mode: ThemeMode::default(),
            show_home_on_launch: true,
            onboarding_complete: false,
            launch_at_login: false,
            install_id: None,
            screenshot_save_location: Self::default_screenshot_save_location(),
            recording_preferences: RecordingPreferences::default(),
            recording_save_location: Self::default_recording_save_location(),
            recording_hotkey: Self::default_recording_hotkey(),
        }
    }
}

impl UserSettings {
    pub fn default_image_hosting_provider_url() -> String {
        global_constants::IMGBB_API_URL.to_string()
    }

    pub fn default_image_hosting_public_key_name() -> String {
        global_constants::IMGBB_PUBLIC_KEY_QUERY_NAME.to_string()
    }

    pub fn default_image_hosting_public_key_value() -> String {
        std::env::var(global_constants::IMGBB_PUBLIC_KEY_ENV_VAR_NAME)
            .unwrap_or_else(|_| global_constants::IMGBB_PUBLIC_KEY.to_string())
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn is_using_default_public_key(&self) -> bool {
        self.image_hosting_public_key_value == Self::default_image_hosting_public_key_value()
    }

    pub fn default_image_hosting_expiration_seconds() -> String {
        global_constants::IMGBB_EXPIRATION_SECONDS.to_string()
    }

    pub fn default_image_hosting_image_field_name() -> String {
        global_constants::IMGBB_IMAGE_FIELD_NAME.to_string()
    }

    pub fn default_screenshot_save_location() -> String {
        downloads_or_home_directory().to_string_lossy().to_string()
    }

    pub fn default_recording_save_location() -> String {
        dirs::video_dir()
            .unwrap_or_else(downloads_or_home_directory)
            .join(global_constants::RECORDINGS_FOLDER_NAME)
            .to_string_lossy()
            .to_string()
    }

    pub fn default_show_home_on_launch() -> bool {
        true
    }

    pub fn default_recording_hotkey() -> String {
        global_constants::HOTKEY_RECORD.to_string()
    }
}

fn downloads_or_home_directory() -> PathBuf {
    dirs::download_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(CURRENT_DIRECTORY)))
}
