use super::*;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Background, Color, Element, Length};

impl AppOrchestrator {
    pub fn render_settings_window(&self) -> Element<'_, OrchestratorMessage> {
        let theme = app_theme::get_theme(&self.settings.theme_mode);
        let temp = self.get_settings_for_rendering();
        let content = self.render_settings_content(temp);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme| {
                let palette = theme.palette();
                iced::widget::container::Style {
                    background: Some(Background::Color(palette.background)),
                    text_color: Some(palette.text),
                    ..Default::default()
                }
            })
            .into()
    }

    fn render_settings_content(
        &self,
        temp: &UserSettings,
    ) -> iced::widget::Column<'_, OrchestratorMessage> {
        let header_section = self.render_settings_header();
        let search_section = self.render_search_settings_section(temp);
        let image_hosting_section = self.render_image_hosting_settings_section(temp);
        let keyboard_section = self.render_keyboard_settings_section(temp);
        let appearance_section = self.render_appearance_settings_section(temp);
        let save_button = self.render_settings_save_button();

        column![
            header_section,
            Space::new().height(Length::Fixed(24.0)),
            search_section,
            Space::new().height(Length::Fixed(16.0)),
            image_hosting_section,
            Space::new().height(Length::Fixed(16.0)),
            keyboard_section,
            Space::new().height(Length::Fixed(16.0)),
            appearance_section,
            Space::new().height(Length::Fixed(28.0)),
            save_button,
        ]
        .spacing(4)
        .padding(32)
        .width(Length::Fill)
        .align_x(Alignment::Center)
    }

    fn render_settings_header(&self) -> iced::widget::Column<'_, OrchestratorMessage> {
        let header_icon = text(global_constants::SETTINGS_WINDOW_ICON).size(48);
        let title = text(global_constants::SETTINGS_WINDOW_TITLE).size(28);
        column![header_icon, title]
            .spacing(8)
            .align_x(Alignment::Center)
    }

    fn render_search_settings_section(
        &self,
        temp: &UserSettings,
    ) -> Element<'_, OrchestratorMessage> {
        use iced::widget::text_input;

        self.render_settings_section(
            global_constants::SETTINGS_SECTION_SEARCH_TITLE,
            global_constants::SETTINGS_SECTION_SEARCH_ICON,
            column![self.render_setting_row(
                global_constants::SETTINGS_LABEL_IMAGE_SEARCH_URL,
                global_constants::SETTINGS_DESCRIPTION_IMAGE_SEARCH_URL,
                text_input(
                    global_constants::DEFAULT_IMAGE_SEARCH_URL,
                    &temp.image_search_url_template,
                )
                .on_input(OrchestratorMessage::UpdateSearchUrl)
                .padding(12)
                .into(),
            ),]
            .spacing(12),
        )
    }

    fn render_image_hosting_settings_section(
        &self,
        temp: &UserSettings,
    ) -> Element<'_, OrchestratorMessage> {
        use iced::widget::{pick_list, text_input};

        let displayed_public_key: &str = if temp.is_using_default_public_key() {
            ""
        } else {
            &temp.image_hosting_public_key_value
        };

        self.render_settings_section(
            global_constants::SETTINGS_SECTION_IMAGE_HOSTING_TITLE,
            global_constants::SETTINGS_SECTION_IMAGE_HOSTING_ICON,
            column![
                self.render_setting_row(
                    global_constants::SETTINGS_LABEL_PROVIDER_URL,
                    global_constants::SETTINGS_DESCRIPTION_PROVIDER_URL,
                    text_input(
                        global_constants::IMGBB_API_URL,
                        &temp.image_hosting_provider_url,
                    )
                    .on_input(OrchestratorMessage::UpdateImageHostingProviderUrl)
                    .padding(12)
                    .into(),
                ),
                self.render_setting_row(
                    global_constants::SETTINGS_LABEL_AUTH_MODE,
                    global_constants::SETTINGS_DESCRIPTION_AUTH_MODE,
                    pick_list(
                        vec![ImageHostingAuthMode::Query, ImageHostingAuthMode::Header],
                        Some(temp.image_hosting_auth_mode.clone()),
                        OrchestratorMessage::UpdateImageHostingAuthMode,
                    )
                    .padding(12)
                    .into(),
                ),
                self.render_setting_row(
                    global_constants::SETTINGS_LABEL_PUBLIC_KEY_NAME,
                    global_constants::SETTINGS_DESCRIPTION_PUBLIC_KEY_NAME,
                    text_input(
                        global_constants::IMGBB_PUBLIC_KEY_QUERY_NAME,
                        &temp.image_hosting_public_key_name
                    )
                    .on_input(OrchestratorMessage::UpdateImageHostingPublicKeyName)
                    .padding(12)
                    .into(),
                ),
                self.render_setting_row(
                    global_constants::SETTINGS_LABEL_PUBLIC_KEY,
                    global_constants::SETTINGS_DESCRIPTION_PUBLIC_KEY,
                    text_input(
                        global_constants::SETTINGS_PUBLIC_KEY_PLACEHOLDER,
                        displayed_public_key
                    )
                    .on_input(OrchestratorMessage::UpdateImageHostingPublicKeyValue)
                    .padding(12)
                    .into(),
                ),
                self.render_setting_row(
                    global_constants::SETTINGS_LABEL_EXPIRATION_SECONDS,
                    global_constants::SETTINGS_DESCRIPTION_EXPIRATION_SECONDS,
                    text_input(
                        global_constants::IMGBB_EXPIRATION_SECONDS,
                        &temp.image_hosting_expiration_seconds
                    )
                    .on_input(OrchestratorMessage::UpdateImageHostingExpirationSeconds)
                    .padding(12)
                    .into(),
                ),
                self.render_setting_row(
                    global_constants::SETTINGS_LABEL_HTTP_METHOD,
                    global_constants::SETTINGS_DESCRIPTION_HTTP_METHOD,
                    pick_list(
                        vec![
                            ImageUploadHttpMethod::Post,
                            ImageUploadHttpMethod::Get,
                            ImageUploadHttpMethod::Put,
                        ],
                        Some(temp.image_hosting_http_method.clone()),
                        OrchestratorMessage::UpdateImageHostingHttpMethod,
                    )
                    .padding(12)
                    .into(),
                ),
                self.render_setting_row(
                    global_constants::SETTINGS_LABEL_IMAGE_FIELD_NAME,
                    global_constants::SETTINGS_DESCRIPTION_IMAGE_FIELD_NAME,
                    text_input(
                        global_constants::SETTINGS_IMAGE_FIELD_NAME_PLACEHOLDER,
                        &temp.image_hosting_image_field_name,
                    )
                    .on_input(OrchestratorMessage::UpdateImageHostingImageFieldName)
                    .padding(12)
                    .into(),
                ),
                text(global_constants::IMAGE_HOSTING_SETTINGS_TIP)
                    .size(11)
                    .style(|_theme: &iced::Theme| iced::widget::text::Style {
                        color: Some(Color::from_rgba(0.8, 0.8, 0.8, 0.9)),
                    }),
            ]
            .spacing(12),
        )
    }

    fn render_keyboard_settings_section(
        &self,
        temp: &UserSettings,
    ) -> Element<'_, OrchestratorMessage> {
        use iced::widget::text_input;

        let hotkey_warning = text(global_constants::SETTINGS_RESTART_REQUIRED_WARNING)
            .size(11)
            .style(|_theme: &iced::Theme| iced::widget::text::Style {
                color: Some(Color::from_rgba(1.0, 0.7, 0.0, 0.8)),
            });

        self.render_settings_section(
            global_constants::SETTINGS_SECTION_KEYBOARD_TITLE,
            global_constants::SETTINGS_SECTION_KEYBOARD_ICON,
            column![self.render_setting_row(
                global_constants::SETTINGS_LABEL_CAPTURE_HOTKEY,
                global_constants::SETTINGS_DESCRIPTION_CAPTURE_HOTKEY,
                column![
                    text_input(
                        global_constants::DEFAULT_CAPTURE_HOTKEY,
                        &temp.capture_hotkey
                    )
                    .on_input(OrchestratorMessage::UpdateHotkey)
                    .padding(12),
                    hotkey_warning,
                ]
                .spacing(4)
                .into(),
            ),]
            .spacing(12),
        )
    }

    fn render_appearance_settings_section(
        &self,
        temp: &UserSettings,
    ) -> Element<'_, OrchestratorMessage> {
        use iced::widget::pick_list;

        self.render_settings_section(
            global_constants::SETTINGS_SECTION_APPEARANCE_TITLE,
            global_constants::SETTINGS_SECTION_APPEARANCE_ICON,
            column![self.render_setting_row(
                global_constants::SETTINGS_LABEL_THEME,
                global_constants::SETTINGS_DESCRIPTION_THEME,
                pick_list(
                    vec![ThemeMode::Dark, ThemeMode::Light],
                    Some(temp.theme_mode.clone()),
                    OrchestratorMessage::UpdateTheme,
                )
                .padding(12)
                .into(),
            ),]
            .spacing(12),
        )
    }

    fn render_settings_save_button(&self) -> iced::widget::Button<'_, OrchestratorMessage> {
        button(
            row![
                text(global_constants::SETTINGS_ICON_SAVE).size(16),
                text(global_constants::SETTINGS_SAVE_CHANGES_LABEL).size(15)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding([14, 36])
        .style(|theme, status| app_theme::primary_button_style(theme, status))
        .on_press(OrchestratorMessage::SaveSettings)
    }

    fn render_settings_section<'a>(
        &self,
        title: &'a str,
        icon: &'a str,
        content: iced::widget::Column<'a, OrchestratorMessage>,
    ) -> Element<'a, OrchestratorMessage> {
        let section_header = row![text(icon).size(18), text(title).size(16)]
            .spacing(8)
            .align_y(Alignment::Center);

        let section_content = container(content)
            .padding([12, 16])
            .width(Length::Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.3))),
                border: iced::Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.3),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            });

        column![section_header, section_content]
            .spacing(8)
            .width(Length::Fill)
            .into()
    }

    fn render_setting_row<'a>(
        &self,
        label: &'a str,
        description: &'a str,
        input: Element<'a, OrchestratorMessage>,
    ) -> Element<'a, OrchestratorMessage> {
        let label_col = column![
            text(label).size(14),
            text(description)
                .size(11)
                .style(|_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(0.6, 0.6, 0.6, 1.0)),
                }),
        ]
        .spacing(2)
        .width(Length::FillPortion(2));

        let input_col = container(input).width(Length::FillPortion(3));

        row![label_col, input_col]
            .spacing(16)
            .align_y(Alignment::Center)
            .into()
    }
}
