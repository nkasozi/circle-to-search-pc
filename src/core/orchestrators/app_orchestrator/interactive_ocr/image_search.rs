use super::super::*;
use super::constants::*;
use crate::presentation::InteractiveOcrMessage;

type ImageSearchOutcome = Result<anyhow::Result<String>, tokio::time::error::Elapsed>;

impl AppOrchestrator {
    pub(in crate::core::orchestrators::app_orchestrator) fn handle_perform_image_search(
        &mut self,
        window_id: Id,
        buffer: CaptureBuffer,
        query: Option<String>,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_IMAGE_SEARCH_STARTED,
            serde_json::json!({
                "window_id": format!("{:?}", window_id),
                "has_query": query.as_ref().map(|query_text| !query_text.is_empty()).unwrap_or(false),
            }),
        );

        let search_provider = Arc::clone(&self.reverse_image_search_provider);
        let correlation_id = self.current_correlation_id();

        Task::batch(vec![
            Task::done(OrchestratorMessage::InteractiveOcrMessage(
                window_id,
                InteractiveOcrMessage::SearchUploading,
            )),
            Task::future(async move {
                let search_future = search_provider.perform_search(&buffer, query.as_deref());
                let timeout_duration =
                    std::time::Duration::from_secs(global_constants::IMAGE_SEARCH_TIMEOUT_SECONDS);
                let search_outcome = tokio::time::timeout(timeout_duration, search_future).await;
                build_image_search_result_message(window_id, search_outcome, &correlation_id)
            }),
        ])
    }
}

fn build_image_search_result_message(
    window_id: Id,
    search_outcome: ImageSearchOutcome,
    correlation_id: &str,
) -> OrchestratorMessage {
    match search_outcome {
        Ok(Ok(_search_url)) => build_search_completed_message(window_id, correlation_id),
        Ok(Err(error)) => build_search_failed_message(window_id, error, correlation_id),
        Err(_) => build_search_timed_out_message(window_id, correlation_id),
    }
}

fn build_search_completed_message(window_id: Id, correlation_id: &str) -> OrchestratorMessage {
    AppOrchestrator::log_info_event_for_correlation(
        correlation_id.to_string(),
        EVENT_IMAGE_SEARCH_COMPLETED,
        serde_json::json!({
            "window_id": format!("{:?}", window_id),
        }),
    );
    OrchestratorMessage::InteractiveOcrMessage(window_id, InteractiveOcrMessage::SearchCompleted)
}

fn build_search_failed_message(
    window_id: Id,
    error: anyhow::Error,
    correlation_id: &str,
) -> OrchestratorMessage {
    AppOrchestrator::log_error_event_for_correlation(
        correlation_id.to_string(),
        EVENT_IMAGE_SEARCH_FAILED,
        serde_json::json!({
            "window_id": format!("{:?}", window_id),
            "error": error.to_string(),
        }),
    );
    let failure_message = format!("{}{}", error, global_constants::IMAGE_SEARCH_FAILURE_SUFFIX);
    OrchestratorMessage::InteractiveOcrMessage(
        window_id,
        InteractiveOcrMessage::SearchFailed(failure_message),
    )
}

fn build_search_timed_out_message(window_id: Id, correlation_id: &str) -> OrchestratorMessage {
    AppOrchestrator::log_error_event_for_correlation(
        correlation_id.to_string(),
        EVENT_IMAGE_SEARCH_TIMEOUT,
        serde_json::json!({
            "window_id": format!("{:?}", window_id),
            "timeout_seconds": global_constants::IMAGE_SEARCH_TIMEOUT_SECONDS,
        }),
    );
    OrchestratorMessage::InteractiveOcrMessage(
        window_id,
        InteractiveOcrMessage::SearchFailed(
            global_constants::IMAGE_SEARCH_TIMEOUT_MESSAGE.to_string(),
        ),
    )
}
