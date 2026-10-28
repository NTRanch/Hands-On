//! Fixed-step rhythm evaluation (Playroom P2's twin): Strobe's
//! beat-sync, reduced to its contract.
//!
//! Judgments are made against a **deterministic tick grid**,
//! never against wall time and never against rendered frames.
//! A beat grid is an interval in ticks; a hit is a tick; the
//! judgment is a pure function of the two. The exercise's
//! "frame-perfect vs tick-perfect" round exists to make one
//! consequence felt: a renderer may *present* a hit on a frame
//! boundary that straddles two ticks, but the core counts the
//! tick the hit lands on — so a hit that looks perfect can be
//! tick-late, and no frame rate can change the verdict.
//!
//! Hits also carry a haptics **envelope** shape (attack /
//! sustain / decay), the same envelope the `torogao-haptics`
//! player emits: stimulation is the envelope evaluated on the
//! tick, not a square wave.

/// One beat grid: beats every `interval_ticks`, starting at
/// tick 0. Windows mirror [`crate::sim::Config`]'s grading:
/// inside the perfect window is Perfect, inside the good
/// window is Good, anything else is a miss.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeatGrid {
    /// Ticks between consecutive beats.
    pub interval_ticks: u64,
    /// |offset| at or below this is a Perfect hit.
    pub perfect_window_ticks: u64,
    /// |offset| at or below this is a Good hit.
    pub good_window_ticks: u64,
}

impl BeatGrid {
    /// The grid at `bpm` beats per minute on a `tick_hz` clock.
    /// The interval is rounded to the nearest tick — the grid is
    /// ticks all the way down, so a tempo that does not divide
    /// the clock still lands on whole ticks, deterministically.
    #[must_use]
    pub fn at_bpm(bpm: u32, tick_hz: u32) -> Self {
        let interval = (u64::from(tick_hz) * 60 + u64::from(bpm) / 2) / u64::from(bpm.max(1));
        Self {
            interval_ticks: interval.max(1),
            perfect_window_ticks: 6,
            good_window_ticks: 12,
        }
    }

    /// Signed offset of `tick` from the *nearest* beat, in
    /// ticks: negative is early, positive is late. On an exact
    /// half-interval tie the earlier beat wins (offset is the
    /// positive half), so the function is total and stable.
    #[must_use]
    pub fn offset_of(&self, tick: u64) -> i64 {
        let interval = self.interval_ticks.max(1);
        let rem = tick % interval;
        if rem * 2 <= interval {
            rem as i64
        } else {
            rem as i64 - interval as i64
        }
    }

    /// Judge one hit tick against the grid.
    #[must_use]
    pub fn judge(&self, tick: u64) -> Judgment {
        let offset = self.offset_of(tick).unsigned_abs();
        if offset <= self.perfect_window_ticks {
            Judgment::Perfect
        } else if offset <= self.good_window_ticks {
            Judgment::Good
        } else {
            Judgment::Miss
        }
    }

    /// The tick of the `n`th beat (0-based).
    #[must_use]
    pub fn beat_tick(&self, n: u64) -> u64 {
        n * self.interval_ticks.max(1)
    }
}

/// How one hit landed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Judgment {
    /// Inside the perfect window.
    Perfect,
    /// Inside the good window.
    Good,
    /// Outside both windows.
    Miss,
}

impl Judgment {
    /// Stimulation strength this judgment carries as an
    /// envelope peak: a Perfect hit is a full envelope, a Good
    /// hit a partial one, a Miss none.
    #[must_use]
    pub fn envelope_peak(self) -> f64 {
        match self {
            Judgment::Perfect => 1.0,
            Judgment::Good => 0.6,
            Judgment::Miss => 0.0,
        }
    }
}

/// A haptics envelope: attack up to the peak, sustain, then
/// decay — all in ticks, all evaluated on the tick. This is
/// the shape a hit's stimulation takes on the host.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Envelope {
    /// Ticks to climb from 0 to the peak.
    pub attack_ticks: u64,
    /// Ticks held at the peak.
    pub sustain_ticks: u64,
    /// Ticks to fall from the peak back to 0.
    pub decay_ticks: u64,
}

impl Envelope {
    /// Total envelope length in ticks.
    #[must_use]
    pub fn total_ticks(&self) -> u64 {
        self.attack_ticks + self.sustain_ticks + self.decay_ticks
    }

    /// Envelope value `t` ticks after the hit, for a hit with
    /// the given peak. Outside the envelope the value is 0.
    #[must_use]
    pub fn at(&self, t: u64, peak: f64) -> f64 {
        if t < self.attack_ticks {
            if self.attack_ticks == 0 {
                peak
            } else {
                peak * t as f64 / self.attack_ticks as f64
            }
        } else if t < self.attack_ticks + self.sustain_ticks {
            peak
        } else if t < self.total_ticks() {
            let into = t - self.attack_ticks - self.sustain_ticks;
            peak * (1.0 - into as f64 / self.decay_ticks.max(1) as f64)
        } else {
            0.0
        }
    }
}

/// The frame-rate independence proof, as data: a hit presented
/// by a renderer running at `frame_hz` is *displayed* on the
/// frame containing its tick, but judged on the tick itself.
/// Returns the frame index the hit displays on — the exercise
/// compares this across frame rates while the judgment stays
/// fixed, which is the whole lesson.
#[must_use]
pub fn display_frame(tick: u64, tick_hz: u32, frame_hz: u32) -> u64 {
    // frame = floor(tick / tick_hz * frame_hz), integer-exact.
    tick * u64::from(frame_hz.max(1)) / u64::from(tick_hz.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> BeatGrid {
        BeatGrid::at_bpm(120, 60) // one beat every 30 ticks
    }

    #[test]
    fn bpm_maps_to_whole_ticks() {
        assert_eq!(grid().interval_ticks, 30);
        assert_eq!(BeatGrid::at_bpm(60, 60).interval_ticks, 60);
        assert_eq!(BeatGrid::at_bpm(240, 60).interval_ticks, 15);
    }

    #[test]
    fn on_beat_hits_are_perfect() {
        let g = grid();
        for n in 0..8u64 {
            assert_eq!(g.judge(g.beat_tick(n)), Judgment::Perfect);
        }
    }

    #[test]
    fn windows_grade_by_tick_offset() {
        let g = grid();
        assert_eq!(g.judge(30 + 6), Judgment::Perfect);
        assert_eq!(g.judge(30 - 6), Judgment::Perfect);
        assert_eq!(g.judge(30 + 12), Judgment::Good);
        assert_eq!(g.judge(30 - 12), Judgment::Good);
        assert_eq!(g.judge(30 + 13), Judgment::Miss);
        assert_eq!(g.judge(30 - 13), Judgment::Miss);
    }

    #[test]
    fn offset_is_signed_and_ties_go_early() {
        let g = grid();
        assert_eq!(g.offset_of(33), 3);
        assert_eq!(g.offset_of(27), -3);
        // Halfway between beats 0 and 30: the tie rule assigns
        // the earlier beat, offset +15.
        assert_eq!(g.offset_of(15), 15);
        assert_eq!(g.offset_of(16), -14);
    }

    // ---- adversarial: the frame rate must not matter ----

    #[test]
    fn judgment_is_identical_at_any_frame_rate() {
        let g = grid();
        // A hit 8 ticks late: Good on the tick grid. Renderers
        // at 30/60/144 fps display it on different frames, but
        // the judgment is a function of the tick only.
        let tick = 38;
        assert_eq!(g.judge(tick), Judgment::Good);
        let frames: Vec<u64> = [30u32, 60, 144]
            .iter()
            .map(|&f| display_frame(tick, 60, f))
            .collect();
        assert_eq!(frames, vec![19, 38, 91]);
        assert_eq!(
            g.judge(tick),
            Judgment::Good,
            "frames moved, verdict did not"
        );
    }

    #[test]
    fn frame_perfect_can_be_tick_late() {
        // At 20 fps a frame spans 3 ticks. A hit presented dead
        // on frame 10's boundary at tick 31 *looks* perfect and
        // is 1 tick late — still Perfect here, but at tick 37
        // (same visual neighborhood at coarse frame rates) it
        // is only Good. The grid, not the frame, is the judge.
        let g = grid();
        assert_eq!(display_frame(31, 60, 20), 10);
        assert_eq!(g.judge(31), Judgment::Perfect);
        assert_eq!(g.judge(37), Judgment::Good);
    }

    #[test]
    fn envelope_attacks_sustains_decays() {
        let env = Envelope {
            attack_ticks: 4,
            sustain_ticks: 2,
            decay_ticks: 4,
        };
        assert_eq!(env.at(0, 1.0), 0.0);
        assert!((env.at(2, 1.0) - 0.5).abs() < f64::EPSILON);
        assert_eq!(env.at(4, 1.0), 1.0);
        assert_eq!(env.at(5, 1.0), 1.0);
        assert!((env.at(8, 1.0) - 0.5).abs() < f64::EPSILON);
        assert_eq!(env.at(10, 1.0), 0.0);
        assert_eq!(env.at(999, 1.0), 0.0);
        // Peak scales linearly; a Miss peaks at 0.
        assert_eq!(env.at(4, Judgment::Miss.envelope_peak()), 0.0);
    }

    #[test]
    fn zero_attack_envelope_starts_at_peak() {
        let env = Envelope {
            attack_ticks: 0,
            sustain_ticks: 3,
            decay_ticks: 1,
        };
        assert_eq!(env.at(0, 0.6), 0.6);
        assert_eq!(env.total_ticks(), 4);
    }
}
