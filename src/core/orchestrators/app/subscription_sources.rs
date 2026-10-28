use futures::channel::mpsc::Sender;
use futures::Stream;
use iced::keyboard::key::{Named, Physical};
use iced::keyboard::{Event as KeyboardEvent, Key};
use iced::window::{self, Id};
use iced::{Event, Subscription};

use super::constants::{
    ONBOARDING_PERMISSION_POLL_INTERVAL, SPINNER_TICK_INTERVAL, SUBSCRIPTION_CHANNEL_CAPACITY,
    TRAY_POLL_INTERVAL,
};
use crate::core::models::HotkeyRegistration;
use crate::core::orchestrators::app_orchestrator::OrchestratorMessage;
use crate::ports::{GlobalHotkeyListener, SystemTray};
use crate::presentation::settings_view::PressedModifiers;
use crate::presentation::OnboardingMessage;

fn map_keyboard_event(event: KeyboardEvent, window_id: Id) -> Option<OrchestratorMessage> {
    match event {
        KeyboardEvent::KeyPressed {
            key: Key::Named(Named::Escape),
            ..
        } => Some(OrchestratorMessage::EscapePressed(window_id)),
        KeyboardEvent::KeyPressed {
            physical_key: Physical::Code(code),
            modifiers,
            repeat: false,
            ..
        } => Some(OrchestratorMessage::KeyPressedInWindow(
            window_id,
            PressedModifiers {
                control: modifiers.control(),
                alt: modifiers.alt(),
                shift: modifiers.shift(),
                command: modifiers.logo(),
            },
            format!("{:?}", code),
        )),
        _ => None,
    }
}

fn map_window_event(
    event: Event,
    _status: iced::event::Status,
    window_id: Id,
) -> Option<OrchestratorMessage> {
    match event {
        Event::Window(window::Event::Closed) => Some(OrchestratorMessage::WindowClosed(window_id)),
        Event::Window(window::Event::Focused) => {
            Some(OrchestratorMessage::WindowFocused(window_id))
        }
        Event::Keyboard(keyboard_event) => map_keyboard_event(keyboard_event, window_id),
        _ => None,
    }
}

fn tray_event_stream() -> impl Stream<Item = OrchestratorMessage> {
    iced::stream::channel(
        SUBSCRIPTION_CHANNEL_CAPACITY,
        |mut output: Sender<OrchestratorMessage>| async move {
            loop {
                tokio::time::sleep(TRAY_POLL_INTERVAL).await;
                if let Some(event) = SystemTray::poll_events() {
                    let _ = output.try_send(OrchestratorMessage::TrayEvent(event));
                }
            }
        },
    )
}

fn spinner_tick_stream() -> impl Stream<Item = OrchestratorMessage> {
    iced::stream::channel(
        SUBSCRIPTION_CHANNEL_CAPACITY,
        |mut output: Sender<OrchestratorMessage>| async move {
            loop {
                tokio::time::sleep(SPINNER_TICK_INTERVAL).await;
                let _ = output.try_send(OrchestratorMessage::SpinnerTick);
            }
        },
    )
}

pub fn window_event_subscription() -> Subscription<OrchestratorMessage> {
    iced::event::listen_with(map_window_event)
}

pub fn tray_poll_subscription() -> Subscription<OrchestratorMessage> {
    Subscription::run(tray_event_stream)
}

pub fn spinner_tick_subscription() -> Subscription<OrchestratorMessage> {
    Subscription::run(spinner_tick_stream)
}

pub fn onboarding_poll_subscription(
    onboarding_window: Option<Id>,
) -> Option<Subscription<OrchestratorMessage>> {
    onboarding_window.map(|window_id| {
        iced::time::every(ONBOARDING_PERMISSION_POLL_INTERVAL)
            .with(window_id)
            .map(|(window_id, _)| {
                OrchestratorMessage::OnboardingMsg(window_id, OnboardingMessage::RefreshPermissions)
            })
    })
}

pub fn global_hotkey_subscription(
    hotkey_registrations: &[HotkeyRegistration],
) -> Option<Subscription<OrchestratorMessage>> {
    match hotkey_registrations.is_empty() {
        true => None,
        false => Some(
            Subscription::run_with(
                hotkey_registrations.to_vec(),
                GlobalHotkeyListener::create_event_stream,
            )
            .map(OrchestratorMessage::GlobalHotkey),
        ),
    }
}
