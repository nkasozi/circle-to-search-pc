use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Vector};

use super::app_theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardingStep {
    Welcome,
    ScreenRecording,
    InputMonitoring,
    AutoStart,
    Complete,
}

impl OnboardingStep {
    pub fn next(self) -> Self {
        match self {
            OnboardingStep::Welcome => OnboardingStep::ScreenRecording,
            OnboardingStep::ScreenRecording => OnboardingStep::InputMonitoring,
            OnboardingStep::InputMonitoring => OnboardingStep::AutoStart,
            OnboardingStep::AutoStart => OnboardingStep::Complete,
            OnboardingStep::Complete => OnboardingStep::Complete,
        }
    }

    pub fn step_number(self) -> usize {
        match self {
            OnboardingStep::Welcome => 1,
            OnboardingStep::ScreenRecording => 2,
            OnboardingStep::InputMonitoring => 3,
            OnboardingStep::AutoStart => 4,
            OnboardingStep::Complete => 5,
        }
    }

    pub fn total_steps() -> usize {
        5
    }
}

#[derive(Debug, Clone)]
pub enum OnboardingMessage {
    NextStep,
    BackToStart,
    OpenScreenRecordingSettings,
    OpenInputMonitoringSettings,
    ToggleLaunchAtLogin(bool),
    FinishOnboarding,
    RefreshPermissions,
}

pub struct OnboardingView {
    current_step: OnboardingStep,
    screen_recording_granted: bool,
    input_monitoring_granted: bool,
    launch_at_login: bool,
    toast_message: Option<(String, bool)>,
}

impl OnboardingView {
    pub fn new(
        screen_recording_granted: bool,
        input_monitoring_granted: bool,
        launch_at_login: bool,
    ) -> Self {
        Self {
            current_step: OnboardingStep::Welcome,
            screen_recording_granted,
            input_monitoring_granted,
            launch_at_login,
            toast_message: None,
        }
    }

    pub fn update_permissions(
        &mut self,
        screen_recording_granted: bool,
        input_monitoring_granted: bool,
    ) {
        let previous_screen = self.screen_recording_granted;
        let previous_input_monitoring = self.input_monitoring_granted;

        self.screen_recording_granted = screen_recording_granted;
        self.input_monitoring_granted = input_monitoring_granted;

        let relevant_permission = match self.current_step {
            OnboardingStep::ScreenRecording => screen_recording_granted,
            OnboardingStep::InputMonitoring => input_monitoring_granted,
            _ => return,
        };

        let was_granted = match self.current_step {
            OnboardingStep::ScreenRecording => previous_screen,
            OnboardingStep::InputMonitoring => previous_input_monitoring,
            _ => return,
        };

        if relevant_permission {
            self.toast_message = Some(("✓ Permission granted!".to_string(), true));
        } else if !was_granted {
            self.toast_message = Some((
                "✗ Permission not yet granted. Please enable it in System Settings.".to_string(),
                false,
            ));
        }
    }

    #[allow(dead_code)]
    pub fn current_step(&self) -> OnboardingStep {
        self.current_step
    }

    pub fn is_launch_at_login_enabled(&self) -> bool {
        self.launch_at_login
    }

    pub fn handle_message(&mut self, message: OnboardingMessage) -> bool {
        match message {
            OnboardingMessage::NextStep => {
                self.current_step = self.current_step.next();
                self.toast_message = None;
                false
            }
            OnboardingMessage::BackToStart => {
                self.current_step = OnboardingStep::Welcome;
                self.toast_message = None;
                false
            }
            OnboardingMessage::ToggleLaunchAtLogin(enabled) => {
                self.launch_at_login = enabled;
                false
            }
            OnboardingMessage::FinishOnboarding => true,
            OnboardingMessage::OpenScreenRecordingSettings
            | OnboardingMessage::OpenInputMonitoringSettings
            | OnboardingMessage::RefreshPermissions => false,
        }
    }

    pub fn view(&self) -> Element<'_, OnboardingMessage> {
        let content = match self.current_step {
            OnboardingStep::Welcome => self.render_welcome_step(),
            OnboardingStep::ScreenRecording => self.render_screen_recording_step(),
            OnboardingStep::InputMonitoring => self.render_input_monitoring_step(),
            OnboardingStep::AutoStart => self.render_auto_start_step(),
            OnboardingStep::Complete => self.render_complete_step(),
        };

        let progress_indicator = self.render_progress_indicator();

        let toast = self.render_toast();

        let main_content = column![toast, progress_indicator, text("").size(16), content]
            .spacing(8)
            .padding(24)
            .width(Length::Fill)
            .align_x(Alignment::Center);

        let scrollable_content = scrollable(main_content)
            .width(Length::Fill)
            .height(Length::Fill);

        container(scrollable_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .style(|_theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
                text_color: Some(Color::WHITE),
                ..Default::default()
            })
            .into()
    }

    fn render_progress_indicator(&self) -> Element<'_, OnboardingMessage> {
        let current_step = self.current_step.step_number();
        let total_steps = OnboardingStep::total_steps();

        let progress_dots: Vec<Element<OnboardingMessage>> = (1..=total_steps)
            .map(|step| {
                let dot_style = if step <= current_step { "●" } else { "○" };
                text(dot_style).size(16).into()
            })
            .collect();

        let progress_row = row(progress_dots).spacing(10);

        let step_text = text(format!("Step {} of {}", current_step, total_steps)).size(14);

        column![progress_row, step_text]
            .spacing(5)
            .align_x(Alignment::Center)
            .into()
    }

    fn render_toast(&self) -> Element<'_, OnboardingMessage> {
        match &self.toast_message {
            Some((message, is_success)) => {
                let (border_color, icon) = if *is_success {
                    (Color::from_rgba(0.122, 0.655, 0.408, 0.8), "✓")
                } else {
                    (Color::from_rgba(0.9, 0.4, 0.1, 0.8), "⚠")
                };

                let toast_text = format!("{} {}", icon, message);

                container(
                    text(toast_text)
                        .size(16)
                        .style(|_theme| iced::widget::text::Style {
                            color: Some(Color::WHITE),
                        }),
                )
                .padding([12, 24])
                .width(Length::Fill)
                .align_x(Alignment::Center)
                .style(move |_theme| iced::widget::container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.85))),
                    border: iced::Border {
                        color: border_color,
                        width: 2.0,
                        radius: 8.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                        offset: iced::Vector::new(0.0, 4.0),
                        blur_radius: 12.0,
                    },
                    text_color: None,
                    snap: false,
                })
                .into()
            }
            None => text("").size(1).into(),
        }
    }

    fn render_welcome_step(&self) -> Element<'_, OnboardingMessage> {
        let title = text("Welcome to Circle to Search").size(28);

        let description = text(
            "This app lets you search anything on your screen using a simple keyboard shortcut.\n\n\
             Before we get started, we need to set up a few permissions to make everything work smoothly.",
        )
        .size(16);

        let features_content = column![
            text("What you'll be able to do:").size(16),
            text("• Press Alt+Shift+S to capture any part of your screen").size(14),
            text("• Extract text from images using OCR").size(14),
            text("• Search Google with selected images").size(14),
            text("• Access the app from the system tray").size(14),
        ]
        .spacing(8);

        let features_panel = container(features_content)
            .padding([16, 20])
            .width(Length::Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.3))),
                border: Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.3),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
                text_color: None,
                snap: false,
            });

        let next_button = button(text("Let's Get Started").size(16))
            .padding([14, 32])
            .style(app_theme::primary_button_style)
            .on_press(OnboardingMessage::NextStep);

        column![
            title,
            text("").size(16),
            description,
            text("").size(16),
            features_panel,
            text("").size(24),
            next_button,
        ]
        .spacing(8)
        .align_x(Alignment::Center)
        .max_width(500)
        .into()
    }

    fn render_screen_recording_step(&self) -> Element<'_, OnboardingMessage> {
        let title = text("Screen Recording Permission").size(24);

        let status_content = if self.screen_recording_granted {
            row![
                text("✓")
                    .size(20)
                    .style(|_theme: &iced::Theme| iced::widget::text::Style {
                        color: Some(Color::from_rgb(0.2, 0.8, 0.4)),
                    }),
                text(" Permission Granted").size(18)
            ]
            .spacing(4)
            .align_y(Alignment::Center)
        } else {
            row![
                text("⚠")
                    .size(20)
                    .style(|_theme: &iced::Theme| iced::widget::text::Style {
                        color: Some(Color::from_rgb(1.0, 0.7, 0.0)),
                    }),
                text(" Permission Required").size(18)
            ]
            .spacing(4)
            .align_y(Alignment::Center)
        };

        let status_panel = container(status_content)
            .padding([12, 20])
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .style(|_theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.3))),
                border: Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.3),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            });

        let description = text(
            "Circle to Search needs permission to capture your screen so you can select \
             areas to search or extract text from.\n\n\
             This permission is required for the core functionality of the app.",
        )
        .size(16);

        let instructions_content = if !self.screen_recording_granted {
            column![
                text("To grant permission:").size(16),
                text("1. Click the button below to open System Settings").size(14),
                text("2. Find 'Circle to Search' in the list").size(14),
                text("3. Toggle the switch to enable access").size(14),
                text("4. Return here and click 'Check Permission'").size(14),
            ]
            .spacing(6)
        } else {
            column![text("You're all set! Click 'Continue' to proceed.").size(14)]
        };

        let instructions_panel = container(instructions_content)
            .padding([16, 20])
            .width(Length::Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.3))),
                border: Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.3),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            });

        let open_settings_button = button(text("Open System Settings").size(16))
            .padding([14, 24])
            .style(app_theme::primary_button_style)
            .on_press(OnboardingMessage::OpenScreenRecordingSettings);

        let check_button = button(text("Check Permission").size(16))
            .padding([14, 24])
            .style(app_theme::purple_button_style)
            .on_press(OnboardingMessage::RefreshPermissions);

        let action_buttons = row![open_settings_button, check_button]
            .spacing(12)
            .align_y(Alignment::Center);

        let continue_button = if self.screen_recording_granted {
            button(text("Continue").size(16))
                .padding([14, 32])
                .style(app_theme::primary_button_style)
                .on_press(OnboardingMessage::NextStep)
        } else {
            button(text("Skip for now").size(14))
                .padding([12, 24])
                .style(app_theme::secondary_button_style)
                .on_press(OnboardingMessage::NextStep)
        };

        column![
            title,
            text("").size(12),
            status_panel,
            text("").size(16),
            description,
            text("").size(16),
            instructions_panel,
            text("").size(20),
            action_buttons,
            text("").size(16),
            continue_button,
        ]
        .spacing(4)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .max_width(500)
        .into()
    }

    fn render_input_monitoring_step(&self) -> Element<'_, OnboardingMessage> {
        let title = text("Input Monitoring Permission").size(24);

        let status_content = if self.input_monitoring_granted {
            row![
                text("✓")
                    .size(20)
                    .style(|_theme: &iced::Theme| iced::widget::text::Style {
                        color: Some(Color::from_rgb(0.2, 0.8, 0.4)),
                    }),
                text(" Permission Granted").size(18)
            ]
            .spacing(4)
            .align_y(Alignment::Center)
        } else {
            row![
                text("⚠")
                    .size(20)
                    .style(|_theme: &iced::Theme| iced::widget::text::Style {
                        color: Some(Color::from_rgb(1.0, 0.7, 0.0)),
                    }),
                text(" Permission Required").size(18)
            ]
            .spacing(4)
            .align_y(Alignment::Center)
        };

        let status_panel = container(status_content)
            .padding([12, 20])
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .style(|_theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.3))),
                border: Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.3),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            });

        let description = text(
            "Circle to Search needs Input Monitoring permission to detect the keyboard shortcut \
             (Alt+Shift+S) that triggers the screen capture.\n\n\
             Without this, you'll need to use the system tray menu to start a capture.",
        )
        .size(16);

        let instructions_content = if !self.input_monitoring_granted {
            column![
                text("To grant permission:").size(16),
                text("1. Click the button below to open System Settings").size(14),
                text("2. Find 'Circle to Search' in the list").size(14),
                text("3. Toggle the switch to enable access").size(14),
                text("4. Return here and click 'Check Permission'").size(14),
            ]
            .spacing(6)
        } else {
            column![text("You're all set! Click 'Continue' to proceed.").size(14)]
        };

        let instructions_panel = container(instructions_content)
            .padding([16, 20])
            .width(Length::Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.3))),
                border: Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.3),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            });

        let open_settings_button = button(text("Open System Settings").size(16))
            .padding([14, 24])
            .style(app_theme::primary_button_style)
            .on_press(OnboardingMessage::OpenInputMonitoringSettings);

        let check_button = button(text("Check Permission").size(16))
            .padding([14, 24])
            .style(app_theme::purple_button_style)
            .on_press(OnboardingMessage::RefreshPermissions);

        let action_buttons = row![open_settings_button, check_button]
            .spacing(12)
            .align_y(Alignment::Center);

        let continue_button = if self.input_monitoring_granted {
            button(text("Continue").size(16))
                .padding([14, 32])
                .style(app_theme::primary_button_style)
                .on_press(OnboardingMessage::NextStep)
        } else {
            button(text("Skip for now").size(14))
                .padding([12, 24])
                .style(app_theme::secondary_button_style)
                .on_press(OnboardingMessage::NextStep)
        };

        column![
            title,
            text("").size(12),
            status_panel,
            text("").size(16),
            description,
            text("").size(16),
            instructions_panel,
            text("").size(20),
            action_buttons,
            text("").size(16),
            continue_button,
        ]
        .spacing(4)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .max_width(500)
        .into()
    }

    fn render_auto_start_step(&self) -> Element<'_, OnboardingMessage> {
        let title = text("Start at Login").size(24);

        let description = text(
            "Circle to Search runs quietly in the background, accessible from your system tray.\n\n\
             Would you like the app to start automatically when you log in?",
        )
        .size(16);

        let checkbox_label = if self.launch_at_login {
            "☑ Launch Circle to Search when I log in"
        } else {
            "☐ Launch Circle to Search when I log in"
        };

        let auto_start_content = column![
            button(text(checkbox_label).size(16))
                .padding([14, 24])
                .on_press(OnboardingMessage::ToggleLaunchAtLogin(
                    !self.launch_at_login
                )),
            text("You can change this setting later in the app preferences.")
                .size(13)
                .style(|_theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(0.6, 0.6, 0.6, 1.0)),
                }),
        ]
        .spacing(12)
        .align_x(Alignment::Center);

        let auto_start_panel = container(auto_start_content)
            .padding([20, 24])
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .style(|_theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.3))),
                border: Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.3),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            });

        let next_button = button(text("Continue").size(16))
            .padding([14, 32])
            .style(app_theme::primary_button_style)
            .on_press(OnboardingMessage::NextStep);

        column![
            title,
            text("").size(16),
            description,
            text("").size(20),
            auto_start_panel,
            text("").size(24),
            next_button,
        ]
        .spacing(4)
        .align_x(Alignment::Center)
        .max_width(500)
        .into()
    }

    fn render_complete_step(&self) -> Element<'_, OnboardingMessage> {
        let title = text("You're All Set!").size(28);

        let description = text("Circle to Search is ready to use.").size(16);

        let screen_status_color = if self.screen_recording_granted {
            Color::from_rgb(0.2, 0.8, 0.4)
        } else {
            Color::from_rgb(1.0, 0.4, 0.4)
        };

        let input_monitoring_color = if self.input_monitoring_granted {
            Color::from_rgb(0.2, 0.8, 0.4)
        } else {
            Color::from_rgb(1.0, 0.4, 0.4)
        };

        let auto_start_color = if self.launch_at_login {
            Color::from_rgb(0.2, 0.8, 0.4)
        } else {
            Color::from_rgba(0.6, 0.6, 0.6, 1.0)
        };

        let screen_status = if self.screen_recording_granted {
            "✓ Screen Recording: Enabled"
        } else {
            "✗ Screen Recording: Not enabled"
        };

        let input_monitoring_status = if self.input_monitoring_granted {
            "✓ Input Monitoring: Enabled"
        } else {
            "✗ Input Monitoring: Not enabled"
        };

        let auto_start_status = if self.launch_at_login {
            "✓ Auto-start: Enabled"
        } else {
            "○ Auto-start: Disabled"
        };

        let permissions_content = column![
            text("Setup Summary:").size(16),
            text(screen_status)
                .size(14)
                .style(move |_theme: &iced::Theme| {
                    iced::widget::text::Style {
                        color: Some(screen_status_color),
                    }
                }),
            text(input_monitoring_status)
                .size(14)
                .style(move |_theme: &iced::Theme| {
                    iced::widget::text::Style {
                        color: Some(input_monitoring_color),
                    }
                }),
            text(auto_start_status)
                .size(14)
                .style(move |_theme: &iced::Theme| {
                    iced::widget::text::Style {
                        color: Some(auto_start_color),
                    }
                }),
        ]
        .spacing(8);

        let permissions_panel = container(permissions_content)
            .padding([16, 20])
            .width(Length::Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.3))),
                border: Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.3),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            });

        let hotkey_content = column![
            text("How to use:").size(16),
            text("• Press Alt+Shift+S to start a capture").size(14),
            text("• Or right-click the system tray icon").size(14),
            text("• Draw a rectangle around what you want to search").size(14),
        ]
        .spacing(8);

        let hotkey_panel = container(hotkey_content)
            .padding([16, 20])
            .width(Length::Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.3))),
                border: Border {
                    color: Color::from_rgba(0.4, 0.4, 0.4, 0.3),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            });

        let finish_button = button(text("Start Using Circle to Search").size(16))
            .padding([14, 32])
            .style(app_theme::primary_button_style)
            .on_press(OnboardingMessage::FinishOnboarding);

        let back_button = button(text("← Back to Setup").size(14))
            .padding([12, 24])
            .style(app_theme::secondary_button_style)
            .on_press(OnboardingMessage::BackToStart);

        column![
            title,
            text("").size(12),
            description,
            text("").size(16),
            permissions_panel,
            text("").size(12),
            hotkey_panel,
            text("").size(20),
            finish_button,
            text("").size(12),
            back_button,
        ]
        .spacing(4)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .max_width(500)
        .into()
    }
}
