//! The expression state machine (SPEC §2.3).
//!
//! Five states in a strict line. The machine advances or regresses
//! exactly one step per transition call, and the simulation calls
//! it in a loop — so crossing two thresholds in one tick still
//! walks through the intermediate face. That is the lesson: an
//! expression *state machine* is a graph, and this graph is a line
//! with no shortcuts, in either direction.
//!
//! Hysteresis: a state is entered at its threshold but only left
//! downward when the meter falls `HYSTERESIS` points below that
//! threshold, so a meter hovering at a boundary cannot flicker the
//! face on and off every tick.

/// State meter range is `0.0..=100.0`.
pub const METER_MAX: f64 = 100.0;
/// Points below an entry threshold the meter must fall before the
/// machine steps back down.
pub const HYSTERESIS: f64 = 10.0;

/// One expression. The discriminants are the progression order.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum Expression {
    /// Entry state.
    #[default]
    Neutral = 0,
    /// Meter ≥ 20.
    Smiling = 1,
    /// Meter ≥ 40.
    Flushed = 2,
    /// Meter ≥ 60.
    Aroused = 3,
    /// Meter ≥ 80. Heart overlays live here.
    Ecstatic = 4,
}

impl Expression {
    /// All states in progression order.
    pub const ALL: [Expression; 5] = [
        Expression::Neutral,
        Expression::Smiling,
        Expression::Flushed,
        Expression::Aroused,
        Expression::Ecstatic,
    ];

    /// The meter value at which this state is entered from below.
    /// `Neutral` is the floor and has no entry threshold.
    #[must_use]
    pub fn entry_threshold(self) -> f64 {
        match self {
            Expression::Neutral => 0.0,
            Expression::Smiling => 20.0,
            Expression::Flushed => 40.0,
            Expression::Aroused => 60.0,
            Expression::Ecstatic => 80.0,
        }
    }

    /// Flow multiplier this state grants the squeeze cycle
    /// (SPEC §2.2 `STATE_GAIN`).
    #[must_use]
    pub fn flow_gain(self) -> f64 {
        match self {
            Expression::Neutral => 1.0,
            Expression::Smiling => 1.15,
            Expression::Flushed => 1.3,
            Expression::Aroused => 1.5,
            Expression::Ecstatic => 1.75,
        }
    }

    /// The next state up, if any.
    #[must_use]
    pub fn next(self) -> Option<Expression> {
        Self::ALL.get((self as usize) + 1).copied()
    }

    /// The next state down, if any.
    #[must_use]
    pub fn prev(self) -> Option<Expression> {
        (self as usize).checked_sub(1).map(|i| Self::ALL[i])
    }
}

/// The machine: a current state plus the pure transition rule.
#[derive(Clone, Copy, Debug, Default)]
pub struct ExpressionMachine {
    state: Expression,
}

impl ExpressionMachine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn state(&self) -> Expression {
        self.state
    }

    /// Reconcile the state with `meter`, moving **at most one
    /// step**. Returns the (possibly unchanged) state. The caller
    /// loops this until it stops changing if a jump of several
    /// thresholds must be walked through in one tick — each
    /// intermediate state is still visited, never skipped.
    pub fn step_toward(&mut self, meter: f64) -> Expression {
        if let Some(next) = self.state.next()
            && meter >= next.entry_threshold()
        {
            self.state = next;
            return self.state;
        }
        if let Some(prev) = self.state.prev()
            && meter < self.state.entry_threshold() - HYSTERESIS
        {
            self.state = prev;
        }
        self.state
    }

    /// Walk [`Self::step_toward`] until the state settles. Every
    /// intermediate state is entered in order.
    pub fn settle(&mut self, meter: f64) -> Expression {
        loop {
            let before = self.state;
            if self.step_toward(meter) == before {
                return self.state;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_step_at_a_time_even_on_a_full_meter() {
        let mut m = ExpressionMachine::new();
        // A single step from a full meter reaches only Smiling.
        assert_eq!(m.step_toward(100.0), Expression::Smiling);
        // Settling walks the whole line in order.
        assert_eq!(m.settle(100.0), Expression::Ecstatic);
    }

    #[test]
    fn hysteresis_holds_state_near_the_boundary() {
        let mut m = ExpressionMachine::new();
        m.settle(45.0);
        assert_eq!(m.state(), Expression::Flushed);
        // 39 is below the 40 entry but inside the hysteresis band.
        assert_eq!(m.settle(39.0), Expression::Flushed);
        // 29 is below 40 - 10, so the machine steps down — once.
        assert_eq!(m.step_toward(29.0), Expression::Smiling);
    }

    #[test]
    fn floor_and_ceiling_are_sticky() {
        let mut m = ExpressionMachine::new();
        assert_eq!(m.settle(0.0), Expression::Neutral);
        m.settle(100.0);
        assert_eq!(m.settle(100.0), Expression::Ecstatic);
    }
}
