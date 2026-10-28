//! The session layer (SPEC §6): the wrapper that turns each
//! level's pure `milking-core` twin into a playable sitting.
//!
//! A [`Session`] is a small deterministic state machine over
//! text commands: an intro, one command at a time, and a
//! verdict. The same session object is driven three ways —
//! interactively by the TUI loop, headlessly by `--demo`, and
//! by the test suite — through [`run_scripted`], so the route
//! that proves a level in CI is the route a player plays.
//!
//! The voice throughout is Daddy Style (SPEC §7.4): terms
//! set in advance by the host, applied consistently, with
//! praise exact and failure landing as consequence — the
//! framing and the engineering are the same discipline.
//!
//! Feedback (SPEC §6.3): a session never touches the host
//! directly. When a session succeeds, the *runner* pays the
//! success out through the shared [`FeedbackEngine`] via
//! [`reward_host`] — on-beat squeezes on the host's own
//! profile, so her ladder, reservoir, climax, and refractory
//! are the payout, exactly as Level 1's direct loop behaves.
//! The engine does not know or care which exercise produced
//! the success. That indifference is the design.

use milking_core::sim::{Config, Hand, InputEvent};

use crate::feedback::FeedbackEngine;
use crate::level::LevelDescriptor;

/// What one command did to the session.
#[derive(Clone, Debug, PartialEq)]
pub struct StepOut {
    /// Narration for the player (may be empty).
    pub text: String,
    /// The sitting is over (success or failure).
    pub done: bool,
    /// The sitting ended in success (only meaningful when
    /// `done`).
    pub success: bool,
}

impl StepOut {
    /// Continue the sitting with narration.
    #[must_use]
    pub fn go(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            done: false,
            success: false,
        }
    }

    /// End the sitting in success.
    #[must_use]
    pub fn pass(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            done: true,
            success: true,
        }
    }

    /// End the sitting in failure.
    #[must_use]
    pub fn fail(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            done: true,
            success: false,
        }
    }
}

/// One playable level sitting.
pub trait Session {
    /// The level's static descriptor (host included).
    fn descriptor(&self) -> &LevelDescriptor;

    /// The sitting's opening text: the exercise, the commands,
    /// and what success looks like.
    fn intro(&self) -> String;

    /// Answer one command line. Total: unknown commands are
    /// answered with help text, never a panic or a silent
    /// ignore.
    fn handle(&mut self, cmd: &str) -> StepOut;
}

/// Drive a session through a scripted route: feed each
/// command until the session ends or the script runs out.
/// Returns the verdict and the full transcript — the demo
/// and the tests assert on both.
#[must_use]
pub fn run_scripted(session: &mut dyn Session, script: &[&str]) -> (bool, Vec<String>) {
    let mut transcript = vec![session.intro()];
    for cmd in script {
        let step = session.handle(cmd);
        transcript.push(format!("> {cmd}\n{}", step.text));
        if step.done {
            return (step.success, transcript);
        }
    }
    (false, transcript)
}

/// Pay a level success out on the host: on-beat squeezes at
/// the ideal interval on her own sensitivity profile, until
/// she climaxes or the bounded payout is exhausted. Returns
/// how many climaxes the payout produced (0 or 1 — the
/// refractory window makes a second impossible inside one
/// payout for every shipped profile).
///
/// The bound is named in advance (Daddy Style): at most
/// [`REWARD_MAX_SQUEEZES`] squeezes. Every shipped host
/// profile climaxes well inside it — the slow LOW regime
/// (gain 0.45, threshold 98) is the long pole, and the gate
/// suite proves each host's payout lands.
pub const REWARD_MAX_SQUEEZES: u64 = 400;

pub fn reward_host(engine: &mut FeedbackEngine, cfg: &Config) -> u32 {
    let before = engine.snapshot().climax_count;
    let start = engine.snapshot().tick;
    for i in 0..REWARD_MAX_SQUEEZES {
        let tick = start + i * cfg.ideal_interval_ticks;
        let hand = if i.is_multiple_of(2) {
            Hand::Left
        } else {
            Hand::Right
        };
        let _ = engine.feed(InputEvent::Squeeze {
            tick,
            hand,
            pressure: 700,
        });
        if engine.snapshot().climax_count > before {
            break;
        }
    }
    let end = engine.snapshot().tick;
    engine.sim_mut().advance_to(end);
    engine.snapshot().climax_count - before
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::playroom_levels;

    #[test]
    fn reward_pays_a_climax_on_every_playroom_host() {
        // The payout is the framework's promise (SPEC §6.3):
        // succeed at her level and she responds. Every host
        // profile — balanced, HIGH, LOW, and the two custom
        // profiles — must climax inside the named bound.
        let cfg = Config::default();
        for desc in playroom_levels() {
            let mut engine = FeedbackEngine::new(desc.host, cfg);
            let gained = reward_host(&mut engine, &cfg);
            assert_eq!(
                gained, 1,
                "{} (P{}) must climax inside the reward bound",
                desc.host.name, desc.id
            );
        }
    }
}
