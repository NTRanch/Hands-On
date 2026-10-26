//! The Studio Mode authoring model (SPEC §3): keyform tracks,
//! easing, loop closure, and baking.
//!
//! A *track* is one animated parameter — in the game, the squeeze
//! parameter. The author places *keyforms* (a time inside the
//! cycle, a value), picks how segments ease between them, and the
//! engine's job is to evaluate the track at any phase. A track
//! *loops* only if it closes: the value where the cycle ends must
//! equal the value where it begins, or every loop iteration pops.
//! [`check_closure`] is that rule as code, and [`bake`] freezes a
//! track into a fixed-step frame table — the exact artifact a
//! scene track in Torogao carries after authoring.

/// How a segment interpolates between two keyforms.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Easing {
    /// Constant speed.
    #[default]
    Linear,
    /// Smoothstep: slow at both ends (`3t² − 2t³`).
    Smooth,
    /// Cubic ease-in-out: slower ends, faster middle.
    EaseInOut,
    /// Hold the starting value until the next keyform.
    Hold,
}

impl Easing {
    /// Map a raw segment fraction `t ∈ [0,1]` through the easing.
    #[must_use]
    pub fn apply(self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::Smooth => t * t * (3.0 - 2.0 * t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Easing::Hold => 0.0,
        }
    }
}

/// One keyform: a value at a time inside the cycle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keyform {
    /// Milliseconds from cycle start. Must lie in `0..=cycle_ms`.
    pub at_ms: u32,
    /// Parameter value, 0..=1.
    pub value: f64,
}

/// Why a track is not well-formed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrackError {
    /// A cycle must have positive length.
    EmptyCycle,
    /// Fewer than two keyforms cannot define a loop.
    TooFewKeyforms,
    /// Keyforms must be strictly increasing in time, inside the
    /// cycle, with values in 0..=1.
    MalformedKeyform {
        /// Index of the offending keyform.
        index: usize,
    },
    /// The cycle does not close: end value ≠ start value.
    LoopOpen {
        /// Value at cycle start.
        start: f64,
        /// Value at cycle end.
        end: f64,
    },
}

/// An authored keyform track for one parameter.
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    cycle_ms: u32,
    keyforms: Vec<Keyform>,
    easing: Easing,
}

impl Track {
    /// Build a track, validating shape.
    ///
    /// # Errors
    /// [`TrackError`] on a zero cycle, fewer than two keyforms,
    /// or any malformed keyform (unsorted, out of cycle, value
    /// outside 0..=1).
    pub fn new(cycle_ms: u32, keyforms: Vec<Keyform>, easing: Easing) -> Result<Self, TrackError> {
        if cycle_ms == 0 {
            return Err(TrackError::EmptyCycle);
        }
        if keyforms.len() < 2 {
            return Err(TrackError::TooFewKeyforms);
        }
        for (i, k) in keyforms.iter().enumerate() {
            let bad_time = k.at_ms > cycle_ms || (i > 0 && k.at_ms <= keyforms[i - 1].at_ms);
            let bad_value = !(0.0..=1.0).contains(&k.value);
            if bad_time || bad_value {
                return Err(TrackError::MalformedKeyform { index: i });
            }
        }
        Ok(Self {
            cycle_ms,
            keyforms,
            easing,
        })
    }

    /// Cycle length in milliseconds.
    #[must_use]
    pub fn cycle_ms(&self) -> u32 {
        self.cycle_ms
    }

    /// The keyforms, in time order.
    #[must_use]
    pub fn keyforms(&self) -> &[Keyform] {
        &self.keyforms
    }

    /// The segment easing.
    #[must_use]
    pub fn easing(&self) -> Easing {
        self.easing
    }

    /// Evaluate the parameter at `phase_ms` within the cycle.
    /// Phases past the cycle wrap. Before the first keyform the
    /// value holds at that keyform; after the last, the segment
    /// runs from the last keyform to the *first keyform's value*
    /// at cycle end — which is exactly why closure matters.
    #[must_use]
    pub fn evaluate(&self, phase_ms: u32) -> f64 {
        let phase = phase_ms % self.cycle_ms.max(1);
        let first = self.keyforms[0];
        if phase <= first.at_ms {
            return first.value;
        }
        for pair in self.keyforms.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if phase <= b.at_ms {
                let span = (b.at_ms - a.at_ms) as f64;
                let t = (phase - a.at_ms) as f64 / span;
                return a.value + (b.value - a.value) * self.easing.apply(t);
            }
        }
        // Tail segment: last keyform → first keyform's value at
        // cycle end (the wrap segment).
        let last = *self.keyforms.last().expect("validated: ≥2 keyforms");
        let span = (self.cycle_ms - last.at_ms) as f64;
        if span <= 0.0 {
            return last.value;
        }
        let t = (phase - last.at_ms) as f64 / span;
        last.value + (first.value - last.value) * self.easing.apply(t)
    }

    /// The value the cycle ends on (the limit at `cycle_ms`,
    /// evaluated on the wrap segment).
    #[must_use]
    pub fn end_value(&self) -> f64 {
        let last = *self.keyforms.last().expect("validated: ≥2 keyforms");
        let first = self.keyforms[0];
        if last.at_ms == self.cycle_ms {
            last.value
        } else {
            first.value
        }
    }

    /// Loop-closure check: a seamless loop needs the end value to
    /// equal the start value. The comparison is exact on purpose
    /// for authored endpoints — the author closes a loop by
    /// placing the final keyform, not by hoping floats agree.
    ///
    /// # Errors
    /// [`TrackError::LoopOpen`] carrying both endpoint values
    /// when they differ — the Studio UI draws the gap from these.
    pub fn check_closure(&self) -> Result<(), TrackError> {
        let start = self.keyforms[0].value;
        let end = self.end_value();
        if (start - end).abs() <= f64::EPSILON {
            Ok(())
        } else {
            Err(TrackError::LoopOpen { start, end })
        }
    }

    /// Tempo score 0..=100 against a target cycle length: exact
    /// match is 100, drifting to 0 at a 2× / ½× miss.
    #[must_use]
    pub fn tempo_score(&self, target_cycle_ms: u32) -> f64 {
        if target_cycle_ms == 0 {
            return 0.0;
        }
        let ratio = f64::from(self.cycle_ms) / f64::from(target_cycle_ms);
        let log_err = ratio.ln().abs() / std::f64::consts::LN_2; // 1.0 at 2× or ½×
        (100.0 * (1.0 - log_err)).clamp(0.0, 100.0)
    }

    /// Bake the track to a fixed-step frame table: one value per
    /// `step_ms`, starting at phase 0, for exactly one cycle. This
    /// is the deterministic artifact an engine plays — after the
    /// bake, evaluation is a table lookup, identical everywhere.
    ///
    /// # Errors
    /// [`TrackError::EmptyCycle`] if `step_ms` is 0 or does not
    /// divide the cycle (a partial last frame would make the
    /// table's loop point ambiguous — refuse, don't round).
    pub fn bake(&self, step_ms: u32) -> Result<Vec<f64>, TrackError> {
        if step_ms == 0 || !self.cycle_ms.is_multiple_of(step_ms) {
            return Err(TrackError::EmptyCycle);
        }
        let frames = (self.cycle_ms / step_ms) as usize;
        Ok((0..frames)
            .map(|i| self.evaluate(i as u32 * step_ms))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn closed_track() -> Track {
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
    fn closed_loop_passes_closure() {
        assert_eq!(closed_track().check_closure(), Ok(()));
    }

    #[test]
    fn open_loop_fails_with_both_endpoints() {
        let track = Track::new(
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
                    value: 0.4,
                },
            ],
            Easing::Linear,
        )
        .unwrap();
        assert_eq!(
            track.check_closure(),
            Err(TrackError::LoopOpen {
                start: 0.0,
                end: 0.4
            })
        );
    }

    #[test]
    fn wrap_segment_returns_to_the_first_value() {
        // No keyform at cycle end: the wrap segment must land on
        // the first keyform's value, so this loop closes.
        let track = Track::new(
            1000,
            vec![
                Keyform {
                    at_ms: 0,
                    value: 0.2,
                },
                Keyform {
                    at_ms: 500,
                    value: 0.9,
                },
            ],
            Easing::Linear,
        )
        .unwrap();
        assert_eq!(track.check_closure(), Ok(()));
        assert!((track.evaluate(999) - 0.2).abs() < 0.01);
    }

    #[test]
    fn malformed_tracks_are_rejected() {
        assert_eq!(
            Track::new(0, vec![], Easing::Linear),
            Err(TrackError::EmptyCycle)
        );
        let unsorted = Track::new(
            1000,
            vec![
                Keyform {
                    at_ms: 500,
                    value: 0.0,
                },
                Keyform {
                    at_ms: 100,
                    value: 1.0,
                },
            ],
            Easing::Linear,
        );
        assert_eq!(unsorted, Err(TrackError::MalformedKeyform { index: 1 }));
        let out_of_range = Track::new(
            1000,
            vec![
                Keyform {
                    at_ms: 0,
                    value: 0.0,
                },
                Keyform {
                    at_ms: 100,
                    value: 1.7,
                },
            ],
            Easing::Linear,
        );
        assert_eq!(out_of_range, Err(TrackError::MalformedKeyform { index: 1 }));
    }

    #[test]
    fn bake_is_a_full_cycle_at_the_step() {
        let table = closed_track().bake(100).unwrap();
        assert_eq!(table.len(), 10);
        assert_eq!(table[0], 0.0);
        assert!((table[4] - 1.0).abs() < f64::EPSILON);
        // A step that does not divide the cycle is refused.
        assert_eq!(closed_track().bake(333), Err(TrackError::EmptyCycle));
        assert_eq!(closed_track().bake(0), Err(TrackError::EmptyCycle));
    }

    #[test]
    fn tempo_score_peaks_at_the_target() {
        let track = closed_track();
        assert_eq!(track.tempo_score(1000), 100.0);
        assert!(track.tempo_score(2000) < 1.0);
    }

    #[test]
    fn easing_endpoints_are_exact() {
        for easing in [Easing::Linear, Easing::Smooth, Easing::EaseInOut] {
            assert_eq!(easing.apply(0.0), 0.0);
            assert_eq!(easing.apply(1.0), 1.0);
        }
        assert_eq!(Easing::Hold.apply(0.99), 0.0);
    }
}
