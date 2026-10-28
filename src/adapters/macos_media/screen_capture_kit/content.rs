use std::sync::mpsc;

use anyhow::{anyhow, Context, Result};
use block2::RcBlock;
use objc2::rc::Retained;
use objc2::AllocAnyThread;
use objc2_foundation::{NSArray, NSError};
use objc2_screen_capture_kit::{
    SCContentFilter, SCDisplay, SCRunningApplication, SCShareableContent, SCWindow,
};

use super::super::asset_reading::ThreadBound;
use super::super::foundation_helpers::{describe_error, COMPLETION_TIMEOUT};

const DISPLAY_NOT_FOUND_ERROR: &str = "the display to record could not be found";
const SCREEN_PERMISSION_HINT: &str = "Screen Recording permission is required";
const DISPLAY_ORIGIN_TOLERANCE: f64 = 1.0;
const LOG_TAG: &str = "[SCREEN_CAPTURE_KIT]";
const CONTENT_FILTER_BUILT_EVENT: &str = "content_filter_built";

type ContentResult = std::result::Result<ThreadBound<Retained<SCShareableContent>>, String>;

pub fn fetch_shareable_content() -> Result<Retained<SCShareableContent>> {
    let (sender, receiver) = mpsc::channel::<ContentResult>();
    let completion = RcBlock::new(
        move |content: *mut SCShareableContent, error: *mut NSError| {
            let outcome = match (unsafe { Retained::retain(content) }, unsafe {
                error.as_ref()
            }) {
                (Some(retained_content), _) => Ok(ThreadBound(retained_content)),
                (None, Some(error)) => Err(describe_error(error)),
                (None, None) => Err(SCREEN_PERMISSION_HINT.to_string()),
            };
            let _ = sender.send(outcome);
        },
    );
    unsafe { SCShareableContent::getShareableContentWithCompletionHandler(&completion) };
    let outcome = receiver
        .recv_timeout(COMPLETION_TIMEOUT)
        .context(SCREEN_PERMISSION_HINT)?;
    outcome
        .map(ThreadBound::into_inner)
        .map_err(|message| anyhow!("{}: {}", SCREEN_PERMISSION_HINT, message))
}

pub fn find_display(
    content: &SCShareableContent,
    origin_x: i32,
    origin_y: i32,
) -> Result<Retained<SCDisplay>> {
    let displays = unsafe { content.displays() };
    displays
        .iter()
        .find(|display| {
            let frame = unsafe { display.frame() };
            (frame.origin.x - origin_x as f64).abs() < DISPLAY_ORIGIN_TOLERANCE
                && (frame.origin.y - origin_y as f64).abs() < DISPLAY_ORIGIN_TOLERANCE
        })
        .or_else(|| displays.firstObject().filter(|_| displays.count() == 1))
        .context(DISPLAY_NOT_FOUND_ERROR)
}

pub fn build_content_filter(
    content: &SCShareableContent,
    display: &SCDisplay,
    visible_own_window_titles: &[String],
) -> Retained<SCContentFilter> {
    let own_process_identifier = std::process::id() as i32;
    let own_applications: Vec<Retained<SCRunningApplication>> = unsafe { content.applications() }
        .iter()
        .filter(|application| unsafe { application.processID() } == own_process_identifier)
        .collect();
    let visible_windows: Vec<Retained<SCWindow>> = unsafe { content.windows() }
        .iter()
        .filter(|window| {
            is_visible_own_window(window, own_process_identifier, visible_own_window_titles)
        })
        .collect();
    let applications_array = NSArray::from_retained_slice(&own_applications);
    let windows_array = NSArray::from_retained_slice(&visible_windows);
    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": CONTENT_FILTER_BUILT_EVENT, "excluded_applications": own_applications.len(), "visible_own_windows": visible_windows.len()})
    );
    unsafe {
        SCContentFilter::initWithDisplay_excludingApplications_exceptingWindows(
            SCContentFilter::alloc(),
            display,
            &applications_array,
            &windows_array,
        )
    }
}

fn is_visible_own_window(
    window: &SCWindow,
    own_process_identifier: i32,
    visible_titles: &[String],
) -> bool {
    let owner_matches = unsafe { window.owningApplication() }
        .is_some_and(|application| unsafe { application.processID() } == own_process_identifier);
    let title_matches = unsafe { window.title() }.is_some_and(|title| {
        visible_titles
            .iter()
            .any(|visible_title| *visible_title == title.to_string())
    });
    owner_matches && title_matches
}
