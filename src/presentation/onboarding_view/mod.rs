mod steps;
mod ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardingStep {
    Welcome,
    ScreenAccess,
    Shortcuts,
    Startup,
}

impl OnboardingStep {
    pub const ALL: [OnboardingStep; 4] = [
        OnboardingStep::Welcome,
        OnboardingStep::ScreenAccess,
        OnboardingStep::Shortcuts,
        OnboardingStep::Startup,
    ];

    pub fn next(self) -> Self {
        match self {
            OnboardingStep::Welcome => OnboardingStep::ScreenAccess,
            OnboardingStep::ScreenAccess => OnboardingStep::Shortcuts,
            OnboardingStep::Shortcuts | OnboardingStep::Startup => OnboardingStep::Startup,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            OnboardingStep::Welcome | OnboardingStep::ScreenAccess => OnboardingStep::Welcome,
            OnboardingStep::Shortcuts => OnboardingStep::ScreenAccess,
            OnboardingStep::Startup => OnboardingStep::Shortcuts,
        }
    }

    pub fn position(self) -> usize {
        Self::ALL.iter().position(|step| *step == self).unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
pub enum OnboardingMessage {
    NextStep,
    PreviousStep,
    RequestScreenAccess,
    OpenScreenRecordingSettings,
    RefreshPermissions,
    RestartApp,
    ToggleLaunchAtLogin(bool),
    ToggleShowHomeOnLaunch(bool),
    FinishOnboarding,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OnboardingView {
    current_step: OnboardingStep,
    screen_access_granted: bool,
    screen_access_requested: bool,
    launch_at_login: bool,
    show_home_on_launch: bool,
    capture_hotkey: String,
    recording_hotkey: String,
}

pub struct OnboardingSetup {
    pub screen_access_granted: bool,
    pub launch_at_login: bool,
    pub show_home_on_launch: bool,
    pub capture_hotkey: String,
    pub recording_hotkey: String,
}

impl OnboardingView {
    pub fn new(setup: OnboardingSetup) -> Self {
        Self {
            current_step: OnboardingStep::Welcome,
            screen_access_granted: setup.screen_access_granted,
            screen_access_requested: false,
            launch_at_login: setup.launch_at_login,
            show_home_on_launch: setup.show_home_on_launch,
            capture_hotkey: setup.capture_hotkey,
            recording_hotkey: setup.recording_hotkey,
        }
    }

    pub fn update_screen_access(&mut self, granted: bool) -> bool {
        let changed = self.screen_access_granted != granted;
        self.screen_access_granted = granted;
        changed
    }

    pub fn is_on_screen_access_step(&self) -> bool {
        self.current_step == OnboardingStep::ScreenAccess
    }

    pub fn is_launch_at_login_enabled(&self) -> bool {
        self.launch_at_login
    }

    pub fn is_show_home_on_launch_enabled(&self) -> bool {
        self.show_home_on_launch
    }

    pub fn handle_message(&mut self, message: &OnboardingMessage) -> bool {
        match message {
            OnboardingMessage::NextStep => self.current_step = self.current_step.next(),
            OnboardingMessage::PreviousStep => self.current_step = self.current_step.previous(),
            OnboardingMessage::RequestScreenAccess => self.screen_access_requested = true,
            OnboardingMessage::ToggleLaunchAtLogin(enabled) => self.launch_at_login = *enabled,
            OnboardingMessage::ToggleShowHomeOnLaunch(enabled) => {
                self.show_home_on_launch = *enabled
            }
            OnboardingMessage::FinishOnboarding => return true,
            OnboardingMessage::OpenScreenRecordingSettings
            | OnboardingMessage::RefreshPermissions
            | OnboardingMessage::RestartApp => {}
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_view(granted: bool) -> OnboardingView {
        OnboardingView::new(OnboardingSetup {
            screen_access_granted: granted,
            launch_at_login: false,
            show_home_on_launch: true,
            capture_hotkey: "Alt+Shift+S".into(),
            recording_hotkey: "Alt+Shift+R".into(),
        })
    }

    #[test]
    fn test_steps_advance_and_go_back() {
        let mut view = build_view(false);
        view.handle_message(&OnboardingMessage::NextStep);
        assert!(view.is_on_screen_access_step());
        view.handle_message(&OnboardingMessage::NextStep);
        view.handle_message(&OnboardingMessage::NextStep);
        view.handle_message(&OnboardingMessage::NextStep);
        assert_eq!(view.current_step, OnboardingStep::Startup);
        view.handle_message(&OnboardingMessage::PreviousStep);
        assert_eq!(view.current_step, OnboardingStep::Shortcuts);
    }

    #[test]
    fn test_finish_reports_completion() {
        let mut view = build_view(true);
        assert!(view.handle_message(&OnboardingMessage::FinishOnboarding));
        assert!(!view.handle_message(&OnboardingMessage::ToggleLaunchAtLogin(true)));
        assert!(view.is_launch_at_login_enabled());
    }

    #[test]
    fn test_screen_access_updates_report_changes() {
        let mut view = build_view(false);
        assert!(view.update_screen_access(true));
        assert!(!view.update_screen_access(true));
    }
}
