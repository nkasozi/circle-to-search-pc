use std::time::Duration;

const DURATION_MARKER: &str = "Duration: ";
const VIDEO_STREAM_MARKER: &str = "Video: ";
const AUDIO_STREAM_MARKER: &str = "Audio: ";
const FRAME_RATE_SUFFIX: &str = " fps";
const DEFAULT_FRAMES_PER_SECOND: f64 = 30.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MediaProbe {
    pub duration: Duration,
    pub width: u32,
    pub height: u32,
    pub frames_per_second: f64,
    pub has_audio: bool,
}

fn parse_clock(clock: &str) -> Option<Duration> {
    let mut parts = clock.trim().split(':');
    let hours: f64 = parts.next()?.parse().ok()?;
    let minutes: f64 = parts.next()?.parse().ok()?;
    let seconds: f64 = parts.next()?.parse().ok()?;
    Some(Duration::from_secs_f64(
        hours * 3_600.0 + minutes * 60.0 + seconds,
    ))
}

fn parse_dimensions(video_line: &str) -> Option<(u32, u32)> {
    video_line
        .split(|character: char| character == ',' || character == ' ')
        .filter_map(|token| {
            let (width, height) = token.split_once('x')?;
            Some((width.parse().ok()?, height.parse().ok()?))
        })
        .find(|(width, height): &(u32, u32)| *width > 0 && *height > 0)
}

fn parse_frame_rate(video_line: &str) -> Option<f64> {
    video_line
        .split(',')
        .find_map(|segment| {
            segment
                .trim()
                .strip_suffix(FRAME_RATE_SUFFIX.trim_start())
                .map(str::trim)
        })
        .and_then(|value| value.trim_end_matches(' ').parse().ok())
}

pub fn parse_media_probe(ffmpeg_output: &str) -> Option<MediaProbe> {
    let duration_text = ffmpeg_output
        .split(DURATION_MARKER)
        .nth(1)?
        .split(',')
        .next()?;
    let video_line = ffmpeg_output
        .lines()
        .find(|line| line.contains(VIDEO_STREAM_MARKER))?;
    let (width, height) = parse_dimensions(video_line)?;
    Some(MediaProbe {
        duration: parse_clock(duration_text)?,
        width,
        height,
        frames_per_second: parse_frame_rate(video_line).unwrap_or(DEFAULT_FRAMES_PER_SECOND),
        has_audio: ffmpeg_output
            .lines()
            .any(|line| line.contains(AUDIO_STREAM_MARKER)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_OUTPUT: &str = "Input #0, mov,mp4,m4a,3gp,3g2,mj2, from 'recording.mp4':\n  Duration: 00:01:14.53, start: 0.000000, bitrate: 1983 kb/s\n  Stream #0:0[0x1](und): Video: h264 (High) (avc1 / 0x31637661), yuv420p(tv, bt709, progressive), 1600x1000, 1800 kb/s, 30 fps, 30 tbr, 15360 tbn (default)\n  Stream #0:1[0x2](und): Audio: aac (LC) (mp4a / 0x6134706D), 48000 Hz, stereo, fltp, 192 kb/s (default)\n";

    #[test]
    fn test_parses_duration_size_rate_and_audio() {
        let probe = parse_media_probe(SAMPLE_OUTPUT).expect("probe");
        assert_eq!(probe.width, 1600);
        assert_eq!(probe.height, 1000);
        assert!((probe.frames_per_second - 30.0).abs() < 0.01);
        assert!(probe.has_audio);
        assert!((probe.duration.as_secs_f64() - 74.53).abs() < 0.001);
    }

    #[test]
    fn test_missing_video_stream_returns_none() {
        assert!(
            parse_media_probe("Duration: 00:00:05.00, start: 0\n  Stream #0:0: Audio: aac")
                .is_none()
        );
    }

    #[test]
    fn test_video_without_audio() {
        let output = "Duration: 00:00:02.00, start: 0\n Stream #0:0: Video: h264, yuv420p, 320x240, 60 fps, 60 tbr\n";
        let probe = parse_media_probe(output).expect("probe");
        assert!(!probe.has_audio);
        assert!((probe.frames_per_second - 60.0).abs() < 0.01);
    }
}
