use std::sync::mpsc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_core_foundation::CFString;
use objc2_foundation::{NSCopying, NSDictionary, NSError, NSNumber, NSString, NSURL};

pub const COMPLETION_TIMEOUT: Duration = Duration::from_secs(20);
const COMPLETION_TIMEOUT_ERROR: &str = "timed out waiting for a system callback";

pub fn bridged_key(key: &'static CFString) -> &'static NSString {
    unsafe { &*(key as *const CFString as *const NSString) }
}

pub fn required_static<T: ?Sized>(value: Option<&'static T>, name: &str) -> Result<&'static T> {
    value.ok_or_else(|| anyhow!("system constant {} is unavailable", name))
}

pub fn number_object(value: i64) -> Retained<AnyObject> {
    Retained::into_super(Retained::into_super(NSNumber::new_i64(value))).into()
}

pub fn string_object(value: &NSString) -> Retained<AnyObject> {
    Retained::into_super(Retained::into_super(value.copy())).into()
}

pub fn dictionary_object(
    dictionary: Retained<NSDictionary<NSString, AnyObject>>,
) -> Retained<AnyObject> {
    Retained::into_super(Retained::into_super(dictionary)).into()
}

pub fn build_dictionary(
    entries: &[(&NSString, Retained<AnyObject>)],
) -> Retained<NSDictionary<NSString, AnyObject>> {
    let keys: Vec<&NSString> = entries.iter().map(|(key, _)| *key).collect();
    let values: Vec<&AnyObject> = entries.iter().map(|(_, value)| &**value).collect();
    NSDictionary::from_slices(&keys, &values)
}

pub fn file_url(path: &std::path::Path) -> Retained<NSURL> {
    NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()))
}

pub fn describe_error(error: &NSError) -> String {
    error.localizedDescription().to_string()
}

pub fn wait_for_unit_completion(start: impl FnOnce(&RcBlock<dyn Fn()>)) -> Result<()> {
    let (sender, receiver) = mpsc::channel::<()>();
    let completion = RcBlock::new(move || {
        let _ = sender.send(());
    });
    start(&completion);
    receiver
        .recv_timeout(COMPLETION_TIMEOUT)
        .map_err(|_| anyhow!(COMPLETION_TIMEOUT_ERROR))
}

pub fn wait_for_error_completion(start: impl FnOnce(&RcBlock<dyn Fn(*mut NSError)>)) -> Result<()> {
    let (sender, receiver) = mpsc::channel::<Option<String>>();
    let completion = RcBlock::new(move |error: *mut NSError| {
        let message = unsafe { error.as_ref() }.map(describe_error);
        let _ = sender.send(message);
    });
    start(&completion);
    match receiver.recv_timeout(COMPLETION_TIMEOUT) {
        Ok(None) => Ok(()),
        Ok(Some(message)) => Err(anyhow!(message)),
        Err(_) => Err(anyhow!(COMPLETION_TIMEOUT_ERROR)),
    }
}
