//! anim — UI/feedback animation for the egui shell.
//!
//! Scope, stated plainly (D-156 honesty): this is *interface*
//! animation — pulses, pops, bursts, fades — drawn with egui
//! shapes at the shell's repaint cadence. It is NOT sprite-strip
//! character animation and invents no character art; the host's
//! presence is a medallion disc whose scale, squash, and sparkle
//! bursts carry the feedback the TUI renders as text.
//!
//! Everything here is a pure function of an elapsed time or a
//! bounded particle state, so the maths is unit-tested without a
//! window: anticipation dips before the move, overshoot passes
//! the target and settles, squash trades height for width, and
//! every particle dies on schedule. The sim in `milking-core`
//! never sees any of this (SPEC §4: rendering never feeds back).

use std::time::{Duration, Instant};

use eframe::egui;

/// Clamp a phase into 0..=1.
fn phase(elapsed: Duration, dur: Duration) -> f32 {
    if dur.is_zero() {
        return 1.0;
    }
    (elapsed.as_secs_f32() / dur.as_secs_f32()).clamp(0.0, 1.0)
}

/// Ease-out with overshoot (the "pop"): rises past 1.0, then
/// settles back onto it. `strength` ~2.0 is a lively pop.
#[must_use]
pub fn ease_out_back(t: f32, strength: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let c = strength;
    let u = t - 1.0;
    1.0 + (c + 1.0) * u * u * u + c * u * u
}

/// Anticipation: a short dip below the start before the main
/// move — the wind-up that makes the move read as intended.
#[must_use]
pub fn anticipate(t: f32, dip: f32, dip_share: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < dip_share {
        let u = t / dip_share;
        -dip * (u * (2.0 - u)) // ease-out into the dip
    } else {
        let u = (t - dip_share) / (1.0 - dip_share);
        -dip + (1.0 + dip) * ease_smooth(u) // rise from the dip to 1
    }
}

/// Smoothstep — the settle at the end of a move.
#[must_use]
pub fn ease_smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A one-shot pulse: scale envelope for a control or medallion
/// reacting to a single event. Anticipation dip, overshoot pop,
/// settle — total `DUR`.
#[derive(Clone, Copy, Debug)]
pub struct Pulse {
    start: Instant,
}

impl Pulse {
    /// The pulse window: dip 60 ms, pop, settled by 380 ms.
    pub const DUR: Duration = Duration::from_millis(380);

    #[must_use]
    pub fn now() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Current scale multiplier (1.0 = at rest).
    #[must_use]
    pub fn scale(&self) -> f32 {
        let t = phase(self.start.elapsed(), Self::DUR);
        if t >= 1.0 {
            return 1.0;
        }
        // Anticipation for the first fifth (a 5% dip), then an
        // overshoot pop that peaks near 1.09 and settles at 1.0.
        if t < 0.2 {
            1.0 - 0.05 * ease_smooth(t / 0.2)
        } else {
            let u = (t - 0.2) / 0.8;
            let pop = ease_out_back(u, 2.2);
            0.95 + 0.05 * pop + 0.8 * (pop - 1.0).max(0.0)
        }
    }

    /// True while the pulse is still moving (drives repaints).
    #[must_use]
    pub fn active(&self) -> bool {
        self.start.elapsed() < Self::DUR
    }
}

/// Squash-and-stretch factors for the medallion on a hard event:
/// returns (width, height) multipliers that preserve area-ish
/// (w*h stays near 1). Peaks mid-window, rests at both ends.
#[must_use]
pub fn squash(elapsed: Duration, dur: Duration, amount: f32) -> (f32, f32) {
    let t = phase(elapsed, dur);
    let s = (t * std::f32::consts::PI).sin() * amount;
    (1.0 + s, 1.0 - s * 0.8)
}

/// One sparkle in a burst: deterministic drift from an index
/// seed (no RNG state to keep in sync across frames).
#[derive(Clone, Copy, Debug)]
struct Particle {
    angle: f32,
    speed: f32,
    size: f32,
    life: Duration,
    born: Instant,
}

/// A bounded burst of sparkles — the payout celebration. At
/// most `MAX` particles, all dead by `MAX_LIFE`; spawning is
/// the only mutation.
#[derive(Clone, Debug)]
pub struct Burst {
    particles: Vec<Particle>,
}

impl Burst {
    /// Hard bound on live particles.
    pub const MAX: usize = 48;
    /// No particle outlives this.
    pub const MAX_LIFE: Duration = Duration::from_millis(900);

    #[must_use]
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
        }
    }

    /// Spawn `n` sparkles (clamped to the bound). Directions
    /// fan from a golden-angle spiral so bursts look scattered,
    /// never gridded.
    pub fn spawn(&mut self, n: usize) {
        let now = Instant::now();
        let base = self.particles.len();
        for i in 0..n.min(Self::MAX - base.min(Self::MAX)) {
            let k = (base + i) as f32;
            let angle = k * 2.399_963; // golden angle
            self.particles.push(Particle {
                angle,
                speed: 40.0 + (k * 37.0).rem_euclid(70.0),
                size: 2.0 + (k * 13.0).rem_euclid(3.5),
                life: Duration::from_millis(450 + ((k * 91.0) as u64).rem_euclid(450)),
                born: now,
            });
        }
    }

    /// Live particles as (offset, radius, alpha) for the
    /// painter; dead ones are reaped here.
    pub fn draw_list(&mut self) -> Vec<(egui::Vec2, f32, f32)> {
        let now = Instant::now();
        self.particles
            .retain(|p| now.duration_since(p.born) < p.life);
        self.particles
            .iter()
            .map(|p| {
                let age = now.duration_since(p.born).as_secs_f32();
                let life = p.life.as_secs_f32();
                let t = (age / life).clamp(0.0, 1.0);
                // Ease-out travel + a gentle fall late in life.
                let dist = p.speed * (1.0 - (1.0 - t) * (1.0 - t));
                let off = egui::vec2(p.angle.cos() * dist, p.angle.sin() * dist + 26.0 * t * t);
                (off, p.size * (1.0 - t * 0.6), 1.0 - ease_smooth(t))
            })
            .collect()
    }

    /// True while any particle lives (drives repaints).
    #[must_use]
    pub fn active(&self) -> bool {
        let now = Instant::now();
        self.particles
            .iter()
            .any(|p| now.duration_since(p.born) < p.life)
    }
}

impl Default for Burst {
    fn default() -> Self {
        Self::new()
    }
}

/// Screen transition: a short veil fade + rise when the screen
/// changes, so navigation reads as travel, not a cut.
#[derive(Clone, Copy, Debug)]
pub struct Transition {
    start: Instant,
}

impl Transition {
    /// Veil window.
    pub const DUR: Duration = Duration::from_millis(240);

    #[must_use]
    pub fn now() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Veil opacity 0..=0.85, high at the cut, gone by `DUR`.
    #[must_use]
    pub fn veil_alpha(&self) -> f32 {
        let t = phase(self.start.elapsed(), Self::DUR);
        0.85 * (1.0 - ease_smooth(t))
    }

    /// Content rise in points (starts 14 low, settles at 0).
    #[must_use]
    pub fn rise(&self) -> f32 {
        let t = phase(self.start.elapsed(), Self::DUR);
        14.0 * (1.0 - ease_smooth(t))
    }

    #[must_use]
    pub fn active(&self) -> bool {
        self.start.elapsed() < Self::DUR
    }
}

/// A value that glides toward its target (meter smoothing):
/// frame-rate independent exponential approach.
#[derive(Clone, Copy, Debug)]
pub struct Glide {
    pub value: f32,
}

impl Glide {
    #[must_use]
    pub fn new(value: f32) -> Self {
        Self { value }
    }

    /// Step toward `target`; `rate` ~6/s reads responsive but
    /// smooth. Returns the new value.
    pub fn step(&mut self, target: f32, dt: f32, rate: f32) -> f32 {
        let k = 1.0 - (-rate * dt.max(0.0)).exp();
        self.value += (target - self.value) * k;
        if (target - self.value).abs() < 0.001 {
            self.value = target;
        }
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overshoot_passes_one_then_settles_on_it() {
        let mut peak = 0.0f32;
        for i in 0..=100 {
            peak = peak.max(ease_out_back(i as f32 / 100.0, 2.2));
        }
        assert!(peak > 1.05, "must overshoot: peak {peak}");
        assert!((ease_out_back(1.0, 2.2) - 1.0).abs() < 1e-6);
        assert!((ease_out_back(0.0, 2.2)).abs() < 1e-6);
    }

    #[test]
    fn anticipation_dips_below_start_first() {
        assert!(anticipate(0.1, 0.08, 0.35) < 0.0);
        assert!((anticipate(1.0, 0.08, 0.35) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn squash_stays_bounded_and_rests_at_ends() {
        let (w0, h0) = squash(Duration::ZERO, Pulse::DUR, 0.18);
        assert!((w0 - 1.0).abs() < 1e-6 && (h0 - 1.0).abs() < 1e-6);
        let (w, h) = squash(Pulse::DUR / 2, Pulse::DUR, 0.18);
        assert!(w > 1.1 && h < 1.0, "mid-window squash: {w} {h}");
        assert!((w * h - 1.0).abs() < 0.05, "area roughly kept");
        let (w1, h1) = squash(Pulse::DUR, Pulse::DUR, 0.18);
        assert!((w1 - 1.0).abs() < 1e-6 && (h1 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn burst_is_bounded_and_dies_on_schedule() {
        let mut b = Burst::new();
        b.spawn(200);
        assert!(b.draw_list().len() <= Burst::MAX);
        // Age every particle past MAX_LIFE by rewinding `born`.
        for p in &mut b.particles {
            p.born = Instant::now() - (Burst::MAX_LIFE + Duration::from_millis(50));
        }
        assert!(b.draw_list().is_empty());
        assert!(!b.active());
    }

    #[test]
    fn transition_veil_fades_to_clear() {
        let tr = Transition::now();
        assert!(tr.veil_alpha() > 0.5);
        let old = Transition {
            start: Instant::now() - (Transition::DUR + Duration::from_millis(10)),
        };
        assert_eq!(old.veil_alpha(), 0.0);
        assert!(!old.active());
    }

    #[test]
    fn glide_converges_without_overshoot() {
        let mut g = Glide::new(0.0);
        let mut v = 0.0;
        for _ in 0..240 {
            v = g.step(1.0, 1.0 / 60.0, 6.0);
            assert!(v <= 1.0, "glide must not overshoot: {v}");
        }
        assert!((v - 1.0).abs() < 0.01);
    }

    #[test]
    fn pulse_rests_at_one_and_finishes() {
        let p = Pulse {
            start: Instant::now() - (Pulse::DUR + Duration::from_millis(5)),
        };
        assert_eq!(p.scale(), 1.0);
        assert!(!p.active());
        let live = Pulse::now();
        assert!(live.active());
        let s = live.scale();
        assert!((0.8..=1.3).contains(&s), "live scale sane: {s}");
    }
}
