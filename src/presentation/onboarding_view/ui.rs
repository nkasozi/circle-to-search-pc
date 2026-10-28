use iced::widget::{column, container, row, Row, Space};
use iced::{Alignment, Element, Length, Theme};

use super::{OnboardingMessage, OnboardingStep, OnboardingView};
use crate::presentation::design::components::labeled_button;
use crate::presentation::design::tokens::{radius, tokens_for_theme};
use crate::presentation::design::{spacing, surfaces, ButtonTone, Icon, IconTint};

const CONTINUE_LABEL: &str = "Continue";
const BACK_LABEL: &str = "Back";
const FINISH_LABEL: &str = "Start using Circle to Search";
const GET_STARTED_LABEL: &str = "Get started";
const SKIP_FOR_NOW_LABEL: &str = "Skip for now";
const PROGRESS_SEGMENT_HEIGHT: f32 = 4.0;
const CONTENT_MAXIMUM_WIDTH: f32 = 520.0;

impl OnboardingView {
    pub fn view(&self) -> Element<'_, OnboardingMessage> {
        let step_content = match self.current_step {
            OnboardingStep::Welcome => self.render_welcome_step(),
            OnboardingStep::ScreenAccess => self.render_screen_access_step(),
            OnboardingStep::Shortcuts => self.render_shortcuts_step(),
            OnboardingStep::Startup => self.render_startup_step(),
        };
        let layout = column![
            self.render_progress(),
            container(step_content)
                .height(Length::Fill)
                .center_y(Length::Fill),
            self.render_footer(),
        ]
        .spacing(spacing::EXTRA_LARGE)
        .padding(spacing::HUGE)
        .max_width(CONTENT_MAXIMUM_WIDTH);
        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .style(surfaces::window_background)
            .into()
    }

    fn render_progress(&self) -> Element<'_, OnboardingMessage> {
        let current_position = self.current_step.position();
        let segments: Vec<Element<'_, OnboardingMessage>> = OnboardingStep::ALL
            .iter()
            .map(|step| render_progress_segment(step.position() <= current_position))
            .collect();
        Row::with_children(segments).spacing(spacing::SMALL).into()
    }

    fn render_footer(&self) -> Element<'_, OnboardingMessage> {
        let (primary_label, primary_message) = self.resolve_primary_action();
        let primary_tone = self.resolve_primary_tone();
        row![
            self.render_back_button(),
            Space::new().width(Length::Fill),
            labeled_button(
                Icon::ChevronRight,
                primary_label,
                primary_message,
                primary_tone,
                resolve_primary_tint(primary_tone)
            )
        ]
        .align_y(Alignment::Center)
        .into()
    }

    fn render_back_button(&self) -> Element<'_, OnboardingMessage> {
        match self.current_step {
            OnboardingStep::Welcome => Space::new().into(),
            _ => labeled_button(
                Icon::ChevronLeft,
                BACK_LABEL,
                OnboardingMessage::PreviousStep,
                ButtonTone::Ghost,
                IconTint::TextSecondary,
            )
            .into(),
        }
    }

    fn resolve_primary_action(&self) -> (&'static str, OnboardingMessage) {
        match (self.current_step, self.screen_access_granted) {
            (OnboardingStep::Welcome, _) => (GET_STARTED_LABEL, OnboardingMessage::NextStep),
            (OnboardingStep::ScreenAccess, false) => {
                (SKIP_FOR_NOW_LABEL, OnboardingMessage::NextStep)
            }
            (OnboardingStep::Startup, _) => (FINISH_LABEL, OnboardingMessage::FinishOnboarding),
            _ => (CONTINUE_LABEL, OnboardingMessage::NextStep),
        }
    }

    fn resolve_primary_tone(&self) -> ButtonTone {
        match (self.current_step, self.screen_access_granted) {
            (OnboardingStep::ScreenAccess, false) => ButtonTone::Secondary,
            _ => ButtonTone::Primary,
        }
    }
}

fn resolve_primary_tint(primary_tone: ButtonTone) -> IconTint {
    match primary_tone {
        ButtonTone::Primary => IconTint::Fixed(iced::Color::WHITE),
        _ => IconTint::TextPrimary,
    }
}

fn render_progress_segment<'a>(is_reached: bool) -> Element<'a, OnboardingMessage> {
    container(Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(PROGRESS_SEGMENT_HEIGHT))
        .style(move |theme: &Theme| {
            let tokens = tokens_for_theme(theme);
            let segment_color = match is_reached {
                true => tokens.accent,
                false => tokens.border,
            };
            iced::widget::container::Style {
                background: Some(iced::Background::Color(segment_color)),
                border: iced::Border {
                    radius: radius::PILL.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}
