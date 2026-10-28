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
//!
//! Menu commands: `save 1..3` / `load 1..3` write and restore
//! the named save slots. Progression, per-host sensitivity
//! settings (the Level 1 dial writes them), the campaign
//! (Continue) state, and the slots all live in the shared
//! store (`hands_on::save`, `hands_on::settings`) — the same
//! files the egui app reads and writes.

#![forbid(unsafe_code)]

use std::io::{BufRead, Write};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use milking_core::affordance::Action;
use milking_core::authoring::{Easing, Keyform, Track};
use milking_core::sim::{Config, Hand, InputEvent, SensitivityProfile};

use hands_on::feedback::FeedbackEngine;
use hands_on::level::{
    Level, LevelDescriptor, MilkingLevel, Track as LevelTrack, playroom_levels, registry,
    study_levels,
};
use hands_on::play::{play_script, playroom_session};
use hands_on::save::{LevelKey, LevelState, Progression, SaveGame, SimPayload, Store};
use hands_on::session::{Session, reward_host, run_scripted};
use hands_on::settings::GameSettings;
use hands_on::study::{study_script, study_session};

/// Wall-clock seconds for save timestamps (the core stays
/// clock-free; the shell supplies this).
fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// The shell's share of the store: settings, progression,
/// and the save context, loaded once at startup and kept
/// current as play happens. With no data directory the game
/// still runs — saves and settings are simply off.
struct Shell {
    store: Option<Store>,
    settings: GameSettings,
    progression: Progression,
    /// Where the player last was: (level, host, summary).
    context: Option<(LevelKey, String, String)>,
    /// The latest P1 sim checkpoint (campaign / P1 exit).
    p1_state: Option<SimPayload>,
    /// A P1 payload from an explicit slot load, restored on
    /// the next Level 1 entry.
    pending_p1: Option<SimPayload>,
}

impl Shell {
    fn load() -> Self {
        let store = Store::default_location();
        let mut shell = Self {
            store,
            settings: GameSettings::defaults(),
            progression: Progression::new(),
            context: None,
            p1_state: None,
            pending_p1: None,
        };
        if let Some(store) = &shell.store {
            match store.load_settings() {
                Ok(settings) => shell.settings = settings,
                Err(e) => println!("settings file refused ({e}) — playing on the tuned defaults"),
            }
            match store.load_campaign() {
                Ok(Some(game)) => {
                    shell.progression = game.progression;
                    if game.level
                        == (LevelKey {
                            track: LevelTrack::Playroom,
                            id: 1,
                        })
                    {
                        shell.p1_state = game.sim;
                    }
                    shell.context = Some((game.level, game.host, game.summary));
                }
                Ok(None) => {}
                Err(e) => println!("campaign file refused ({e}) — starting a fresh campaign"),
            }
        }
        shell
    }

    fn persist_campaign(&self) {
        let Some(store) = &self.store else { return };
        let (level, host, summary) = self.context.clone().unwrap_or((
            LevelKey {
                track: LevelTrack::Playroom,
                id: 1,
            },
            "Opal".to_string(),
            "campaign".to_string(),
        ));
        let sim = if level
            == (LevelKey {
                track: LevelTrack::Playroom,
                id: 1,
            }) {
            self.p1_state.clone()
        } else {
            None
        };
        let game = SaveGame {
            name: "Campaign".to_string(),
            saved_at_unix: now_unix(),
            level,
            host,
            summary,
            progression: self.progression.clone(),
            sim,
        };
        if let Err(e) = store.save_campaign(&game) {
            println!("  (campaign save failed: {e})");
        }
    }

    fn persist_settings(&self) {
        if let Some(store) = &self.store
            && let Err(e) = store.save_settings(&self.settings)
        {
            println!("  (settings save failed: {e})");
        }
    }

    /// Record a level completion: progression, context, and
    /// the campaign file move together.
    fn complete_level(&mut self, key: LevelKey, host: &str, summary: String) {
        let fresh = self.progression.mark_completed(key);
        self.context = Some((key, host.to_string(), summary));
        self.persist_campaign();
        if fresh {
            println!(
                "— {} complete: {}/17 in your progression —",
                key.code(),
                self.progression.count()
            );
        }
    }

    /// Checkpoint the P1 session (on the win edge and on
    /// leaving the parlour): her sim state joins the
    /// campaign so Continue resumes her, not a fresh sim.
    fn checkpoint_p1(&mut self, engine: &FeedbackEngine, won: bool) {
        let snap = engine.snapshot();
        self.p1_state = Some(SimPayload::capture(engine.sim()));
        let summary = format!(
            "bottle {:.1} · climaxes {} · {}",
            snap.bottle,
            snap.climax_count,
            if won { "session won" } else { "in progress" }
        );
        let key = LevelKey {
            track: LevelTrack::Playroom,
            id: 1,
        };
        if won {
            self.complete_level(key, engine.host().name, summary);
        } else {
            self.context = Some((key, engine.host().name.to_string(), summary));
            self.persist_campaign();
        }
    }
}

fn state_tag(progression: Option<&Progression>, l: &LevelDescriptor) -> String {
    match progression {
        None => {
            if l.built {
                "playable".to_string()
            } else {
                "designed".to_string()
            }
        }
        Some(p) => match p.state_of(l) {
            LevelState::Completed => "complete ✓".to_string(),
            LevelState::Available => {
                if l.built {
                    "playable".to_string()
                } else {
                    "designed".to_string()
                }
            }
            LevelState::Locked => {
                let pre = p.prerequisite(l).map(|k| k.code()).unwrap_or_default();
                format!("locked — complete {pre} first")
            }
        },
    }
}

fn print_menu(progression: Option<&Progression>) {
    println!("=== hands-on — learn the systems by playing them ===");
    println!("-- The Playroom (game systems + deterministic animation, taught by playing) --");
    for l in playroom_levels() {
        println!(
            "  {}. {}  [{}]  host: {} — {}",
            l.id,
            l.title,
            state_tag(progression, &l),
            l.host.name,
            l.source_system
        );
    }
    println!("-- The Study (Rust mastery lessons — the compiler corrects you at the desk) --");
    for l in study_levels() {
        println!(
            "  S{}. Lesson {}: {}  [{}]  host: {} — {}",
            l.id,
            l.id,
            l.title,
            state_tag(progression, &l),
            l.host.name,
            l.source_system
        );
    }
    println!(
        "Pick a Playroom number (1..9), a Study lesson (S1..S8), save <1..3>, load <1..3>, or q."
    );
}

/// Run one session-layer level interactively: the session
/// owns the exercise, the runner owns the host. On success
/// the payout lands through the shared feedback engine
/// (SPEC §6.3) — her climax is the reward instrument.
fn run_session(
    lines: &mut impl Iterator<Item = std::io::Result<String>>,
    mut session: Box<dyn Session>,
    shell: &mut Shell,
) {
    let cfg = Config::default();
    let desc = *session.descriptor();
    let mut engine = FeedbackEngine::new(desc.host, cfg);
    engine
        .sim_mut()
        .set_profile(shell.settings.effective_profile(&desc.host));
    println!("{}", session.intro());
    print!("{}", engine.render(&cfg));
    while let Some(Ok(line)) = lines.next() {
        let cmd = line.trim();
        if cmd == "q" || cmd == "menu" {
            return;
        }
        if cmd.is_empty() {
            continue;
        }
        let step = session.handle(cmd);
        if !step.text.is_empty() {
            println!("{}", step.text);
        }
        if step.done {
            if step.success {
                let gained = reward_host(&mut engine, &cfg);
                // TUI cue marker: the app shell plays the
                // climax cue for this same event; the terminal
                // names it so both shells speak one grammar.
                println!("♪ cue: climax");
                println!(
                    "— payout: {} responds through the feedback engine (climaxes +{gained}) —",
                    desc.host.name
                );
                print!("{}", engine.render(&cfg));
                shell.complete_level(
                    LevelKey::of(&desc),
                    desc.host.name,
                    format!("session complete · payout +{gained}"),
                );
            } else {
                println!("♪ cue: failure");
            }
            return;
        }
        print!("> ");
        let _ = std::io::stdout().flush();
    }
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
fn play_level_1(lines: &mut impl Iterator<Item = std::io::Result<String>>, shell: &mut Shell) {
    let cfg = Config::default();
    let level = MilkingLevel::new();
    let host = level.descriptor().host;
    let mut engine = FeedbackEngine::new(host, cfg);
    // Her profile comes from the shared settings store —
    // the same values the app Settings screen edits, and
    // the dial below writes back to.
    engine
        .sim_mut()
        .set_profile(shell.settings.effective_profile(&host));
    // Resume: an explicit slot load outranks the campaign
    // checkpoint; either restores her sim exactly.
    if let Some(payload) = shell.pending_p1.take().or_else(|| shell.p1_state.clone()) {
        match payload.restore(cfg) {
            Ok(sim) => {
                *engine.sim_mut() = sim;
                println!("— her saved session resumes where you left it —");
            }
            Err(e) => println!("— saved session refused ({e}); starting fresh —"),
        }
    }
    let mut recorded = engine.snapshot().won;
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
            "q" | "menu" => break,
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
                // The dial writes the shared settings store
                // (clamped there); the engine re-derives its
                // profile from it. One source of truth.
                let mut t = shell.settings.tuning_for(host.name);
                t.gain += if cmd == "+" { 0.15 } else { -0.15 };
                shell.settings.set_tuning(host.name, t);
                shell.persist_settings();
                engine
                    .sim_mut()
                    .set_profile(shell.settings.effective_profile(&host));
                show_dial(&engine);
            }
            "T" | "t" => {
                let mut t = shell.settings.tuning_for(host.name);
                t.threshold += if cmd == "T" { -5.0 } else { 5.0 };
                shell.settings.set_tuning(host.name, t);
                shell.persist_settings();
                engine
                    .sim_mut()
                    .set_profile(shell.settings.effective_profile(&host));
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
        if !recorded && engine.snapshot().won {
            recorded = true;
            shell.checkpoint_p1(&engine, true);
        }
        print!("> ");
        let _ = std::io::stdout().flush();
    }
    // Leaving the parlour checkpoints her session into the
    // campaign — Continue resumes this exact sim.
    shell.checkpoint_p1(&engine, recorded);
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
/// loop for Opal, bake it twice from the named seed, and pass
/// only if closure holds AND the twin runs are byte-identical.
/// Determinism is the pass gate, mechanically.
fn play_loop_studio(lines: &mut impl Iterator<Item = std::io::Result<String>>, shell: &mut Shell) {
    const SEED: &str = "opal-loop-a1";
    let p4 = playroom_levels()[3];
    let mut recorded = false;
    println!("Playroom P4 — Loop Studio slice. Host: Mireille (this slice runs on Opal).");
    println!("Seed: {SEED} (named, fixed — every run starts from it)");
    println!("Command: run <cycle_ms> <easing> <ms=value,...>   ·   menu/q to leave");
    println!(
        "Pass gate: loop closure AND twin-run byte-identical bake. Opal's terms \
         (Daddy Style): the gate is named before the bake, it does not move, and \
         its verdict is a consequence of the loop you authored — never a mood."
    );
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
                                "  GATE: PASS — Opal runs your loop seamlessly, identically, twice. \
                                 Praise, exact and earned: the loop kept the terms, so the terms keep the loop. (seed {SEED})"
                            );
                            if !recorded {
                                recorded = true;
                                shell.complete_level(
                                    LevelKey::of(&p4),
                                    p4.host.name,
                                    "loop studio gate passed".to_string(),
                                );
                            }
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
    print_menu(None);
    let level = MilkingLevel::new();
    let mut engine = FeedbackEngine::new(level.descriptor().host, cfg);
    // Opal on a livelier dial for a compact demo.
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
    // The session layer, proven headlessly: one scripted
    // playthrough per wrapped level, each reaching its
    // success state, each paying its host out through the
    // feedback engine.
    let mut passed = 0u32;
    for id in [2u8, 3, 5, 6, 7, 8, 9] {
        let mut session = playroom_session(id).expect("playroom session exists");
        let (success, _) = run_scripted(session.as_mut(), play_script(id));
        let desc = *session.descriptor();
        let mut engine = FeedbackEngine::new(desc.host, cfg);
        let gained = if success {
            reward_host(&mut engine, &cfg)
        } else {
            0
        };
        println!(
            "P{id} demo: scripted route success={success} · host {} climax payout +{gained} · ♪ cue: climax",
            desc.host.name
        );
        assert!(success, "P{id} scripted route must succeed in the demo");
        assert_eq!(gained, 1, "P{id} host payout must land");
        passed += 1;
    }
    for id in 1..=8u8 {
        let mut session = study_session(id).expect("study session exists");
        let (success, _) = run_scripted(session.as_mut(), study_script(id));
        let desc = *session.descriptor();
        let mut engine = FeedbackEngine::new(desc.host, cfg);
        let gained = if success {
            reward_host(&mut engine, &cfg)
        } else {
            0
        };
        println!(
            "S{id} demo: scripted route success={success} · host {} climax payout +{gained} · ♪ cue: climax",
            desc.host.name
        );
        assert!(success, "S{id} scripted route must succeed in the demo");
        assert_eq!(gained, 1, "S{id} host payout must land");
        passed += 1;
    }
    println!("session demo: {passed}/15 wrapped levels pass their scripted routes");
}

/// `save <slot>` — write the current campaign state (last
/// level context + progression + the P1 checkpoint) into a
/// named slot.
fn cmd_save(shell: &Shell, slot: u8) {
    let Some(store) = &shell.store else {
        println!("saves are disabled this run (no app-data directory)");
        return;
    };
    let (level, host, summary) = shell.context.clone().unwrap_or((
        LevelKey {
            track: LevelTrack::Playroom,
            id: 1,
        },
        "Opal".to_string(),
        "no level played yet".to_string(),
    ));
    let sim = if level
        == (LevelKey {
            track: LevelTrack::Playroom,
            id: 1,
        }) {
        shell.p1_state.clone()
    } else {
        None
    };
    let game = SaveGame {
        name: format!("Slot {slot}"),
        saved_at_unix: now_unix(),
        level,
        host,
        summary,
        progression: shell.progression.clone(),
        sim,
    };
    match store.save_slot(slot, &game) {
        Ok(()) => println!("saved to slot {slot}: {}", game.describe()),
        Err(e) => println!("save refused: {e}"),
    }
}

/// `load <slot>` — restore a slot as the live campaign:
/// progression, context, and (for P1) her saved sim, which
/// the next Level 1 entry resumes.
fn cmd_load(shell: &mut Shell, slot: u8) {
    let Some(store) = &shell.store else {
        println!("saves are disabled this run (no app-data directory)");
        return;
    };
    match store.load_slot(slot) {
        Ok(Some(game)) => {
            println!("loaded slot {slot}: {}", game.describe());
            shell.progression = game.progression.clone();
            shell.context = Some((game.level, game.host.clone(), game.summary.clone()));
            if game.level
                == (LevelKey {
                    track: LevelTrack::Playroom,
                    id: 1,
                })
            {
                shell.p1_state = game.sim.clone();
                shell.pending_p1 = game.sim;
                if shell.pending_p1.is_some() {
                    println!("Level 1 will resume her saved session.");
                }
            } else {
                shell.pending_p1 = None;
            }
            shell.persist_campaign();
        }
        Ok(None) => println!("slot {slot} is empty"),
        Err(e) => println!("load refused: {e}"),
    }
}

fn parse_slot(cmd: &str, verb: &str) -> Option<u8> {
    let mut parts = cmd.split_whitespace();
    let _ = parts.next();
    match parts.next().and_then(|p| p.parse::<u8>().ok()) {
        Some(slot) => Some(slot),
        None => {
            println!("usage: {verb} <1..3>");
            None
        }
    }
}

/// A menu selection as a level key: `1`..`9` for the
/// Playroom, `S1`..`S8` for the Study.
fn parse_selection(cmd: &str) -> Option<LevelKey> {
    if let Some(rest) = cmd.strip_prefix(['S', 's']) {
        return rest.parse::<u8>().ok().map(|id| LevelKey {
            track: LevelTrack::Study,
            id,
        });
    }
    cmd.parse::<u8>().ok().map(|id| LevelKey {
        track: LevelTrack::Playroom,
        id,
    })
}

fn main() {
    if std::env::args().any(|a| a == "--demo") {
        demo();
        return;
    }
    let mut shell = Shell::load();
    match &shell.store {
        Some(store) => println!("saves + settings: {}", store.root().display()),
        None => println!("no app-data directory found — saves and settings are off this run"),
    }
    if shell.progression.count() > 0 {
        println!(
            "welcome back — {}/17 levels complete; Level 1 resumes her last session",
            shell.progression.count()
        );
    }
    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();
    print_menu(Some(&shell.progression));
    while let Some(Ok(line)) = lines.next() {
        let cmd = line.trim();
        match cmd {
            "q" => break,
            "" => continue,
            "menu" => {}
            _ if cmd.starts_with("save") => {
                if let Some(slot) = parse_slot(cmd, "save") {
                    cmd_save(&shell, slot);
                }
                continue;
            }
            _ if cmd.starts_with("load") => {
                if let Some(slot) = parse_slot(cmd, "load") {
                    cmd_load(&mut shell, slot);
                }
                continue;
            }
            _ => {
                let Some(key) = parse_selection(cmd) else {
                    println!("unknown selection: {cmd}");
                    continue;
                };
                let Some(desc) = registry()
                    .into_iter()
                    .find(|l| l.track == key.track && l.id == key.id)
                else {
                    println!("unknown selection: {cmd}");
                    continue;
                };
                if shell.progression.state_of(&desc) == LevelState::Locked {
                    let pre = shell
                        .progression
                        .prerequisite(&desc)
                        .map(|k| k.code())
                        .unwrap_or_default();
                    println!("{} is locked — complete {pre} first.", key.code());
                    continue;
                }
                match (key.track, key.id) {
                    (LevelTrack::Playroom, 1) => play_level_1(&mut lines, &mut shell),
                    (LevelTrack::Playroom, 4) => play_loop_studio(&mut lines, &mut shell),
                    (LevelTrack::Playroom, id) => {
                        if let Some(session) = playroom_session(id) {
                            run_session(&mut lines, session, &mut shell);
                        }
                    }
                    (LevelTrack::Study, id) => {
                        if let Some(session) = study_session(id) {
                            run_session(&mut lines, session, &mut shell);
                        } else {
                            println!("Lesson S{id} is designed, not built — see CURRICULUM.md.");
                        }
                    }
                }
            }
        }
        print_menu(Some(&shell.progression));
    }
}
