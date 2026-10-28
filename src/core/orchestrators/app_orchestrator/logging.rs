use super::*;

fn build_log_entry(
    correlation_id: &str,
    event: &str,
    details: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "event": event,
        "correlation_id": correlation_id,
        "details": details,
    })
}

impl AppOrchestrator {
    pub(super) fn log_info_event(&self, event: &str, details: serde_json::Value) -> bool {
        log::info!(
            "{}",
            build_log_entry(&self.current_correlation_id, event, details)
        );
        true
    }

    pub(super) fn log_error_event(&self, event: &str, details: serde_json::Value) -> bool {
        log::error!(
            "{}",
            build_log_entry(&self.current_correlation_id, event, details)
        );
        true
    }

    pub(super) fn log_info_event_for_correlation(
        correlation_id: String,
        event: &str,
        details: serde_json::Value,
    ) -> bool {
        log::info!("{}", build_log_entry(&correlation_id, event, details));
        true
    }

    pub(super) fn log_error_event_for_correlation(
        correlation_id: String,
        event: &str,
        details: serde_json::Value,
    ) -> bool {
        log::error!("{}", build_log_entry(&correlation_id, event, details));
        true
    }

    pub(super) fn log_debug_event(&self, event: &str, details: serde_json::Value) -> bool {
        log::debug!(
            "{}",
            build_log_entry(&self.current_correlation_id, event, details)
        );
        true
    }
}
