use super::*;

mod debug_format;

#[derive(Clone)]
pub enum OrchestratorMessage {
    #[allow(dead_code)]
    OpenMainWindow,
    CreateHiddenWindow,
    CaptureScreen,
    PerformCapture,
    OpenCaptureOverlay(i32, i32, CaptureBuffer),
    CaptureError(String),
    CaptureOverlayMessage(Id, CaptureViewMessage),
    ConfirmSelection(Id),
    ShowCroppedImage(CaptureBuffer, Rectangle),
    ProcessOcr(Id, CaptureBuffer),
    OcrComplete(Id, Result<OcrResult, String>),
    OcrServiceReady(Arc<dyn OcrService>),
    OcrServiceFailed(String),
    InteractiveOcrMessage(Id, crate::presentation::InteractiveOcrMessage),
    PerformImageSearch(Id, CaptureBuffer, Option<String>),
    SpinnerTick,
    #[allow(dead_code)]
    CloseWindow(Id),
    WindowClosed(Id),
    WindowFocused(Id),
    GlobalHotkey(GlobalHotkeyAction),
    Recording(RecordingMessage),
    Home(crate::presentation::home_view::HomeMessage),
    RecentRecordingsLoaded(Vec<RecordingSummary>),
    Player(Id, crate::presentation::player_view::PlayerMessage),
    PlayerTick,
    PlayerFrameAllocated(Id, Option<iced::widget::image::Allocation>),
    EscapePressed(Id),
    HotkeysApplied(Vec<HotkeyRegistrationFailure>),
    ReapplyHotkeys,
    Settings(crate::presentation::settings_view::SettingsMessage),
    KeyPressedInWindow(
        Id,
        crate::presentation::settings_view::PressedModifiers,
        String,
    ),
    TrayEvent(TrayEvent),
    #[allow(dead_code)]
    HideMainWindow,
    OpenOnboarding,
    OnboardingMsg(Id, OnboardingMessage),
    CopyImageToClipboard(Id, CaptureBuffer, Vec<crate::presentation::DrawStroke>),
    SaveImageToFile(Id, CaptureBuffer, Vec<crate::presentation::DrawStroke>),
    #[allow(dead_code)]
    OpenWindowPicker,
    WindowPickerMsg(Id, WindowPickerMessage),
    WindowsListLoaded(Id, Vec<WindowInfo>),
    FocusWindowAndCapture(String),
    #[allow(dead_code)]
    CaptureSelectedWindow(u32),
    WindowCaptureComplete(CaptureBuffer),
    WindowCaptureError(String),
}
