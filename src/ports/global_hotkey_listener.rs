use std::str::FromStr;
use std::time::Duration;

use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use iced::futures::{channel::mpsc, SinkExt, Stream};
use iced::stream;

use crate::core::models::{
    GlobalHotkeyAction, HotkeyBinding, HotkeyRegistration, HotkeyRegistrationFailure,
};

#[cfg(test)]
mod tests;

const LOG_TAG: &str = "[GLOBAL_HOTKEY]";
const EVENT_MANAGER_CREATED: &str = "global_hotkey_manager_created";
const EVENT_HOTKEYS_CLEARED: &str = "global_hotkeys_cleared";
const EVENT_HOTKEYS_UNREGISTER_FAILED: &str = "global_hotkeys_unregister_failed";
const EVENT_HOTKEY_REGISTERED: &str = "global_hotkey_registered";
const EVENT_HOTKEY_REGISTRATION_FAILED: &str = "global_hotkey_registration_failed";
const EVENT_POLL_INTERVAL_MILLISECONDS: u64 = 40;
const EVENT_CHANNEL_CAPACITY: usize = 8;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct HotkeyApplyOutcome {
    pub registrations: Vec<HotkeyRegistration>,
    pub failures: Vec<HotkeyRegistrationFailure>,
}

pub struct GlobalHotkeyListener {
    manager: GlobalHotKeyManager,
    active_hotkeys: Vec<HotKey>,
}

impl GlobalHotkeyListener {
    pub fn build() -> anyhow::Result<Self> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|creation_error| anyhow::anyhow!(creation_error.to_string()))?;
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": EVENT_MANAGER_CREATED})
        );
        Ok(Self {
            manager,
            active_hotkeys: Vec::new(),
        })
    }

    pub fn apply_bindings(&mut self, bindings: &[HotkeyBinding]) -> HotkeyApplyOutcome {
        let unregistered_count = self.unregister_active_hotkeys();
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": EVENT_HOTKEYS_CLEARED, "count": unregistered_count})
        );
        bindings
            .iter()
            .fold(HotkeyApplyOutcome::default(), |outcome, binding| {
                self.register_binding(outcome, binding)
            })
    }

    fn unregister_active_hotkeys(&mut self) -> usize {
        let previous_hotkeys = std::mem::take(&mut self.active_hotkeys);
        let previous_count = previous_hotkeys.len();
        if let Err(unregister_error) = self.manager.unregister_all(&previous_hotkeys) {
            log::warn!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": EVENT_HOTKEYS_UNREGISTER_FAILED, "error": unregister_error.to_string()})
            );
        }
        previous_count
    }

    fn register_binding(
        &mut self,
        mut outcome: HotkeyApplyOutcome,
        binding: &HotkeyBinding,
    ) -> HotkeyApplyOutcome {
        match self.parse_and_register(&binding.accelerator) {
            Ok(hotkey_id) => {
                log::info!(
                    "{} {}",
                    LOG_TAG,
                    serde_json::json!({"event": EVENT_HOTKEY_REGISTERED, "accelerator": binding.accelerator, "action": format!("{:?}", binding.action)})
                );
                outcome.registrations.push(HotkeyRegistration {
                    hotkey_id,
                    action: binding.action,
                });
            }
            Err(reason) => {
                log::error!(
                    "{} {}",
                    LOG_TAG,
                    serde_json::json!({"event": EVENT_HOTKEY_REGISTRATION_FAILED, "accelerator": binding.accelerator, "reason": reason})
                );
                outcome.failures.push(HotkeyRegistrationFailure {
                    action: binding.action,
                    accelerator: binding.accelerator.clone(),
                    reason,
                });
            }
        }
        outcome
    }

    fn parse_and_register(&mut self, accelerator: &str) -> Result<u32, String> {
        let hotkey = HotKey::from_str(accelerator).map_err(|error| error.to_string())?;
        self.manager
            .register(hotkey)
            .map_err(|error| error.to_string())?;
        self.active_hotkeys.push(hotkey);
        Ok(hotkey.id())
    }

    pub fn create_event_stream(
        registrations: &Vec<HotkeyRegistration>,
    ) -> impl Stream<Item = GlobalHotkeyAction> {
        let registrations = registrations.clone();
        stream::channel(
            EVENT_CHANNEL_CAPACITY,
            move |mut output: mpsc::Sender<GlobalHotkeyAction>| async move {
                loop {
                    let Some(action) = Self::wait_for_next_action(&registrations).await else {
                        continue;
                    };
                    let _ = output.send(action).await;
                }
            },
        )
    }

    async fn wait_for_next_action(
        registrations: &[HotkeyRegistration],
    ) -> Option<GlobalHotkeyAction> {
        tokio::time::sleep(Duration::from_millis(EVENT_POLL_INTERVAL_MILLISECONDS)).await;
        let event = GlobalHotKeyEvent::receiver().try_recv().ok()?;
        resolve_pressed_action(&event, registrations)
    }
}

pub fn resolve_pressed_action(
    event: &GlobalHotKeyEvent,
    registrations: &[HotkeyRegistration],
) -> Option<GlobalHotkeyAction> {
    if event.state != HotKeyState::Pressed {
        return None;
    }
    registrations
        .iter()
        .find(|registration| registration.hotkey_id == event.id)
        .map(|registration| registration.action)
}
