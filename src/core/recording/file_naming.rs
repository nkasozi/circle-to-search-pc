use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;

const RECORDING_FILE_NAME_FORMAT: &str = "Recording %Y-%m-%d at %H.%M.%S";
const RECORDING_FILE_EXTENSION: &str = "mp4";
const MAXIMUM_SUFFIX_ATTEMPTS: u32 = 1_000;

pub fn build_recording_file_stem(local_time: &NaiveDateTime) -> String {
    local_time.format(RECORDING_FILE_NAME_FORMAT).to_string()
}

pub fn build_unique_recording_path(
    directory: &Path,
    local_time: &NaiveDateTime,
    path_exists: impl Fn(&Path) -> bool,
) -> PathBuf {
    let stem = build_recording_file_stem(local_time);
    let first_choice = directory.join(format!("{}.{}", stem, RECORDING_FILE_EXTENSION));
    if !path_exists(&first_choice) {
        return first_choice;
    }
    (2..MAXIMUM_SUFFIX_ATTEMPTS)
        .map(|suffix| {
            directory.join(format!(
                "{} ({}).{}",
                stem, suffix, RECORDING_FILE_EXTENSION
            ))
        })
        .find(|candidate| !path_exists(candidate))
        .unwrap_or(first_choice)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn sample_time() -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 10, 21)
            .and_then(|date| date.and_hms_opt(14, 3, 22))
            .unwrap_or_default()
    }

    #[test]
    fn test_file_stem_is_human_friendly() {
        assert_eq!(
            build_recording_file_stem(&sample_time()),
            "Recording 2026-10-21 at 14.03.22"
        );
    }

    #[test]
    fn test_unique_path_adds_suffix_when_taken() {
        let directory = Path::new("/recordings");
        let taken = directory.join("Recording 2026-10-21 at 14.03.22.mp4");
        let path =
            build_unique_recording_path(directory, &sample_time(), |candidate| candidate == taken);
        assert_eq!(
            path,
            directory.join("Recording 2026-10-21 at 14.03.22 (2).mp4")
        );
    }

    #[test]
    fn test_unique_path_uses_plain_name_when_free() {
        let directory = Path::new("/recordings");
        let path = build_unique_recording_path(directory, &sample_time(), |_| false);
        assert_eq!(path, directory.join("Recording 2026-10-21 at 14.03.22.mp4"));
    }
}
