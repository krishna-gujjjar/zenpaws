use std::time::Instant;

use crate::PetState;

/// Local-only cursor coordinates for pet behavior decisions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Runtime behavior quality setting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PerformanceMode {
    Full,
    Reduced,
}

/// Local behavior output. Coordinates are never synchronized.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BehaviorDecision {
    pub state: PetState,
    pub target: Point,
}

/// Chooses local pet movement state from cursor distance and velocity.
#[derive(Debug)]
pub struct CursorBehavior {
    last_cursor: Option<(Point, Instant)>,
}

impl CursorBehavior {
    /// Creates a cursor behavior controller without any sampled cursor position.
    #[must_use]
    pub const fn new() -> Self {
        Self { last_cursor: None }
    }

    /// Produces the next local-only movement decision.
    #[must_use]
    pub fn update(
        &mut self,
        pet: Point,
        cursor: Point,
        now: Instant,
        mode: PerformanceMode,
    ) -> BehaviorDecision {
        let cursor_distance = distance(pet, cursor);
        let velocity = self.last_cursor.map_or(0.0, |(previous, sampled_at)| {
            let seconds = now.duration_since(sampled_at).as_secs_f64();
            if seconds == 0.0 {
                0.0
            } else {
                distance(previous, cursor) / seconds
            }
        });
        self.last_cursor = Some((cursor, now));

        let state = if cursor_distance < 8.0 {
            PetState::Idle
        } else if mode == PerformanceMode::Full && velocity > 1_200.0 {
            PetState::Run
        } else {
            PetState::Walk
        };
        BehaviorDecision {
            state,
            target: cursor,
        }
    }
}

impl Default for CursorBehavior {
    fn default() -> Self {
        Self::new()
    }
}

fn distance(left: Point, right: Point) -> f64 {
    (left.x - right.x).hypot(left.y - right.y)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{CursorBehavior, PerformanceMode, Point};
    use crate::PetState;

    #[test]
    fn chooses_local_movement_without_syncing_coordinates() {
        let mut behavior = CursorBehavior::new();
        let now = Instant::now();
        let pet = Point { x: 0.0, y: 0.0 };

        assert_eq!(
            behavior.update(pet, pet, now, PerformanceMode::Full).state,
            PetState::Idle
        );
        assert_eq!(
            behavior
                .update(
                    pet,
                    Point { x: 20.0, y: 0.0 },
                    now + Duration::from_millis(100),
                    PerformanceMode::Full
                )
                .state,
            PetState::Walk
        );
        assert_eq!(
            behavior
                .update(
                    pet,
                    Point { x: 500.0, y: 0.0 },
                    now + Duration::from_millis(200),
                    PerformanceMode::Full
                )
                .state,
            PetState::Run
        );
        assert_eq!(
            behavior
                .update(
                    pet,
                    Point { x: 1_000.0, y: 0.0 },
                    now + Duration::from_millis(300),
                    PerformanceMode::Reduced
                )
                .state,
            PetState::Walk
        );
    }
}
