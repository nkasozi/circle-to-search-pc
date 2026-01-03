use std::sync::Arc;

use iced::window::Id;
use iced::{Element, Task};

use crate::adapters::{GoogleLensSearchProvider, ImgbbImageHostingService, TesseractOcrService};
use crate::core::interfaces::adapters::OcrService;
use crate::core::models::OcrResult;
use crate::core::orchestrators::app_orchestrator::{AppOrchestrator, OrchestratorMessage};
use crate::ports::{
    GlobalKeyboardEvent, GlobalKeyboardListener, SystemMousePositionProvider, XcapScreenCapturer,
};
use crate::user_settings;

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
    _tray: Option<crate::system_tray::SystemTray>,
}

impl CircleApp {
    pub fn build() -> (Self, Task<OrchestratorMessage>) {
        log::info!("[APP] Initializing application");

        let settings = user_settings::UserSettings::load().unwrap_or_else(|e| {
            log::warn!("[APP] Failed to load settings: {}, using defaults", e);
            user_settings::UserSettings::default()
        });

        let image_hosting_service = Arc::new(ImgbbImageHostingService::new());
        let reverse_image_search_provider = Arc::new(GoogleLensSearchProvider::new(
            image_hosting_service,
            settings.image_search_url_template.clone(),
        ));

        let should_show_window = !settings.run_in_system_tray;

        let orchestrator = AppOrchestrator::build(
            Arc::new(XcapScreenCapturer::initialize()),
            Arc::new(SystemMousePositionProvider::initialize()),
            Arc::new(DummyOcrService),
            reverse_image_search_provider,
            settings,
        );

        let tray = match crate::system_tray::SystemTray::build() {
            Ok(tray) => {
                log::info!("[APP] System tray initialized successfully");
                Some(tray)
            }
            Err(e) => {
                log::error!("[APP] Failed to initialize system tray: {}", e);
                None
            }
        };

        let mut tasks = vec![Task::future(async {
            match TesseractOcrService::build() {
                Ok(service) => {
                    log::info!("[APP] Tesseract OCR service initialized successfully");
                    OrchestratorMessage::OcrServiceReady(Arc::new(service) as Arc<dyn OcrService>)
                }
                Err(e) => {
                    log::error!("[APP] Failed to initialize Tesseract OCR service: {}", e);
                    OrchestratorMessage::OcrServiceFailed(e.to_string())
                }
            }
        })];

        if should_show_window {
            log::info!("[APP] System tray mode disabled, showing main window");
            tasks.push(Task::done(OrchestratorMessage::OpenMainWindow));
        } else {
            log::info!("[APP] Running in system tray mode, window hidden");
        }

        (
            Self {
                orchestrator,
                _tray: tray,
            },
            Task::batch(tasks),
        )
    }

    pub fn handle_update(&mut self, message: OrchestratorMessage) -> Task<OrchestratorMessage> {
        self.orchestrator.update(message)
    }

    pub fn render_view(&self, window_id: Id) -> Element<'_, OrchestratorMessage> {
        self.orchestrator.render_view(window_id)
    }

    pub fn handle_subscription(&self) -> iced::Subscription<OrchestratorMessage> {
        use iced::window;

        iced::Subscription::batch([
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
                            if let Some(event) = crate::system_tray::SystemTray::poll_events() {
                                let _ = output.try_send(OrchestratorMessage::TrayEvent(event));
                            }
                        }
                    },
                )
            }),
        ])
    }
}
