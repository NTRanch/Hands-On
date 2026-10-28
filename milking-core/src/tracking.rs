//! Track the Performer (Study S7's twin): the live-mode
//! tracking state machine — states, transitions, calibration
//! states — written so the compiler rejects an unhandled
//! transition.
//!
//! The machine has five states and a total transition
//! function: every `(state, event)` pair is answered, and the
//! answer for an impossible event is a typed refusal naming
//! the pair — never a silent ignore, never a panic. The
//! transition table below is written as one exhaustive match;
//! adding a state is a compile error until every event is
//! answered for it. Exhaustiveness *is* the lesson — and in
//! the level it reads as the host's composure: a machine that
//! can answer everything cannot be flustered.
//!
//! Calibration uses let chains (Rust 2024): a calibration
//! completes only when the sample count *and* the stability
//! bound both hold in one condition.

/// The tracking states.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackState {
    /// No performer acquired.
    Idle,
    /// Acquiring: samples are being collected.
    Calibrating,
    /// Locked on and tracking.
    Tracking,
    /// The performer was lost; reacquisition is underway.
    Lost,
    /// A fault latched: only a reset answers from here.
    Faulted,
}

/// The events the tracking surface can receive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackEvent {
    /// A performer appeared; begin acquiring.
    Acquire,
    /// One calibration sample arrived (with its jitter).
    Sample {
        /// This sample's jitter, abstract units.
        jitter: u32,
    },
    /// A tracking frame arrived on the beat.
    Frame,
    /// The performer left the frame.
    PerformerLost,
    /// The performer is visible again.
    PerformerSeen,
    /// Something underneath faulted.
    Fault,
    /// A human reset the machine.
    Reset,
}

/// Calibration terms, negotiated in advance: the bounded
/// work of the sitting.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Calibration {
    /// Samples required before lock is possible.
    pub samples_needed: u32,
    /// Maximum jitter a sample may carry to count as stable.
    pub max_jitter: u32,
}

impl Default for Calibration {
    fn default() -> Self {
        Self {
            samples_needed: 5,
            max_jitter: 10,
        }
    }
}

/// Why a transition was refused.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrackError {
    /// This event means nothing in this state, and the
    /// machine says so rather than ignoring it.
    Unhandled {
        /// The state that refused.
        state: TrackState,
        /// The event refused.
        event: TrackEvent,
    },
}

/// The tracking machine.
#[derive(Clone, Debug)]
pub struct Tracker {
    state: TrackState,
    terms: Calibration,
    stable_samples: u32,
    frames: u64,
}

impl Tracker {
    /// A fresh machine in [`TrackState::Idle`].
    #[must_use]
    pub fn new(terms: Calibration) -> Self {
        Self {
            state: TrackState::Idle,
            terms,
            stable_samples: 0,
            frames: 0,
        }
    }

    /// Current state.
    #[must_use]
    pub fn state(&self) -> TrackState {
        self.state
    }

    /// Stable calibration samples collected so far.
    #[must_use]
    pub fn stable_samples(&self) -> u32 {
        self.stable_samples
    }

    /// Frames tracked since lock.
    #[must_use]
    pub fn frames(&self) -> u64 {
        self.frames
    }

    /// Answer one event. Total: every pair is handled or
    /// refused typed.
    ///
    /// # Errors
    /// [`TrackError::Unhandled`] for an event that means
    /// nothing in the current state.
    pub fn apply(&mut self, event: TrackEvent) -> Result<TrackState, TrackError> {
        use TrackEvent as E;
        use TrackState as S;
        let next = match (self.state, event) {
            (S::Idle, E::Acquire) => {
                self.stable_samples = 0;
                S::Calibrating
            }
            (S::Calibrating, E::Sample { jitter }) => {
                // Let chain (Rust 2024): lock only when the
                // count AND the stability bound both hold.
                if jitter <= self.terms.max_jitter
                    && let Some(n) = self.stable_samples.checked_add(1)
                {
                    self.stable_samples = n;
                }
                if self.stable_samples >= self.terms.samples_needed {
                    S::Tracking
                } else {
                    S::Calibrating
                }
            }
            (S::Tracking, E::Frame) => {
                self.frames += 1;
                S::Tracking
            }
            (S::Tracking, E::PerformerLost) => S::Lost,
            (S::Lost, E::PerformerSeen) => {
                // Reacquisition re-calibrates from zero: lost
                // means the lock's evidence is stale.
                self.stable_samples = 0;
                S::Calibrating
            }
            (_, E::Fault) => S::Faulted,
            (S::Faulted, E::Reset) => {
                self.stable_samples = 0;
                self.frames = 0;
                S::Idle
            }
            (state, event) => {
                return Err(TrackError::Unhandled { state, event });
            }
        };
        self.state = next;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tracker() -> Tracker {
        Tracker::new(Calibration::default())
    }

    fn lock(t: &mut Tracker) {
        t.apply(TrackEvent::Acquire).unwrap();
        for _ in 0..5 {
            t.apply(TrackEvent::Sample { jitter: 3 }).unwrap();
        }
        assert_eq!(t.state(), TrackState::Tracking);
    }

    #[test]
    fn the_happy_path_locks_and_tracks() {
        let mut t = tracker();
        lock(&mut t);
        t.apply(TrackEvent::Frame).unwrap();
        t.apply(TrackEvent::Frame).unwrap();
        assert_eq!(t.frames(), 2);
    }

    #[test]
    fn jittery_samples_do_not_count() {
        let mut t = tracker();
        t.apply(TrackEvent::Acquire).unwrap();
        for _ in 0..4 {
            t.apply(TrackEvent::Sample { jitter: 3 }).unwrap();
        }
        // A wild sample: answered, but it does not count.
        let s = t.apply(TrackEvent::Sample { jitter: 99 }).unwrap();
        assert_eq!(s, TrackState::Calibrating);
        assert_eq!(t.stable_samples(), 4);
        t.apply(TrackEvent::Sample { jitter: 1 }).unwrap();
        assert_eq!(t.state(), TrackState::Tracking);
    }

    #[test]
    fn loss_forces_recalibration() {
        let mut t = tracker();
        lock(&mut t);
        t.apply(TrackEvent::PerformerLost).unwrap();
        assert_eq!(t.state(), TrackState::Lost);
        t.apply(TrackEvent::PerformerSeen).unwrap();
        assert_eq!(t.state(), TrackState::Calibrating);
        assert_eq!(t.stable_samples(), 0);
    }

    // ---- adversarial ----

    #[test]
    fn meaningless_events_are_refused_typed() {
        let mut t = tracker();
        assert_eq!(
            t.apply(TrackEvent::Frame),
            Err(TrackError::Unhandled {
                state: TrackState::Idle,
                event: TrackEvent::Frame
            })
        );
        assert_eq!(
            t.apply(TrackEvent::Reset),
            Err(TrackError::Unhandled {
                state: TrackState::Idle,
                event: TrackEvent::Reset
            })
        );
        assert_eq!(t.state(), TrackState::Idle, "a refusal changes nothing");
    }

    #[test]
    fn fault_latches_until_a_human_reset() {
        let mut t = tracker();
        lock(&mut t);
        t.apply(TrackEvent::Fault).unwrap();
        assert_eq!(t.state(), TrackState::Faulted);
        assert!(t.apply(TrackEvent::Frame).is_err());
        assert!(t.apply(TrackEvent::Acquire).is_err());
        t.apply(TrackEvent::Reset).unwrap();
        assert_eq!(t.state(), TrackState::Idle);
        assert_eq!(t.frames(), 0);
    }

    #[test]
    fn reset_outside_fault_is_unhandled() {
        // Reset is a fault-recovery verb, not a general
        // escape hatch: mid-tracking it refuses.
        let mut t = tracker();
        lock(&mut t);
        assert!(t.apply(TrackEvent::Reset).is_err());
        assert_eq!(t.state(), TrackState::Tracking);
    }
}
