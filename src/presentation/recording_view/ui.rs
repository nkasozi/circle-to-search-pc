use iced::widget::{button, container, image, row, stack, text, Space};
use iced::{Alignment, Border, Color, Element, Length, Shadow};

use super::state::{build_recording_status_label, format_elapsed_duration};
use super::{RecordingView, RecordingViewMessage};
use crate::core::models::recording::RecordingState;

const BUTTON_LABEL_RECORD: &str = "\u{23FA} Record";
const BUTTON_LABEL_PAUSE: &str = "\u{23F8} Pause";
const BUTTON_LABEL_RESUME: &str = "\u{25B6} Resume";
const BUTTON_LABEL_STOP: &str = "\u{23F9} Stop";
const BUTTON_LABEL_CLOSE: &str = "\u{2715} Close";
const BUTTON_LABEL_CLEAR: &str = "\u{1F5D1} Clear";
const LABEL_MIC_ON: &str = "\u{1F3A4} Mic On";
const LABEL_MIC_OFF: &str = "\u{1F507} Mic Off";
const LABEL_CAM_ON: &str = "\u{1F4F7} Cam On";
const LABEL_CAM_OFF: &str = "\u{1F4F7} Cam Off";

const TOOLBAR_BACKGROUND: Color = Color::from_rgb(0.12, 0.12, 0.14);
const TOOLBAR_BORDER_COLOR: Color = Color::from_rgb(0.25, 0.25, 0.28);
const RECORD_BUTTON_COLOR: Color = Color::from_rgb(0.85, 0.15, 0.15);
const RECORD_BUTTON_HOVER: Color = Color::from_rgb(0.95, 0.20, 0.20);
const STOP_BUTTON_COLOR: Color = Color::from_rgb(0.75, 0.15, 0.15);
const PAUSE_BUTTON_COLOR: Color = Color::from_rgb(0.20, 0.45, 0.70);
const ACTIVE_TOGGLE_COLOR: Color = Color::from_rgb(0.15, 0.55, 0.35);
const INACTIVE_TOGGLE_COLOR: Color = Color::from_rgb(0.35, 0.35, 0.38);
const TIMER_TEXT_COLOR: Color = Color::from_rgb(0.90, 0.30, 0.30);
const PREVIEW_BACKGROUND: Color = Color::from_rgb(0.08, 0.08, 0.10);

const TOOLBAR_BORDER_WIDTH: f32 = 1.0;
const TOOLBAR_BORDER_RADIUS: f32 = 8.0;
const BUTTON_BORDER_RADIUS: f32 = 6.0;
const BUTTON_FONT_SIZE: f32 = 14.0;
const TIMER_FONT_SIZE: f32 = 18.0;
const STATUS_FONT_SIZE: f32 = 12.0;
const TOOLBAR_SPACING: f32 = 8.0;
const TOOLBAR_PADDING: f32 = 10.0;

impl RecordingView {
    pub fn render_ui(&self) -> Element<'_, RecordingViewMessage> {
        let preview_layer = self.build_preview_layer();
        let toolbar_layer = self.build_control_toolbar();

        let positioned_toolbar = container(toolbar_layer)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(iced::Padding {
                top: 0.0,
                right: 0.0,
                bottom: 16.0,
                left: 0.0,
            })
            .align_x(Alignment::Center)
            .align_y(Alignment::End);

        let mut layers: Vec<Element<'_, RecordingViewMessage>> = vec![preview_layer.into()];
        layers.push(positioned_toolbar.into());

        if let Some(error_text) = &self.error_message {
            layers.push(self.build_error_banner(error_text).into());
        }

        if let Some(output_path) = &self.output_file_path {
            layers.push(self.build_success_banner(output_path).into());
        }

        stack(layers).into()
    }

    fn build_preview_layer(&self) -> Element<'_, RecordingViewMessage> {
        match &self.current_preview_frame {
            Some(frame) => {
                let handle = frame.to_image_handle();
                container(
                    image(handle)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .content_fit(iced::ContentFit::Contain),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| container_style(PREVIEW_BACKGROUND))
                .into()
            }
            None => container(
                text(build_recording_status_label(&self.recording_state))
                    .size(STATUS_FONT_SIZE)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(Color::WHITE),
                    }),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .style(|_theme| container_style(PREVIEW_BACKGROUND))
            .into(),
        }
    }

    fn build_control_toolbar(&self) -> Element<'_, RecordingViewMessage> {
        let timer_display = text(format_elapsed_duration(&self.elapsed_duration))
            .size(TIMER_FONT_SIZE)
            .style(|_theme| iced::widget::text::Style {
                color: Some(TIMER_TEXT_COLOR),
            });

        let primary_action_button = self.build_primary_action_button();
        let mic_toggle = self.build_mic_toggle();
        let webcam_toggle = self.build_webcam_toggle();
        let clear_button = build_toolbar_button(
            BUTTON_LABEL_CLEAR,
            INACTIVE_TOGGLE_COLOR,
            INACTIVE_TOGGLE_COLOR,
            RecordingViewMessage::ClearAnnotations,
        );
        let close_button = build_toolbar_button(
            BUTTON_LABEL_CLOSE,
            INACTIVE_TOGGLE_COLOR,
            INACTIVE_TOGGLE_COLOR,
            RecordingViewMessage::Close,
        );

        let toolbar_row = row![
            primary_action_button,
            timer_display,
            Space::new().width(Length::Fixed(TOOLBAR_SPACING)),
            mic_toggle,
            webcam_toggle,
            Space::new().width(Length::Fixed(TOOLBAR_SPACING)),
            clear_button,
            close_button,
        ]
        .spacing(TOOLBAR_SPACING)
        .align_y(Alignment::Center);

        container(toolbar_row)
            .padding(TOOLBAR_PADDING)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(TOOLBAR_BACKGROUND)),
                border: Border {
                    color: TOOLBAR_BORDER_COLOR,
                    width: TOOLBAR_BORDER_WIDTH,
                    radius: TOOLBAR_BORDER_RADIUS.into(),
                },
                shadow: Shadow::default(),
                text_color: None,
                snap: false,
            })
            .into()
    }

    fn build_primary_action_button(&self) -> Element<'_, RecordingViewMessage> {
        match &self.recording_state {
            RecordingState::Idle => build_toolbar_button(
                BUTTON_LABEL_RECORD,
                RECORD_BUTTON_COLOR,
                RECORD_BUTTON_HOVER,
                RecordingViewMessage::StartRecording,
            ),
            RecordingState::Recording => {
                let pause_btn = build_toolbar_button(
                    BUTTON_LABEL_PAUSE,
                    PAUSE_BUTTON_COLOR,
                    PAUSE_BUTTON_COLOR,
                    RecordingViewMessage::PauseRecording,
                );
                let stop_btn = build_toolbar_button(
                    BUTTON_LABEL_STOP,
                    STOP_BUTTON_COLOR,
                    STOP_BUTTON_COLOR,
                    RecordingViewMessage::StopRecording,
                );
                row![pause_btn, stop_btn].spacing(4.0).into()
            }
            RecordingState::Paused => {
                let resume_btn = build_toolbar_button(
                    BUTTON_LABEL_RESUME,
                    ACTIVE_TOGGLE_COLOR,
                    ACTIVE_TOGGLE_COLOR,
                    RecordingViewMessage::ResumeRecording,
                );
                let stop_btn = build_toolbar_button(
                    BUTTON_LABEL_STOP,
                    STOP_BUTTON_COLOR,
                    STOP_BUTTON_COLOR,
                    RecordingViewMessage::StopRecording,
                );
                row![resume_btn, stop_btn].spacing(4.0).into()
            }
            RecordingState::Stopped => build_toolbar_button(
                BUTTON_LABEL_CLOSE,
                INACTIVE_TOGGLE_COLOR,
                INACTIVE_TOGGLE_COLOR,
                RecordingViewMessage::Close,
            ),
        }
    }

    fn build_mic_toggle(&self) -> Element<'_, RecordingViewMessage> {
        let label = match self.microphone_enabled {
            true => LABEL_MIC_ON,
            false => LABEL_MIC_OFF,
        };
        let color = match self.microphone_enabled {
            true => ACTIVE_TOGGLE_COLOR,
            false => INACTIVE_TOGGLE_COLOR,
        };
        build_toolbar_button(label, color, color, RecordingViewMessage::ToggleMicrophone)
    }

    fn build_webcam_toggle(&self) -> Element<'_, RecordingViewMessage> {
        let label = match self.webcam_enabled {
            true => LABEL_CAM_ON,
            false => LABEL_CAM_OFF,
        };
        let color = match self.webcam_enabled {
            true => ACTIVE_TOGGLE_COLOR,
            false => INACTIVE_TOGGLE_COLOR,
        };
        build_toolbar_button(label, color, color, RecordingViewMessage::ToggleWebcam)
    }
}

fn build_toolbar_button<'a>(
    label: &'a str,
    background_color: Color,
    hover_color: Color,
    on_press_message: RecordingViewMessage,
) -> Element<'a, RecordingViewMessage> {
    button(
        text(label)
            .size(BUTTON_FONT_SIZE)
            .style(|_theme| iced::widget::text::Style {
                color: Some(Color::WHITE),
            }),
    )
    .padding([4, 12])
    .style(move |_theme: &iced::Theme, status| match status {
        iced::widget::button::Status::Active | iced::widget::button::Status::Disabled => {
            iced::widget::button::Style {
                background: Some(iced::Background::Color(background_color)),
                text_color: Color::WHITE,
                border: Border {
                    color: background_color,
                    width: TOOLBAR_BORDER_WIDTH,
                    radius: BUTTON_BORDER_RADIUS.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            }
        }
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(iced::Background::Color(hover_color)),
                text_color: Color::WHITE,
                border: Border {
                    color: hover_color,
                    width: TOOLBAR_BORDER_WIDTH,
                    radius: BUTTON_BORDER_RADIUS.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            }
        }
    })
    .on_press(on_press_message)
    .into()
}

fn container_style(background: Color) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(iced::Background::Color(background)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: None,
        snap: false,
    }
}

impl RecordingView {
    fn build_error_banner(&self, error_text: &str) -> Element<'_, RecordingViewMessage> {
        let banner = container(
            text(format!("\u{26A0} {}", error_text))
                .size(STATUS_FONT_SIZE)
                .style(|_theme| iced::widget::text::Style {
                    color: Some(Color::WHITE),
                }),
        )
        .padding(8)
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.70, 0.15, 0.15))),
            border: Border {
                color: Color::from_rgb(0.85, 0.20, 0.20),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            text_color: None,
            snap: false,
        });

        container(banner)
            .width(Length::Fill)
            .padding(iced::Padding {
                top: 16.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            })
            .align_x(Alignment::Center)
            .into()
    }

    fn build_success_banner(&self, output_path: &str) -> Element<'_, RecordingViewMessage> {
        let banner = container(
            text(format!("\u{2713} Saved to {}", output_path))
                .size(STATUS_FONT_SIZE)
                .style(|_theme| iced::widget::text::Style {
                    color: Some(Color::WHITE),
                }),
        )
        .padding(8)
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.10, 0.50, 0.30))),
            border: Border {
                color: Color::from_rgb(0.15, 0.60, 0.35),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Shadow::default(),
            text_color: None,
            snap: false,
        });

        container(banner)
            .width(Length::Fill)
            .padding(iced::Padding {
                top: 16.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            })
            .align_x(Alignment::Center)
            .into()
    }
}
