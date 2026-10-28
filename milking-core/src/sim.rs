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

use std::fmt;

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
    /// Ticks without a landed stimulation before arousal decay
    /// applies. Arousal decays *at rest* (SPEC §2.3: the meter
    /// decays when no successful action has landed for a while),
    /// never mid-rhythm: at the ideal interval a squeeze lands
    /// every 30 ticks, well inside this grace, so continued
    /// stimulation always builds — decay can never cancel a
    /// build in progress and park her under the threshold.
    pub idle_grace_ticks: u64,
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
            idle_grace_ticks: 90,
        }
    }
}

/// A tunable reaction profile (Amendment 1; SPEC §5).
///
/// This is the shape of a Torogao reaction profile: a *gain* on
/// incoming stimulation, a *threshold* at which the governed
/// climax expression triggers, a *reservoir* that stores the
/// milk she produces (its level is the visible fullness), and a
/// *refractory* period after climax during which sensitivity is
/// temporarily reduced and re-triggering is impossible.
///
/// The milk economy is conservative — milk is only ever created
/// by production (per tick, while milking contact holds, at a
/// rate set by the arousal tier) and only ever leaves by being
/// collected into the bottle or spilled from it:
///
/// ```text
/// total_produced == bottle + reservoir + spilled_away
/// ```
///
/// at every tick. Production splits on the tick it happens: the
/// expressed share is collected into the bottle, and the
/// `reservoir_rate` share is retained in the reservoir. Climax
/// pays a fraction of the reservoir into the bottle as the
/// burst; the un-paid remainder *stays stored* — it is the head
/// start on the refill, not a rounding error.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SensitivityProfile {
    /// Arousal added per effective squeeze, before state gain.
    /// Must be > 0 for the arousal arc to run; 0 is a lawful
    /// degenerate (she never rises, never climaxes — production
    /// is unaffected, it is not gain-driven).
    pub gain: f64,
    /// Arousal level (0..=100 meter scale) at which climax
    /// triggers. The effective threshold is
    /// `min(threshold, METER_MAX)`: values above 100 behave as
    /// 100. A threshold of 0 makes the first stimulation that
    /// raises the meter off zero culminate immediately.
    pub threshold: f64,
    /// Maximum stored reservoir volume (milk units).
    pub reservoir_capacity: f64,
    /// Share of each tick's produced milk that is retained in
    /// the reservoir instead of being expressed straight into
    /// the bottle — her retention share, 0..=1. This is the
    /// reservoir's refill rate: the reservoir refills after a
    /// climax at exactly this share of ongoing production.
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

/// Version of the opaque sim-state payload produced by
/// [`Sim::state_bytes`] and consumed by
/// [`Sim::from_state_bytes`]. The payload is the save system's
/// property: shells carry it without interpreting it, and any
/// other version is a typed refusal, never a guess. Bump on
/// any layout change.
pub const STATE_VERSION: u32 = 1;

/// Byte length of a version-1 state payload (fixed layout).
const STATE_BYTES_LEN: usize = 159;

/// Why a sim-state payload was refused. A corrupt or foreign
/// payload is always this typed refusal, never a panic and
/// never a partially-restored sim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StateError {
    /// Not a whole state record: too short, truncated, or
    /// carrying trailing bytes.
    Malformed,
    /// A field carried a value the sim cannot hold (a
    /// non-finite float, an unknown expression discriminant,
    /// a bad option tag). The field is named.
    InvalidField(&'static str),
    /// The payload was written under another state version.
    UnsupportedVersion(u32),
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::Malformed => write!(f, "sim state payload is malformed"),
            StateError::InvalidField(name) => {
                write!(f, "sim state payload has an invalid {name}")
            }
            StateError::UnsupportedVersion(v) => {
                write!(
                    f,
                    "sim state version {v} is not supported (this build reads {STATE_VERSION})"
                )
            }
        }
    }
}

impl std::error::Error for StateError {}

/// Cursor over a state payload whose total length has already
/// been verified against [`STATE_BYTES_LEN`], so every read is
/// in bounds by construction.
struct StateReader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl StateReader<'_> {
    fn byte(&mut self) -> u8 {
        let v = self.bytes[self.pos];
        self.pos += 1;
        v
    }

    fn u32(&mut self) -> u32 {
        let v = u32::from_le_bytes(
            self.bytes[self.pos..self.pos + 4]
                .try_into()
                .expect("length-checked payload"),
        );
        self.pos += 4;
        v
    }

    fn u64(&mut self) -> u64 {
        let v = u64::from_le_bytes(
            self.bytes[self.pos..self.pos + 8]
                .try_into()
                .expect("length-checked payload"),
        );
        self.pos += 8;
        v
    }

    fn f64(&mut self) -> f64 {
        f64::from_bits(self.u64())
    }

    fn optional_tick(&mut self, name: &'static str) -> Result<Option<u64>, StateError> {
        match self.byte() {
            0 => {
                let _ = self.u64();
                Ok(None)
            }
            1 => Ok(Some(self.u64())),
            _ => Err(StateError::InvalidField(name)),
        }
    }
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
    /// Ticks since the last landed stimulation (an accepted
    /// squeeze or a side action). Arousal decay applies only
    /// once this passes `Config::idle_grace_ticks` — decay is a
    /// rest phenomenon, never a tax on rhythm.
    idle_ticks: u64,
    /// Flow multiplier of the most recent accepted squeeze's
    /// hand(s) (SPEC §2.4: both hands express +50%). The squeeze
    /// *parameter* is a 0..=1 envelope and cannot carry the
    /// multiplier, so the yield computation reads it from here;
    /// it lasts exactly as long as the envelope it rode in on.
    flow_boost: f64,
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
            idle_ticks: 0,
            flow_boost: 1.0,
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

    /// The full sim state as an opaque, version-tagged payload
    /// for the save system. Unlike [`trace_bytes`](Self::trace_bytes)
    /// this is lossless: it carries the cadence history
    /// (last-spank / last-squeeze ticks), the absolute
    /// refractory deadline, the expression machine's exact
    /// standing, and the win flag, so a restored sim continues
    /// bit-for-bit like the original. Shells must treat the
    /// bytes as opaque; only this module reads them back.
    #[must_use]
    pub fn state_bytes(&self) -> Vec<u8> {
        let p = &self.profile;
        let mut out = Vec::with_capacity(STATE_BYTES_LEN);
        out.extend_from_slice(&STATE_VERSION.to_le_bytes());
        for v in [p.gain, p.threshold, p.reservoir_capacity, p.reservoir_rate] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out.extend_from_slice(&p.refractory_ticks.to_le_bytes());
        out.extend_from_slice(&self.tick.to_le_bytes());
        for v in [
            self.squeeze,
            self.bottle,
            self.total_produced,
            self.meter,
            self.reservoir,
            self.last_climax_intensity,
        ] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out.extend_from_slice(&(self.machine.state() as u32).to_le_bytes());
        out.extend_from_slice(&self.climax_count.to_le_bytes());
        out.extend_from_slice(&self.refractory_until.to_le_bytes());
        out.extend_from_slice(&self.idle_ticks.to_le_bytes());
        out.extend_from_slice(&self.flow_boost.to_le_bytes());
        for opt in [self.last_spank_tick, self.last_squeeze_tick] {
            // if/else rather than a match on the option: the
            // payload bytes are identical either way, and the
            // bare `None` pattern trips a rust-analyzer
            // non_snake_case false positive (the study.rs
            // precedent in the wave gates).
            if let Some(t) = opt {
                out.push(1);
                out.extend_from_slice(&t.to_le_bytes());
            } else {
                out.push(0);
                out.extend_from_slice(&0u64.to_le_bytes());
            }
        }
        out.extend_from_slice(&self.spills.to_le_bytes());
        out.extend_from_slice(&self.combo.to_le_bytes());
        out.push(u8::from(self.won));
        debug_assert_eq!(out.len(), STATE_BYTES_LEN);
        out
    }

    /// Rebuild a sim from a payload written by
    /// [`state_bytes`](Self::state_bytes), under the caller's
    /// `cfg` (configuration is a build contract, not save
    /// data). Any deviation — wrong version, wrong length, a
    /// field the sim cannot hold — is a [`StateError`], never
    /// a panic and never a partial restore.
    pub fn from_state_bytes(cfg: Config, bytes: &[u8]) -> Result<Sim, StateError> {
        if bytes.len() < 4 {
            return Err(StateError::Malformed);
        }
        let version = u32::from_le_bytes(bytes[0..4].try_into().expect("4-byte prefix"));
        if version != STATE_VERSION {
            return Err(StateError::UnsupportedVersion(version));
        }
        if bytes.len() != STATE_BYTES_LEN {
            return Err(StateError::Malformed);
        }
        let mut r = StateReader { bytes, pos: 4 };
        let finite = |v: f64, name: &'static str| -> Result<f64, StateError> {
            if v.is_finite() {
                Ok(v)
            } else {
                Err(StateError::InvalidField(name))
            }
        };
        let profile = SensitivityProfile {
            gain: finite(r.f64(), "gain")?,
            threshold: finite(r.f64(), "threshold")?,
            reservoir_capacity: finite(r.f64(), "reservoir_capacity")?,
            reservoir_rate: finite(r.f64(), "reservoir_rate")?,
            refractory_ticks: r.u64(),
        };
        let tick = r.u64();
        let squeeze = finite(r.f64(), "squeeze")?;
        let bottle = finite(r.f64(), "bottle")?;
        let total_produced = finite(r.f64(), "total_produced")?;
        let meter = finite(r.f64(), "meter")?;
        let reservoir = finite(r.f64(), "reservoir")?;
        let last_climax_intensity = finite(r.f64(), "last_climax_intensity")?;
        let expression = match r.u32() {
            0 => Expression::Neutral,
            1 => Expression::Smiling,
            2 => Expression::Flushed,
            3 => Expression::Aroused,
            4 => Expression::Ecstatic,
            _ => return Err(StateError::InvalidField("expression")),
        };
        let climax_count = r.u32();
        let refractory_until = r.u64();
        let idle_ticks = r.u64();
        let flow_boost = finite(r.f64(), "flow_boost")?;
        let last_spank_tick = r.optional_tick("last_spank_tick")?;
        let last_squeeze_tick = r.optional_tick("last_squeeze_tick")?;
        let spills = r.u32();
        let combo = r.u32();
        let won = match r.byte() {
            0 => false,
            1 => true,
            _ => return Err(StateError::InvalidField("won")),
        };
        Ok(Sim {
            cfg,
            profile,
            tick,
            squeeze,
            bottle,
            total_produced,
            meter,
            machine: ExpressionMachine::from_state(expression),
            reservoir,
            climax_count,
            last_climax_intensity,
            refractory_until,
            last_spank_tick,
            last_squeeze_tick,
            spills,
            combo,
            won,
            idle_ticks,
            flow_boost,
        })
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

    /// Advance one tick with no input: production, rest decay,
    /// expression, and culmination.
    ///
    /// Production (lactation): while the squeeze parameter is in
    /// the productive zone — milking contact — she produces at
    /// `squeeze × tier_gain × flow_base × hand_boost`, where the
    /// tier gain is her current expression's (arousal drives
    /// production) and the boost is the hand(s) that set the
    /// envelope. Each tick's yield splits: the expressed share
    /// is collected into the bottle, the profile's
    /// `reservoir_rate` share is retained in the reservoir (up
    /// to its capacity; at capacity, production throttles to
    /// the expressed stream — nothing is produced into a void).
    /// No contact, no production: an idle tick makes no milk,
    /// however high the meter sits.
    pub fn step(&mut self) {
        if self.won {
            return;
        }
        self.tick += 1;
        self.idle_ticks = self.idle_ticks.saturating_add(1);
        // Squeeze parameter decays toward rest.
        self.squeeze = (self.squeeze - self.cfg.squeeze_decay_per_tick).max(0.0);
        // Production while the parameter is in the productive zone.
        if self.squeeze >= self.cfg.productive_zone {
            let tier_gain = self.machine.state().flow_gain();
            let expressed = self.squeeze * tier_gain * self.cfg.flow_base * self.flow_boost;
            let retained = (expressed * self.profile.reservoir_rate)
                .min(self.profile.reservoir_capacity - self.reservoir)
                .max(0.0);
            self.reservoir += retained;
            self.bottle = (self.bottle + expressed).min(self.cfg.bottle_capacity);
            self.total_produced += expressed + retained;
            if self.bottle >= self.cfg.bottle_capacity {
                self.won = true;
            }
        }
        // Meter decays slowly, but only at rest: decay is gated
        // on the idle clock, so a build in progress is never
        // taxed between squeezes. Expression walks (never
        // skips): at most ONE ladder step per tick, so descents
        // after a climax ease down the ladder over several ticks
        // instead of cutting between faces.
        if self.idle_ticks > self.cfg.idle_grace_ticks {
            self.meter = (self.meter - self.cfg.meter_decay_per_tick).max(0.0);
        }
        self.machine.step_toward(self.meter);
        // Culmination is a property of the arousal state, not of
        // whichever input happened to cross the line: a meter
        // parked at or over the threshold (side actions built it,
        // a live dial change lowered the line under it) resolves
        // on the clock. The meter can only be up there because
        // stimulation put it there — it never rises on its own —
        // so this can never fire from rest.
        let _ = self.check_culmination();
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

    /// Add arousal through the sensitivity profile. Arousal only:
    /// milk storage is production's business (see [`Sim::step`]),
    /// never a second accounting of the same stimulation.
    /// Returns `Some((intensity, burst))` if this addition
    /// culminates in a climax.
    fn add_stimulation(&mut self, base: f64) -> Option<(f64, f64)> {
        self.idle_ticks = 0;
        let scaled = base * self.effective_gain();
        self.meter = (self.meter + scaled).clamp(0.0, METER_MAX);
        self.machine.step_toward(self.meter);
        self.check_culmination()
    }

    /// The culmination check — the one door into orgasm, called
    /// after every arousal change and on every tick, so *how*
    /// the meter got to the line (squeezes, side actions, a dial
    /// change under a parked meter) cannot matter. Fires only
    /// when the meter is at or over the effective threshold, the
    /// refractory window is not running, and the meter is off
    /// zero — arousal at rest is exactly zero, and zero is never
    /// over any threshold that matters, so rest alone can never
    /// culminate her.
    fn check_culmination(&mut self) -> Option<(f64, f64)> {
        if self.in_refractory() || self.meter <= 0.0 {
            return None;
        }
        if self.meter < self.profile.threshold.min(METER_MAX) {
            return None;
        }
        Some(self.culminate())
    }

    /// Orgasm, as a first-class event. Resolution profile:
    /// intensity and burst scale with the reservoir at trigger —
    /// the slow build pays out bigger; the burst is a *transfer*
    /// (reservoir → bottle) of milk already produced and already
    /// counted, and the un-paid remainder stays stored; the
    /// meter drops 45 points; the expression eases back down the
    /// ladder one step now and the rest over the refractory
    /// ticks; the refractory window opens (gain halved,
    /// re-trigger impossible inside it).
    fn culminate(&mut self) -> (f64, f64) {
        let fullness = if self.profile.reservoir_capacity > 0.0 {
            self.reservoir / self.profile.reservoir_capacity
        } else {
            0.0
        };
        let intensity = 0.35 + 0.65 * fullness;
        let burst = self.reservoir * (0.6 + 0.4 * intensity);
        self.reservoir -= burst;
        self.bottle = (self.bottle + burst).min(self.cfg.bottle_capacity);
        if self.bottle >= self.cfg.bottle_capacity {
            self.won = true;
        }
        self.meter = (self.meter - 45.0).max(0.0);
        // One step now; `step` eases her down the rest of
        // the ladder over the refractory ticks.
        self.machine.step_toward(self.meter);
        self.climax_count += 1;
        self.last_climax_intensity = intensity;
        self.refractory_until = self.tick + self.profile.refractory_ticks;
        (intensity, burst)
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
        // (if/else rather than a match on the option, for the
        // same rust-analyzer bare-`None` false positive noted in
        // `state_bytes`.)
        let grade = if let Some(last) = self.last_squeeze_tick {
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
        } else {
            Outcome::Good // the first squeeze sets the beat
        };
        self.last_squeeze_tick = Some(self.tick);
        self.combo += 1;
        // The squeeze parameter jumps with pressure; flow itself
        // happens in `step`. Depth is a fraction of the sensor's
        // nominal range, clamped to it: the parameter is a 0..=1
        // envelope by contract, and an out-of-range pressure
        // reading (a u16 can carry 65× the nominal depth) must
        // not inflate it — overpressure grading is the ceiling
        // rule's business in the grade above, not a multiplier.
        // The hand(s) ride along separately as the yield boost
        // for as long as this envelope lasts (SPEC §2.4: both
        // hands express +50% — the envelope cannot carry that,
        // it is already saturated at full depth).
        let depth = (f64::from(pressure) / 1000.0).min(1.0);
        self.squeeze = 0.55 + 0.45 * depth;
        self.flow_boost = hand.flow_multiplier();
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
        // the meter at a fixed fraction of the profile gain and
        // make no milk (SPEC §2.4 — they don't fill). Arousal is
        // arousal, though: a side action that carries her over
        // the threshold culminates exactly like a squeeze does,
        // and the outcome says so — the climax cue grammar keys
        // off the event, not the input class.
        self.idle_ticks = 0;
        self.meter = (self.meter + base * 0.5 * self.effective_gain()).clamp(0.0, METER_MAX);
        self.machine.step_toward(self.meter);
        if let Some((intensity, burst)) = self.check_culmination() {
            return Ok(Outcome::Climax { intensity, burst });
        }
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

    /// A sim driven into a rich mid-session state: flow in the
    /// bottle, a combo running, cadence history on both clocks,
    /// and a customized profile (so the payload's profile half
    /// is exercised too).
    fn rich_sim() -> Sim {
        let profile = SensitivityProfile {
            gain: 1.4,
            threshold: 65.0,
            reservoir_capacity: 140.0,
            reservoir_rate: 0.45,
            refractory_ticks: 240,
        };
        let mut sim = Sim::new(Config::default(), profile);
        sim.run(&perfect_stream(24), 24 * 30).unwrap();
        // Side actions may legitimately refuse (locked at the
        // current expression); whether each lands or not, the
        // state that results is the state under test.
        let _ = sim.apply(InputEvent::Side {
            tick: 24 * 30,
            action: Action::Caress,
        });
        let _ = sim.apply(InputEvent::Side {
            tick: 24 * 30 + 5,
            action: Action::Spank,
        });
        sim.advance_to(25 * 30);
        sim
    }

    #[test]
    fn state_bytes_round_trip_restores_exactly() {
        let sim = rich_sim();
        let bytes = sim.state_bytes();
        assert_eq!(bytes.len(), STATE_BYTES_LEN);
        assert_eq!(
            u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            STATE_VERSION
        );
        let mut restored =
            Sim::from_state_bytes(Config::default(), &bytes).expect("own payload must restore");
        assert_eq!(restored.snapshot(), sim.snapshot());
        assert_eq!(restored.trace_bytes(), sim.trace_bytes());
        assert_eq!(restored.profile(), sim.profile());
        // Continuation is bit-for-bit: the same further inputs
        // land identically on the original and the restore —
        // this is what proves the cadence history and the
        // refractory deadline survived, since both steer the
        // very next outcomes.
        let mut original = sim;
        let tail = perfect_stream(10);
        for event in &tail {
            let tick = event.tick() + 26 * 30;
            let shifted = match *event {
                InputEvent::Squeeze { hand, pressure, .. } => InputEvent::Squeeze {
                    tick,
                    hand,
                    pressure,
                },
                other => other,
            };
            assert_eq!(original.apply(shifted), restored.apply(shifted));
        }
        original.advance_to(40 * 30);
        restored.advance_to(40 * 30);
        assert_eq!(restored.snapshot(), original.snapshot());
        assert_eq!(restored.state_bytes(), original.state_bytes());
    }

    /// The refusal a payload earns (the sim itself has no
    /// `PartialEq`, so tests compare errors, not results).
    fn restore_err(bytes: &[u8]) -> StateError {
        Sim::from_state_bytes(Config::default(), bytes).expect_err("payload must be refused")
    }

    #[test]
    fn state_bytes_refuses_foreign_versions() {
        let bytes = rich_sim().state_bytes();
        for v in [0u32, 2, 99, u32::MAX] {
            let mut bad = bytes.clone();
            bad[0..4].copy_from_slice(&v.to_le_bytes());
            assert_eq!(
                restore_err(&bad),
                StateError::UnsupportedVersion(v),
                "version {v}"
            );
        }
    }

    #[test]
    fn state_bytes_refuses_malformed_payloads() {
        let bytes = rich_sim().state_bytes();
        for bad in [
            Vec::new(),
            vec![1, 0, 0],
            bytes[..bytes.len() - 1].to_vec(),
            bytes[..40].to_vec(),
        ] {
            assert_eq!(
                restore_err(&bad),
                StateError::Malformed,
                "len {}",
                bad.len()
            );
        }
        let mut extended = bytes.clone();
        extended.push(0);
        assert_eq!(restore_err(&extended), StateError::Malformed);
    }

    #[test]
    fn state_bytes_refuses_unholdable_fields() {
        let bytes = rich_sim().state_bytes();
        // Gain (offset 4..12) as NaN.
        let mut bad = bytes.clone();
        bad[4..12].copy_from_slice(&f64::NAN.to_le_bytes());
        assert_eq!(restore_err(&bad), StateError::InvalidField("gain"));
        // Meter (offset 76..84) as infinity.
        let mut bad = bytes.clone();
        bad[76..84].copy_from_slice(&f64::INFINITY.to_le_bytes());
        assert_eq!(restore_err(&bad), StateError::InvalidField("meter"));
        // Expression discriminant (offset 100..104) out of range.
        let mut bad = bytes;
        bad[100..104].copy_from_slice(&9u32.to_le_bytes());
        assert_eq!(restore_err(&bad), StateError::InvalidField("expression"));
    }
}
