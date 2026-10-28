use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackClock {
    Playing {
        started_at: Instant,
        start_position: Duration,
    },
    Paused {
        position: Duration,
    },
}

impl PlaybackClock {
    pub fn paused_at(position: Duration) -> Self {
        PlaybackClock::Paused { position }
    }

    pub fn position(&self, now: Instant, duration: Duration) -> Duration {
        let raw_position = match self {
            PlaybackClock::Playing {
                started_at,
                start_position,
            } => *start_position + now.saturating_duration_since(*started_at),
            PlaybackClock::Paused { position } => *position,
        };
        raw_position.min(duration)
    }

    pub fn play(self, now: Instant, duration: Duration) -> Self {
        match self {
            PlaybackClock::Paused { position } => PlaybackClock::Playing {
                started_at: now,
                start_position: position.min(duration),
            },
            playing => playing,
        }
    }

    pub fn pause(self, now: Instant, duration: Duration) -> Self {
        PlaybackClock::Paused {
            position: self.position(now, duration),
        }
    }

    pub fn is_playing(&self) -> bool {
        matches!(self, PlaybackClock::Playing { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playing_clock_advances_and_caps_at_duration() {
        let origin = Instant::now();
        let duration = Duration::from_secs(10);
        let clock = PlaybackClock::paused_at(Duration::from_secs(2)).play(origin, duration);
        assert_eq!(
            clock.position(origin + Duration::from_secs(3), duration),
            Duration::from_secs(5)
        );
        assert_eq!(
            clock.position(origin + Duration::from_secs(30), duration),
            duration
        );
    }

    #[test]
    fn test_pause_freezes_position() {
        let origin = Instant::now();
        let duration = Duration::from_secs(10);
        let clock = PlaybackClock::paused_at(Duration::ZERO)
            .play(origin, duration)
            .pause(origin + Duration::from_secs(4), duration);
        assert_eq!(
            clock.position(origin + Duration::from_secs(9), duration),
            Duration::from_secs(4)
        );
        assert!(!clock.is_playing());
    }
}
