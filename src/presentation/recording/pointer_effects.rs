use std::time::{Duration, Instant};

use iced::Point;

pub const RIPPLE_LIFETIME: Duration = Duration::from_millis(550);
const RIPPLE_START_RADIUS: f32 = 10.0;
const RIPPLE_END_RADIUS: f32 = 42.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClickRipple {
    pub center: Point,
    pub started_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RippleAppearance {
    pub radius: f32,
    pub opacity: f32,
}

impl ClickRipple {
    pub fn appearance(&self, now: Instant) -> Option<RippleAppearance> {
        let age = now.saturating_duration_since(self.started_at);
        if age >= RIPPLE_LIFETIME {
            return None;
        }
        let progress = age.as_secs_f32() / RIPPLE_LIFETIME.as_secs_f32();
        let eased_progress = 1.0 - (1.0 - progress).powi(3);
        Some(RippleAppearance {
            radius: RIPPLE_START_RADIUS
                + (RIPPLE_END_RADIUS - RIPPLE_START_RADIUS) * eased_progress,
            opacity: 1.0 - progress,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PointerEffects {
    pub cursor: Option<Point>,
    pub is_pressed: bool,
    pub ripples: Vec<ClickRipple>,
    pub now: Option<Instant>,
}

impl PointerEffects {
    pub fn apply_sample(
        mut self,
        local_position: Option<Point>,
        is_pressed: bool,
        now: Instant,
    ) -> Self {
        let press_started = is_pressed && !self.is_pressed;
        if let (true, Some(position)) = (press_started, local_position) {
            self.ripples.push(ClickRipple {
                center: position,
                started_at: now,
            });
        }
        self.ripples
            .retain(|ripple| now.saturating_duration_since(ripple.started_at) < RIPPLE_LIFETIME);
        self.cursor = local_position;
        self.is_pressed = is_pressed;
        self.now = Some(now);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_press_edge_creates_one_ripple() {
        let now = Instant::now();
        let effects = PointerEffects::default()
            .apply_sample(Some(Point::new(5.0, 5.0)), false, now)
            .apply_sample(
                Some(Point::new(5.0, 5.0)),
                true,
                now + Duration::from_millis(16),
            )
            .apply_sample(
                Some(Point::new(6.0, 5.0)),
                true,
                now + Duration::from_millis(32),
            );
        assert_eq!(effects.ripples.len(), 1);
    }

    #[test]
    fn test_ripples_expire() {
        let now = Instant::now();
        let effects = PointerEffects::default()
            .apply_sample(Some(Point::ORIGIN), true, now)
            .apply_sample(
                Some(Point::ORIGIN),
                false,
                now + RIPPLE_LIFETIME + Duration::from_millis(1),
            );
        assert!(effects.ripples.is_empty());
    }

    #[test]
    fn test_ripple_grows_and_fades() {
        let now = Instant::now();
        let ripple = ClickRipple {
            center: Point::ORIGIN,
            started_at: now,
        };
        let early = ripple
            .appearance(now + Duration::from_millis(50))
            .expect("visible");
        let late = ripple
            .appearance(now + Duration::from_millis(400))
            .expect("visible");
        assert!(late.radius > early.radius);
        assert!(late.opacity < early.opacity);
        assert!(ripple.appearance(now + RIPPLE_LIFETIME).is_none());
    }

    #[test]
    fn test_press_outside_display_creates_no_ripple() {
        let effects = PointerEffects::default().apply_sample(None, true, Instant::now());
        assert!(effects.ripples.is_empty());
    }
}
