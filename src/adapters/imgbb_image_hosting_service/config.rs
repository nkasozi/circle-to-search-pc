use crate::core::models::{ImageHostingAuthMode, ImageUploadHttpMethod, UserSettings};

#[derive(Debug, Clone)]
pub struct ImgbbImageHostingConfig {
    pub provider_url: String,
    pub auth_mode: ImageHostingAuthMode,
    pub public_key_name: String,
    pub public_key_value: String,
    pub expiration_seconds: String,
    pub http_method: ImageUploadHttpMethod,
    pub image_field_name: String,
}

impl ImgbbImageHostingConfig {
    pub fn from_user_settings(settings: &UserSettings) -> Self {
        Self {
            provider_url: settings.image_hosting_provider_url.clone(),
            auth_mode: settings.image_hosting_auth_mode.clone(),
            public_key_name: settings.image_hosting_public_key_name.clone(),
            public_key_value: settings.image_hosting_public_key_value.clone(),
            expiration_seconds: settings.image_hosting_expiration_seconds.clone(),
            http_method: settings.image_hosting_http_method.clone(),
            image_field_name: settings.image_hosting_image_field_name.clone(),
        }
    }
}
