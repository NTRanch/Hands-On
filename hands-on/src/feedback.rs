//! The shared feedback engine (SPEC §6): the girl is the reward
//! instrument.
//!
//! Every level's exercise produces success/failure events; this
//! engine is the one place those events become stimulation. It
//! drives the host girl's deterministic state inside
//! `milking-core` (arousal ladder, governed expressions, the
//! sensitivity reservoir, climax and refractory) and renders the
//! result as text: bars, named states, a style line naming the
//! House Cel beats as they occur (SPEC §7), an event log, and a
//! **fake-haptics readout** — the scalar the torogao-haptics
//! router *would* emit for her current state, clamped at the
//! router's 0.20 ceiling. Fakes only: nothing in this crate can
//! reach a device, by construction (there is no transport here).
//!
//! Because the engine wraps the fixed-step sim, feedback lands on
//! the same deterministic tick as the exercise event that caused
//! it. Do the exercise well and she responds — on the tick, every
//! time, identically in every replay.

use milking_core::expression::Expression;
use milking_core::sim::{Config, InputError, InputEvent, Outcome, Sim, Snapshot};

use crate::level::HostGirl;

/// The torogao-haptics ceiling the fake readout clamps to.
pub const FAKE_CEILING: f64 = 0.20;

/// Exercise events in, girl state out.
pub struct FeedbackEngine {
    host: HostGirl,
    sim: Sim,
    log: Vec<String>,
}

impl FeedbackEngine {
    /// Start a session with `host` on her default profile.
    #[must_use]
    pub fn new(host: HostGirl, cfg: Config) -> Self {
        Self {
            host,
            sim: Sim::new(cfg, host.profile),
            log: vec![format!(
                "{} settles in. (gain {:.2}, threshold {:.0}, reservoir {:.0})",
                host.name,
                host.profile.gain,
                host.profile.threshold,
                host.profile.reservoir_capacity
            )],
        }
    }

    /// The host of this session.
    #[must_use]
    pub fn host(&self) -> HostGirl {
        self.host
    }

    /// The wrapped simulation.
    #[must_use]
    pub fn sim(&self) -> &Sim {
        &self.sim
    }

    /// Mutable access for dials and clock advances.
    pub fn sim_mut(&mut self) -> &mut Sim {
        &mut self.sim
    }

    /// Current girl state.
    #[must_use]
    pub fn snapshot(&self) -> Snapshot {
        self.sim.snapshot()
    }

    /// The most recent log lines, oldest first.
    #[must_use]
    pub fn log_tail(&self, n: usize) -> &[String] {
        let start = self.log.len().saturating_sub(n);
        &self.log[start..]
    }

    /// What the torogao-haptics router WOULD emit for her right
    /// now: her squeeze parameter × expression gain, clamped at
    /// the ceiling — a readout, never a transmission.
    #[must_use]
    pub fn fake_haptic_scalar(&self) -> f64 {
        let s = self.sim.snapshot();
        (s.squeeze * s.expression.flow_gain() * 0.15).min(FAKE_CEILING)
    }

    /// Feed one exercise event into the girl. The outcome is
    /// narrated into the log in her terms, and returned for the
    /// exercise UI to grade.
    pub fn feed(&mut self, event: InputEvent) -> Result<Outcome, InputError> {
        let name = self.host.name;
        let outcome = self.sim.apply(event);
        match outcome {
            Ok(Outcome::Perfect) => self.log.push(format!("{name}: a shiver — perfect tempo.")),
            Ok(Outcome::Good) => self.log.push(format!("{name}: pleased hum.")),
            Ok(Outcome::Spill) => {
                self.log
                    .push(format!("{name}: flinch — spill! Off-beat or too hard."));
            }
            Ok(Outcome::SideApplied) => self.log.push(format!("{name}: leans into the touch.")),
            Ok(Outcome::Climax { intensity, burst }) => {
                self.log.push(format!(
                    "{name}: CLIMAX (torogao face) — melted pleasure: half-lidded upturned eyes, luminous heart pupils, open mouth, heavy blush, sparkles everywhere. intensity {intensity:.2} — reservoir burst +{burst:.1} into the bottle."
                ));
            }
            Err(InputError::Locked(_)) => {
                self.log.push(format!(
                    "{name}: shakes her head — that action is still locked."
                ));
            }
            Err(e) => self.log.push(format!("input refused: {e:?}")),
        }
        outcome
    }

    /// The house style, legible in text (SPEC §7): the feedback
    /// panel names the House Cel style beats as they occur, so
    /// the eye grammar reads even in ASCII. The eye rule holds
    /// at every state — heart-shaped pupils, her own eye color;
    /// only the *intensity* travels with arousal.
    fn style_line(expression: Expression) -> &'static str {
        match expression {
            Expression::Neutral => {
                "House Cel — heart pupils (the eye rule), gaze level, blush not yet risen"
            }
            Expression::Smiling => {
                "House Cel — heart pupils, first catchlight sparkles, blush rising at the edges"
            }
            Expression::Flushed => {
                "House Cel — heart pupils glowing under heavy lids, blush rising deep, sparkles multiplying"
            }
            Expression::Aroused => {
                "House Cel — heart pupils bright, gaze turning up, blush full, sparkles dancing"
            }
            Expression::Ecstatic => {
                "House Cel — TOROGAO: half-lidded upturned eyes, luminous heart pupils, open mouth, heavy blush, sparkles everywhere"
            }
        }
    }

    /// Render a fraction as a 24-cell bar.
    fn bar(frac: f64) -> String {
        let filled = (frac.clamp(0.0, 1.0) * 24.0).round() as usize;
        format!("[{}{}]", "#".repeat(filled), "-".repeat(24 - filled))
    }

    /// The full text panel: host, governed expression by name,
    /// every meter that matters, the fake-haptics readout, and
    /// the event log tail.
    #[must_use]
    pub fn render(&self, cfg: &Config) -> String {
        let s = self.snapshot();
        let mut out = String::new();
        out.push_str(&format!(
            "{} — expression: {:?}   tick {}\n",
            self.host.name, s.expression, s.tick
        ));
        out.push_str(&format!("  style    {}\n", Self::style_line(s.expression)));
        out.push_str(&format!(
            "  arousal  {} {:5.1}/100\n",
            Self::bar(s.meter / 100.0),
            s.meter
        ));
        out.push_str(&format!(
            "  reservoir{} {:6.1} stored (bar = fullness)\n",
            Self::bar(s.fullness),
            s.reservoir,
        ));
        out.push_str(&format!(
            "  bottle   {} {:6.1}/{}   combo {}   spills {}   climaxes {}\n",
            Self::bar(s.bottle / cfg.bottle_capacity),
            s.bottle,
            cfg.bottle_capacity,
            s.combo,
            s.spills,
            s.climax_count
        ));
        if s.refractory_remaining > 0 {
            out.push_str(&format!(
                "  refractory: {} ticks (sensitivity halved)\n",
                s.refractory_remaining
            ));
        }
        out.push_str(&format!(
            "  haptics> FAKE router would emit scalar {:.3} (ceiling {:.2}) — no device attached\n",
            self.fake_haptic_scalar(),
            FAKE_CEILING
        ));
        for line in self.log_tail(3) {
            out.push_str(&format!("  · {line}\n"));
        }
        if s.won {
            out.push_str("  *** BOTTLE FULL — level complete ***\n");
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use milking_core::sim::{Hand, SensitivityProfile};

    fn opal() -> HostGirl {
        HostGirl {
            name: "Opal",
            sketch: "test host",
            profile: SensitivityProfile::high(),
        }
    }

    #[test]
    fn fake_haptic_readout_never_exceeds_the_ceiling() {
        let mut engine = FeedbackEngine::new(opal(), Config::default());
        for i in 0..40u64 {
            let _ = engine.feed(InputEvent::Squeeze {
                tick: i * 30,
                hand: Hand::Left,
                pressure: 1000,
            });
            assert!(engine.fake_haptic_scalar() <= FAKE_CEILING + f64::EPSILON);
        }
    }

    #[test]
    fn success_events_move_the_girl_and_log_it() {
        let mut engine = FeedbackEngine::new(opal(), Config::default());
        engine
            .feed(InputEvent::Squeeze {
                tick: 0,
                hand: Hand::Left,
                pressure: 700,
            })
            .unwrap();
        assert!(engine.snapshot().meter > 0.0);
        assert!(engine.log_tail(1)[0].contains("Opal"));
    }

    #[test]
    fn style_line_holds_the_eye_rule_at_every_state() {
        for e in Expression::ALL {
            let line = FeedbackEngine::style_line(e);
            assert!(line.contains("heart pupils"), "{e:?}: {line}");
            assert!(line.contains("House Cel"), "{e:?}: {line}");
        }
        assert!(FeedbackEngine::style_line(Expression::Ecstatic).contains("TOROGAO"));
        let panel = FeedbackEngine::new(opal(), Config::default()).render(&Config::default());
        assert!(panel.contains("style    House Cel"), "panel: {panel}");
    }

    #[test]
    fn climax_is_reachable_and_narrated() {
        let mut engine = FeedbackEngine::new(opal(), Config::default());
        for i in 0..60u64 {
            let _ = engine.feed(InputEvent::Squeeze {
                tick: i * 30,
                hand: if i.is_multiple_of(2) {
                    Hand::Left
                } else {
                    Hand::Right
                },
                pressure: 700,
            });
            if engine.snapshot().climax_count > 0 {
                break;
            }
        }
        assert!(engine.snapshot().climax_count > 0);
        assert!(
            engine
                .log_tail(10)
                .iter()
                .any(|l| l.contains("CLIMAX (torogao face)"))
        );
    }
}
