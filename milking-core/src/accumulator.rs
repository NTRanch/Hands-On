//! The Accumulator (Study S3's twin): `torogao-engine`'s
//! fixed-step evaluation, as arithmetic.
//!
//! The contract is integer-only: real time arrives as
//! microseconds, the engine consumes whole steps of a fixed
//! duration, and the remainder carries. No floats anywhere —
//! a float accumulator *drifts*, and the lesson's second half
//! proves it by running a naive float version beside this one
//! and counting where they disagree.
//!
//! Frame counts here are produced with a **gen block**
//! (Rust 2024): the step stream is a generator expression of
//! the contract, not a hand-rolled loop with a counter to get
//! wrong.

/// A fixed-step accumulator over integer microseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Accumulator {
    step_us: u64,
    carry_us: u64,
    total_steps: u64,
}

impl Accumulator {
    /// An accumulator with the given fixed step.
    #[must_use]
    pub fn new(step_us: u64) -> Self {
        Self {
            step_us: step_us.max(1),
            carry_us: 0,
            total_steps: 0,
        }
    }

    /// The engine's house step: 60 Hz ≈ 16 667 µs (the
    /// integer the contract names — 1 000 000 / 60 truncated,
    /// and the truncation is *in* the contract, so every
    /// implementation agrees).
    #[must_use]
    pub fn at_60hz() -> Self {
        Self::new(16_667)
    }

    /// Feed `elapsed_us` of real time; returns how many whole
    /// steps the engine owes. The remainder carries — nothing
    /// is rounded away.
    pub fn feed(&mut self, elapsed_us: u64) -> u64 {
        let total = self.carry_us + elapsed_us;
        let steps = total / self.step_us;
        self.carry_us = total % self.step_us;
        self.total_steps += steps;
        steps
    }

    /// Microseconds currently carried (always < step).
    #[must_use]
    pub fn carry_us(&self) -> u64 {
        self.carry_us
    }

    /// Total steps emitted since construction.
    #[must_use]
    pub fn total_steps(&self) -> u64 {
        self.total_steps
    }

    /// The step stream as a gen block: feeding a sequence of
    /// elapsed times yields each burst of owed steps in order.
    /// (Rust 2024 gen blocks — the frame-table generation the
    /// curriculum names.)
    pub fn step_stream(step_us: u64, elapsed: Vec<u64>) -> impl Iterator<Item = u64> {
        gen move {
            let mut acc = Accumulator::new(step_us);
            for e in elapsed {
                yield acc.feed(e);
            }
        }
    }
}

/// The naive float accumulator the lesson argues against:
/// seconds as `f64`, steps by division. It exists to be caught
/// drifting — the test suite runs it beside the integer
/// contract and exhibits the disagreement.
#[derive(Clone, Copy, Debug)]
pub struct NaiveFloatAccumulator {
    step_s: f64,
    carry_s: f64,
    /// Total steps emitted.
    pub total_steps: u64,
}

impl NaiveFloatAccumulator {
    /// A naive accumulator with the given step in seconds.
    #[must_use]
    pub fn new(step_s: f64) -> Self {
        Self {
            step_s,
            carry_s: 0.0,
            total_steps: 0,
        }
    }

    /// Feed elapsed seconds; returns whole steps owed.
    pub fn feed(&mut self, elapsed_s: f64) -> u64 {
        let total = self.carry_s + elapsed_s;
        let steps = (total / self.step_s).floor();
        self.carry_s = total - steps * self.step_s;
        self.total_steps += steps as u64;
        steps as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_steps_and_carry() {
        let mut acc = Accumulator::new(100);
        assert_eq!(acc.feed(250), 2);
        assert_eq!(acc.carry_us(), 50);
        assert_eq!(acc.feed(50), 1);
        assert_eq!(acc.carry_us(), 0);
        assert_eq!(acc.total_steps(), 3);
    }

    #[test]
    fn a_second_at_60hz_is_fiftynine_steps_plus_carry() {
        // The contract's step is 1_000_000 / 60 truncated to
        // 16_667 µs, so 60 steps cover 1_000_020 µs: one exact
        // second is 59 steps and 16_647 µs of carry. Every
        // implementation of the contract agrees on this —
        // that agreement, not round numbers, is the point.
        let mut acc = Accumulator::at_60hz();
        let mut steps = 0;
        for slice in [16_000u64, 17_000, 16_700, 16_633, 16_667, 17_000, 900_000] {
            steps += acc.feed(slice);
        }
        assert_eq!(steps, 59);
        assert_eq!(acc.carry_us(), 16_647);
        assert_eq!(acc.feed(20), 1, "20 more µs completes the 60th step");
        assert_eq!(acc.total_steps(), 60);
    }

    #[test]
    fn the_gen_block_stream_agrees_with_the_loop() {
        let elapsed = vec![100u64, 250, 75, 1000];
        let streamed: Vec<u64> = Accumulator::step_stream(100, elapsed.clone()).collect();
        let mut acc = Accumulator::new(100);
        let looped: Vec<u64> = elapsed.iter().map(|e| acc.feed(*e)).collect();
        assert_eq!(streamed, looped);
        assert_eq!(streamed, vec![1, 2, 1, 10]);
    }

    // ---- adversarial ----

    #[test]
    fn zero_step_is_clamped_to_one() {
        let mut acc = Accumulator::new(0);
        assert_eq!(acc.feed(5), 5);
    }

    #[test]
    fn the_float_version_drifts_and_the_integer_one_does_not() {
        // 0.3 s frames at a 0.1 s step: 0.3 / 0.1 in binary
        // floating point is 2.9999999999999996, so the naive
        // accumulator floors to 2 steps and banks a phantom
        // carry; over 1000 frames it emits 2999 steps where
        // the integer contract emits exactly 3000. One step,
        // lost to representation — that loss is the lesson.
        let mut int_acc = Accumulator::new(100_000);
        let mut float_acc = NaiveFloatAccumulator::new(0.1);
        let mut int_total = 0u64;
        let mut float_total = 0u64;
        for _ in 0..1000 {
            int_total += int_acc.feed(300_000);
            float_total += float_acc.feed(0.3);
        }
        assert_eq!(int_total, 3000);
        assert_eq!(int_acc.carry_us(), 0);
        assert_eq!(float_total, 2999, "the float version lost a step to drift");
    }

    #[test]
    fn huge_feeds_stay_exact() {
        let mut acc = Accumulator::new(16_667);
        let steps = acc.feed(u64::MAX / 2);
        assert_eq!(steps, (u64::MAX / 2) / 16_667);
        assert_eq!(acc.carry_us(), (u64::MAX / 2) % 16_667);
    }
}
