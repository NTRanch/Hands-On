//! Integration gates for milking-core: validation + adversarial.
//!
//! The adversarial half thinks like a cheater and a fuzzer: mash
//! the input, over-pressurize, replay stale events, try locked
//! actions, author loops that do not close, and try to make the
//! state machine skip a face. Every one of those must fail in the
//! typed, observable way the SPEC promises — never silently, never
//! by panic.

use milking_core::affordance::{Action, AffordanceError};
use milking_core::authoring::{Easing, Keyform, Track};
use milking_core::sim::{Config, Hand, InputError, InputEvent, Outcome, SensitivityProfile, Sim};

fn squeeze(tick: u64, hand: Hand, pressure: u16) -> InputEvent {
    InputEvent::Squeeze {
        tick,
        hand,
        pressure,
    }
}

/// Perfectly timed alternating squeezes.
fn perfect_stream(n: u64) -> Vec<InputEvent> {
    (0..n)
        .map(|i| {
            squeeze(
                i * 30,
                if i.is_multiple_of(2) {
                    Hand::Left
                } else {
                    Hand::Right
                },
                700,
            )
        })
        .collect()
}

/// Squeeze on the beat until a climax fires; return the number of
/// squeezes it took, or None if `max` was not enough.
fn squeezes_to_climax(profile: SensitivityProfile, max: u64) -> Option<u64> {
    let mut sim = Sim::new(Config::default(), profile);
    for event in perfect_stream(max) {
        sim.apply(event).ok()?;
        if sim.snapshot().climax_count > 0 {
            // Count = index of the triggering squeeze + 1.
            return Some(event.tick() / 30 + 1);
        }
    }
    None
}

// ---- Timing: the window is the game ------------------------------

#[test]
fn out_of_window_squeeze_produces_spill_not_flow() {
    let mut sim = Sim::new(Config::default(), SensitivityProfile::default());
    sim.apply(squeeze(0, Hand::Left, 700)).unwrap();
    // Interval 50 vs ideal 30: error 20, outside the Good window
    // (12) — a spill, and a spill produces nothing.
    let outcome = sim.apply(squeeze(50, Hand::Right, 700)).unwrap();
    assert_eq!(outcome, Outcome::Spill);
    assert_eq!(sim.snapshot().spills, 1);
    // The spilled squeeze produced nothing itself: the bottle
    // holds only what the first squeeze made, minus the spill.
    let snap = sim.snapshot();
    assert!(snap.bottle <= snap.total_produced);
}

#[test]
fn badly_off_beat_squeeze_spills() {
    let mut sim = Sim::new(Config::default(), SensitivityProfile::default());
    sim.apply(squeeze(0, Hand::Left, 700)).unwrap();
    // Interval 55 vs ideal 30: error 25, far outside Good (12).
    let outcome = sim.apply(squeeze(55, Hand::Right, 700)).unwrap();
    assert_eq!(outcome, Outcome::Spill);
    assert_eq!(sim.snapshot().spills, 1);
    // The spilled squeeze itself produced nothing: the bottle
    // holds only what earlier accepted squeezes made, minus the
    // spill subtraction — never more than total production.
    let snap = sim.snapshot();
    assert!(snap.bottle <= snap.total_produced);
}

#[test]
fn mashing_and_overpressure_spill() {
    let mut sim = Sim::new(Config::default(), SensitivityProfile::default());
    sim.apply(squeeze(0, Hand::Left, 700)).unwrap();
    // Mash: 5 ticks later, far below the 15-tick floor.
    assert_eq!(
        sim.apply(squeeze(5, Hand::Right, 700)).unwrap(),
        Outcome::Spill
    );
    // Over-pressure: on-beat but 950 > the 900 ceiling.
    assert_eq!(
        sim.apply(squeeze(35, Hand::Left, 950)).unwrap(),
        Outcome::Spill
    );
    assert_eq!(sim.snapshot().spills, 2);
}

#[test]
fn locked_actions_and_both_hands_refuse_until_unlocked() {
    let mut sim = Sim::new(Config::default(), SensitivityProfile::default());
    let caress = InputEvent::Side {
        tick: 0,
        action: Action::Caress,
    };
    assert!(matches!(
        sim.apply(caress),
        Err(InputError::Locked(AffordanceError::Locked { .. }))
    ));
    assert!(matches!(
        sim.apply(squeeze(0, Hand::Both, 700)),
        Err(InputError::Locked(_))
    ));
    // A milk action smuggled into a side slot is a typed refusal.
    let smuggled = InputEvent::Side {
        tick: 0,
        action: Action::MilkLeft,
    };
    assert_eq!(
        sim.apply(smuggled),
        Err(InputError::NotASideAction(Action::MilkLeft))
    );
}

#[test]
fn state_machine_cannot_skip_states() {
    // Observe the expression TICK BY TICK through a whole
    // high-sensitivity session, including the post-climax descent:
    // consecutive ticks may differ by at most one ladder step, in
    // either direction. The descent after a climax must ease down
    // through the intermediate faces, never cut to Neutral.
    let mut sim = Sim::new(Config::default(), SensitivityProfile::high());
    // Prime the beat, then observe every tick while squeezing.
    sim.apply(squeeze(0, Hand::Left, 700)).unwrap();
    let mut prev = sim.snapshot().expression;
    let mut saw_climax_descent = false;
    let mut peak = prev;
    for i in 1..=600u64 {
        let t = i * 30;
        // Alternate hands on the beat for the first stretch, then
        // stop and let her settle (the descent under observation).
        if i <= 12 {
            sim.apply(squeeze(
                t,
                if i.is_multiple_of(2) {
                    Hand::Left
                } else {
                    Hand::Right
                },
                700,
            ))
            .unwrap();
            let now = sim.snapshot().expression;
            assert!(
                (now as i32 - prev as i32).abs() <= 1,
                "expression skipped at apply: {prev:?} -> {now:?}"
            );
            prev = now;
            peak = peak.max(now);
        }
        // Tick-by-tick observation of the settle/descent.
        for _ in 0..30 {
            sim.step();
            let now = sim.snapshot().expression;
            assert!(
                (now as i32 - prev as i32).abs() <= 1,
                "expression skipped at tick {}: {prev:?} -> {now:?}",
                sim.snapshot().tick
            );
            if sim.snapshot().climax_count > 0 && now < peak {
                saw_climax_descent = true;
            }
            prev = now;
        }
        if i > 12 && sim.snapshot().expression as i32 == 0 {
            break;
        }
    }
    assert!(sim.snapshot().climax_count > 0, "session should climax");
    assert!(saw_climax_descent, "descent should be observable");
}

// ---- Authoring: closure is a rule, not a hope --------------------

#[test]
fn mismatched_end_keyform_fails_closure() {
    let track = Track::new(
        1000,
        vec![
            Keyform {
                at_ms: 0,
                value: 0.1,
            },
            Keyform {
                at_ms: 500,
                value: 0.9,
            },
            Keyform {
                at_ms: 1000,
                value: 0.8,
            },
        ],
        Easing::Linear,
    )
    .unwrap();
    assert!(track.check_closure().is_err());
}

// ---- Amendment 1: the sensitivity regimes -------------------------

#[test]
fn high_sensitivity_climaxes_in_strictly_fewer_squeezes() {
    let high = squeezes_to_climax(SensitivityProfile::high(), 300).expect("high should climax");
    let low = squeezes_to_climax(SensitivityProfile::low(), 400).expect("low should climax");
    assert!(
        high < low,
        "high regime took {high} squeezes, low took {low}"
    );
    assert!(high <= 20, "high regime should be quick, took {high}");
}

#[test]
fn low_regime_climax_is_bigger_in_intensity_and_burst() {
    // Run each regime to its climax and capture the burst outcome.
    let capture = |profile: SensitivityProfile| {
        let mut sim = Sim::new(Config::default(), profile);
        for event in perfect_stream(400) {
            if let Ok(Outcome::Climax { intensity, burst }) = sim.apply(event) {
                return (intensity, burst);
            }
        }
        panic!("regime never climaxed");
    };
    let (high_i, high_b) = capture(SensitivityProfile::high());
    let (low_i, low_b) = capture(SensitivityProfile::low());
    assert!(low_i > high_i, "intensity: low {low_i} vs high {high_i}");
    assert!(low_b > high_b, "burst: low {low_b} vs high {high_b}");
}

#[test]
fn refractory_blocks_immediate_retrigger() {
    let mut sim = Sim::new(Config::default(), SensitivityProfile::high());
    // Drive to the first climax.
    let mut climaxed_at = None;
    for event in perfect_stream(60) {
        if let Ok(Outcome::Climax { .. }) = sim.apply(event) {
            climaxed_at = Some(event.tick());
            break;
        }
    }
    let t0 = climaxed_at.expect("high regime should climax");
    // Keep squeezing perfectly through the refractory window:
    // no second climax may fire inside it.
    let refractory = SensitivityProfile::high().refractory_ticks;
    let mut t = t0;
    let mut i = 0u64;
    while t + 30 < t0 + refractory {
        t += 30;
        i += 1;
        sim.apply(squeeze(
            t,
            if i.is_multiple_of(2) {
                Hand::Left
            } else {
                Hand::Right
            },
            700,
        ))
        .unwrap();
        assert_eq!(sim.snapshot().climax_count, 1, "re-triggered in refractory");
    }
}

#[test]
fn determinism_twin_run_byte_equal_with_sensitivity() {
    let events = perfect_stream(50);
    let run = |profile: SensitivityProfile| {
        let mut sim = Sim::new(Config::default(), profile);
        sim.run(&events, 2000).unwrap();
        sim.trace_bytes()
    };
    assert_eq!(
        run(SensitivityProfile::low()),
        run(SensitivityProfile::low())
    );
    // And a different profile really is a different run.
    assert_ne!(
        run(SensitivityProfile::low()),
        run(SensitivityProfile::high())
    );
}

#[test]
fn bottle_win_ends_the_session() {
    // A huge flow config so the win is reachable in a short test.
    let cfg = Config {
        flow_base: 50.0,
        ..Config::default()
    };
    let mut sim = Sim::new(cfg, SensitivityProfile::default());
    // Apply until the win lands; the core then refuses the rest
    // of the stream with SessionOver, which is the point.
    for event in perfect_stream(30) {
        match sim.apply(event) {
            Ok(_) => {}
            Err(InputError::SessionOver) => break,
            Err(other) => panic!("unexpected error: {other:?}"),
        }
    }
    assert!(sim.snapshot().won);
    assert_eq!(
        sim.apply(squeeze(2000, Hand::Left, 700)),
        Err(InputError::SessionOver)
    );
}
