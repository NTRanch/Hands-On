//! The funscript writer (Playroom P8's twin): Lumen's
//! funscript writer plus the `torogao-haptics` player's clamp.
//!
//! A funscript is a JSON timeline of `(at_ms, pos)` actions,
//! `pos` in 0..=100 — a motion curve renamed into a haptic
//! track. The writer samples an authored [`Track`] at a fixed
//! step; the player lawfully emits only what the router
//! contract allows, so the level's second half applies the
//! router ceiling and makes the player *hear* (read) the
//! difference between the authored curve and the lawful
//! emission: actions above the ceiling move, and the writer
//! reports every move — clamping is arithmetic, never a secret.

use crate::authoring::{Track, TrackError};
use crate::router::RouterConfig;

/// One funscript action: a position at a time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Action {
    /// Milliseconds from track start.
    pub at_ms: u32,
    /// Position, 0..=100 (the funscript range).
    pub pos: u8,
}

/// A funscript: actions in time order plus the metadata the
/// format carries.
#[derive(Clone, Debug, PartialEq)]
pub struct Funscript {
    /// The actions, ascending in `at_ms`.
    pub actions: Vec<Action>,
    /// Track duration in milliseconds.
    pub duration_ms: u32,
    /// The named seed/range note the writer stamps.
    pub range: u8,
}

impl Funscript {
    /// Sample a track at `step_ms` into a funscript: one
    /// action per step, `pos` = value × 100 rounded. The final
    /// action lands exactly on the cycle end.
    ///
    /// # Errors
    /// Propagates [`Track::bake`]'s refusal of a step that
    /// does not divide the cycle.
    pub fn from_track(track: &Track, step_ms: u32) -> Result<Self, TrackError> {
        let table = track.bake(step_ms)?;
        let mut actions: Vec<Action> = table
            .iter()
            .enumerate()
            .map(|(i, v)| Action {
                at_ms: i as u32 * step_ms,
                pos: (v.clamp(0.0, 1.0) * 100.0).round() as u8,
            })
            .collect();
        // The endpoint: at cycle end the value is the track's
        // end value — include it so players that interpolate to
        // the duration see the authored endpoint.
        actions.push(Action {
            at_ms: track.cycle_ms(),
            pos: (track.end_value().clamp(0.0, 1.0) * 100.0).round() as u8,
        });
        Ok(Self {
            actions,
            duration_ms: track.cycle_ms(),
            range: 100,
        })
    }

    /// Apply the router contract's ceiling: every position is
    /// scaled into the lawful range (ceiling × 100). Returns
    /// the clamped script and the number of actions that moved
    /// — the difference between the authored curve and the
    /// lawful emission, counted honestly.
    #[must_use]
    pub fn apply_router_clamp(&self, cfg: RouterConfig) -> (Self, usize) {
        let cap = (cfg.ceiling.clamp(0.0, 1.0) * 100.0).round() as u8;
        let mut moved = 0usize;
        let actions: Vec<Action> = self
            .actions
            .iter()
            .map(|a| {
                if a.pos > cap {
                    moved += 1;
                    Action { pos: cap, ..*a }
                } else {
                    *a
                }
            })
            .collect();
        (
            Self {
                actions,
                duration_ms: self.duration_ms,
                range: cap,
            },
            moved,
        )
    }

    /// Serialize to the funscript JSON shape
    /// (`{"version":"1.0","inverted":false,"range":N,"actions":[{"at":ms,"pos":p},...]}`).
    /// Hand-rolled and exact: the bytes are a teaching artifact
    /// and twin-run comparisons hash them.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut out = String::with_capacity(64 + self.actions.len() * 24);
        out.push_str("{\"version\":\"1.0\",\"inverted\":false,\"range\":");
        out.push_str(&self.range.to_string());
        out.push_str(",\"actions\":[");
        for (i, a) in self.actions.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str("{\"at\":");
            out.push_str(&a.at_ms.to_string());
            out.push_str(",\"pos\":");
            out.push_str(&a.pos.to_string());
            out.push('}');
        }
        out.push_str("]}");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authoring::{Easing, Keyform};

    fn track() -> Track {
        Track::new(
            1000,
            vec![
                Keyform {
                    at_ms: 0,
                    value: 0.0,
                },
                Keyform {
                    at_ms: 400,
                    value: 1.0,
                },
                Keyform {
                    at_ms: 1000,
                    value: 0.0,
                },
            ],
            Easing::Smooth,
        )
        .unwrap()
    }

    #[test]
    fn sampling_hits_the_authored_landmarks() {
        let fs = Funscript::from_track(&track(), 100).unwrap();
        assert_eq!(fs.actions.len(), 11); // 10 steps + endpoint
        assert_eq!(fs.actions[0], Action { at_ms: 0, pos: 0 });
        assert_eq!(
            fs.actions[4],
            Action {
                at_ms: 400,
                pos: 100
            }
        );
        assert_eq!(
            fs.actions[10],
            Action {
                at_ms: 1000,
                pos: 0
            }
        );
        assert_eq!(fs.duration_ms, 1000);
    }

    #[test]
    fn json_is_the_funscript_shape() {
        let fs = Funscript::from_track(&track(), 500).unwrap();
        let json = fs.to_json();
        assert!(json.starts_with("{\"version\":\"1.0\",\"inverted\":false,\"range\":100,"));
        assert!(json.contains("{\"at\":500,\"pos\":93}"), "{json}");
        assert!(json.ends_with("]}"));
    }

    #[test]
    fn twin_writes_are_byte_identical() {
        let a = Funscript::from_track(&track(), 50).unwrap().to_json();
        let b = Funscript::from_track(&track(), 50).unwrap().to_json();
        assert_eq!(a, b);
    }

    // ---- adversarial ----

    #[test]
    fn the_router_clamp_moves_peaks_and_counts_them() {
        let fs = Funscript::from_track(&track(), 100).unwrap();
        let (clamped, moved) = fs.apply_router_clamp(RouterConfig::default());
        assert!(moved > 0, "the authored peak exceeds the 0.20 ceiling");
        assert!(clamped.actions.iter().all(|a| a.pos <= 20));
        assert_eq!(clamped.range, 20);
        // The clamp is a cap, not a rescale: quiet actions stay.
        assert_eq!(clamped.actions[0].pos, 0);
        assert_eq!(clamped.actions[10].pos, 0);
    }

    #[test]
    fn a_step_that_does_not_divide_is_refused() {
        assert_eq!(
            Funscript::from_track(&track(), 333),
            Err(TrackError::EmptyCycle)
        );
    }

    #[test]
    fn clamping_twice_moves_nothing_the_second_time() {
        let fs = Funscript::from_track(&track(), 100).unwrap();
        let (once, _) = fs.apply_router_clamp(RouterConfig::default());
        let (_, moved_twice) = once.apply_router_clamp(RouterConfig::default());
        assert_eq!(moved_twice, 0);
    }
}
