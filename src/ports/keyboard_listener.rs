use iced::futures::{channel::mpsc, SinkExt, Stream, StreamExt};
use iced::stream;
use rdev::{listen, EventType, Key};

use crate::global_constants::{
    LOG_TAG_KEYBOARD, MESSAGE_KEYBOARD_ALT_PRESSED, MESSAGE_KEYBOARD_ALT_RELEASED,
    MESSAGE_KEYBOARD_ESCAPE_PRESSED, MESSAGE_KEYBOARD_HOTKEY_DETECTED,
    MESSAGE_KEYBOARD_SHIFT_PRESSED, MESSAGE_KEYBOARD_SHIFT_RELEASED,
};

#[derive(Debug, Clone)]
pub enum GlobalKeyboardEvent {
    CaptureHotkeyPressed,
    EscapePressed,
}

pub struct GlobalKeyboardListener;

impl GlobalKeyboardListener {
    #[allow(dead_code)]
    pub fn initialize() -> Self {
        log::debug!("{} initializing keyboard listener", LOG_TAG_KEYBOARD);
        Self
    }

    pub fn create_event_stream() -> impl Stream<Item = GlobalKeyboardEvent> {
        stream::channel(
            1,
            |mut output_channel: mpsc::Sender<GlobalKeyboardEvent>| async move {
                let (keyboard_sender, mut keyboard_receiver) = mpsc::channel(1);

                Self::spawn_keyboard_listener_thread(keyboard_sender);

                let mut state = KeyboardState::default();

                loop {
                    let keyboard_event = keyboard_receiver.select_next_some().await;
                    if let Some(global_event) = state.process_event(keyboard_event) {
                        let _ = output_channel.send(global_event).await;
                    }
                }
            },
        )
    }

    fn spawn_keyboard_listener_thread(mut keyboard_sender: mpsc::Sender<rdev::Event>) {
        std::thread::spawn(move || {
            log::info!(
                "{} Starting global keyboard listener thread",
                LOG_TAG_KEYBOARD
            );
            if let Err(e) = listen(move |event| {
                let _ = keyboard_sender.try_send(event);
            }) {
                log::error!("{} Failed to start keyboard listener: {:?}. This is expected if another instance is already running.", LOG_TAG_KEYBOARD, e);
            }
        });
    }
}

#[derive(Default)]
struct KeyboardState {
    is_alt_pressed: bool,
    is_shift_pressed: bool,
}

impl KeyboardState {
    fn process_event(&mut self, event: rdev::Event) -> Option<GlobalKeyboardEvent> {
        match event.event_type {
            EventType::KeyPress(key) => self.handle_key_press(key),
            EventType::KeyRelease(key) => self.handle_key_release(key),
            _ => None,
        }
    }

    fn handle_key_press(&mut self, key: Key) -> Option<GlobalKeyboardEvent> {
        match key {
            Key::Alt => {
                log::debug!("{} {}", LOG_TAG_KEYBOARD, MESSAGE_KEYBOARD_ALT_PRESSED);
                self.is_alt_pressed = true;
                None
            }
            Key::ShiftLeft | Key::ShiftRight => {
                log::debug!("{} {}", LOG_TAG_KEYBOARD, MESSAGE_KEYBOARD_SHIFT_PRESSED);
                self.is_shift_pressed = true;
                None
            }
            Key::KeyS if self.is_alt_pressed && self.is_shift_pressed => {
                log::info!("{} {}", LOG_TAG_KEYBOARD, MESSAGE_KEYBOARD_HOTKEY_DETECTED);
                Some(GlobalKeyboardEvent::CaptureHotkeyPressed)
            }
            Key::Escape => {
                log::debug!("{} {}", LOG_TAG_KEYBOARD, MESSAGE_KEYBOARD_ESCAPE_PRESSED);
                Some(GlobalKeyboardEvent::EscapePressed)
            }
            _ => None,
        }
    }

    fn handle_key_release(&mut self, key: Key) -> Option<GlobalKeyboardEvent> {
        match key {
            Key::Alt => {
                log::debug!("{} {}", LOG_TAG_KEYBOARD, MESSAGE_KEYBOARD_ALT_RELEASED);
                self.is_alt_pressed = false;
            }
            Key::ShiftLeft | Key::ShiftRight => {
                log::debug!("{} {}", LOG_TAG_KEYBOARD, MESSAGE_KEYBOARD_SHIFT_RELEASED);
                self.is_shift_pressed = false;
            }
            _ => {}
        }
        None
    }
}
