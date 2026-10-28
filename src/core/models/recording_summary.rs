use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const BYTES_PER_KILOBYTE: f64 = 1024.0;
const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 3_600;
const SECONDS_PER_DAY: u64 = 86_400;
const RELATIVE_TIME_JUST_NOW: &str = "just now";
const RELATIVE_TIME_MINUTES_SUFFIX: &str = " min ago";
const RELATIVE_TIME_HOURS_SUFFIX: &str = " h ago";
const RELATIVE_TIME_DAYS_SUFFIX: &str = " d ago";
const FILE_SIZE_UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordingSummary {
    pub file_path: PathBuf,
    pub file_name: String,
    pub modified_at: SystemTime,
    pub size_bytes: u64,
}

pub fn format_file_size(size_bytes: u64) -> String {
    let mut scaled_size = size_bytes as f64;
    let mut unit_index = 0usize;
    while scaled_size >= BYTES_PER_KILOBYTE && unit_index < FILE_SIZE_UNITS.len() - 1 {
        scaled_size /= BYTES_PER_KILOBYTE;
        unit_index += 1;
    }
    match unit_index {
        0 => format!("{} {}", size_bytes, FILE_SIZE_UNITS[0]),
        _ => format!("{:.1} {}", scaled_size, FILE_SIZE_UNITS[unit_index]),
    }
}

pub fn format_relative_time(moment: SystemTime, now: SystemTime) -> String {
    let elapsed_seconds = now
        .duration_since(moment)
        .unwrap_or(Duration::ZERO)
        .as_secs();
    match elapsed_seconds {
        seconds if seconds < SECONDS_PER_MINUTE => RELATIVE_TIME_JUST_NOW.to_string(),
        seconds if seconds < SECONDS_PER_HOUR => format!(
            "{}{}",
            seconds / SECONDS_PER_MINUTE,
            RELATIVE_TIME_MINUTES_SUFFIX
        ),
        seconds if seconds < SECONDS_PER_DAY => {
            format!(
                "{}{}",
                seconds / SECONDS_PER_HOUR,
                RELATIVE_TIME_HOURS_SUFFIX
            )
        }
        seconds => format!("{}{}", seconds / SECONDS_PER_DAY, RELATIVE_TIME_DAYS_SUFFIX),
    }
}

pub fn format_elapsed_clock(elapsed: Duration) -> String {
    let total_seconds = elapsed.as_secs();
    let hours = total_seconds / SECONDS_PER_HOUR;
    let minutes = (total_seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
    let seconds = total_seconds % SECONDS_PER_MINUTE;
    match hours {
        0 => format!("{:02}:{:02}", minutes, seconds),
        _ => format!("{}:{:02}:{:02}", hours, minutes, seconds),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size_bytes() {
        assert_eq!(format_file_size(512), "512 B");
    }

    #[test]
    fn test_format_file_size_megabytes() {
        assert_eq!(format_file_size(5 * 1024 * 1024 + 512 * 1024), "5.5 MB");
    }

    #[test]
    fn test_format_file_size_caps_at_gigabytes() {
        assert_eq!(format_file_size(3 * 1024 * 1024 * 1024 * 1024), "3072.0 GB");
    }

    #[test]
    fn test_format_relative_time_buckets() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
        let at = |seconds_ago: u64| now - Duration::from_secs(seconds_ago);
        assert_eq!(format_relative_time(at(5), now), "just now");
        assert_eq!(format_relative_time(at(125), now), "2 min ago");
        assert_eq!(format_relative_time(at(7_300), now), "2 h ago");
        assert_eq!(format_relative_time(at(200_000), now), "2 d ago");
    }

    #[test]
    fn test_format_relative_time_future_moment_is_just_now() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        let future = now + Duration::from_secs(50);
        assert_eq!(format_relative_time(future, now), "just now");
    }

    #[test]
    fn test_format_elapsed_clock() {
        assert_eq!(format_elapsed_clock(Duration::from_secs(65)), "01:05");
        assert_eq!(format_elapsed_clock(Duration::from_secs(3_725)), "1:02:05");
        assert_eq!(format_elapsed_clock(Duration::ZERO), "00:00");
    }
}
