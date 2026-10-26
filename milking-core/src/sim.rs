//! The fixed-step squeeze-cycle simulation (SPEC §2, §5).
//!
//! Time advances in discrete 60 Hz ticks and *only* in ticks.
//! Input arrives as [`InputEvent`]s stamped with the tick they
//! land on; [`Sim::run`] sorts nothing and trusts nothing — events
//! are applied in stream order at their tick, and a stale event
//! (a tick already passed) is rejected typed. Given the same
//! config and the same event stream, [`Sim::trace_bytes`] is
//! byte-identical across runs: that is the determinism contract
//! the whole teaching package stands on.
//!
//! The squeeze itself is a *parameter*, not a picture. An accepted
//! squeeze drives `squeeze` toward 1.0; it then decays back toward
//! 0. Each tick the parameter sits in the productive zone, flow is
//! produced. The chest jiggle the player sees in the HTML build is
//! a spring driven by this parameter — the core only owns the
//! parameter, which is the correct split for the Torogao port:
//! `torogao-format` owns parameter shapes, `torogao-engine`
//! evaluates them on a fixed step, renderers merely read.

use crate::affordance::{Action, AffordanceError, require_unlocked};
use crate::expression::{Expression, ExpressionMachine, METER_MAX};

/// Simulation constants. These mirror `game.html` exactly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// Fixed-step rate: ticks per second.
    pub tick_hz: u32,
    /// Ideal ticks between squeezes (30 ticks = 500 ms at 60 Hz).
    pub ideal_interval_ticks: u64,
    /// |error| at or below this is Perfect.
    pub perfect_window_ticks: u64,
    /// |error| at or below this is Good; beyond it is a spill.
    pub good_window_ticks: u64,
    /// A squeeze sooner than this after the last is mashing.
    pub mash_floor_ticks: u64,
    /// Pressure (0..=1000) above this converts a squeeze to a spill.
    pub pressure_ceiling: u16,
    /// Bottle capacity in flow units; reaching it wins.
    pub bottle_capacity: f64,
    /// Flow per tick at squeeze = 1.0 before the state gain.
    pub flow_base: f64,
    /// Lowest squeeze value that still produces flow.
    pub productive_zone: f64,
    /// Bottle units removed by one spill.
    pub spill_amount: f64,
    /// Meter points per Perfect / Good squeeze.
    pub meter_perfect: f64,
    /// Meter points per Good squeeze.
    pub meter_good: f64,
    /// Meter points from one caress / one spank.
    pub meter_caress: f64,
    /// Meter points from one spank.
    pub meter_spank: f64,
    /// Two spanks closer than this (ticks) spill.
    pub spank_spam_ticks: u64,
    /// Meter decay per tick while idle.
    pub meter_decay_per_tick: f64,
    /// Squeeze-parameter decay per tick toward 0.
    pub squeeze_decay_per_tick: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tick_hz: 60,
            ideal_interval_ticks: 30,
            perfect_window_ticks: 6,
            good_window_ticks: 12,
            mash_floor_ticks: 15,
            pressure_ceiling: 900,
            bottle_capacity: 1000.0,
            flow_base: 0.55,
            productive_zone: 0.35,
            spill_amount: 15.0,
            meter_perfect: 2.0,
            meter_good: 1.0,
            meter_caress: 6.0,
            meter_spank: 10.0,
            spank_spam_ticks: 48,
            meter_decay_per_tick: 0.4 / 60.0,
            squeeze_decay_per_tick: 0.08,
        }
    }
}

/// A tunable reaction profile (Amendment 1; SPEC §5).
///
/// This is the shape of a Torogao reaction profile: a *gain* on
/// incoming stimulation, a *threshold* at which the governed
/// climax expression triggers, a *reservoir* that accumulates
/// stimulation into stored volume (and visible fullness), and a
/// *refractory* period after climax during which sensitivity is
/// temporarily reduced and re-triggering is impossible.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SensitivityProfile {
    /// Arousal added per effective squeeze, before state gain.
    pub gain: f64,
    /// Arousal level (0..=100 meter scale plus overshoot) at
    /// which climax triggers.
    pub threshold: f64,
    /// Maximum stored reservoir volume.
    pub reservoir_capacity: f64,
    /// Fraction of each effective squeeze's stimulation that is
    /// stored in the reservoir instead of passing straight on.
    pub reservoir_rate: f64,
    /// Ticks after climax during which gain is halved and climax
    /// cannot re-trigger.
    pub refractory_ticks: u64,
}

impl Default for SensitivityProfile {
    fn default() -> Self {
        Self {
            gain: 1.0,
            threshold: 96.0,
            reservoir_capacity: 220.0,
            reservoir_rate: 0.55,
            refractory_ticks: 300,
        }
    }
}

impl SensitivityProfile {
    /// The twitchy regime: high gain, low threshold. A few
    /// effective squeezes send her over, with a small reservoir —
    /// so the climax is quick but the burst is small.
    #[must_use]
    pub fn high() -> Self {
        Self {
            gain: 2.6,
            threshold: 42.0,
            reservoir_rate: 0.3,
            ..Self::default()
        }
    }

    /// The slow-build regime: low gain, high threshold. The
    /// reservoir visibly fills over a long session, and the
    /// eventual climax pays out proportionally.
    #[must_use]
    pub fn low() -> Self {
        Self {
            gain: 0.45,
            threshold: 98.0,
            reservoir_capacity: 420.0,
            reservoir_rate: 0.9,
            ..Self::default()
        }
    }
}

/// Which hand(s) a squeeze input used.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Hand {
    /// Left hand only.
    Left,
    /// Right hand only.
    Right,
    /// Both hands (unlocks at Aroused; tighter window, more flow).
    Both,
}

impl Hand {
    fn action(self) -> Action {
        match self {
            Hand::Left => Action::MilkLeft,
            Hand::Right => Action::MilkRight,
            Hand::Both => Action::MilkBoth,
        }
    }

    fn flow_multiplier(self) -> f64 {
        match self {
            Hand::Both => 1.5,
            _ => 1.0,
        }
    }

    /// Both-hands squeezes grade on a window scaled by this.
    fn window_scale(self) -> f64 {
        match self {
            Hand::Both => 0.75,
            _ => 1.0,
        }
    }
}

/// One timestamped input. `tick` is the fixed-step tick the input
/// lands on; the stream must be non-decreasing in tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputEvent {
    /// A squeeze attempt.
    Squeeze {
        /// Tick the squeeze lands on.
        tick: u64,
        /// Which hand(s).
        hand: Hand,
        /// Charge pressure, 0..=1000.
        pressure: u16,
    },
    /// A side action (caress / spank).
    Side {
        /// Tick the action lands on.
        tick: u64,
        /// Which side action (must be Caress or Spank).
        action: Action,
    },
}

impl InputEvent {
    /// The tick this event lands on.
    #[must_use]
    pub fn tick(&self) -> u64 {
        match self {
            InputEvent::Squeeze { tick, .. } | InputEvent::Side { tick, .. } => *tick,
        }
    }
}

/// How one input was resolved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    /// Perfectly timed squeeze.
    Perfect,
    /// In-window squeeze.
    Good,
    /// Out-of-window / over-pressure / mashed squeeze: milk lost.
    Spill,
    /// Side action applied.
    SideApplied,
    /// A climax fired as a result of this input.
    Climax {
        /// Intensity 0..=1, a function of reservoir at trigger.
        intensity: f64,
        /// Bottle units the burst added.
        burst: f64,
    },
}

/// Why an input was rejected before it could do anything.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputError {
    /// The event's tick is behind the simulation clock.
    StaleTick {
        /// The event's tick.
        event_tick: u64,
        /// The simulation's current tick.
        current_tick: u64,
    },
    /// The action is locked at the current expression state.
    Locked(AffordanceError),
    /// A side-action slot was given a non-side action.
    NotASideAction(Action),
    /// The session is already won; no further input applies.
    SessionOver,
}

/// Observable simulation state. Everything the renderers (HTML,
/// TUI, and eventually Torogao's engine) need, nothing they don't.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Snapshot {
    /// Current tick.
    pub tick: u64,
    /// Squeeze parameter, 0..=1.
    pub squeeze: f64,
    /// Bottle fill, 0..=capacity.
    pub bottle: f64,
    /// Total milk ever produced (the bucket's number).
    pub total_produced: f64,
    /// State/arousal meter, 0..=100.
    pub meter: f64,
    /// Current expression.
    pub expression: Expression,
    /// Stored reservoir volume, 0..=capacity.
    pub reservoir: f64,
    /// Reservoir fullness fraction, 0..=1 — the visible
    /// size/fullness parameter in the renderers.
    pub fullness: f64,
    /// Climaxes so far.
    pub climax_count: u32,
    /// Intensity of the most recent climax, 0..=1.
    pub last_climax_intensity: f64,
    /// Ticks of refractory remaining (gain halved, no re-trigger).
    pub refractory_remaining: u64,
    /// Spills so far.
    pub spills: u32,
    /// Current combo of consecutive in-window squeezes.
    pub combo: u32,
    /// True once the bottle has filled.
    pub won: bool,
}

/// The simulation. Owns no rendering, no clock, no randomness —
/// ticks and events in, snapshots out.
#[derive(Clone, Debug)]
pub struct Sim {
    cfg: Config,
    profile: SensitivityProfile,
    tick: u64,
    squeeze: f64,
    bottle: f64,
    total_produced: f64,
    meter: f64,
    machine: ExpressionMachine,
    reservoir: f64,
    climax_count: u32,
    last_climax_intensity: f64,
    refractory_until: u64,
    last_spank_tick: Option<u64>,
    spills: u32,
    combo: u32,
    last_squeeze_tick: Option<u64>,
    won: bool,
}

impl Sim {
    /// A fresh session.
    #[must_use]
    pub fn new(cfg: Config, profile: SensitivityProfile) -> Self {
        Self {
            cfg,
            profile,
            tick: 0,
            squeeze: 0.0,
            bottle: 0.0,
            total_produced: 0.0,
            meter: 0.0,
            machine: ExpressionMachine::new(),
            reservoir: 0.0,
            climax_count: 0,
            last_climax_intensity: 0.0,
            refractory_until: 0,
            last_spank_tick: None,
            spills: 0,
            combo: 0,
            last_squeeze_tick: None,
            won: false,
        }
    }

    /// The active sensitivity profile.
    #[must_use]
    pub fn profile(&self) -> SensitivityProfile {
        self.profile
    }

    /// Replace the sensitivity profile live (the UI dial does
    /// exactly this). Reservoir and meter are kept; capacity
    /// changes clamp the reservoir down, never up.
    pub fn set_profile(&mut self, profile: SensitivityProfile) {
        self.profile = profile;
        if self.reservoir > profile.reservoir_capacity {
            self.reservoir = profile.reservoir_capacity;
        }
    }

    /// Current observable state.
    #[must_use]
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            tick: self.tick,
            squeeze: self.squeeze,
            bottle: self.bottle,
            total_produced: self.total_produced,
            meter: self.meter,
            expression: self.machine.state(),
            reservoir: self.reservoir,
            fullness: if self.profile.reservoir_capacity > 0.0 {
                self.reservoir / self.profile.reservoir_capacity
            } else {
                0.0
            },
            climax_count: self.climax_count,
            last_climax_intensity: self.last_climax_intensity,
            refractory_remaining: self.refractory_until.saturating_sub(self.tick),
            spills: self.spills,
            combo: self.combo,
            won: self.won,
        }
    }

    /// A canonical byte encoding of the full state, for the
    /// determinism twin-run test and for save/replay checks.
    #[must_use]
    pub fn trace_bytes(&self) -> Vec<u8> {
        let s = self.snapshot();
        let mut out = Vec::with_capacity(96);
        out.extend_from_slice(&s.tick.to_le_bytes());
        out.extend_from_slice(&s.squeeze.to_le_bytes());
        out.extend_from_slice(&s.bottle.to_le_bytes());
        out.extend_from_slice(&s.total_produced.to_le_bytes());
        out.extend_from_slice(&s.meter.to_le_bytes());
        out.extend_from_slice(&(s.expression as u32).to_le_bytes());
        out.extend_from_slice(&s.reservoir.to_le_bytes());
        out.extend_from_slice(&s.climax_count.to_le_bytes());
        out.extend_from_slice(&s.last_climax_intensity.to_le_bytes());
        out.extend_from_slice(&s.spills.to_le_bytes());
        out.extend_from_slice(&s.combo.to_le_bytes());
        out
    }

    fn in_refractory(&self) -> bool {
        self.tick < self.refractory_until
    }

    /// Effective gain right now: the profile gain, halved during
    /// refractory (sensitivity temporarily reduced, SPEC §5).
    fn effective_gain(&self) -> f64 {
        if self.in_refractory() {
            self.profile.gain * 0.5
        } else {
            self.profile.gain
        }
    }

    /// Advance one tick with no input: decay, flow, expression.
    pub fn step(&mut self) {
        if self.won {
            return;
        }
        self.tick += 1;
        // Squeeze parameter decays toward rest.
        self.squeeze = (self.squeeze - self.cfg.squeeze_decay_per_tick).max(0.0);
        // Flow while the parameter is in the productive zone.
        if self.squeeze >= self.cfg.productive_zone {
            let gain = self.machine.state().flow_gain();
            let produced = self.squeeze * gain * self.cfg.flow_base;
            self.bottle = (self.bottle + produced).min(self.cfg.bottle_capacity);
            self.total_produced += produced;
            if self.bottle >= self.cfg.bottle_capacity {
                self.won = true;
            }
        }
        // Meter decays slowly; expression walks (never skips):
        // at most ONE ladder step per tick, so descents after a
        // climax ease down the ladder over several ticks instead
        // of cutting between faces.
        self.meter = (self.meter - self.cfg.meter_decay_per_tick).max(0.0);
        self.machine.step_toward(self.meter);
    }

    /// Advance the clock to `tick`, stepping idle ticks. Events
    /// must be applied at their own tick before advancing past it.
    pub fn advance_to(&mut self, tick: u64) {
        while self.tick < tick && !self.won {
            self.step();
        }
    }

    fn spill(&mut self) {
        self.bottle = (self.bottle - self.cfg.spill_amount).max(0.0);
        self.spills += 1;
        self.combo = 0;
    }

    /// Add arousal through the sensitivity profile: part passes to
    /// the meter, part is stored in the reservoir. Returns
    /// `Some((intensity, burst))` if this addition triggers a
    /// climax — the reservoir then drains into the bottle as the
    /// burst, the meter drops, and refractory begins.
    fn add_stimulation(&mut self, base: f64) -> Option<(f64, f64)> {
        let scaled = base * self.effective_gain();
        let stored = (scaled * self.profile.reservoir_rate)
            .min(self.profile.reservoir_capacity - self.reservoir);
        self.reservoir += stored.max(0.0);
        self.meter = (self.meter + scaled).min(METER_MAX);
        self.machine.step_toward(self.meter);

        if !self.in_refractory() && self.meter >= self.profile.threshold.min(METER_MAX) {
            // Climax: intensity and burst scale with the reservoir
            // at trigger — the slow build pays out bigger.
            let fullness = if self.profile.reservoir_capacity > 0.0 {
                self.reservoir / self.profile.reservoir_capacity
            } else {
                0.0
            };
            let intensity = 0.35 + 0.65 * fullness;
            let burst = self.reservoir * (0.6 + 0.4 * intensity);
            self.bottle = (self.bottle + burst).min(self.cfg.bottle_capacity);
            self.total_produced += burst;
            if self.bottle >= self.cfg.bottle_capacity {
                self.won = true;
            }
            self.reservoir = 0.0;
            self.meter = (self.meter - 45.0).max(0.0);
            // One step now; `step` eases her down the rest of
            // the ladder over the refractory ticks.
            self.machine.step_toward(self.meter);
            self.climax_count += 1;
            self.last_climax_intensity = intensity;
            self.refractory_until = self.tick + self.profile.refractory_ticks;
            return Some((intensity, burst));
        }
        None
    }

    /// Apply one input event at its tick.
    ///
    /// # Errors
    /// Typed rejections: stale tick, locked action, a non-side
    /// action in a side slot, or input after the win.
    pub fn apply(&mut self, event: InputEvent) -> Result<Outcome, InputError> {
        if self.won {
            return Err(InputError::SessionOver);
        }
        let tick = event.tick();
        if tick < self.tick {
            return Err(InputError::StaleTick {
                event_tick: tick,
                current_tick: self.tick,
            });
        }
        self.advance_to(tick);
        if self.won {
            return Err(InputError::SessionOver);
        }
        match event {
            InputEvent::Squeeze { hand, pressure, .. } => self.apply_squeeze(hand, pressure),
            InputEvent::Side { action, .. } => self.apply_side(action),
        }
    }

    fn apply_squeeze(&mut self, hand: Hand, pressure: u16) -> Result<Outcome, InputError> {
        require_unlocked(hand.action(), self.machine.state()).map_err(InputError::Locked)?;
        // Timing grade against the previous accepted squeeze.
        let grade = match self.last_squeeze_tick {
            None => Outcome::Good, // the first squeeze sets the beat
            Some(last) => {
                let interval = self.tick - last;
                if interval < self.cfg.mash_floor_ticks || pressure > self.cfg.pressure_ceiling {
                    self.last_squeeze_tick = Some(self.tick);
                    self.spill();
                    return Ok(Outcome::Spill);
                }
                let err = interval.abs_diff(self.cfg.ideal_interval_ticks) as f64;
                let scale = hand.window_scale();
                if err <= self.cfg.perfect_window_ticks as f64 * scale {
                    Outcome::Perfect
                } else if err <= self.cfg.good_window_ticks as f64 * scale {
                    Outcome::Good
                } else {
                    self.last_squeeze_tick = Some(self.tick);
                    self.spill();
                    return Ok(Outcome::Spill);
                }
            }
        };
        self.last_squeeze_tick = Some(self.tick);
        self.combo += 1;
        // The squeeze parameter jumps with pressure, scaled by the
        // both-hands multiplier; flow itself happens in `step`.
        let depth = f64::from(pressure) / 1000.0;
        self.squeeze = (0.55 + 0.45 * depth) * hand.flow_multiplier().min(1.0);
        let base = match grade {
            Outcome::Perfect => self.cfg.meter_perfect,
            _ => self.cfg.meter_good,
        };
        if let Some((intensity, burst)) = self.add_stimulation(base) {
            return Ok(Outcome::Climax { intensity, burst });
        }
        Ok(grade)
    }

    fn apply_side(&mut self, action: Action) -> Result<Outcome, InputError> {
        match action {
            Action::Caress | Action::Spank => {}
            other => return Err(InputError::NotASideAction(other)),
        }
        require_unlocked(action, self.machine.state()).map_err(InputError::Locked)?;
        if action == Action::Spank {
            if let Some(last) = self.last_spank_tick
                && self.tick - last < self.cfg.spank_spam_ticks
            {
                self.last_spank_tick = Some(self.tick);
                self.spill();
                return Ok(Outcome::Spill);
            }
            self.last_spank_tick = Some(self.tick);
        }
        let base = if action == Action::Caress {
            self.cfg.meter_caress
        } else {
            self.cfg.meter_spank
        };
        // Side actions raise state but are not squeezes: they feed
        // the meter at a fixed fraction of the profile gain and do
        // not store reservoir (SPEC §2.4 — they don't fill).
        self.meter = (self.meter + base * 0.5 * self.effective_gain()).min(METER_MAX);
        self.machine.step_toward(self.meter);
        Ok(Outcome::SideApplied)
    }

    /// Run a whole event stream to `end_tick` and return the final
    /// snapshot. Convenience for tests and replays.
    ///
    /// # Errors
    /// Propagates the first [`InputError`] from the stream.
    pub fn run(&mut self, events: &[InputEvent], end_tick: u64) -> Result<Snapshot, InputError> {
        for event in events {
            self.apply(*event)?;
        }
        self.advance_to(end_tick);
        Ok(self.snapshot())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn squeeze(tick: u64) -> InputEvent {
        InputEvent::Squeeze {
            tick,
            hand: Hand::Left,
            pressure: 700,
        }
    }

    /// A stream of perfectly timed squeezes, one per ideal
    /// interval, alternating hands.
    fn perfect_stream(n: u64) -> Vec<InputEvent> {
        (0..n)
            .map(|i| InputEvent::Squeeze {
                tick: i * 30,
                hand: if i.is_multiple_of(2) {
                    Hand::Left
                } else {
                    Hand::Right
                },
                pressure: 700,
            })
            .collect()
    }

    #[test]
    fn first_squeeze_is_good_and_sets_the_beat() {
        let mut sim = Sim::new(Config::default(), SensitivityProfile::default());
        assert_eq!(sim.apply(squeeze(0)).unwrap(), Outcome::Good);
    }

    #[test]
    fn on_beat_squeezes_flow_and_mashing_spills() {
        let mut sim = Sim::new(Config::default(), SensitivityProfile::default());
        sim.run(&perfect_stream(20), 20 * 30).unwrap();
        let good = sim.snapshot();
        assert!(good.bottle > 30.0, "bottle should fill: {}", good.bottle);
        assert_eq!(good.spills, 0);

        let mut masher = Sim::new(Config::default(), SensitivityProfile::default());
        let mash: Vec<InputEvent> = (0..20).map(|i| squeeze(i * 5)).collect();
        masher.run(&mash, 200).unwrap();
        let bad = masher.snapshot();
        assert!(bad.spills > 0);
        assert!(bad.bottle < good.bottle);
    }

    #[test]
    fn stale_events_are_rejected() {
        let mut sim = Sim::new(Config::default(), SensitivityProfile::default());
        sim.apply(squeeze(60)).unwrap();
        assert_eq!(
            sim.apply(squeeze(30)),
            Err(InputError::StaleTick {
                event_tick: 30,
                current_tick: 60,
            })
        );
    }

    #[test]
    fn reservoir_never_exceeds_capacity() {
        let profile = SensitivityProfile {
            gain: 50.0,
            threshold: 99.0,
            reservoir_capacity: 10.0,
            reservoir_rate: 1.0,
            refractory_ticks: 10_000,
        };
        let mut sim = Sim::new(Config::default(), profile);
        sim.run(&perfect_stream(40), 40 * 30).unwrap();
        assert!(sim.snapshot().reservoir <= 10.0 + f64::EPSILON);
    }

    #[test]
    fn twin_runs_are_byte_identical() {
        let events = perfect_stream(30);
        let mut a = Sim::new(Config::default(), SensitivityProfile::default());
        let mut b = Sim::new(Config::default(), SensitivityProfile::default());
        a.run(&events, 1000).unwrap();
        b.run(&events, 1000).unwrap();
        assert_eq!(a.trace_bytes(), b.trace_bytes());
    }
}
