use std::sync::Arc;

use iced::window::Id;
use iced::{Element, Task};

use crate::adapters::{
    macos_app_behavior, GoogleLensSearchProvider, ImgbbImageHostingService, TesseractOcrService,
};
use crate::core::interfaces::adapters::OcrService;
use crate::core::models::{OcrResult, UserSettings};
use crate::core::orchestrators::app_orchestrator::{AppOrchestrator, OrchestratorMessage};
use crate::ports::{
    GlobalKeyboardEvent, GlobalKeyboardListener, SystemMousePositionProvider, SystemTray,
    XcapScreenCapturer,
};

struct DummyOcrService;

#[async_trait::async_trait]
impl OcrService for DummyOcrService {
    async fn extract_text_from_image(
        &self,
        _image: &image::DynamicImage,
    ) -> anyhow::Result<OcrResult> {
        anyhow::bail!("OCR service not initialized yet")
    }
}

pub struct CircleApp {
    orchestrator: AppOrchestrator,
    _tray: Option<SystemTray>,
    keyboard_listener_enabled: bool,
}

fn check_all_permissions_granted() -> bool {
    #[cfg(target_os = "macos")]
    {
        use crate::adapters::macos_permissions::macos::{
            check_input_monitoring_permission, check_screen_recording_permission,
        };

        let screen_recording_granted = check_screen_recording_permission();
        let input_monitoring_granted = check_input_monitoring_permission();

        log::info!(
            "[APP] Permission check: screen_recording={}, input_monitoring={}",
            screen_recording_granted,
            input_monitoring_granted
        );

        screen_recording_granted && input_monitoring_granted
    }

    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

impl CircleApp {
    pub fn build() -> (Self, Task<OrchestratorMessage>) {
        log::info!("[APP] Initializing application");

        macos_app_behavior::macos::hide_dock_icon();

        let settings = UserSettings::load().unwrap_or_else(|e| {
            log::warn!("[APP] Failed to load settings: {}, using defaults", e);
            UserSettings::default()
        });

        let onboarding_complete = settings.onboarding_complete;

        let image_hosting_service = Arc::new(ImgbbImageHostingService::new());
        let reverse_image_search_provider = Arc::new(GoogleLensSearchProvider::new(
            image_hosting_service,
            settings.image_search_url_template.clone(),
        ));

        let orchestrator = AppOrchestrator::build(
            Arc::new(XcapScreenCapturer::initialize()),
            Arc::new(SystemMousePositionProvider::initialize()),
            Arc::new(DummyOcrService),
            reverse_image_search_provider,
            settings,
        );

        let tray = match SystemTray::build() {
            Ok(tray) => {
                log::info!("[APP] System tray initialized successfully");
                Some(tray)
            }
            Err(e) => {
                log::error!("[APP] Failed to initialize system tray: {}", e);
                None
            }
        };

        let mut tasks = vec![
            Task::done(OrchestratorMessage::CreateHiddenWindow),
            Task::future(async {
                match TesseractOcrService::build() {
                    Ok(service) => {
                        log::info!("[APP] Tesseract OCR service initialized successfully");
                        OrchestratorMessage::OcrServiceReady(
                            Arc::new(service) as Arc<dyn OcrService>
                        )
                    }
                    Err(e) => {
                        log::error!("[APP] Failed to initialize Tesseract OCR service: {}", e);
                        OrchestratorMessage::OcrServiceFailed(e.to_string())
                    }
                }
            }),
        ];

        let all_permissions_granted = check_all_permissions_granted();
        let needs_onboarding = !onboarding_complete || !all_permissions_granted;
        let keyboard_listener_enabled = !needs_onboarding;

        if needs_onboarding {
            log::info!(
                "[APP] Permissions missing or onboarding incomplete, showing onboarding window"
            );
            tasks.push(Task::done(OrchestratorMessage::OpenOnboarding));
        } else {
            log::info!("[APP] Permissions OK, showing main window");
            tasks.push(Task::done(OrchestratorMessage::OpenMainWindow));
        }

        (
            Self {
                orchestrator,
                _tray: tray,
                keyboard_listener_enabled,
            },
            Task::batch(tasks),
        )
    }

    pub fn handle_update(&mut self, message: OrchestratorMessage) -> Task<OrchestratorMessage> {
        if matches!(message, OrchestratorMessage::EnableKeyboardListener) {
            log::info!("[APP] Enabling keyboard listener after onboarding");
            self.keyboard_listener_enabled = true;
            return Task::none();
        }
        self.orchestrator.update(message)
    }

    pub fn render_view(&self, window_id: Id) -> Element<'_, OrchestratorMessage> {
        self.orchestrator.render_view(window_id)
    }

    pub fn handle_subscription(&self) -> iced::Subscription<OrchestratorMessage> {
        use iced::window;

        let mut subscriptions = vec![
            iced::event::listen_with(|event, _status, id| {
                if let iced::Event::Window(window::Event::Closed) = event {
                    return Some(OrchestratorMessage::WindowClosed(id));
                }
                None
            }),
            iced::Subscription::run(|| {
                iced::stream::channel(
                    10,
                    |mut output: futures::channel::mpsc::Sender<OrchestratorMessage>| async move {
                        loop {
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                            if let Some(event) = SystemTray::poll_events() {
                                let _ = output.try_send(OrchestratorMessage::TrayEvent(event));
                            }
                        }
                    },
                )
            }),
            iced::Subscription::run(|| {
                iced::stream::channel(
                    10,
                    |mut output: futures::channel::mpsc::Sender<OrchestratorMessage>| async move {
                        loop {
                            tokio::time::sleep(std::time::Duration::from_millis(80)).await;
                            let _ = output.try_send(OrchestratorMessage::SpinnerTick);
                        }
                    },
                )
            }),
        ];

        if self.keyboard_listener_enabled {
            subscriptions.push(
                iced::Subscription::run(GlobalKeyboardListener::create_event_stream).map(|event| {
                    match event {
                        GlobalKeyboardEvent::CaptureHotkeyPressed => {
                            OrchestratorMessage::Keyboard(GlobalKeyboardEvent::CaptureHotkeyPressed)
                        }
                        GlobalKeyboardEvent::EscapePressed => {
                            OrchestratorMessage::Keyboard(GlobalKeyboardEvent::EscapePressed)
                        }
                    }
                }),
            );
        }

        iced::Subscription::batch(subscriptions)
    }
}
