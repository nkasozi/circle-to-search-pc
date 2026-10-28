use iced::widget::{text, text_input};
use iced::Element;

use super::controls::{segmented, setting_row, settings_card};
use super::{SettingsChange, SettingsMessage};
use crate::core::models::{ImageHostingAuthMode, ImageUploadHttpMethod, UserSettings};
use crate::global_constants;
use crate::presentation::design::inputs::field_style;
use crate::presentation::design::{font_size, spacing, surfaces};

const SEARCH_TITLE: &str = "Reverse image search";
const HOSTING_TITLE: &str = "Image hosting (advanced)";
const FIELD_WIDTH: f32 = 260.0;
const GET_LABEL: &str = "GET";
const PUT_LABEL: &str = "PUT";
const POST_LABEL: &str = "POST";
const HEADER_LABEL: &str = "Header";
const QUERY_LABEL: &str = "Query";

fn text_field<'a>(
    placeholder: &'a str,
    value: &'a str,
    on_input: fn(String) -> SettingsMessage,
) -> Element<'a, SettingsMessage> {
    text_input(placeholder, value)
        .on_input(on_input)
        .padding([spacing::SMALL, spacing::MEDIUM])
        .size(font_size::CAPTION)
        .width(iced::Length::Fixed(FIELD_WIDTH))
        .style(field_style)
        .into()
}

pub fn render_search_tab<'a>(draft: &'a UserSettings) -> Vec<Element<'a, SettingsMessage>> {
    vec![render_search_card(draft), render_hosting_card(draft)]
}

fn render_search_card(draft: &UserSettings) -> Element<'_, SettingsMessage> {
    settings_card(
        SEARCH_TITLE,
        vec![setting_row(
            global_constants::SETTINGS_LABEL_IMAGE_SEARCH_URL,
            global_constants::SETTINGS_DESCRIPTION_IMAGE_SEARCH_URL,
            text_field(
                global_constants::DEFAULT_IMAGE_SEARCH_URL,
                &draft.image_search_url_template,
                |value| SettingsMessage::Change(SettingsChange::SearchUrl(value)),
            ),
        )],
    )
}

fn render_hosting_card(draft: &UserSettings) -> Element<'_, SettingsMessage> {
    let tip: Element<'_, SettingsMessage> = text(global_constants::IMAGE_HOSTING_SETTINGS_TIP)
        .size(font_size::CAPTION)
        .style(surfaces::muted_text)
        .into();
    let rows = std::iter::once(tip)
        .chain(render_provider_rows(draft))
        .chain(render_key_rows(draft))
        .chain(render_request_rows(draft))
        .collect();
    settings_card(HOSTING_TITLE, rows)
}

fn render_provider_rows(draft: &UserSettings) -> Vec<Element<'_, SettingsMessage>> {
    vec![
        setting_row(
            global_constants::SETTINGS_LABEL_PROVIDER_URL,
            global_constants::SETTINGS_DESCRIPTION_PROVIDER_URL,
            text_field(
                global_constants::IMGBB_API_URL,
                &draft.image_hosting_provider_url,
                |value| SettingsMessage::Change(SettingsChange::HostingProviderUrl(value)),
            ),
        ),
        setting_row(
            global_constants::SETTINGS_LABEL_AUTH_MODE,
            global_constants::SETTINGS_DESCRIPTION_AUTH_MODE,
            segmented(
                &[
                    (ImageHostingAuthMode::Query, QUERY_LABEL),
                    (ImageHostingAuthMode::Header, HEADER_LABEL),
                ],
                draft.image_hosting_auth_mode,
                |mode| SettingsMessage::Change(SettingsChange::HostingAuthMode(mode)),
            ),
        ),
    ]
}

fn render_key_rows(draft: &UserSettings) -> Vec<Element<'_, SettingsMessage>> {
    vec![
        setting_row(
            global_constants::SETTINGS_LABEL_PUBLIC_KEY_NAME,
            global_constants::SETTINGS_DESCRIPTION_PUBLIC_KEY_NAME,
            text_field(
                global_constants::IMGBB_PUBLIC_KEY_QUERY_NAME,
                &draft.image_hosting_public_key_name,
                |value| SettingsMessage::Change(SettingsChange::HostingKeyName(value)),
            ),
        ),
        setting_row(
            global_constants::SETTINGS_LABEL_PUBLIC_KEY,
            global_constants::SETTINGS_DESCRIPTION_PUBLIC_KEY,
            text_field(
                global_constants::SETTINGS_PUBLIC_KEY_PLACEHOLDER,
                &draft.image_hosting_public_key_value,
                |value| SettingsMessage::Change(SettingsChange::HostingKeyValue(value)),
            ),
        ),
    ]
}

fn render_request_rows(draft: &UserSettings) -> Vec<Element<'_, SettingsMessage>> {
    vec![
        setting_row(
            global_constants::SETTINGS_LABEL_EXPIRATION_SECONDS,
            global_constants::SETTINGS_DESCRIPTION_EXPIRATION_SECONDS,
            text_field(
                global_constants::IMGBB_EXPIRATION_SECONDS,
                &draft.image_hosting_expiration_seconds,
                |value| SettingsMessage::Change(SettingsChange::HostingExpirationSeconds(value)),
            ),
        ),
        setting_row(
            global_constants::SETTINGS_LABEL_HTTP_METHOD,
            global_constants::SETTINGS_DESCRIPTION_HTTP_METHOD,
            segmented(
                &[
                    (ImageUploadHttpMethod::Post, POST_LABEL),
                    (ImageUploadHttpMethod::Put, PUT_LABEL),
                    (ImageUploadHttpMethod::Get, GET_LABEL),
                ],
                draft.image_hosting_http_method,
                |method| SettingsMessage::Change(SettingsChange::HostingHttpMethod(method)),
            ),
        ),
        setting_row(
            global_constants::SETTINGS_LABEL_IMAGE_FIELD_NAME,
            global_constants::SETTINGS_DESCRIPTION_IMAGE_FIELD_NAME,
            text_field(
                global_constants::SETTINGS_IMAGE_FIELD_NAME_PLACEHOLDER,
                &draft.image_hosting_image_field_name,
                |value| SettingsMessage::Change(SettingsChange::HostingImageFieldName(value)),
            ),
        ),
    ]
}
