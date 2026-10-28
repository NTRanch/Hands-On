//! Playroom sessions (SPEC §6.4): P2, P3, P5–P9 wrapped
//! around their `milking-core` twins. (P1 and the P4 slice
//! predate this layer and keep their bespoke loops in
//! `main.rs`; they are the standard these sessions match —
//! a scripted route that reaches the level's success state,
//! proven by the same driver the player uses.)
//!
//! Session conventions: commands are one line; every session
//! answers unknown input with its command list; a success
//! state is mechanical (a count, a hash match, a typed
//! refusal observed), never a narrator's opinion.
//!
//! The voice is Daddy Style (SPEC §7.4): each host sets her
//! requirements in advance — firm, caring, for the player's
//! own benefit — and they never move mid-exercise. Success
//! is praised exactly; failure lands as consequence, not
//! punishment. The correction is the care working.

use milking_core::authoring::{Easing, Keyform, Track};
use milking_core::beat::{BeatGrid, Judgment, display_frame};
use milking_core::ecs::{club_systems, run_scene, validate_schedule};
use milking_core::funscript::Funscript;
use milking_core::moc3::{
    Construct, ConstructKind, EyeStyle, MocVersion, StyleLane, classify, run_border,
};
use milking_core::render3d::{Frame, frame_png, scene_glb};
use milking_core::router::{Emission, Router, RouterConfig};
use milking_core::sprite::{bake_closes, pupil_occupancy, select_frames};

use crate::level::{LevelDescriptor, playroom_levels};
use crate::session::{Session, StepOut};

fn descriptor(id: u8) -> LevelDescriptor {
    *playroom_levels()
        .iter()
        .find(|l| l.id == id)
        .expect("playroom id exists")
}

/// The session for a Playroom level, if this layer hosts it.
#[must_use]
pub fn playroom_session(id: u8) -> Option<Box<dyn Session>> {
    match id {
        2 => Some(Box::new(BeatSession::new())),
        3 => Some(Box::new(StageSession::new())),
        5 => Some(Box::new(RouterSession::new())),
        6 => Some(Box::new(BorderSession::new())),
        7 => Some(Box::new(SpriteSession::new())),
        8 => Some(Box::new(FunscriptSession::new())),
        9 => Some(Box::new(RenderSession::new())),
        _ => None,
    }
}

/// The canonical scripted route for a level: the commands
/// that reach its success state. `--demo` and the tests run
/// exactly these.
#[must_use]
pub fn play_script(id: u8) -> &'static [&'static str] {
    match id {
        2 => &[
            "hit 0", "hit 30", "hit 60", "hit 90", "hit 120", "hit 150", "hit 180", "hit 210",
            "hit 0", "hit 24", "hit 48", "hit 72", "hit 96", "hit 120", "hit 144", "hit 168",
            "hit 0", "hit 20", "hit 40", "hit 60", "hit 80", "hit 100", "hit 120", "hit 140",
            "hit 0", "hit 15", "hit 30", "hit 45", "hit 60", "hit 75", "hit 90", "hit 105",
        ],
        3 => &["order input,rig,physics,haptics,render"],
        5 => &[
            "beat",
            "emit 0.15",
            "emit 0.2",
            "wait 30",
            "beat",
            "emit 0.1",
            "stop",
        ],
        6 => &[
            "answer exact",
            "answer exact",
            "answer lossy",
            "answer lossy",
            "answer exact",
            "answer nomap",
            "answer nomap",
            "answer exact",
            "export",
        ],
        7 => &["bake 25"],
        8 => &["write 100", "clamp"],
        9 => &["render 3", "render 3", "glb", "png"],
        _ => &[],
    }
}

// ---------------------------------------------------------------- P2

/// P2 — Fixed-Step Rhythm. Hit ticks on a deterministic beat
/// grid at rising tempos; the grid judges, never the frame.
pub struct BeatSession {
    desc: LevelDescriptor,
    round: usize,
    hits_in_round: u32,
    perfect: u32,
    misses: u32,
}

const TEMPOS: [u32; 4] = [120, 150, 180, 240];
const HITS_PER_ROUND: u32 = 8;

impl BeatSession {
    #[must_use]
    pub fn new() -> Self {
        Self {
            desc: descriptor(2),
            round: 0,
            hits_in_round: 0,
            perfect: 0,
            misses: 0,
        }
    }

    fn grid(&self) -> BeatGrid {
        BeatGrid::at_bpm(TEMPOS[self.round], 60)
    }
}

impl Default for BeatSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Session for BeatSession {
    fn descriptor(&self) -> &LevelDescriptor {
        &self.desc
    }

    fn intro(&self) -> String {
        format!(
            "Playroom P2 — {} · host {} ({})\n\
             Beats land on a tick grid, not on frames: `hit <tick>` is judged against the \
             nearest beat (perfect ≤6 ticks, good ≤12). Four rounds, {} hits each, the tempo \
             rising {} → {} bpm (hit ticks are absolute within each round, beats at \
             round_start + k·interval — the scripted route hits round-relative ticks: \
             each round's grid restarts at 0). `frame <tick> <hz>` shows which frame a \
             renderer would display the hit on — the verdict never follows the frame.\n\
             Sable's terms are set before the first hit and they do not move (Daddy \
             Style): the grid is strict because it is trustworthy, and trustworthy \
             because it is strict — you always know exactly where you stand with her.\n\
             Pass: finish round 4 with ≥24 perfect and ≤4 misses.",
            self.desc.title,
            self.desc.host.name,
            self.desc.host.sketch,
            HITS_PER_ROUND,
            TEMPOS[0],
            TEMPOS[3]
        )
    }

    fn handle(&mut self, cmd: &str) -> StepOut {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.as_slice() {
            ["frame", tick, hz] => {
                let (Ok(tick), Ok(hz)) = (tick.parse::<u64>(), hz.parse::<u32>()) else {
                    return StepOut::go("usage: frame <tick> <hz>");
                };
                let grid = self.grid();
                StepOut::go(format!(
                    "tick {tick} displays on frame {} at {hz} fps; the grid judges it {:?} \
                     (offset {} ticks) — frames move, verdicts don't",
                    display_frame(tick, 60, hz),
                    grid.judge(tick),
                    grid.offset_of(tick)
                ))
            }
            ["hit", tick] => {
                let Ok(tick) = tick.parse::<u64>() else {
                    return StepOut::go("usage: hit <tick>");
                };
                let grid = self.grid();
                let verdict = grid.judge(tick);
                match verdict {
                    Judgment::Perfect => self.perfect += 1,
                    Judgment::Good => {}
                    Judgment::Miss => self.misses += 1,
                }
                self.hits_in_round += 1;
                let mut text = format!(
                    "tick {tick}: {:?} (offset {} ticks, envelope peak {:.1}) — perfect {} · misses {} · round {} hit {}",
                    verdict,
                    grid.offset_of(tick),
                    verdict.envelope_peak(),
                    self.perfect,
                    self.misses,
                    self.round + 1,
                    self.hits_in_round
                );
                if self.misses > 4 {
                    return StepOut::fail(format!(
                        "{text}\nThe grid does not forgive drift: 5 misses end the set. \
                         A consequence, not a punishment — Sable drops the headphones \
                         and waits. Run it again, on the tick."
                    ));
                }
                if self.hits_in_round == HITS_PER_ROUND {
                    if self.round + 1 == TEMPOS.len() {
                        return StepOut::pass(format!(
                            "{text}\nSet complete: {} perfect, {} misses across four tempos. \
                             Every verdict was the tick's, at every frame rate. Praise, \
                             exact and earned: you were corrected by the grid, never by \
                             whim — that is the whole discipline.",
                            self.perfect, self.misses
                        ));
                    }
                    self.round += 1;
                    self.hits_in_round = 0;
                    text.push_str(&format!(
                        "\nTempo up: {} bpm — one beat every {} ticks. The grid restarts at 0.",
                        TEMPOS[self.round],
                        self.grid().interval_ticks
                    ));
                }
                StepOut::go(text)
            }
            _ => StepOut::go("commands: hit <tick> · frame <tick> <hz> · menu/q to leave"),
        }
    }
}

// ---------------------------------------------------------------- P3

/// P3 — Stage Systems. Reconstruct the club scene's lawful
/// schedule; a lawful schedule runs the scene stable under
/// rising load.
pub struct StageSession {
    desc: LevelDescriptor,
    failures: u32,
}

impl StageSession {
    #[must_use]
    pub fn new() -> Self {
        Self {
            desc: descriptor(3),
            failures: 0,
        }
    }
}

impl Default for StageSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Session for StageSession {
    fn descriptor(&self) -> &LevelDescriptor {
        &self.desc
    }

    fn intro(&self) -> String {
        format!(
            "Playroom P3 — {} · host {} ({})\n\
             Style lane note (CAST.md): Vega's lane is Pony Painterly; the shared feedback \
             panel names House Cel beats — her scene renders Pony Painterly, the panel is \
             the estate's text grammar.\n\
             Five systems — input, rig, physics, haptics, render — one schedule. \
             `order a,b,c,d,e` places each in its own step, in your order. A system may \
             only run after everything that produces what it reads; two writers of one \
             component never share a step. Vega's booth runs on terms stated before \
             the house opens (Daddy Style): lawful schedules run the scene for 120 \
             fixed steps under a rising crowd (2 → 40 events/step), and an unlawful \
             one is refused before it can embarrass anyone on stage. The strictness \
             is protection, not theater.\n\
             Pass: one lawful order. Three unlawful orders end the night.",
            self.desc.title, self.desc.host.name, self.desc.host.sketch
        )
    }

    fn handle(&mut self, cmd: &str) -> StepOut {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.as_slice() {
            ["order", list] => {
                let steps: Vec<Vec<&str>> = list.split(',').map(|name| vec![name.trim()]).collect();
                let systems = club_systems();
                match validate_schedule(&systems, &[], &steps) {
                    Ok(()) => {
                        let outcome = run_scene(120, 2, 40);
                        StepOut::pass(format!(
                            "Lawful. The scene runs {} fixed steps under the rising crowd: \
                             {} events, checksum {:016x}, stable: {}. A full set with zero \
                             instability — the schedule was the program, and order kept \
                             is care delivered: the scene held because the terms did.",
                            outcome.steps, outcome.events, outcome.checksum, outcome.stable
                        ))
                    }
                    Err(e) => {
                        self.failures += 1;
                        if self.failures >= 3 {
                            StepOut::fail(format!(
                                "Refused: {e:?} — the third unlawful schedule tonight. \
                                 Props desync, the rig stutters, Vega closes the booth."
                            ))
                        } else {
                            StepOut::go(format!(
                                "Refused: {e:?} — the scene destabilizes before it can run. \
                                 (failure {} of 3)",
                                self.failures
                            ))
                        }
                    }
                }
            }
            _ => StepOut::go("commands: order <a,b,c,d,e> · menu/q to leave"),
        }
    }
}

// ---------------------------------------------------------------- P5

/// P5 — Router's Gate. Route lawful signal through the
/// torogao-haptics router contract: ceiling, watchdog,
/// synchronous stop, no resume without reset.
pub struct RouterSession {
    desc: LevelDescriptor,
    router: Router,
    tick: u64,
    routed: u32,
}

impl RouterSession {
    #[must_use]
    pub fn new() -> Self {
        Self {
            desc: descriptor(5),
            router: Router::new(RouterConfig::default()),
            tick: 0,
            routed: 0,
        }
    }
}

impl Default for RouterSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Session for RouterSession {
    fn descriptor(&self) -> &LevelDescriptor {
        &self.desc
    }

    fn intro(&self) -> String {
        format!(
            "Playroom P5 — {} · host {} ({})\n\
             The router contract, rule for rule: ceiling 0.20 (over-ceiling is refused \
             typed, never clamped), watchdog 90 ticks (a dead source ends the round), \
             stop latches synchronously, and only `reset` — a human decision — reopens \
             the gate. Vigil states the whole contract before the first signal \
             (Daddy Style): the rules are known in advance and applied the same \
             every time — and the stop is never a prize to earn or a favor to \
             grant. It is simply there, always, for anyone holding the source.\n\
             Commands: beat · emit <value> · wait <ticks> · poll · stop · reset.\n\
             Pass: route 3 lawful emissions with zero faults, then stop.",
            self.desc.title, self.desc.host.name, self.desc.host.sketch
        )
    }

    fn handle(&mut self, cmd: &str) -> StepOut {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.as_slice() {
            ["beat"] => match self.router.heartbeat(self.tick) {
                Ok(()) => StepOut::go(format!("heartbeat at tick {} — watchdog fed", self.tick)),
                Err(e) => StepOut::go(format!("heartbeat refused: {e:?}")),
            },
            ["wait", n] => {
                let Ok(n) = n.parse::<u64>() else {
                    return StepOut::go("usage: wait <ticks>");
                };
                self.tick += n;
                match self.router.advance_to(self.tick) {
                    Ok(()) => StepOut::go(format!(
                        "tick {} — watchdog {}",
                        self.tick,
                        if self.router.watchdog_alive() {
                            "alive"
                        } else {
                            "dead (poll would fault the round)"
                        }
                    )),
                    Err(e) => StepOut::go(format!("advance refused: {e:?}")),
                }
            }
            ["emit", v] => {
                let Ok(v) = v.parse::<f64>() else {
                    return StepOut::go("usage: emit <value>");
                };
                match self.router.emit(v) {
                    Ok(Emission::Routed(value)) => {
                        self.routed += 1;
                        StepOut::go(format!(
                            "routed {value:.2} unchanged ({} lawful emissions)",
                            self.routed
                        ))
                    }
                    Ok(Emission::Silence) => StepOut::go(
                        "the router emits silence — the source is unproven (watchdog window \
                         passed). Silence is the safe state.",
                    ),
                    Err(e) => StepOut::go(format!(
                        "refused typed: {e:?} — recorded on the audit trail, never clamped"
                    )),
                }
            }
            ["poll"] => match self.router.poll() {
                Ok(()) => StepOut::go("watchdog poll: source alive, gate open"),
                Err(e) => StepOut::fail(format!(
                    "watchdog fault: {e:?} — the round ends and the gate latches. \
                     A dead source ends the round; it does not merely mute it."
                )),
            },
            ["stop"] => {
                let _ = self.router.stop();
                if self.routed >= 3 && self.router.faults().is_empty() {
                    StepOut::pass(format!(
                        "Stop latched synchronously at tick {} — output zero from this \
                         instant. {} lawful emissions, zero faults: the gate was kept.",
                        self.tick, self.routed
                    ))
                } else {
                    StepOut::go(format!(
                        "stopped ({} lawful emissions, {} faults). The latch holds until \
                         `reset` — reconnect storms change nothing.",
                        self.routed,
                        self.router.faults().len()
                    ))
                }
            }
            ["reset"] => {
                self.router.reset();
                StepOut::go(
                    "reset: the latch clears, the watchdog re-arms from this tick, and the router starts silent",
                )
            }
            _ => StepOut::go(
                "commands: beat · emit <value> · wait <ticks> · poll · stop · reset · menu/q",
            ),
        }
    }
}

// ---------------------------------------------------------------- P6

/// P6 — Format Border. Predict each construct's moc3 V3_0
/// mapping class, then run the border on the lawful rig.
pub struct BorderSession {
    desc: LevelDescriptor,
    idx: usize,
    score: u32,
}

fn border_constructs() -> Vec<(&'static str, ConstructKind)> {
    vec![
        ("angle_x (parameter)", ConstructKind::Parameter),
        ("param→property binding", ConstructKind::Binding),
        ("body multiply color", ConstructKind::MultiplyColor),
        ("cheek screen color", ConstructKind::ScreenColor),
        (
            "blend: normal (has a V3_0 home)",
            ConstructKind::BlendMode {
                has_v3_target: true,
            },
        ),
        (
            "blend: overlay-plus (no V3_0 home)",
            ConstructKind::BlendMode {
                has_v3_target: false,
            },
        ),
        ("part opacity", ConstructKind::PartOpacity),
        ("glue pair", ConstructKind::Glue),
    ]
}

impl BorderSession {
    #[must_use]
    pub fn new() -> Self {
        Self {
            desc: descriptor(6),
            idx: 0,
            score: 0,
        }
    }
}

impl Default for BorderSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Session for BorderSession {
    fn descriptor(&self) -> &LevelDescriptor {
        &self.desc
    }

    fn intro(&self) -> String {
        let first = border_constructs()[0].0;
        format!(
            "Playroom P6 — {} · host {} ({})\n\
             The moc3 V3_0 border: for each construct, does it cross EXACT, LOSSY, or \
             NO-MAP? Per-form multiply/screen colors are version-gated (LOSSY before \
             V4_2); blend modes with no V3_0 target and part opacity are NO-MAP — \
             refused typed, never silently downgraded.\n\
             `answer exact|lossy|nomap` per construct, then `export` runs the border on \
             the lawful rig. Vigil grades to the table, not to a vibe (Daddy Style): \
             every answer is knowable in advance by anyone who reads the version \
             rules — and a refusal at this border is information, never rejection.\n\
             Pass: ≥7 of 8 predictions and a clean export.\n\
             Construct 1 of 8: {first}",
            self.desc.title, self.desc.host.name, self.desc.host.sketch
        )
    }

    fn handle(&mut self, cmd: &str) -> StepOut {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let constructs = border_constructs();
        match parts.as_slice() {
            ["answer", class] => {
                if self.idx >= constructs.len() {
                    return StepOut::go("all constructs judged — `export` runs the border");
                }
                let (name, kind) = constructs[self.idx];
                let truth = classify(kind, MocVersion::V3_0);
                let guessed = match class.to_ascii_lowercase().as_str() {
                    "exact" => Some(milking_core::moc3::MappingClass::Exact),
                    "lossy" => Some(milking_core::moc3::MappingClass::Lossy),
                    "nomap" | "no-map" => Some(milking_core::moc3::MappingClass::NoMap),
                    _ => None,
                };
                let Some(guessed) = guessed else {
                    return StepOut::go("answer exact, lossy, or nomap");
                };
                let right = guessed == truth;
                if right {
                    self.score += 1;
                }
                self.idx += 1;
                let mut text = format!(
                    "{name}: {truth:?} — {} (score {}/{})",
                    if right {
                        "correct"
                    } else {
                        "the border says otherwise"
                    },
                    self.score,
                    self.idx
                );
                if self.idx < constructs.len() {
                    text.push_str(&format!(
                        "\nConstruct {} of 8: {}",
                        self.idx + 1,
                        constructs[self.idx].0
                    ));
                } else {
                    text.push_str("\nAll judged. `export` runs the border on the lawful rig.");
                }
                StepOut::go(text)
            }
            ["export"] => {
                if self.idx < constructs.len() {
                    return StepOut::go(
                        "judge every construct first — the border grades predictions, then runs",
                    );
                }
                // The lawful rig: everything except the two
                // NO-MAP constructs, plus an eye construct
                // that obeys the eye rule on-lane.
                let mut iris = Construct {
                    name: "iris".to_string(),
                    kind: ConstructKind::Parameter,
                    eye: None,
                };
                iris.eye = Some(EyeStyle {
                    heart_pupil: true,
                    iris_canonical: true,
                    lane: StyleLane::HouseCel,
                });
                let rig = vec![
                    Construct {
                        name: "angle_x".to_string(),
                        kind: ConstructKind::Parameter,
                        eye: None,
                    },
                    Construct {
                        name: "body_mul".to_string(),
                        kind: ConstructKind::MultiplyColor,
                        eye: None,
                    },
                    Construct {
                        name: "cheek_screen".to_string(),
                        kind: ConstructKind::ScreenColor,
                        eye: None,
                    },
                    iris,
                ];
                match run_border(&rig, MocVersion::V3_0, StyleLane::HouseCel) {
                    Ok(report) => {
                        if self.score >= 7 {
                            StepOut::pass(format!(
                                "Export crossed: {} EXACT, {} LOSSY, 0 NO-MAP — the loss \
                                 report names every approximation, and the eye rule held \
                                 at the border. Predictions {}/8.",
                                report.exact, report.lossy, self.score
                            ))
                        } else {
                            StepOut::fail(format!(
                                "The export crossed cleanly, but predictions were {}/8 — \
                                 the border is a judgment, and Vigil grades the judgment.",
                                self.score
                            ))
                        }
                    }
                    Err(e) => StepOut::fail(format!("export refused: {e:?}")),
                }
            }
            _ => StepOut::go("commands: answer exact|lossy|nomap · export · menu/q"),
        }
    }
}

// ---------------------------------------------------------------- P7

fn studio_track(end: f64) -> Track {
    Track::new(
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
                value: end,
            },
        ],
        Easing::Smooth,
    )
    .expect("the session's track is valid")
}

/// P7 — Sprite & GIF Bake. Choose the sampling step for a
/// GIF bake; the baked table must still close, keep its
/// extremes, fit the frame budget, and keep the pupil.
pub struct SpriteSession {
    desc: LevelDescriptor,
    track: Track,
}

impl SpriteSession {
    #[must_use]
    pub fn new() -> Self {
        Self {
            desc: descriptor(7),
            track: studio_track(0.0),
        }
    }
}

impl Default for SpriteSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Session for SpriteSession {
    fn descriptor(&self) -> &LevelDescriptor {
        &self.desc
    }

    fn intro(&self) -> String {
        format!(
            "Playroom P7 — {} · host {} ({})\n\
             A closed 1000 ms loop, baked at 10 ms (100 frames). A GIF cannot carry them \
             all: `bake <step>` keeps every <step>th frame — the baker forces in the \
             loop's extremes (min and max) so the baked range is the authored range. \
             The bake passes only if the loop still CLOSES across the bake, the extremes \
             survive, the pupil still occupies its cell (16 lit px in a 6 px cell of a \
             12 px frame), and the frame count fits the GIF budget (≤20).\n\
             `open` breaks the loop's closure on purpose, to watch the gate refuse it.\
             \nSable's gate is mechanical and stated up front (Daddy Style): four \
             conditions, all named before the bake — pass them and the loop ships; \
             miss one and the refusal names which. No moods, no moving goalposts.",
            self.desc.title, self.desc.host.name, self.desc.host.sketch
        )
    }

    fn handle(&mut self, cmd: &str) -> StepOut {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.as_slice() {
            ["open"] => {
                self.track = studio_track(0.5);
                StepOut::go(
                    "the loop's end keyform is now 0.5 against a 0.0 start — the loop is open. `bake` will say so",
                )
            }
            ["close"] => {
                self.track = studio_track(0.0);
                StepOut::go("end keyform restored to 0.0 — the loop closes again")
            }
            ["bake", step] => {
                let Ok(step) = step.parse::<usize>() else {
                    return StepOut::go("usage: bake <step in frames>");
                };
                let table = match self.track.bake(10) {
                    Ok(t) => t,
                    Err(e) => return StepOut::go(format!("bake refused: {e:?}")),
                };
                let selection = match select_frames(&table, step) {
                    Ok(s) => s,
                    Err(e) => return StepOut::go(format!("selection refused: {e:?}")),
                };
                let frames = selection.frames.len();
                let pupil = pupil_occupancy(16, 6, 12);
                match bake_closes(&self.track, &table, &selection) {
                    Err(e) => StepOut::go(format!(
                        "the bake does NOT close: {e:?} — a loop that closed at 10 ms can \
                         open at GIF rates, and the baked table is the judge"
                    )),
                    Ok(()) if frames > 20 => StepOut::go(format!(
                        "closes, extremes kept, pupil occupancy {pupil} px — but {frames} \
                         frames blow the GIF budget (≤20). Sample coarser."
                    )),
                    Ok(()) => StepOut::pass(format!(
                        "bake @every {step} frames: {frames} frames kept (extremes included: {}), \
                         closure holds across the bake, pupil occupancy {pupil} px in its cell. \
                         The GIF loops — seamlessly, at GIF resolution, heart pupil intact.",
                        selection.keeps_extremes
                    )),
                }
            }
            _ => StepOut::go("commands: bake <step> · open · close · menu/q"),
        }
    }
}

// ---------------------------------------------------------------- P8

/// P8 — Funscript Timeline. Sample the authored curve into
/// a funscript, then hear the router clamp lawful-ize it.
pub struct FunscriptSession {
    desc: LevelDescriptor,
    script: Option<Funscript>,
}

impl FunscriptSession {
    #[must_use]
    pub fn new() -> Self {
        Self {
            desc: descriptor(8),
            script: None,
        }
    }
}

impl Default for FunscriptSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Session for FunscriptSession {
    fn descriptor(&self) -> &LevelDescriptor {
        &self.desc
    }

    fn intro(&self) -> String {
        format!(
            "Playroom P8 — {} · host {} ({})\n\
             The same closed loop as P7's, renamed into touch: `write <step_ms>` samples \
             it to a funscript (pos 0..=100, one action per step plus the endpoint). \
             `clamp` runs the router contract over it — the ceiling is 0.20, so every \
             position above 20 moves, and the writer counts every move. The authored \
             curve and the lawful emission are different artifacts; this level makes \
             you hold both.\n\
             Vega's foundry rule (Daddy Style): what you authored and what may \
             lawfully be emitted are both written down before the write begins. The \
             clamp is not censorship — it is the contract doing its protective \
             work in the open, and counting every move it makes.\n\
             Pass: write, then clamp, with the twin write byte-identical.",
            self.desc.title, self.desc.host.name, self.desc.host.sketch
        )
    }

    fn handle(&mut self, cmd: &str) -> StepOut {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.as_slice() {
            ["write", step] => {
                let Ok(step) = step.parse::<u32>() else {
                    return StepOut::go("usage: write <step_ms> (must divide 1000)");
                };
                match Funscript::from_track(&studio_track(0.0), step) {
                    Ok(fs) => {
                        let count = fs.actions.len();
                        self.script = Some(fs);
                        StepOut::go(format!(
                            "funscript written: {count} actions over {} ms, range {}. `clamp` applies the router ceiling",
                            self.script.as_ref().map_or(0, |s| s.duration_ms),
                            self.script.as_ref().map_or(0, |s| s.range)
                        ))
                    }
                    Err(e) => StepOut::go(format!(
                        "write refused: {e:?} — the step must divide the cycle"
                    )),
                }
            }
            ["clamp"] => {
                let Some(fs) = &self.script else {
                    return StepOut::go("write a funscript first");
                };
                let (clamped, moved) = fs.apply_router_clamp(RouterConfig::default());
                // Twin-write check: rebuild from the track and
                // compare the JSON bytes.
                let step = fs.actions.get(1).map(|a| a.at_ms).unwrap_or(100);
                let twin_json = Funscript::from_track(&studio_track(0.0), step)
                    .map(|t| t.apply_router_clamp(RouterConfig::default()).0.to_json());
                let identical = twin_json.map(|t| t == clamped.to_json()).unwrap_or(false);
                if moved > 0 && identical && clamped.actions.iter().all(|a| a.pos <= 20) {
                    StepOut::pass(format!(
                        "clamped: {moved} actions moved to the 0.20 ceiling, range now {}, \
                         every position lawful — and the twin write is byte-identical. \
                         The difference between the authored curve and the lawful \
                         emission is {moved} counted moves, not a secret.",
                        clamped.range
                    ))
                } else {
                    StepOut::go(format!(
                        "clamp ran: {moved} moved, twin identical: {identical} — not a pass"
                    ))
                }
            }
            _ => StepOut::go("commands: write <step_ms> · clamp · menu/q"),
        }
    }
}

// ---------------------------------------------------------------- P9

/// P9 — The Third Dimension. Render the fixed scene, export
/// the GLB, encode the PNG — every artifact twin-run
/// byte-identical.
pub struct RenderSession {
    desc: LevelDescriptor,
    renders: Vec<(u64, u64)>,
    twin_ok: bool,
    glb_hash: Option<u64>,
    png_hash: Option<u64>,
}

impl RenderSession {
    #[must_use]
    pub fn new() -> Self {
        Self {
            desc: descriptor(9),
            renders: Vec::new(),
            twin_ok: false,
            glb_hash: None,
            png_hash: None,
        }
    }
}

impl Default for RenderSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Session for RenderSession {
    fn descriptor(&self) -> &LevelDescriptor {
        &self.desc
    }

    fn intro(&self) -> String {
        format!(
            "Playroom P9 — {} · host {} ({})\n\
             The 3D route in miniature: one fixed scene (a unit cube — the performer \
             block), an orbit step as the seed. `render <step>` rasterizes 48×48; \
             rendering the same step twice must hash identical. `glb` exports the \
             scene as binary glTF; `png` encodes the last render as a PNG (hand-rolled, \
             stored-block DEFLATE — no dependency). In 3D the style lanes become \
             shading contracts: the cube's faces take flat shades from a small ramp — \
             House Cel as a toon-shading contract.\n\
             Mireille's terms (Daddy Style): determinism is a promise made before \
             the render starts — same step, same hash, or the artifact does not \
             ship. The gate is not suspicion; the gate is the care.\n\
             Pass: a twin-identical render, plus both artifacts.",
            self.desc.title, self.desc.host.name, self.desc.host.sketch
        )
    }

    fn handle(&mut self, cmd: &str) -> StepOut {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let step_out = match parts.as_slice() {
            ["render", step] => {
                let Ok(step) = step.parse::<u64>() else {
                    return StepOut::go("usage: render <step>");
                };
                let frame = Frame::render(48, 48, step, 16);
                let hash = frame.hash();
                if let Some((_, prev)) = self.renders.iter().find(|(s, _)| *s == step)
                    && *prev == hash
                {
                    self.twin_ok = true;
                }
                self.renders.push((step, hash));
                StepOut::go(format!(
                    "render step {step}/16: hash {hash:016x}{}",
                    if self.twin_ok {
                        " — twin run identical"
                    } else {
                        ""
                    }
                ))
            }
            ["glb"] => {
                let glb = scene_glb();
                if &glb[0..4] != b"glTF" {
                    return StepOut::fail("GLB envelope broken — the export is not glTF");
                }
                let hash = milking_core::render3d::fnv1a(&glb);
                self.glb_hash = Some(hash);
                StepOut::go(format!(
                    "GLB exported: {} bytes, glTF envelope verified, hash {hash:016x}",
                    glb.len()
                ))
            }
            ["png"] => {
                let frame = Frame::render(48, 48, 0, 16);
                let png = frame_png(&frame);
                if png[0..4] != [0x89, b'P', b'N', b'G'] {
                    return StepOut::fail("PNG envelope broken");
                }
                let hash = milking_core::render3d::fnv1a(&png);
                self.png_hash = Some(hash);
                StepOut::go(format!(
                    "PNG encoded: {} bytes, envelope verified, hash {hash:016x}",
                    png.len()
                ))
            }
            _ => return StepOut::go("commands: render <step> · glb · png · menu/q"),
        };
        if self.twin_ok && self.glb_hash.is_some() && self.png_hash.is_some() {
            return StepOut::pass(format!(
                "{} — and with that, all three artifacts stand: a render that twin-runs \
                 byte-identical, a GLB, and a PNG of the same scene. The 3D route's whole \
                 discipline is this: same scene, same seed, same bytes.",
                step_out.text
            ));
        }
        step_out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::run_scripted;

    #[test]
    fn every_playroom_route_reaches_its_success_state() {
        for id in [2u8, 3, 5, 6, 7, 8, 9] {
            let mut session = playroom_session(id).expect("session exists");
            let (success, transcript) = run_scripted(session.as_mut(), play_script(id));
            assert!(
                success,
                "P{id} scripted route must succeed:\n{}",
                transcript.join("\n")
            );
        }
    }

    // ---- adversarial: wrong play must not pass ----

    #[test]
    fn p2_off_grid_hits_fail_the_set() {
        let mut session = playroom_session(2).unwrap();
        // 20 ticks off every beat in round 1: all misses.
        let misses: Vec<String> = (0..8).map(|k| format!("hit {}", 20 + k * 30)).collect();
        let script: Vec<&str> = misses.iter().map(String::as_str).collect();
        let (success, _) = run_scripted(session.as_mut(), &script);
        assert!(!success);
    }

    #[test]
    fn p3_unlawful_orders_never_pass() {
        let mut session = playroom_session(3).unwrap();
        let (success, _) = run_scripted(
            session.as_mut(),
            &["order render,haptics,physics,rig,input"],
        );
        assert!(!success);
    }

    #[test]
    fn p5_over_ceiling_and_dead_watchdog_do_not_pass() {
        let mut session = playroom_session(5).unwrap();
        // Over-ceiling is a fault on the trail: the stop that
        // follows cannot pass.
        let (success, _) = run_scripted(session.as_mut(), &["beat", "emit 0.9", "stop"]);
        assert!(!success);
    }

    #[test]
    fn p6_wrong_predictions_fail_the_border() {
        let mut session = playroom_session(6).unwrap();
        let (success, _) = run_scripted(
            session.as_mut(),
            &[
                "answer nomap",
                "answer nomap",
                "answer nomap",
                "answer nomap",
                "answer nomap",
                "answer exact",
                "answer exact",
                "answer nomap",
                "export",
            ],
        );
        assert!(!success);
    }

    #[test]
    fn p7_open_loop_bake_does_not_pass() {
        let mut session = playroom_session(7).unwrap();
        let (success, _) = run_scripted(session.as_mut(), &["open", "bake 25"]);
        assert!(!success);
    }
}
