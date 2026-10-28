//! Systems gates for the lactation/milking economy and the
//! arousal→orgasm arc (D-177): the two systems as *modeled*
//! mechanics, exercised through the public API only.
//!
//! The first half is validation — build curves, tier crossings,
//! culmination and its resolution profile, refill, conservation,
//! parameter effect. The second half is adversarial — rest,
//! spills, degenerate profiles, dial churn, restore — the ways
//! the systems must refuse to misbehave.
//!
//! Host profiles below mirror `hands-on/src/level.rs` (the
//! shipped tuning) so the curves proven here are the curves the
//! game actually ships.

use milking_core::affordance::Action;
use milking_core::expression::Expression;
use milking_core::sim::{
    Config, Hand, InputEvent, Outcome, STATE_VERSION, SensitivityProfile, Sim,
};

const EPS: f64 = 1e-9;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= EPS
}

/// Opal's shipped profile (the P1 teaching host).
fn opal() -> SensitivityProfile {
    SensitivityProfile {
        gain: 1.0,
        threshold: 96.0,
        reservoir_capacity: 220.0,
        reservoir_rate: 0.55,
        refractory_ticks: 300,
    }
}

/// Sable's shipped profile (the high-sensitivity regime).
fn sable() -> SensitivityProfile {
    SensitivityProfile {
        gain: 2.6,
        threshold: 42.0,
        reservoir_capacity: 220.0,
        reservoir_rate: 0.30,
        refractory_ticks: 300,
    }
}

/// Vega's shipped profile (the low-sensitivity regime).
fn vega() -> SensitivityProfile {
    SensitivityProfile {
        gain: 0.45,
        threshold: 98.0,
        reservoir_capacity: 420.0,
        reservoir_rate: 0.90,
        refractory_ticks: 300,
    }
}

/// Mireille's shipped profile.
fn mireille() -> SensitivityProfile {
    SensitivityProfile {
        gain: 1.4,
        threshold: 65.0,
        reservoir_capacity: 140.0,
        reservoir_rate: 0.45,
        refractory_ticks: 240,
    }
}

/// Vigil's shipped profile.
fn vigil() -> SensitivityProfile {
    SensitivityProfile {
        gain: 0.9,
        threshold: 85.0,
        reservoir_capacity: 260.0,
        reservoir_rate: 0.60,
        refractory_ticks: 360,
    }
}

fn squeeze(tick: u64, hand: Hand, pressure: u16) -> InputEvent {
    InputEvent::Squeeze {
        tick,
        hand,
        pressure,
    }
}

fn caress(tick: u64) -> InputEvent {
    InputEvent::Side {
        tick,
        action: Action::Caress,
    }
}

/// Apply `n` perfectly-timed left-hand squeezes starting at
/// `start` (one per ideal interval) and return the outcomes.
fn perfect_run(sim: &mut Sim, start: u64, n: u64) -> Vec<Outcome> {
    (0..n)
        .map(|i| {
            sim.apply(squeeze(start + i * 30, Hand::Left, 700))
                .expect("perfect stream applies")
        })
        .collect()
}

/// Squeezes (perfect stream, from tick 0) until the climax
/// outcome lands; `None` if `max` squeezes pass without one.
fn squeezes_to_climax(profile: SensitivityProfile, max: u64) -> Option<u64> {
    let mut sim = Sim::new(Config::default(), profile);
    for i in 0..max {
        let out = sim
            .apply(squeeze(i * 30, Hand::Left, 700))
            .expect("perfect stream applies");
        if matches!(out, Outcome::Climax { .. }) {
            return Some(i + 1);
        }
    }
    None
}

fn ladder(e: Expression) -> i32 {
    match e {
        Expression::Neutral => 0,
        Expression::Smiling => 1,
        Expression::Flushed => 2,
        Expression::Aroused => 3,
        Expression::Ecstatic => 4,
    }
}

// ----------------------------------------------------------------
// Validation
// ----------------------------------------------------------------

/// The build is steady (meter strictly rises squeeze over
/// squeeze inside the rhythm — rest decay never taxes a build in
/// progress) and the face walks the ladder one rung at a time,
/// in order, up and back down: no skipped tiers, no cuts.
#[test]
fn arousal_builds_steadily_and_walks_the_tiers_in_order() {
    let mut sim = Sim::new(Config::default(), opal());
    let mut faces: Vec<Expression> = Vec::new();
    let mut meters: Vec<f64> = Vec::new();
    for i in 0..49u64 {
        let out = sim
            .apply(squeeze(i * 30, Hand::Left, 700))
            .expect("perfect stream applies");
        faces.push(sim.snapshot().expression);
        meters.push(sim.snapshot().meter);
        if i < 48 {
            for _ in 0..30 {
                sim.step();
                faces.push(sim.snapshot().expression);
            }
        } else {
            assert!(
                matches!(out, Outcome::Climax { .. }),
                "the 49th perfect squeeze culminates Opal, got {out:?}"
            );
        }
    }
    // Steady build: the first twenty squeezes each add meter.
    for w in meters[..20].windows(2) {
        assert!(w[1] > w[0], "meter must rise every squeeze: {w:?}");
    }
    // The descent eases down over the refractory ticks.
    for _ in 0..400 {
        sim.step();
        faces.push(sim.snapshot().expression);
    }
    // One rung at a time, everywhere in the recording.
    for w in faces.windows(2) {
        assert!(
            (ladder(w[1]) - ladder(w[0])).abs() <= 1,
            "expression cut from {:?} to {:?}",
            w[0],
            w[1]
        );
    }
    // Every tier visited, first visits in ladder order.
    let mut firsts: Vec<Expression> = Vec::new();
    for f in &faces {
        if !firsts.contains(f) {
            firsts.push(*f);
        }
    }
    assert_eq!(
        firsts,
        vec![
            Expression::Neutral,
            Expression::Smiling,
            Expression::Flushed,
            Expression::Aroused,
            Expression::Ecstatic,
        ]
    );
}

/// The tuning interface measurably tunes: doubling the gain
/// halves the climb, lowering the threshold shortens it — the
/// exact counts are the determinism contract (a perfect stream
/// adds `gain × (2k − 1)` meter after k squeezes, and rest decay
/// never fires inside the 30-tick rhythm).
#[test]
fn gain_and_threshold_move_time_to_orgasm() {
    let base = SensitivityProfile {
        gain: 1.0,
        threshold: 60.0,
        ..opal()
    };
    assert_eq!(squeezes_to_climax(base, 200), Some(31));
    let hotter = SensitivityProfile { gain: 2.0, ..base };
    assert_eq!(squeezes_to_climax(hotter, 200), Some(16));
    let lower_line = SensitivityProfile {
        threshold: 30.0,
        ..base
    };
    assert_eq!(squeezes_to_climax(lower_line, 200), Some(16));
}

/// The stuck-equilibrium regression: at a gain where the old
/// unconditional decay exactly cancelled the per-interval build
/// (0.1 × perfect 2.0 = 0.2 gained, 30 ticks × 0.4/60 = 0.2
/// decayed), she hovered under the line forever under continued
/// perfect stimulation. With decay gated on rest, she arrives —
/// slowly, as her tuning says, but she arrives.
#[test]
fn low_gain_still_arrives_no_stuck_equilibrium() {
    let faint = SensitivityProfile {
        gain: 0.1,
        threshold: 60.0,
        ..opal()
    };
    let n = squeezes_to_climax(faint, 700).expect("a faint gain still culminates");
    assert!(
        (295..=310).contains(&n),
        "gain 0.1 to a 60 threshold is a ~301-squeeze climb, got {n}"
    );
}

/// Orgasm resolution, exactly: triggered by a side action at a
/// fixed tick (no clock advance, no production between the
/// before/after snapshots), the burst is a pure transfer —
/// bottle gains exactly the burst, reservoir loses exactly the
/// burst, total produced does not move — the meter drops the
/// profiled 45, and the refractory window opens in full.
#[test]
fn orgasm_resolution_is_exact_and_conservative() {
    let mut sim = Sim::new(Config::default(), opal());
    perfect_run(&mut sim, 0, 45); // meter 89, Ecstatic, no climax yet
    let tick = sim.snapshot().tick;
    assert_eq!(sim.snapshot().climax_count, 0);
    // Two caresses land as side actions (92, 95)…
    assert_eq!(sim.apply(caress(tick)).unwrap(), Outcome::SideApplied);
    assert_eq!(sim.apply(caress(tick)).unwrap(), Outcome::SideApplied);
    let before = sim.snapshot();
    assert!(close(before.meter, 95.0));
    // …the third crosses 96 and culminates.
    let out = sim.apply(caress(tick)).unwrap();
    // A match rather than let-else: this rust-analyzer build
    // mis-analyzes the diverging let-else arm (E0308 "expected !,
    // found ()" on code rustc accepts) — the theodolite gate is
    // a real gate, so the test is written in the shape it reads.
    let (intensity, burst) = match out {
        Outcome::Climax { intensity, burst } => (intensity, burst),
        other => panic!("the crossing caress culminates, got {other:?}"),
    };
    let after = sim.snapshot();
    let fullness = before.reservoir / 220.0;
    assert!(close(intensity, 0.35 + 0.65 * fullness));
    assert!(close(burst, before.reservoir * (0.6 + 0.4 * intensity)));
    assert!(
        close(after.bottle - before.bottle, burst),
        "bottle gains the burst"
    );
    assert!(
        close(before.reservoir - after.reservoir, burst),
        "reservoir pays the burst"
    );
    assert!(
        close(after.total_produced, before.total_produced),
        "a climax transfers milk, it does not create it"
    );
    assert!(close(after.meter, 53.0), "98 over the line, minus 45");
    assert_eq!(after.climax_count, 1);
    assert_eq!(after.refractory_remaining, 300);
}

/// Culmination is not a squeeze privilege: caresses alone, from
/// a primed smile, carry her over the line — the side-action
/// path runs the same culmination check as every other path.
#[test]
fn side_actions_alone_can_culminate() {
    let mut sim = Sim::new(Config::default(), opal());
    perfect_run(&mut sim, 0, 11); // meter 21: Smiling, caress unlocked
    let tick = sim.snapshot().tick;
    assert_eq!(sim.snapshot().expression, Expression::Smiling);
    for _ in 0..24 {
        assert_eq!(sim.apply(caress(tick)).unwrap(), Outcome::SideApplied);
    }
    // 21 + 25 × 3 = 96: the twenty-fifth caress is the crossing.
    let out = sim.apply(caress(tick)).unwrap();
    assert!(
        matches!(out, Outcome::Climax { .. }),
        "caresses culminate her too, got {out:?}"
    );
    assert_eq!(sim.snapshot().climax_count, 1);
}

/// A meter parked over a *lowered* line resolves on the clock:
/// the live dial drops the threshold under a built meter and the
/// very next tick culminates her — no further input of any kind.
/// (The app's clock-advance path has carried a handler for
/// clock-caused climaxes since the completion build; this is
/// the gate that makes it reachable.)
#[test]
fn parked_meter_resolves_on_the_clock() {
    let mut sim = Sim::new(Config::default(), opal());
    perfect_run(&mut sim, 0, 45); // meter 89 < 96: parked, no climax
    assert_eq!(sim.snapshot().climax_count, 0);
    sim.set_profile(SensitivityProfile {
        threshold: 50.0,
        ..opal()
    });
    let tick = sim.snapshot().tick;
    sim.advance_to(tick + 1);
    let s = sim.snapshot();
    assert_eq!(s.climax_count, 1, "one idle tick resolves the parked meter");
    assert!(close(s.meter, 44.0), "89 over the line, minus 45");
    assert!(
        s.last_climax_intensity > 0.35,
        "a built reservoir pays intensity"
    );
}

/// After a climax the reservoir refills from ongoing production
/// at the profile's retention share — the un-paid remainder is
/// the head start, and continued milking tops it back up while
/// the refractory window runs (production is not gain-driven,
/// so refractory does not stall the refill).
#[test]
fn reservoir_refills_after_climax() {
    let mut sim = Sim::new(Config::default(), opal());
    let outcomes = perfect_run(&mut sim, 0, 49);
    assert!(matches!(outcomes[48], Outcome::Climax { .. }));
    let at_climax = sim.snapshot();
    assert!(
        at_climax.reservoir > 0.0,
        "the burst leaves a remainder stored"
    );
    let start = at_climax.tick + 30;
    perfect_run(&mut sim, start, 20);
    let s = sim.snapshot();
    assert_eq!(s.climax_count, 1, "refractory + the 45 drop hold the count");
    assert!(
        s.reservoir > at_climax.reservoir + 10.0,
        "refill is real: {:.1} -> {:.1}",
        at_climax.reservoir,
        s.reservoir
    );
}

/// The milk economy conserves at every event of a long mixed
/// run — squeezes, caresses, two climaxes, deliberate spills:
/// `total_produced == bottle + reservoir + spilled_away`, with
/// the test tracking the spill ledger itself. Production is the
/// only source; the bottle, the reservoir, and spills are the
/// only sinks.
#[test]
fn milk_is_conserved_across_the_whole_run() {
    let mut sim = Sim::new(Config::default(), opal());
    let spill_loss = Config::default().spill_amount;
    let mut spilled_away = 0.0;
    let check = |sim: &Sim, spilled_away: f64| {
        let s = sim.snapshot();
        assert!(
            close(s.total_produced, s.bottle + s.reservoir + spilled_away),
            "conservation broke: total {:.6} vs bottle {:.6} + reservoir {:.6} + spilled {:.6}",
            s.total_produced,
            s.bottle,
            s.reservoir,
            spilled_away
        );
    };
    for i in 0..120u64 {
        let tick = i * 30;
        let out = sim.apply(squeeze(tick, Hand::Left, 700)).unwrap();
        if out == Outcome::Spill {
            spilled_away += spill_loss;
        }
        check(&sim, spilled_away);
        if i % 10 == 9 {
            let _ = sim.apply(caress(tick));
            check(&sim, spilled_away);
        }
        if i == 20 || i == 40 {
            // A deliberate mash 5 ticks later: a spill, milk lost
            // from the bottle, ledger grows by the spill loss.
            let out = sim.apply(squeeze(tick + 5, Hand::Left, 700)).unwrap();
            assert_eq!(out, Outcome::Spill);
            spilled_away += spill_loss;
            check(&sim, spilled_away);
        }
    }
    let s = sim.snapshot();
    assert!(s.climax_count >= 2, "the run crosses the line repeatedly");
    assert!(
        spilled_away > 0.0,
        "the spill leg of the ledger was exercised"
    );
}

/// Both hands mean more milk (SPEC §2.4's +50%): two sims primed
/// identically to Aroused, then tailed with one hand vs both —
/// the both-hands tail collects half again as much over the same
/// intervals. (Before this build the multiplier was clamped to
/// 1.0 by the squeeze-parameter formula and Both was strictly
/// worse than one hand: same flow, tighter window.)
#[test]
fn both_hands_express_more_than_one_hand() {
    let profile = SensitivityProfile {
        gain: 0.5,
        threshold: 100.0,
        reservoir_capacity: 420.0,
        reservoir_rate: 0.30,
        refractory_ticks: 300,
    };
    let mut one = Sim::new(Config::default(), profile);
    let mut both = Sim::new(Config::default(), profile);
    for sim in [&mut one, &mut both] {
        perfect_run(sim, 0, 80); // meter 79.5: Aroused, Both unlocked
        assert_eq!(sim.snapshot().expression, Expression::Aroused);
    }
    // First tail squeeze sets each sim's hand envelope; measure
    // from there so every measured interval runs under the
    // tail's own boost.
    one.apply(squeeze(2400, Hand::Left, 700)).unwrap();
    both.apply(squeeze(2400, Hand::Both, 700)).unwrap();
    let one0 = one.snapshot();
    let both0 = both.snapshot();
    for i in 1..10u64 {
        one.apply(squeeze(2400 + i * 30, Hand::Left, 700)).unwrap();
        both.apply(squeeze(2400 + i * 30, Hand::Both, 700)).unwrap();
    }
    let one_delta = one.snapshot().bottle - one0.bottle;
    let both_delta = both.snapshot().bottle - both0.bottle;
    assert!(
        one_delta > 5.0,
        "the one-hand tail collects, got {one_delta}"
    );
    assert!(
        both_delta > one_delta * 1.4,
        "both hands out-express one hand: {both_delta:.2} vs {one_delta:.2}"
    );
}

/// At rest the economy idles honestly: no contact, no new milk
/// (however high the meter sits), the reservoir neither grows
/// nor leaks, and the meter ebbs once the idle grace passes.
#[test]
fn no_production_and_no_climax_at_rest_after_buildup() {
    let mut sim = Sim::new(Config::default(), opal());
    perfect_run(&mut sim, 0, 45); // meter 89, just under her 96 line
    let tick = sim.snapshot().tick;
    sim.advance_to(tick + 30);
    let a = sim.snapshot();
    sim.advance_to(tick + 630);
    let b = sim.snapshot();
    assert!(close(b.bottle, a.bottle), "no contact, no collection");
    assert!(close(b.reservoir, a.reservoir), "no contact, no retention");
    assert!(close(b.total_produced, a.total_produced));
    assert_eq!(b.climax_count, 0, "89 < 96 and falling: no climax");
    assert!(
        b.meter < a.meter,
        "rest decay ebbs the meter: {} -> {}",
        a.meter,
        b.meter
    );
}

// ----------------------------------------------------------------
// Adversarial
// ----------------------------------------------------------------

/// No stimulation, no orgasm — and no anything: a fresh sim
/// advanced a hundred thousand ticks is bit-for-bit at rest.
#[test]
fn no_orgasm_without_stimulation() {
    let mut sim = Sim::new(Config::default(), opal());
    sim.advance_to(100_000);
    let s = sim.snapshot();
    assert_eq!(s.climax_count, 0);
    assert!(close(s.meter, 0.0));
    assert!(close(s.reservoir, 0.0));
    assert!(close(s.bottle, 0.0));
    assert!(close(s.total_produced, 0.0));
    assert_eq!(s.expression, Expression::Neutral);
}

/// A mashed stream never builds: only the beat-setting first
/// squeeze lands, everything after spills, the meter never
/// climbs past that first point, and no climax ever fires.
#[test]
fn spills_never_build_anything() {
    let mut sim = Sim::new(Config::default(), opal());
    let mut outcomes = Vec::new();
    for i in 0..200u64 {
        outcomes.push(sim.apply(squeeze(i * 5, Hand::Left, 700)).unwrap());
    }
    assert_eq!(
        outcomes[0],
        Outcome::Good,
        "the first squeeze sets the beat"
    );
    assert!(
        outcomes[1..].iter().all(|o| *o == Outcome::Spill),
        "a 5-tick mash is all spills"
    );
    let s = sim.snapshot();
    assert_eq!(s.climax_count, 0);
    assert!(
        s.meter <= 1.0,
        "one Good point and nothing more, got {}",
        s.meter
    );
    sim.advance_to(20_000);
    assert_eq!(sim.snapshot().climax_count, 0);
    assert!(sim.snapshot().meter < 1.0, "rest ebbs even that point away");
}

/// Degenerate profiles stay inside the bounds and keep their
/// meanings: gain 0 disables the arc but not production, a
/// negative gain cannot drive the meter under the floor, a
/// threshold above the meter's cap means the cap, a threshold
/// of 0 fires on the first rise but never from rest, and a
/// zero-capacity reservoir bursts zero at full honesty.
#[test]
fn degenerate_profiles_stay_in_bounds() {
    // Gain 0: the arc is disabled by data; the dairy is not.
    let mut sim = Sim::new(
        Config::default(),
        SensitivityProfile {
            gain: 0.0,
            threshold: 60.0,
            ..opal()
        },
    );
    perfect_run(&mut sim, 0, 100);
    let s = sim.snapshot();
    assert_eq!(s.climax_count, 0);
    assert!(close(s.meter, 0.0));
    assert!(s.bottle > 0.0, "production is not gain-driven");
    assert!(s.reservoir > 0.0, "retention is not gain-driven");

    // Negative gain: stimulation lands, the meter clamps at 0.
    let mut sim = Sim::new(
        Config::default(),
        SensitivityProfile {
            gain: -2.0,
            threshold: 60.0,
            ..opal()
        },
    );
    perfect_run(&mut sim, 0, 50);
    let s = sim.snapshot();
    assert_eq!(s.climax_count, 0);
    assert!(close(s.meter, 0.0), "the floor holds, got {}", s.meter);
    assert!(s.meter.is_finite() && s.reservoir.is_finite() && s.bottle.is_finite());

    // Threshold 150: the effective line is the meter cap, 100 —
    // reached on the 51st perfect squeeze (2k − 1 clamps at 100).
    let over = SensitivityProfile {
        gain: 1.0,
        threshold: 150.0,
        ..opal()
    };
    assert_eq!(squeezes_to_climax(over, 200), Some(51));

    // Threshold 0: rest alone never fires (the meter is 0 and
    // zero is not a rise); the first stimulation culminates.
    let mut sim = Sim::new(
        Config::default(),
        SensitivityProfile {
            gain: 1.0,
            threshold: 0.0,
            ..opal()
        },
    );
    sim.advance_to(10_000);
    assert_eq!(sim.snapshot().climax_count, 0, "rest cannot culminate her");
    let out = sim.apply(squeeze(10_000, Hand::Left, 700)).unwrap();
    assert!(
        matches!(out, Outcome::Climax { .. }),
        "first rise crosses a 0 line"
    );
    sim.advance_to(110_000);
    assert_eq!(sim.snapshot().climax_count, 1, "and rest cannot re-fire it");

    // Zero-capacity reservoir: nothing stores, the burst is 0,
    // the intensity is the floor — no NaN from the fullness 0/0.
    let mut sim = Sim::new(
        Config::default(),
        SensitivityProfile {
            gain: 1.0,
            threshold: 60.0,
            reservoir_capacity: 0.0,
            ..opal()
        },
    );
    let mut fired = None;
    for i in 0..60u64 {
        let out = sim.apply(squeeze(i * 30, Hand::Left, 700)).unwrap();
        assert!(close(sim.snapshot().reservoir, 0.0));
        if let Outcome::Climax { intensity, burst } = out {
            fired = Some((intensity, burst));
            break;
        }
    }
    let (intensity, burst) = fired.expect("she still culminates with no storage");
    assert!(close(intensity, 0.35));
    assert!(close(burst, 0.0));
}

/// Dial churn and input chaos together: 3000 pseudo-random
/// events (an in-test LCG — no randomness crate, the stream is
/// fixed) across all five shipped profiles swapped live, every
/// pressure a u16 can carry, side actions attempted blind.
/// After every event the state must sit inside its bounds.
#[test]
fn chaos_stream_never_leaves_the_bounds() {
    let profiles = [opal(), sable(), vega(), mireille(), vigil()];
    let pressures = [0u16, 300, 700, 900, 1000, 5000, u16::MAX];
    let hands = [Hand::Left, Hand::Right, Hand::Both];
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = move || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        state >> 33
    };
    let mut sim = Sim::new(Config::default(), profiles[0]);
    let mut profile = profiles[0];
    let mut tick = 0u64;
    for i in 0..3000u64 {
        if i % 500 == 499 {
            profile = profiles[(next() % 5) as usize];
            sim.set_profile(profile);
        }
        tick += next() % 37;
        let event = if next() % 4 == 0 {
            InputEvent::Side {
                tick,
                action: if next() % 2 == 0 {
                    Action::Caress
                } else {
                    Action::Spank
                },
            }
        } else {
            squeeze(
                tick,
                hands[(next() % 3) as usize],
                pressures[(next() % 7) as usize],
            )
        };
        let _ = sim.apply(event);
        let s = sim.snapshot();
        assert!(
            s.meter.is_finite() && (0.0..=100.0).contains(&s.meter),
            "meter {}",
            s.meter
        );
        assert!(
            s.reservoir.is_finite()
                && s.reservoir >= 0.0
                && s.reservoir <= profile.reservoir_capacity,
            "reservoir {} vs cap {}",
            s.reservoir,
            profile.reservoir_capacity
        );
        assert!(
            s.bottle.is_finite() && (0.0..=1000.0).contains(&s.bottle),
            "bottle {}",
            s.bottle
        );
        assert!(
            (0.0..=1.0).contains(&s.squeeze),
            "squeeze param {}",
            s.squeeze
        );
        assert!((0.0..=1.0).contains(&s.fullness), "fullness {}", s.fullness);
        if s.won {
            break;
        }
    }
}

/// The first squeeze is ungraded — it sets the beat — so a
/// max-range pressure reading lands there. Its depth must clamp
/// to the envelope's contract: the squeeze parameter never
/// exceeds 1.0, however large the raw reading. (Before this
/// build, depth 65.5 produced a squeeze parameter of ~30.)
#[test]
fn first_squeeze_at_max_pressure_stays_in_range() {
    let mut sim = Sim::new(Config::default(), opal());
    let out = sim.apply(squeeze(0, Hand::Left, u16::MAX)).unwrap();
    assert_eq!(out, Outcome::Good, "the first squeeze sets the beat");
    let s = sim.snapshot();
    assert!(s.squeeze <= 1.0, "parameter clamped, got {}", s.squeeze);
    assert!(
        s.squeeze > 0.99,
        "full depth is full depth, got {}",
        s.squeeze
    );
    // The same reading inside the rhythm is over the ceiling.
    let out = sim.apply(squeeze(30, Hand::Left, u16::MAX)).unwrap();
    assert_eq!(out, Outcome::Spill);
}

/// Determinism across the full systems: a long mixed run on the
/// slowest shipped profile — build, climax, refractory, refill,
/// side actions — replays byte-identically, in both the trace
/// encoding and the save-state payload.
#[test]
fn twin_run_with_the_full_systems_is_byte_identical() {
    fn run() -> Sim {
        let mut sim = Sim::new(Config::default(), vega());
        for i in 0..180u64 {
            let tick = i * 30;
            sim.apply(squeeze(tick, Hand::Left, 700)).unwrap();
            if i % 15 == 14 {
                let _ = sim.apply(caress(tick));
            }
        }
        sim.advance_to(180 * 30 + 600);
        sim
    }
    let a = run();
    let b = run();
    assert!(
        a.snapshot().climax_count >= 1,
        "the slow regime still culminates"
    );
    assert_eq!(a.trace_bytes(), b.trace_bytes());
    assert_eq!(a.state_bytes(), b.state_bytes());
}

/// The save-state contract this build shares with the menu
/// commission: the payload layout did not change (the version
/// stays 1 — the systems build changed behavior, not shape),
/// and a sim restored mid-build culminates identically to the
/// original on the same next input.
#[test]
fn restored_sim_culminates_identically_and_version_holds() {
    assert_eq!(
        STATE_VERSION, 1,
        "behavior changed; the payload shape did not"
    );
    let mut original = Sim::new(Config::default(), opal());
    perfect_run(&mut original, 0, 48); // meter 95, one caress from the line
    let bytes = original.state_bytes();
    let mut restored = Sim::from_state_bytes(Config::default(), &bytes).unwrap();
    let tick = original.snapshot().tick;
    let a = original.apply(caress(tick)).unwrap();
    let b = restored.apply(caress(tick)).unwrap();
    assert!(
        matches!(a, Outcome::Climax { .. }),
        "the original culminates: {a:?}"
    );
    assert_eq!(a, b, "the restored sim culminates identically");
    assert_eq!(original.snapshot(), restored.snapshot());
    assert_eq!(original.state_bytes(), restored.state_bytes());
}
