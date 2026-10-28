//! Study sessions (SPEC §6.5): Lessons S1–S8 wrapped around
//! their `milking-core` twins. The framing is the pedagogy:
//! you are summoned to the Study, the compiler corrects you
//! at the desk, and the correction is the care working as
//! intended. Each lesson's exercise is its twin's contract,
//! played by hand once, with the typed refusals left in.
//! The discipline taught is Daddy Style (SPEC §7.4): strict
//! requirements set in advance by a firm, caring authority,
//! applied consistently, never arbitrary — success praised
//! exactly, failure landing as named consequence, never
//! punishment.

use milking_core::accumulator::{Accumulator, NaiveFloatAccumulator};
use milking_core::aftercare::{Challenge, ChallengeClass, judge};
use milking_core::authoring::{Easing, Keyform, Track};
use milking_core::border::{FieldType, Tool, ToolBorder, Value};
use milking_core::frame::{Frame, FrameError};
use milking_core::funscript::Funscript;
use milking_core::license::{LicenseHeader, Use, check};
use milking_core::puppet::{PuppetFile, apply_edit};
use milking_core::queues::{QueueError, StreamRouter};
use milking_core::router::{Router, RouterConfig};
use milking_core::tracking::{Calibration, TrackEvent, TrackState, Tracker};

use crate::level::{LevelDescriptor, study_levels};
use crate::session::{Session, StepOut};

fn descriptor(id: u8) -> LevelDescriptor {
    *study_levels()
        .iter()
        .find(|l| l.id == id)
        .expect("study id exists")
}

/// The session for a Study lesson.
#[must_use]
pub fn study_session(id: u8) -> Option<Box<dyn Session>> {
    match id {
        1 => Some(Box::new(PuppetSession::new())),
        2 => Some(Box::new(LicenseSession::new())),
        3 => Some(Box::new(AccumulatorSession::new())),
        4 => Some(Box::new(FrameSession::new())),
        5 => Some(Box::new(QueuesSession::new())),
        6 => Some(Box::new(BorderStudySession::new())),
        7 => Some(Box::new(TrackingSession::new())),
        8 => Some(Box::new(AftercareSession::new())),
        _ => None,
    }
}

/// The canonical scripted route for a lesson.
#[must_use]
pub fn study_script(id: u8) -> &'static [&'static str] {
    match id {
        1 => &[
            "hold gain",
            "set gain 2.6",
            "set lane House Cel",
            "get gain",
            "check",
        ],
        2 => &[
            "answer admit",
            "answer UnknownLicense",
            "answer MissingAttribution",
            "answer admit",
            "answer RedistributionForbidden",
            "answer TrainingForbidden",
        ],
        3 => &["feed 1000", "prove"],
        4 => &[
            "render", "orphan 2", "render", "drop 3", "drop 2", "render", "dropall",
        ],
        5 => &[
            "offer 0.1",
            "offer 0.2",
            "deliver",
            "offer 0.3",
            "offer 0.4",
            "offer 0.5",
            "offer 0.6",
            "stop",
            "deliver",
        ],
        6 => &[
            "call echo text=hello",
            "call beat_at index=4",
            "call echo",
            "call echo text=again",
        ],
        7 => &[
            "acquire", "sample 3", "sample 3", "sample 3", "sample 3", "sample 3", "frame",
            "frame", "frame", "lost", "seen", "sample 3", "sample 3", "sample 3", "sample 3",
            "sample 3",
        ],
        8 => &["run", "judge"],
        _ => &[],
    }
}

// ---------------------------------------------------------------- S1

/// S1 — Own the Puppet. Borrow a document's fields, edit the
/// owned document, re-parse: the borrow does not survive the
/// mutation, and the signature says so.
pub struct PuppetSession {
    desc: LevelDescriptor,
    doc: String,
    held: Option<(String, String)>,
}

impl PuppetSession {
    #[must_use]
    pub fn new() -> Self {
        Self {
            desc: descriptor(1),
            doc: "# Opal rig\nname = Opal\ngain = 1.0\nrole = host\n".to_string(),
            held: None,
        }
    }

    /// Re-parse the document and read one field as an owned
    /// copy — the session layer's convenience; the lesson's
    /// borrow discipline is in how `hold` + `set` interact.
    fn field(&self, key: &str) -> Result<Option<String>, String> {
        match PuppetFile::parse(&self.doc) {
            Ok(parsed) => Ok(parsed.get(key).map(str::to_string)),
            Err(e) => Err(format!("{e:?}")),
        }
    }
}

impl Default for PuppetSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Session for PuppetSession {
    fn descriptor(&self) -> &LevelDescriptor {
        &self.desc
    }

    fn intro(&self) -> String {
        format!(
            "Lesson S1 — {} · host {} ({})\n\
             A puppet file, borrowed whole: `get <key>` reads a field (a borrow of a \
             borrow — nothing is copied), `hold <key>` keeps a value in your hand, \
             `set <key> <value>` edits the owned document. An edit ends every borrow: \
             the document must be re-parsed before it can be read again. That is not \
             bureaucracy; it is the ownership rule with its working clothes on.\n\
             Opal teaches it the Daddy Style way: what you may hold and what you \
             may change are stated before your hands move — the borrow ends at the \
             edit because the terms always said it would.\n\
             The desk's terms: make gain read 2.6 and add lane = House Cel, then `check`.",
            self.desc.title, self.desc.host.name, self.desc.host.sketch
        )
    }

    fn handle(&mut self, cmd: &str) -> StepOut {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.as_slice() {
            ["get", key] => match self.field(key) {
                Ok(v) => match v {
                    Some(v) => StepOut::go(format!("{key} = {v} (borrowed from the document)")),
                    _ => StepOut::go(format!("no field named {key}")),
                },
                Err(e) => StepOut::go(format!("the document does not parse: {e}")),
            },
            ["hold", key] => match self.field(key) {
                Ok(v) => match v {
                    Some(v) => {
                        self.held = Some(((*key).to_string(), v.clone()));
                        StepOut::go(format!(
                            "you hold {key} = {v} in your hand — a borrow. An edit will end it"
                        ))
                    }
                    _ => StepOut::go(format!("no field named {key}")),
                },
                Err(e) => StepOut::go(format!("the document does not parse: {e}")),
            },
            ["set", key, values @ ..] => {
                let value = values.join(" ");
                let held_note = if let Some((held_key, _)) = self.held.take() {
                    format!(
                        " The value you held ({held_key}) is gone — the edit ended the \
                         borrow, exactly as the signature promised."
                    )
                } else {
                    String::new()
                };
                self.doc = apply_edit(std::mem::take(&mut self.doc), key, &value);
                StepOut::go(format!(
                    "edited: {key} = {value} (the document is re-parsed on the next read).{held_note}"
                ))
            }
            ["check"] => match PuppetFile::parse(&self.doc) {
                Ok(parsed) => {
                    if parsed.get("gain") == Some("2.6") && parsed.get("lane") == Some("House Cel")
                    {
                        StepOut::pass(
                            "gain reads 2.6, lane reads House Cel, and every read after \
                             an edit went through a fresh parse. The puppet is yours — \
                             because you owned the document and only ever borrowed \
                             the fields.",
                        )
                    } else {
                        StepOut::go(format!(
                            "not yet: gain = {:?}, lane = {:?} — the terms are gain 2.6 \
                             and lane House Cel",
                            parsed.get("gain"),
                            parsed.get("lane")
                        ))
                    }
                }
                Err(e) => StepOut::go(format!("the document does not parse: {e:?}")),
            },
            _ => {
                StepOut::go("commands: get <key> · hold <key> · set <key> <value> · check · menu/q")
            }
        }
    }
}

// ---------------------------------------------------------------- S2

struct LicenseCase {
    header: LicenseHeader,
    use_: Use,
    label: &'static str,
}

fn license_cases() -> Vec<LicenseCase> {
    let full = |license: &str| LicenseHeader {
        license: license.to_string(),
        attribution_present: true,
        redistribution_allowed: true,
        ai_training_allowed: true,
    };
    vec![
        LicenseCase {
            header: full("Apache-2.0"),
            use_: Use::Ship,
            label: "Apache-2.0, attribution present, all terms open — use: Ship",
        },
        LicenseCase {
            header: full("Proprietary"),
            use_: Use::Ship,
            label: "Proprietary, attribution present — use: Ship",
        },
        LicenseCase {
            header: LicenseHeader {
                attribution_present: false,
                ..full("MIT")
            },
            use_: Use::Ship,
            label: "MIT, attribution ABSENT — use: Ship",
        },
        LicenseCase {
            header: LicenseHeader {
                attribution_present: false,
                ..full("CC0-1.0")
            },
            use_: Use::Ship,
            label: "CC0-1.0, attribution absent — use: Ship",
        },
        LicenseCase {
            header: LicenseHeader {
                redistribution_allowed: false,
                ..full("CC-BY-4.0")
            },
            use_: Use::Redistribute,
            label: "CC-BY-4.0, redistribution NOT allowed — use: Redistribute",
        },
        LicenseCase {
            header: LicenseHeader {
                ai_training_allowed: false,
                ..full("CC-BY-4.0")
            },
            use_: Use::Train,
            label: "CC-BY-4.0, AI training NOT allowed — use: Train",
        },
    ]
}

/// S2 — Fail Closed. Judge doctored headers; the variant is
/// the diagnosis.
pub struct LicenseSession {
    desc: LevelDescriptor,
    idx: usize,
    score: u32,
}

impl LicenseSession {
    #[must_use]
    pub fn new() -> Self {
        Self {
            desc: descriptor(2),
            idx: 0,
            score: 0,
        }
    }
}

impl Default for LicenseSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Session for LicenseSession {
    fn descriptor(&self) -> &LevelDescriptor {
        &self.desc
    }

    fn intro(&self) -> String {
        format!(
            "Lesson S2 — {} · host {} ({})\n\
             The license gate never fails with a string: it refuses with a variant, and \
             the variant is the diagnosis. For each header: `answer admit`, or \
             `answer <Variant>` — UnknownLicense, MissingAttribution, \
             RedistributionForbidden, TrainingForbidden. Contract order: known license \
             → attribution → use terms. An unknown license is a refusal: fail closed \
             means the default answer to anything unrecognized is no.\n\
             Vigil grades exactly (Daddy Style): the standard is 6 of 6, known \
             before the first header is shown. A miss is a consequence with a \
             variant name, never a punishment — the correction tells you \
             precisely which term was broken, and that precision is the care.\n\
             Case 1 of 6: {}",
            self.desc.title,
            self.desc.host.name,
            self.desc.host.sketch,
            license_cases()[0].label
        )
    }

    fn handle(&mut self, cmd: &str) -> StepOut {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let cases = license_cases();
        match parts.as_slice() {
            ["answer", answer] => {
                if self.idx >= cases.len() {
                    return StepOut::go("all headers judged");
                }
                let case = &cases[self.idx];
                let truth = check(&case.header, case.use_);
                let right = match (&truth, *answer) {
                    (Ok(()), "admit") => true,
                    (Err(e), a) => format!("{e:?}")
                        .split([' ', '{', '('])
                        .next()
                        .is_some_and(|variant| variant == a),
                    _ => false,
                };
                if right {
                    self.score += 1;
                }
                self.idx += 1;
                let verdict_text = match &truth {
                    Ok(()) => "admitted".to_string(),
                    Err(e) => format!("refused: {e:?}"),
                };
                let mut text = format!(
                    "case {}: the gate {verdict_text} — {} (score {}/{})",
                    self.idx,
                    if right {
                        "exact"
                    } else {
                        "not the variant the gate speaks"
                    },
                    self.score,
                    self.idx
                );
                if self.idx < cases.len() {
                    text.push_str(&format!(
                        "\nCase {} of 6: {}",
                        self.idx + 1,
                        cases[self.idx].label
                    ));
                    StepOut::go(text)
                } else if self.score == cases.len() as u32 {
                    StepOut::pass(format!(
                        "{text}\nEvery header judged to its exact variant. The gate \
                         fails closed and speaks in types — and now, so do you."
                    ))
                } else {
                    StepOut::fail(format!(
                        "{text}\n{}/6 is not the desk's standard. Vague failures cool \
                         Vigil on the spot; exact ones are the lesson.",
                        self.score
                    ))
                }
            }
            _ => StepOut::go("commands: answer admit|<Variant> · menu/q"),
        }
    }
}

// ---------------------------------------------------------------- S3

/// S3 — The Accumulator. Integer contract vs the drifting
/// float: run both over 1000 frames and count the loss.
pub struct AccumulatorSession {
    desc: LevelDescriptor,
    int_acc: Accumulator,
    float_acc: NaiveFloatAccumulator,
    fed: u64,
}

impl AccumulatorSession {
    #[must_use]
    pub fn new() -> Self {
        Self {
            desc: descriptor(3),
            int_acc: Accumulator::new(100_000),
            float_acc: NaiveFloatAccumulator::new(0.1),
            fed: 0,
        }
    }
}

impl Default for AccumulatorSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Session for AccumulatorSession {
    fn descriptor(&self) -> &LevelDescriptor {
        &self.desc
    }

    fn intro(&self) -> String {
        format!(
            "Lesson S3 — {} · host {} ({})\n\
             Two accumulators, one workload: frames of 0.3 s against a 0.1 s step. \
             The integer contract (100 000 µs steps, the remainder carries) and a naive \
             float version (seconds as f64) run side by side. `feed <frames>` runs \
             frames through both; `prove` reads the counts at 1000 frames.\n\
             The lesson is not that floats are bad. It is that 0.3 / 0.1 in binary is \
             2.9999999999999996, and a floor over a thousand frames keeps the change.\
             \nSable's clock is Daddy Style made audible: the step is named in \
             advance and never renegotiated mid-dance — which is why the integer \
             keeps perfect time while the float, improvising, loses a step.",
            self.desc.title, self.desc.host.name, self.desc.host.sketch
        )
    }

    fn handle(&mut self, cmd: &str) -> StepOut {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.as_slice() {
            ["feed", n] => {
                let Ok(n) = n.parse::<u64>() else {
                    return StepOut::go("usage: feed <frames>");
                };
                for _ in 0..n {
                    self.int_acc.feed(300_000);
                    self.float_acc.feed(0.3);
                }
                self.fed += n;
                StepOut::go(format!(
                    "fed {} frames ({} total): integer {} steps, float {} steps, \
                     integer carry {} µs",
                    n,
                    self.fed,
                    self.int_acc.total_steps(),
                    self.float_acc.total_steps,
                    self.int_acc.carry_us()
                ))
            }
            ["prove"] => {
                if self.fed < 1000 {
                    return StepOut::go(format!(
                        "the proof is read at 1000 frames ({} fed) — drift is a \
                         long-game property",
                        self.fed
                    ));
                }
                let int_total = self.int_acc.total_steps();
                let float_total = self.float_acc.total_steps;
                if int_total == 3000 && float_total == 2999 && self.int_acc.carry_us() == 0 {
                    StepOut::pass(format!(
                        "1000 frames: the integer contract emitted exactly {int_total} \
                         steps with 0 µs carried; the float version emitted {float_total} \
                         — one step, lost to representation, banked as phantom carry. \
                         Sable danced to the integer clock the whole time."
                    ))
                } else {
                    StepOut::go(format!(
                        "counts: integer {int_total}, float {float_total}, carry {} µs — \
                         the contract expects 3000 / 2999 / 0 at exactly 1000 frames",
                        self.int_acc.carry_us()
                    ))
                }
            }
            _ => StepOut::go("commands: feed <frames> · prove · menu/q"),
        }
    }
}

// ---------------------------------------------------------------- S4

/// S4 — Hold the Frame. Own the resource lifecycle: ordered
/// drops, an orphan's flicker, and the absent unsafe block.
pub struct FrameSession {
    desc: LevelDescriptor,
    frame: Frame,
    clean_render: bool,
    orphan_seen: bool,
    reclaimed: bool,
}

impl FrameSession {
    #[must_use]
    pub fn new() -> Self {
        Self {
            desc: descriptor(4),
            frame: Frame::new(),
            clean_render: false,
            orphan_seen: false,
            reclaimed: false,
        }
    }
}

impl Default for FrameSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Session for FrameSession {
    fn descriptor(&self) -> &LevelDescriptor {
        &self.desc
    }

    fn intro(&self) -> String {
        format!(
            "Lesson S4 — {} · host {} ({})\n\
             One frame, four owned resources: 0 device, 1 buffer, 2 texture, \
             3 pipeline. Drops run children-first — a resource dies only after \
             everything acquired after it. `orphan <id>` vanishes a handle's owner \
             without a drop: the frame flickers (render refuses, naming the orphan) \
             until a lawful drop reclaims it.\n\
             The desk's route: render clean, orphan 2, watch the flicker, reclaim by \
             dropping 3 then 2, render again, then `dropall`.\n\
             And the one unsafe block? In the real renderer an FFI present call stands \
             at render — here it is absent by law (this crate forbids unsafe), named \
             and fenced in the twin's source. Absent is a lawful outcome.\n\
             Mireille's desk runs Daddy Style: every resource has an owner, every \
             death is ordered in advance, and the flicker is a consequence with a \
             name — reclaim lawfully and she holds the frame again, steady as \
             praise.",
            self.desc.title, self.desc.host.name, self.desc.host.sketch
        )
    }

    fn handle(&mut self, cmd: &str) -> StepOut {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.as_slice() {
            ["render"] => match self.frame.render() {
                Ok(sum) => {
                    if !self.orphan_seen {
                        self.clean_render = true;
                    }
                    StepOut::go(format!(
                        "frame presents (checksum {sum:016x}) — every handle alive and owned"
                    ))
                }
                Err(FrameError::Orphaned { id }) => {
                    if id == 2 {
                        self.orphan_seen = true;
                    }
                    StepOut::go(format!(
                        "FLICKER: render refused — handle {id} is orphaned. She holds \
                         the frame only while every resource has an owner. Drop the \
                         handles above it, then drop it, to reclaim it."
                    ))
                }
                Err(e) => StepOut::go(format!("render refused: {e:?}")),
            },
            ["orphan", id] => {
                let Ok(id) = id.parse::<usize>() else {
                    return StepOut::go("usage: orphan <id>");
                };
                match self.frame.orphan(id) {
                    Ok(()) => StepOut::go(format!(
                        "handle {id} orphaned — its owner is gone, the resource is not freed"
                    )),
                    Err(e) => StepOut::go(format!("orphan refused: {e:?}")),
                }
            }
            ["drop", id] => {
                let Ok(id) = id.parse::<usize>() else {
                    return StepOut::go("usage: drop <id>");
                };
                match self.frame.drop_handle(id) {
                    Ok(()) => {
                        if id == 2 && self.orphan_seen {
                            self.reclaimed = true;
                        }
                        StepOut::go(format!(
                            "dropped handle {id} (drop order so far: {:?})",
                            self.frame.drop_log()
                        ))
                    }
                    Err(e) => StepOut::go(format!(
                        "drop refused: {e:?} — children first; a refused drop changes nothing"
                    )),
                }
            }
            ["dropall"] => {
                let order = self.frame.drop_all();
                if self.clean_render
                    && self.orphan_seen
                    && self.reclaimed
                    && self.frame.drop_log() == [3, 2, 1, 0]
                {
                    StepOut::pass(format!(
                        "final drops {order:?}; the frame's full audit reads [3, 2, 1, 0] \
                         — children before the device they were made from, the orphan \
                         reclaimed on the way. Every resource had an owner at every \
                         moment it was used."
                    ))
                } else {
                    StepOut::go(format!(
                        "dropped {order:?} (audit: {:?}) — the desk's route is: clean \
                         render, the orphan's flicker, the reclaim, then the full order",
                        self.frame.drop_log()
                    ))
                }
            }
            _ => StepOut::go("commands: render · orphan <id> · drop <id> · dropall · menu/q"),
        }
    }
}

// ---------------------------------------------------------------- S5

/// S5 — Bounded Queues. Name the bound, feel the typed
/// backpressure, rehearse the synchronous stop mid-stream.
pub struct QueuesSession {
    desc: LevelDescriptor,
    router: StreamRouter,
    full_seen: bool,
    stopped_discarded: Option<usize>,
}

impl QueuesSession {
    #[must_use]
    pub fn new() -> Self {
        Self {
            desc: descriptor(5),
            router: StreamRouter::new(4),
            full_seen: false,
            stopped_discarded: None,
        }
    }
}

impl Default for QueuesSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Session for QueuesSession {
    fn descriptor(&self) -> &LevelDescriptor {
        &self.desc
    }

    fn intro(&self) -> String {
        format!(
            "Lesson S5 — {} · host {} ({})\n\
             A stream router with a queue bound of 4, named before it runs. `offer \
             <value>` queues an emission; at the bound, the next offer is refused \
             typed — backpressure, never a silent drop and never a silent grow. \
             `deliver` hands the oldest emission to the consumer. `stop` is \
             synchronous and total: it latches now, discards the pending queue, and \
             nothing is delivered after it, ever.\n\
             The rehearsal: deliver one, fill the queue to its refusal, then stop \
             mid-stream and try to deliver again. Hear the shape of Vigil's \
             discipline (Daddy Style): every bound is named before the stream \
             starts — and the stop answers instantly, to anyone, every time. A \
             boundary that must be earned is not a boundary; hers is simply kept.",
            self.desc.title, self.desc.host.name, self.desc.host.sketch
        )
    }

    fn handle(&mut self, cmd: &str) -> StepOut {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.as_slice() {
            ["offer", v] => {
                let Ok(v) = v.parse::<f64>() else {
                    return StepOut::go("usage: offer <value>");
                };
                match self.router.offer(v) {
                    Ok(()) => StepOut::go(format!("queued {v}")),
                    Err(QueueError::Full { bound }) => {
                        self.full_seen = true;
                        StepOut::go(format!(
                            "refused typed: the queue is at its named bound ({bound}). \
                             The held items are unharmed — slow down, don't buffer forever"
                        ))
                    }
                    Err(e) => StepOut::go(format!("refused: {e:?}")),
                }
            }
            ["deliver"] => match self.router.deliver() {
                Some(v) => StepOut::go(format!(
                    "delivered {v} (delivered so far: {:?})",
                    self.router.delivered()
                )),
                _ => {
                    if self.stopped_discarded.is_some()
                        && self.full_seen
                        && self.router.delivered() == [0.1]
                    {
                        StepOut::pass(format!(
                            "nothing — and nothing is the verdict. The stop discarded \
                             {} pending emissions and froze delivery; what was \
                             delivered before it ({:?}) stands, and not one value \
                             arrived late. The stop is synchronous and total.",
                            self.stopped_discarded.unwrap_or(0),
                            self.router.delivered()
                        ))
                    } else {
                        StepOut::go("nothing to deliver (queue empty, or the stop has latched)")
                    }
                }
            },
            ["stop"] => {
                let discarded = self.router.stop();
                self.stopped_discarded = Some(discarded);
                StepOut::go(format!(
                    "STOP latched — {discarded} pending emissions discarded, stated \
                     plainly. Try `deliver` now."
                ))
            }
            _ => StepOut::go("commands: offer <value> · deliver · stop · menu/q"),
        }
    }
}

// ---------------------------------------------------------------- S6

/// S6 — Async Borders. A schema-first tool surface over
/// milking-core's own border twin: validation at the border,
/// the handler synchronous, the drain deterministic.
pub struct BorderStudySession {
    desc: LevelDescriptor,
    ok_calls: u32,
    refusal_seen: bool,
}

impl BorderStudySession {
    #[must_use]
    pub fn new() -> Self {
        Self {
            desc: descriptor(6),
            ok_calls: 0,
            refusal_seen: false,
        }
    }
}

impl Default for BorderStudySession {
    fn default() -> Self {
        Self::new()
    }
}

fn study_border() -> ToolBorder {
    use milking_core::border::Field;
    let mut border = ToolBorder::new();
    border.register(Tool {
        name: "echo",
        schema: vec![Field {
            name: "text",
            ty: FieldType::Text,
            required: true,
        }],
        handler: |args| {
            args.iter()
                .find(|(n, _)| n == "text")
                .map(|(_, v)| match v {
                    Value::Text(s) => s.clone(),
                    _ => String::new(),
                })
                .unwrap_or_default()
        },
    });
    border.register(Tool {
        name: "beat_at",
        schema: vec![
            Field {
                name: "index",
                ty: FieldType::Int,
                required: true,
            },
            Field {
                name: "loud",
                ty: FieldType::Bool,
                required: false,
            },
        ],
        handler: |args| {
            let index = args
                .iter()
                .find(|(n, _)| n == "index")
                .map(|(_, v)| match v {
                    Value::Int(i) => *i,
                    _ => 0,
                })
                .unwrap_or(0);
            format!("beat {index} lands at tick {}", index * 30)
        },
    });
    border
}

impl Session for BorderStudySession {
    fn descriptor(&self) -> &LevelDescriptor {
        &self.desc
    }

    fn intro(&self) -> String {
        format!(
            "Lesson S6 — {} · host {} ({})\n\
             Two tools at a schema border: echo (text: Text, required) and beat_at \
             (index: Int, required; loud: Bool, optional). `call <tool> [k=v …]` \
             submits and drains one call: the border validates against the schema \
             BEFORE the handler runs — a missing field, a wrong type, an undeclared \
             field are all refused at the border and the handler never sees them. \
             The core stays synchronous; only the border is async-shaped, so the \
             drain's order is the submission order, deterministically.\n\
             The sitting: three lawful calls, and one schema lie refused typed \
             (try `call echo` with its required field missing). Sable answers on \
             the beat or refuses at the border (Daddy Style): the schema is \
             published before the sitting, so a lawful call is never a surprise \
             and a refusal is never a mood.",
            self.desc.title, self.desc.host.name, self.desc.host.sketch
        )
    }

    fn handle(&mut self, cmd: &str) -> StepOut {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.as_slice() {
            ["call", tool, rest @ ..] => {
                let mut args: Vec<(String, Value)> = Vec::new();
                for pair in rest {
                    let Some((k, v)) = pair.split_once('=') else {
                        return StepOut::go("arguments are k=v pairs");
                    };
                    let value = if let Ok(i) = v.parse::<i64>() {
                        Value::Int(i)
                    } else if let Ok(b) = v.parse::<bool>() {
                        Value::Bool(b)
                    } else {
                        Value::Text((*v).to_string())
                    };
                    args.push((k.to_string(), value));
                }
                let mut border = study_border();
                border.submit(tool, args);
                let results = border.drain();
                match &results[0] {
                    Ok(text) => {
                        self.ok_calls += 1;
                        let text = format!(
                            "the border validated, the handler answered: {text} \
                             ({} lawful calls)",
                            self.ok_calls
                        );
                        if self.ok_calls >= 3 && self.refusal_seen {
                            StepOut::pass(format!(
                                "{text}\nSchemas first, async only at the border, the \
                                 core synchronous — Sable answered every call on the \
                                 beat, and the one lie never reached a handler."
                            ))
                        } else {
                            StepOut::go(text)
                        }
                    }
                    Err(e) => {
                        self.refusal_seen = true;
                        StepOut::go(format!(
                            "refused at the border: {e:?} — the handler never saw the \
                             call. A schema lie makes her skip, visibly."
                        ))
                    }
                }
            }
            _ => StepOut::go("commands: call <tool> [k=v …] · menu/q"),
        }
    }
}

// ---------------------------------------------------------------- S7

/// S7 — Track the Performer. Drive the tracking machine
/// through lock, loss, and re-lock; unhandled transitions
/// refuse typed.
pub struct TrackingSession {
    desc: LevelDescriptor,
    tracker: Tracker,
    locked_once: bool,
    was_lost: bool,
    relocked: bool,
}

impl TrackingSession {
    #[must_use]
    pub fn new() -> Self {
        Self {
            desc: descriptor(7),
            tracker: Tracker::new(Calibration::default()),
            locked_once: false,
            was_lost: false,
            relocked: false,
        }
    }
}

impl Default for TrackingSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Session for TrackingSession {
    fn descriptor(&self) -> &LevelDescriptor {
        &self.desc
    }

    fn intro(&self) -> String {
        format!(
            "Lesson S7 — {} · host {} ({})\n\
             The tracking machine: Idle → Calibrating → Tracking, with Lost and \
             Faulted at the edges. Calibration terms, negotiated in advance: 5 \
             stable samples, jitter ≤ 10 — a wild sample is answered but does not \
             count (a let chain, Rust 2024: count AND stability in one condition). \
             Every (state, event) pair is answered; what means nothing here refuses \
             typed and changes nothing.\n\
             Commands: acquire · sample <jitter> · frame · lost · seen · fault · \
             reset. The sitting: lock, track 3 frames, lose her, and re-lock — \
             loss means the lock's evidence is stale, so recalibration starts at \
             zero. That is Daddy Style in the booth: terms negotiated before the \
             show, applied the same on the first lock as on the re-lock — Vega's \
             composure is just consistency, kept.",
            self.desc.title, self.desc.host.name, self.desc.host.sketch
        )
    }

    fn handle(&mut self, cmd: &str) -> StepOut {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let event = match parts.as_slice() {
            ["acquire"] => TrackEvent::Acquire,
            ["sample", j] => {
                let Ok(j) = j.parse::<u32>() else {
                    return StepOut::go("usage: sample <jitter>");
                };
                TrackEvent::Sample { jitter: j }
            }
            ["frame"] => TrackEvent::Frame,
            ["lost"] => TrackEvent::PerformerLost,
            ["seen"] => TrackEvent::PerformerSeen,
            ["fault"] => TrackEvent::Fault,
            ["reset"] => TrackEvent::Reset,
            _ => {
                return StepOut::go(
                    "commands: acquire · sample <jitter> · frame · lost · seen · fault · reset · menu/q",
                );
            }
        };
        match self.tracker.apply(event) {
            Ok(state) => {
                if state == TrackState::Lost {
                    self.was_lost = true;
                }
                if state == TrackState::Tracking {
                    if !self.locked_once {
                        self.locked_once = true;
                    } else if self.was_lost && self.tracker.frames() >= 3 {
                        self.relocked = true;
                    }
                }
                let mut text = format!(
                    "state: {state:?} (stable samples {}, frames tracked {})",
                    self.tracker.stable_samples(),
                    self.tracker.frames()
                );
                if self.relocked {
                    return StepOut::pass(format!(
                        "{text}\nRe-locked after loss, from a full recalibration — the \
                         booth view held through every transition, and every event \
                         was answered. Exhaustiveness reads as her composure."
                    ));
                }
                if state == TrackState::Tracking && self.locked_once {
                    text.push_str(" — locked on");
                }
                StepOut::go(text)
            }
            Err(e) => StepOut::go(format!(
                "refused typed: {e:?} — the machine answers everything, and this \
                 pair's answer is no. Nothing changed."
            )),
        }
    }
}

// ---------------------------------------------------------------- S8

fn aftercare_challenges() -> Vec<Challenge> {
    vec![
        Challenge {
            name: "closed loops close",
            class: ChallengeClass::Validation,
        },
        Challenge {
            name: "router clamps emission",
            class: ChallengeClass::Validation,
        },
        Challenge {
            name: "open loops are refused",
            class: ChallengeClass::Adversarial,
        },
        Challenge {
            name: "over-ceiling is refused typed",
            class: ChallengeClass::Adversarial,
        },
    ]
}

fn run_aftercare_checks() -> Vec<bool> {
    let closed = Track::new(
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
    .expect("starter track");
    let open = Track::new(
        1000,
        vec![
            Keyform {
                at_ms: 0,
                value: 0.0,
            },
            Keyform {
                at_ms: 1000,
                value: 0.5,
            },
        ],
        Easing::Linear,
    )
    .expect("open track");
    let clamped = Funscript::from_track(&closed, 100)
        .map(|fs| {
            let (c, _) = fs.apply_router_clamp(RouterConfig::default());
            c.range == 20 && c.actions.iter().all(|a| a.pos <= 20)
        })
        .unwrap_or(false);
    let mut router = Router::new(RouterConfig::default());
    let _ = router.heartbeat(0);
    let over_ceiling_refused = matches!(
        router.emit(0.21),
        Err(milking_core::router::RouterError::OverCeiling { .. })
    );
    vec![
        closed.check_closure().is_ok(),
        clamped,
        open.check_closure().is_err(),
        over_ceiling_refused,
    ]
}

/// S8 — Aftercare. The capstone: run the real gate suite
/// (both halves), submit to the judge, and learn what a
/// missing half makes unjudgeable.
pub struct AftercareSession {
    desc: LevelDescriptor,
    results: Option<Vec<bool>>,
}

impl AftercareSession {
    #[must_use]
    pub fn new() -> Self {
        Self {
            desc: descriptor(8),
            results: None,
        }
    }
}

impl Default for AftercareSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Session for AftercareSession {
    fn descriptor(&self) -> &LevelDescriptor {
        &self.desc
    }

    fn intro(&self) -> String {
        format!(
            "Lesson S8 — {} · host {} ({}), with all five hosts attending\n\
             After the intensity: tests, review, verification. The gate suite has \
             two halves by estate law — validation (the system does what it claims) \
             and adversarial (thinking like the person who wants the terms broken). \
             A suite with only one half is unjudgeable, not half a suite. The \
             verdict is the conjunction: one failure anywhere is a failed gate, \
             named. `run` executes the four challenges against the real twins — \
             closure, the router clamp, the open loop's refusal, the over-ceiling \
             refusal. `sabotage <n>` flips one result, so you can watch a gate \
             fail honestly before you `judge`.\n\
             The board's standard is Daddy Style at its most explicit: both \
             halves, named in advance, one conjunction for a verdict. And \
             aftercare itself is the lesson's point — the deliberate work a firm \
             hand does after intensity so the next sitting starts safe. It is \
             never a reward for performing well; it is care, scheduled.",
            self.desc.title, self.desc.host.name, self.desc.host.sketch
        )
    }

    fn handle(&mut self, cmd: &str) -> StepOut {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.as_slice() {
            ["run"] => {
                let results = run_aftercare_checks();
                let challenges = aftercare_challenges();
                let lines: Vec<String> = challenges
                    .iter()
                    .zip(&results)
                    .map(|(c, ok)| {
                        format!(
                            "  [{:?}] {} — {}",
                            c.class,
                            c.name,
                            if *ok { "pass" } else { "FAIL" }
                        )
                    })
                    .collect();
                self.results = Some(results);
                StepOut::go(format!(
                    "the suite ran against the real twins:\n{}\n`judge` submits it",
                    lines.join("\n")
                ))
            }
            ["sabotage", n] => {
                let Ok(n) = n.parse::<usize>() else {
                    return StepOut::go("usage: sabotage <challenge number 1..=4>");
                };
                match &mut self.results {
                    Some(results) if (1..=4).contains(&n) => {
                        results[n - 1] = false;
                        StepOut::go(format!(
                            "challenge {n} now reports failure — sabotage is a \
                             diagnostic, and the judge will name it"
                        ))
                    }
                    _ => StepOut::go("run the suite first (and pick 1..=4)"),
                }
            }
            ["judge"] => {
                let Some(results) = &self.results else {
                    return StepOut::go("nothing to judge — `run` first");
                };
                match judge(&aftercare_challenges(), results) {
                    Ok(verdict) => {
                        if verdict.passed() {
                            StepOut::pass(format!(
                                "the gate PASSES: {} validation + {} adversarial \
                                 challenges, 0 failures, both halves present and \
                                 run. All five hosts concur — the review board's \
                                 verdict is the conjunction, and it holds. That \
                                 is aftercare: the deliberate work that makes the \
                                 next sitting safe.",
                                verdict.validation_run, verdict.adversarial_run
                            ))
                        } else {
                            let failed: Vec<&str> = verdict
                                .outcomes
                                .iter()
                                .filter(|o| !o.passed)
                                .map(|o| o.name)
                                .collect();
                            StepOut::go(format!(
                                "the gate FAILS on: {} — one failure anywhere is a \
                                 failed gate, stated with its name. `run` again to \
                                 re-verify honestly",
                                failed.join(", ")
                            ))
                        }
                    }
                    Err(e) => StepOut::go(format!("the suite is unjudgeable: {e:?}")),
                }
            }
            _ => StepOut::go("commands: run · sabotage <n> · judge · menu/q"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::run_scripted;

    #[test]
    fn every_study_route_reaches_its_success_state() {
        for id in 1..=8u8 {
            let mut session = study_session(id).expect("session exists");
            let (success, transcript) = run_scripted(session.as_mut(), study_script(id));
            assert!(
                success,
                "S{id} scripted route must succeed:\n{}",
                transcript.join("\n")
            );
        }
    }

    // ---- adversarial ----

    #[test]
    fn s2_wrong_variants_fail_the_desk() {
        let mut session = study_session(2).unwrap();
        let (success, _) = run_scripted(
            session.as_mut(),
            &[
                "answer admit",
                "answer admit",
                "answer admit",
                "answer admit",
                "answer admit",
                "answer admit",
            ],
        );
        assert!(!success);
    }

    #[test]
    fn s4_dropping_the_device_first_never_completes() {
        let mut session = study_session(4).unwrap();
        let (success, _) = run_scripted(session.as_mut(), &["render", "drop 0", "dropall"]);
        assert!(!success);
    }

    #[test]
    fn s8_a_sabotaged_suite_fails_the_gate() {
        let mut session = study_session(8).unwrap();
        let (success, _) = run_scripted(session.as_mut(), &["run", "sabotage 3", "judge"]);
        assert!(!success);
    }
}
