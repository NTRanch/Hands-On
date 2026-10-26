//! hands-on — the estate's game-system learning crate
//! (IDEA-0093). Levels codify real engine systems as minigames;
//! a shared feedback engine turns exercise success into live
//! stimulation feedback on each level's host girl.
//!
//! This binary is the TUI the Neovim routes launch. It runs in
//! any terminal, including a Neovim terminal buffer (Torogao
//! Addition V pattern). I/O is plain line-buffered stdin/stdout —
//! zero extra dependencies, scriptable, and smoke-testable:
//!
//! ```text
//! hands-on --demo        scripted session, prints state, exits
//! printf '1\nf\nj\nq\n' | hands-on
//! ```
//!
//! Level 1 commands (one per line): f/j squeeze L/R, F/J hard
//! squeeze, b both hands, c caress, s spank, w wait one beat,
//! +/- gain dial, T/t threshold dial, d dial, studio … (loop
//! exercise), menu, q.

#![forbid(unsafe_code)]

mod feedback;
mod level;

use std::io::{BufRead, Write};
use std::time::Instant;

use milking_core::affordance::Action;
use milking_core::authoring::{Easing, Keyform, Track};
use milking_core::sim::{Config, Hand, InputEvent, SensitivityProfile};

use feedback::FeedbackEngine;
use level::{Level, MilkingLevel, Track as LevelTrack, playroom_levels, registry, study_levels};

fn print_menu() {
    println!("=== hands-on — learn the systems by playing them ===");
    println!("-- The Playroom (game systems + deterministic animation, taught by playing) --");
    for l in playroom_levels() {
        println!(
            "  {}. {}  [{}]  host: {} — {}",
            l.id,
            l.title,
            if l.built { "playable" } else { "designed" },
            l.host.name,
            l.source_system
        );
    }
    println!("-- The Study (Rust mastery lessons — designed, not built) --");
    for l in study_levels() {
        println!(
            "  S{}. Lesson {}: {}  [locked/design]  host: {} — {}",
            l.id, l.id, l.title, l.host.name, l.source_system
        );
    }
    println!("Pick a Playroom number (1..9), a Study lesson (S1..S8), or q.");
}

fn show_dial(engine: &FeedbackEngine) {
    let p = engine.sim().profile();
    println!(
        "sensitivity dial ({}): gain {:.2}  threshold {:.0}  reservoir cap {:.0}  refractory {} ticks",
        engine.host().name,
        p.gain,
        p.threshold,
        p.reservoir_capacity,
        p.refractory_ticks
    );
}

fn studio(args: &[&str]) {
    if args.len() < 3 {
        println!("usage: studio <cycle_ms> <linear|smooth|easeinout|hold> <ms=value,...>");
        return;
    }
    let cycle_ms: u32 = args[0].parse().unwrap_or(0);
    let easing = match args[1] {
        "smooth" => Easing::Smooth,
        "easeinout" => Easing::EaseInOut,
        "hold" => Easing::Hold,
        _ => Easing::Linear,
    };
    let mut keyforms = Vec::new();
    for pair in args[2].split(',') {
        let mut it = pair.split('=');
        if let (Some(ms), Some(v)) = (it.next(), it.next())
            && let (Ok(ms), Ok(v)) = (ms.parse::<u32>(), v.parse::<f64>())
        {
            keyforms.push(Keyform {
                at_ms: ms,
                value: v,
            });
        }
    }
    match Track::new(cycle_ms, keyforms, easing) {
        Ok(track) => {
            match track.check_closure() {
                Ok(()) => println!("  loop closure: PASS (end value == start value)"),
                Err(e) => println!("  loop closure: FAIL ({e:?}) — this loop would pop"),
            }
            println!(
                "  tempo score vs the 1000 ms ideal cycle: {:.0}/100",
                track.tempo_score(1000)
            );
            if let Ok(table) = track.bake(50) {
                let glyphs = " .:-=+*#%@";
                let line: String = table
                    .iter()
                    .map(|v| {
                        let idx = (v.clamp(0.0, 1.0) * (glyphs.len() - 1) as f64).round() as usize;
                        glyphs.chars().nth(idx).unwrap_or(' ')
                    })
                    .collect();
                println!("  bake @50 ms: |{line}|");
            }
        }
        Err(e) => println!("  track rejected: {e:?}"),
    }
}

/// Play Level 1 interactively on stdin/stdout until `menu`/`q`.
fn play_level_1(lines: &mut impl Iterator<Item = std::io::Result<String>>) {
    let cfg = Config::default();
    let level = MilkingLevel::new();
    let host = level.descriptor().host;
    let mut engine = FeedbackEngine::new(host, cfg);
    let mut clock = Instant::now();
    println!(
        "Level 1: {} — hosted by {} ({})",
        level.descriptor().title,
        host.name,
        host.sketch
    );
    println!("Commands: f j F J b c s w + - T t d studio … menu q");
    show_dial(&engine);
    print!("{}", engine.render(&cfg));

    while let Some(Ok(line)) = lines.next() {
        // Fold elapsed real time into fixed ticks (capped), so
        // tempo is graded by the core — mashing Enter spills.
        let elapsed_ms = clock.elapsed().as_millis() as u64;
        clock = Instant::now();
        let elapsed_ticks = (elapsed_ms * u64::from(cfg.tick_hz) / 1000).min(60 * 30);
        let now_tick = engine.snapshot().tick;
        engine.sim_mut().advance_to(now_tick + elapsed_ticks);

        let cmd = line.trim();
        let tick = engine.snapshot().tick;
        match cmd {
            "q" | "menu" => return,
            "f" | "j" | "F" | "J" | "b" => {
                let (hand, pressure) = match cmd {
                    "f" => (Hand::Left, 700),
                    "j" => (Hand::Right, 700),
                    "F" => (Hand::Left, 950),
                    "J" => (Hand::Right, 950),
                    _ => (Hand::Both, 700),
                };
                let _ = engine.feed(InputEvent::Squeeze {
                    tick,
                    hand,
                    pressure,
                });
            }
            "c" | "s" => {
                let action = if cmd == "c" {
                    Action::Caress
                } else {
                    Action::Spank
                };
                let _ = engine.feed(InputEvent::Side { tick, action });
            }
            "w" => engine.sim_mut().advance_to(tick + cfg.ideal_interval_ticks),
            "+" | "-" => {
                let mut p = engine.sim().profile();
                p.gain = (p.gain + if cmd == "+" { 0.15 } else { -0.15 }).clamp(0.1, 4.0);
                engine.sim_mut().set_profile(p);
                show_dial(&engine);
            }
            "T" | "t" => {
                let mut p = engine.sim().profile();
                p.threshold =
                    (p.threshold + if cmd == "T" { -5.0 } else { 5.0 }).clamp(20.0, 100.0);
                engine.sim_mut().set_profile(p);
                show_dial(&engine);
            }
            "d" => show_dial(&engine),
            _ if cmd.starts_with("studio") => {
                let args: Vec<&str> = cmd.split_whitespace().skip(1).collect();
                studio(&args);
            }
            "" => {}
            _ => println!("  unknown command: {cmd}"),
        }
        print!("{}", engine.render(&cfg));
        print!("> ");
        let _ = std::io::stdout().flush();
    }
}

/// FNV-1a 64-bit hash — tiny, deterministic, dependency-free.
/// Loop Studio's twin-run gate compares these over baked
/// frame tables.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Playroom P4 — Loop Studio, playable slice: author a keyform
/// loop for Betsy, bake it twice from the named seed, and pass
/// only if closure holds AND the twin runs are byte-identical.
/// Determinism is the pass gate, mechanically.
fn play_loop_studio(lines: &mut impl Iterator<Item = std::io::Result<String>>) {
    const SEED: &str = "betsy-loop-a1";
    println!("Playroom P4 — Loop Studio slice. Host: Mireille (this slice runs on Betsy).");
    println!("Seed: {SEED} (named, fixed — every run starts from it)");
    println!("Command: run <cycle_ms> <easing> <ms=value,...>   ·   menu/q to leave");
    println!("Pass gate: loop closure AND twin-run byte-identical bake.");
    while let Some(Ok(line)) = lines.next() {
        let cmd = line.trim();
        match cmd {
            "q" | "menu" => return,
            _ if cmd.starts_with("run") => {
                let args: Vec<&str> = cmd.split_whitespace().skip(1).collect();
                if args.len() < 3 {
                    println!("usage: run <cycle_ms> <linear|smooth|easeinout|hold> <ms=value,...>");
                    continue;
                }
                let cycle_ms: u32 = args[0].parse().unwrap_or(0);
                let easing = match args[1] {
                    "smooth" => Easing::Smooth,
                    "easeinout" => Easing::EaseInOut,
                    "hold" => Easing::Hold,
                    _ => Easing::Linear,
                };
                let mut keyforms = Vec::new();
                for pair in args[2].split(',') {
                    let mut it = pair.split('=');
                    if let (Some(ms), Some(v)) = (it.next(), it.next())
                        && let (Ok(ms), Ok(v)) = (ms.parse::<u32>(), v.parse::<f64>())
                    {
                        keyforms.push(Keyform {
                            at_ms: ms,
                            value: v,
                        });
                    }
                }
                let track = match Track::new(cycle_ms, keyforms, easing) {
                    Ok(t) => t,
                    Err(e) => {
                        println!("  track rejected: {e:?}");
                        continue;
                    }
                };
                let closure = track.check_closure();
                println!(
                    "  closure: {}",
                    match &closure {
                        Ok(()) => "PASS".to_string(),
                        Err(e) => format!("FAIL ({e:?})"),
                    }
                );
                // Bake twice from the same seed-state and compare
                // the frame tables byte for byte.
                let bake = |t: &Track| {
                    t.bake(10).map(|table| {
                        let bytes: Vec<u8> = table.iter().flat_map(|v| v.to_le_bytes()).collect();
                        (table, bytes)
                    })
                };
                match (bake(&track), bake(&track)) {
                    (Ok((table, bytes_a)), Ok((_, bytes_b))) => {
                        let hash_a = fnv1a(&bytes_a);
                        let hash_b = fnv1a(&bytes_b);
                        println!("  frame table @10 ms ({} frames):", table.len());
                        for (i, v) in table.iter().enumerate() {
                            if i % 10 == 0 {
                                println!();
                                print!("   ");
                            }
                            print!(" {v:4.2}");
                        }
                        println!();
                        println!("  twin-run hash: run1 {hash_a:016x} · run2 {hash_b:016x}");
                        if closure.is_ok() && bytes_a == bytes_b {
                            println!(
                                "  GATE: PASS — Betsy runs your loop seamlessly, identically, twice. (seed {SEED})"
                            );
                        } else {
                            println!(
                                "  GATE: FAIL — {}",
                                if closure.is_err() {
                                    "the loop does not close"
                                } else {
                                    "twin runs diverged"
                                }
                            );
                        }
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        println!("  bake refused: {e:?} (step must divide the cycle)");
                    }
                }
            }
            "" => {}
            _ => println!("  unknown Loop Studio command: {cmd}"),
        }
        print!("studio> ");
        let _ = std::io::stdout().flush();
    }
}

/// Scripted headless session: proves the framework end-to-end
/// with no terminal — menu, Level 1 on the ideal beat, the dial,
/// a studio closure check, and a final state print.
fn demo() {
    let cfg = Config::default();
    print_menu();
    let level = MilkingLevel::new();
    let mut engine = FeedbackEngine::new(level.descriptor().host, cfg);
    // Betsy on a livelier dial for a compact demo.
    engine.sim_mut().set_profile(SensitivityProfile::high());
    for i in 0..14u64 {
        let hand = if i.is_multiple_of(2) {
            Hand::Left
        } else {
            Hand::Right
        };
        let _ = engine.feed(InputEvent::Squeeze {
            tick: i * cfg.ideal_interval_ticks,
            hand,
            pressure: 700,
        });
    }
    engine.sim_mut().advance_to(14 * cfg.ideal_interval_ticks);
    print!("{}", engine.render(&cfg));
    studio(&["1000", "smooth", "0=0,400=1,1000=0"]);
    let s = engine.snapshot();
    println!(
        "demo ok: tick {} expression {:?} bottle {:.1} climaxes {} spills {}",
        s.tick, s.expression, s.bottle, s.climax_count, s.spills
    );
    // P4 Loop Studio gate, proven headlessly: bake a closed
    // track twice and compare hashes.
    let track = Track::new(
        1000,
        vec![
            Keyform {
                at_ms: 0,
                value: 0.0,
            },
            Keyform {
                at_ms: 400,
                value: 1.0,
            },
            Keyform {
                at_ms: 1000,
                value: 0.0,
            },
        ],
        Easing::Smooth,
    )
    .expect("starter track is valid");
    let bytes = |t: &Track| -> Vec<u8> {
        t.bake(10)
            .expect("10 ms divides 1000 ms")
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect()
    };
    let (a, b) = (bytes(&track), bytes(&track));
    println!(
        "P4 demo: closure {:?} · twin-run hashes {:016x} / {:016x} · identical: {}",
        track.check_closure(),
        fnv1a(&a),
        fnv1a(&b),
        a == b
    );
}

fn main() {
    if std::env::args().any(|a| a == "--demo") {
        demo();
        return;
    }
    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();
    print_menu();
    while let Some(Ok(line)) = lines.next() {
        match line.trim() {
            "q" => break,
            "1" => play_level_1(&mut lines),
            "4" => {
                play_loop_studio(&mut lines);
                print_menu();
            }
            "2" | "3" | "5" | "6" | "7" | "8" | "9" => {
                let id: u8 = line.trim().parse().unwrap_or(0);
                if let Some(l) = playroom_levels().iter().find(|l| l.id == id) {
                    println!(
                        "Playroom P{} ({}) is designed, not built yet — see CURRICULUM.md. Host: {} ({}).",
                        l.id, l.title, l.host.name, l.host.sketch
                    );
                }
                print_menu();
            }
            lesson if lesson.starts_with(['S', 's']) => {
                let id: u8 = lesson[1..].parse().unwrap_or(0);
                if let Some(l) = registry()
                    .iter()
                    .find(|l| l.track == LevelTrack::Study && l.id == id)
                {
                    println!(
                        "Lesson S{} ({}) is codified but locked — designed, not built in this pass. See CURRICULUM.md's Study section. Host: {} ({}).",
                        l.id, l.title, l.host.name, l.host.sketch
                    );
                } else {
                    println!("unknown Lesson: {lesson}");
                }
                print_menu();
            }
            "" => {}
            other => println!("unknown selection: {other}"),
        }
        if line.trim() == "1" {
            print_menu();
        }
    }
}
