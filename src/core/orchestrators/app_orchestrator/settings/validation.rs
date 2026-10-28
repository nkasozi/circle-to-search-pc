use super::*;

impl AppOrchestrator {
    pub(in crate::core::orchestrators::app_orchestrator) fn validate_image_hosting_settings(
        settings: &UserSettings,
    ) -> Result<(), String> {
        if settings.image_hosting_provider_url.trim().is_empty() {
            return Err(global_constants::IMAGE_HOSTING_VALIDATION_URL_EMPTY.to_string());
        }

        if reqwest::Url::parse(settings.image_hosting_provider_url.trim()).is_err() {
            return Err(global_constants::IMAGE_HOSTING_VALIDATION_URL_INVALID.to_string());
        }

        if settings.image_hosting_public_key_name.trim().is_empty() {
            return Err(global_constants::IMAGE_HOSTING_VALIDATION_KEY_NAME_EMPTY.to_string());
        }

        if settings.image_hosting_public_key_value.trim().is_empty() {
            return Err(global_constants::IMAGE_HOSTING_VALIDATION_KEY_EMPTY.to_string());
        }

        if settings.image_hosting_expiration_seconds.trim().is_empty() {
            return Err(global_constants::IMAGE_HOSTING_VALIDATION_EXPIRATION_EMPTY.to_string());
        }

        if !is_positive_expiration(&settings.image_hosting_expiration_seconds) {
            return Err(global_constants::IMAGE_HOSTING_VALIDATION_EXPIRATION_INVALID.to_string());
        }

        Ok(())
    }

    pub(in crate::core::orchestrators::app_orchestrator) fn should_rebuild_search_provider(
        previous_settings: &UserSettings,
        next_settings: &UserSettings,
    ) -> bool {
        next_settings.image_search_url_template != previous_settings.image_search_url_template
            || next_settings.image_hosting_provider_url
                != previous_settings.image_hosting_provider_url
            || next_settings.image_hosting_auth_mode != previous_settings.image_hosting_auth_mode
            || next_settings.image_hosting_public_key_name
                != previous_settings.image_hosting_public_key_name
            || next_settings.image_hosting_public_key_value
                != previous_settings.image_hosting_public_key_value
            || next_settings.image_hosting_expiration_seconds
                != previous_settings.image_hosting_expiration_seconds
            || next_settings.image_hosting_http_method
                != previous_settings.image_hosting_http_method
            || next_settings.image_hosting_image_field_name
                != previous_settings.image_hosting_image_field_name
    }
}

fn is_positive_expiration(expiration_seconds: &str) -> bool {
    expiration_seconds
        .trim()
        .parse::<u64>()
        .is_ok_and(|parsed_seconds| parsed_seconds > 0)
}
