use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingClock {
    Running {
        resumed_at: Instant,
        accumulated: Duration,
    },
    Paused {
        accumulated: Duration,
    },
}

impl RecordingClock {
    pub fn pause(self, now: Instant) -> Self {
        match self {
            RecordingClock::Running {
                resumed_at,
                accumulated,
            } => RecordingClock::Paused {
                accumulated: accumulated + now.saturating_duration_since(resumed_at),
            },
            paused => paused,
        }
    }

    pub fn resume(self, now: Instant) -> Self {
        match self {
            RecordingClock::Paused { accumulated } => RecordingClock::Running {
                resumed_at: now,
                accumulated,
            },
            running => running,
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self, RecordingClock::Running { .. })
    }

    pub fn active_elapsed(&self, now: Instant) -> Duration {
        match self {
            RecordingClock::Running {
                resumed_at,
                accumulated,
            } => *accumulated + now.saturating_duration_since(*resumed_at),
            RecordingClock::Paused { accumulated } => *accumulated,
        }
    }
}

pub fn frames_due(active_elapsed: Duration, frames_per_second: u32) -> u64 {
    let elapsed_frames = active_elapsed.as_secs_f64() * frames_per_second as f64;
    elapsed_frames.floor() as u64 + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paused_time_is_excluded() {
        let origin = Instant::now();
        let at = |milliseconds: u64| origin + Duration::from_millis(milliseconds);
        let clock = RecordingClock::Paused {
            accumulated: Duration::ZERO,
        }
        .resume(at(0))
        .pause(at(1_000))
        .resume(at(3_000));
        assert_eq!(
            clock.active_elapsed(at(4_500)),
            Duration::from_millis(2_500)
        );
        assert!(clock.is_running());
    }

    #[test]
    fn test_pause_and_resume_are_idempotent() {
        let origin = Instant::now();
        let at = |milliseconds: u64| origin + Duration::from_millis(milliseconds);
        let paused = RecordingClock::Paused {
            accumulated: Duration::ZERO,
        }
        .resume(at(0))
        .pause(at(500))
        .pause(at(900));
        assert_eq!(paused.active_elapsed(at(5_000)), Duration::from_millis(500));
        let resumed = paused.resume(at(1_000)).resume(at(2_000));
        assert_eq!(
            resumed.active_elapsed(at(1_500)),
            Duration::from_millis(1_000)
        );
    }

    #[test]
    fn test_frames_due_counts_the_first_frame_immediately() {
        assert_eq!(frames_due(Duration::ZERO, 30), 1);
        assert_eq!(frames_due(Duration::from_millis(1_000), 30), 31);
        assert_eq!(frames_due(Duration::from_millis(1_016), 60), 61);
    }
}
