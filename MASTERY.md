# hands-on — MASTERY arcs (IDEA-0093, design document)

Date: 2026-10-26. Status: **design only** — this file proposes the
mastery layer above the 17-level core map (Playroom P1–P9, Study
S1–S8, in CURRICULUM.md). It builds nothing, renumbers nothing,
and casts nobody: host assignment belongs to the cast pass
(CAST.md), and merging this file's content into CURRICULUM.md is
the reconciliation step (checklist at the end).

## Why this layer exists

The core map teaches the estate's systems at encounter depth:
you play P3 and you *feel* that the schedule is the program; you
play P6 and you can classify a construct at the moc3 border. The
Owner's ruling (2026-10-26) is that Hands-On must go further —
it bakes in **mastery**: of Bevy, of the tooling being built for
the D→Rust Torogao port, of the moc3 / animation / rendering
tooling across Vulkan, OpenGL, OpenCL/Mesa, and CUDA, and —
ruled the same day, 13:05–13:06 — of the haptics integrations
(Buttplug.io and the Lovense API) and the cross-platform Rust
UI work that ships them (Arc D). Mastery
here is not "more levels." It is the same subjects, revisited at
depth, until the student can do the estate's real work — because
Hands-On is also the Owner's evaluation surface for Torogao:
what is prototyped and judged here graduates, and what is
mastered here is the standard it graduates to.

## How the arcs are built (the spiral)

Each arc is a **spiral**, basics first, per the Owner's standing
ramp rule. A subject is met four times:

1. **Encounter** — inside a core level, by playing it (the core
   map already does this; the arcs name the encounters).
2. **Guided practice** — small exercises with the training
   wheels on: the shape is given, the student fills the core.
3. **Mastery module** — the real work at production depth,
   against the estate's real artifacts and gates.
4. **Capstone** — one integrative build, judged both ways
   (below).

Every module states its prerequisites. **Nothing advanced may
assume an unlisted prerequisite** — if a module needs a fact,
an earlier module (or a core level) teaches it, or the module
teaches it itself and says so.

Every module also states its two judges, because Hands-On has
two:

- **The Owner judges by playing** — mechanic feel, animation,
  sound. His verdict is the one that has always governed this
  estate's media: a gate number never overrules his eye or ear.
- **The automated gate judges** — tests, twin-run hashes,
  benchmarks, differential comparisons. Exact counts, never
  rounded.

**Mastery is claimed only when both pass.** A module whose gate
is green but whose play verdict is *strike* is not mastered; a
module that feels wonderful but cannot prove its hash is not
mastered either. Verdicts are recorded per module in the same
shape as the cast records' Evaluation blocks (`pending — Owner
play` until he plays it).

Scope marks: **CORE** = inside the 17-level ramp's reach (an
encounter, or guided practice the core map can grow without new
slots). **ADVANCED** = post-core specialization. **CAPSTONE** =
the integrative proof. Where an arc needs slots the core map
does not have, they are listed at the end as PROPOSED additions
— the core map is not silently extended.

The discipline throughout is **Daddy Style** (SPEC §7.4): strict
requirements, known in advance, applied consistently — follow
them and the work thrives; break them and the poor outcome
arrives mechanically, as consequence, not punishment. Several
modules below are that sentence wearing work clothes: the
license gate, the clippy burndown, the Vulkan pin.

---

## Arc A — Bevy mastery

P3 Stage Systems is the encounter: Vega's club scene, systems
ordered on a text board, queries and change detection declared,
the fixed-timestep schedule as a contract. The arc grows that
one level into full Bevy fluency — the stack Strobe is built on
and the stack Torogao's app layer rhymes with (winit/wgpu/egui
there; the ECS discipline transfers even where the crates
differ).

Encounter (CORE, already in the core map): **P3 itself** —
ECS ordering, declared queries, change detection, the
FixedUpdate discipline felt as scene stability under load.

### Guided practice

| Module | Builds on | Exercise (play / code) | Owner judges | Gate judges | Torogao / Strobe transfer |
|---|---|---|---|---|---|
| A1 ECS mental model | P3 | Rebuild P1's feedback routing as entities, components, and systems in a headless Bevy app: stimulation is a component, the ladder is a system, the readout is a system | Whether the scene *reads* as the same machine he played in P1 — same beats, same responses | World-state tests after N fixed ticks; component inventory matches the design table | Strobe scene data design; torogao-engine's model thinking |
| A2 App & Plugin architecture | A1 | Split the club scene into plugins (Input, Rig, Physics, Haptics, RenderPrep) with declared dependencies, loaded in P3's order | The schedule board from P3, now real: does the scene still hold together when he reorders a plugin on purpose? | Plugin registration test: no duplicate systems, no missing dependency, load order deterministic | torogao-app feature organization; Strobe's plugin split |
| A3 Schedules: FixedUpdate vs Update | A2, P2 | Move the simulation into FixedUpdate; presentation interpolation lives in Update; run the same input at three render rates | Motion smoothness at each rate — the sim must feel identical, only the presentation changes | Replay test: fixed-step traces byte-identical across render rates | torogao-engine fixed-step evaluation; Strobe beat sync (P2's twin, at depth) |
| A4 Queries, filters, change detection | A1 | Rewrite the scene's queries with `With`/`Without`/`Changed`; make one system react *only* to changed profiles | Her response timing: change-driven reactions must land on the tick the change happens, not a frame late | Change-detection tests counting exactly which ticks fire | Engine dirty-flag evaluation; reaction-profile plumbing |
| A5 Events & messages | A4 | Replace direct calls with events: input events in, climax events out; buffered vs immediate, ordering made explicit | Event *feel*: chained reactions (unlock → sound → face) in the right order, no double-fires | Event-ordering tests; no event lost or duplicated across a soak run | torogao-haptics router event sources; MCP tool-call events |
| A6 Resources & time | A3 | Global state as resources: the clock, the profile store, the session; explicit defaults and reset between runs | A restarted session must feel like a fresh room — no residue of the last sitting | Resource default/reset tests; two sequential sessions produce identical traces from identical inputs | torogao-app session state; live-mode entry-state discipline |
| A7 Assets & hot paths | A2 | Load the scene's assets through a loading state: handles, strong/weak, what happens on a missing asset | The loading moment itself — honest progress, no frozen frame pretending | Headless asset-load tests incl. the missing-asset typed failure | Pack/scene loading in torogao-cli and torogao-app |
| A8 Hierarchy & transform propagation | A1 | Parent rig parts into a hierarchy; propagate transforms; break a parent on purpose and read the wreckage | Whether follow-through still reads as attachment, not pasting (the cowbell lesson, in ECS form) | Transform-propagation tests down a 4-deep chain | INP node trees: parent deformers, part hierarchies |
| A9 Spawn / despawn discipline | A5, A8 | The crowd: spawn and despawn under rising event load (P3's crowd, mechanized) without leaks or panics | Crowd life at load — no pops, no ghosts, no frozen extras | Entity-count invariant over a long soak; despawn leaves zero orphans | Scene object lifecycle in Strobe and the Playroom |

### Mastery modules

| Module | Builds on | Exercise (play / code) | Owner judges | Gate judges | Torogao / Strobe transfer |
|---|---|---|---|---|---|
| A10 Sprites, materials, custom WGSL | A7, C10 | A custom shader material on a scene sprite — the House Cel toon contract as a Bevy material | The look: on-lane, eye rule intact, at scene distance | Shader validates (naga); golden-image diff within stated tolerance | Strobe's visual identity; Torogao stage materials (P9's specs made real) |
| A11 Bevy UI | A2 | Rebuild P3's schedule board as real Bevy UI: drag systems between sets, watch the scene answer | Board usability — the tool must feel like an instrument, not a form | Interaction tests via simulated input events | torogao-app's Nerd-mode panel thinking (egui there; the discipline transfers) |
| A12 Audio in Bevy — cues, then the reaction engine | A5 | First the encounter form: wire scene audio cues to events (beat, unlock, climax, fault), scheduled by decision, not by side effect. Then the thread's runtime half, in prototype form: a profile-driven **reaction-audio engine** — layered loops tracking the accumulator (breath rate and pitch, wetness) plus event one-shots (a synced vocal reaction per physical event), all on the **same fixed-step clock as the visuals**, so eyes, body, and voice cannot drift apart | The sound itself: cue character and timing — and, for the engine, the reaction system's two cast-wide tests: is the buildup *audible*, and does the voice peak with each event, then trail off? (his ear is the gate) | Headless tests asserting cue *scheduling decisions* and engine state→mix decisions — no audio device required to prove the logic | Addition D audio discipline (locked voices, loudness manifests); the cast-wide reaction system (CAST.md); Strobe audio |
| A13 Input systems | A3 | One input map for the scene: keyboard/gamepad unified, rebinding, input as recorded data | Control feel — the squeeze rhythm from P1 must survive the move to real input plumbing | Input-replay tests: recorded stream reproduces the session trace | Playroom input design; Strobe controls |
| A14 State machines in Bevy | A6, S7 | Scene states (Loading / Playing / Paused / Fault) with `OnEnter`/`OnExit` cleanup; the fault state is P5's stop, scenewide | Fault behavior: the stop must feel total and instant, the recovery deliberate | Transition-matrix test: every state pair, cleanup verified | Live-mode tracking states; the Playroom's entry/exit state discipline |
| A15 Headless testing of apps & schedules | A2–A9 | A `MinimalPlugins` harness that runs the whole club scene with no window and asserts schedule behavior | Nothing to play — he judges the *report*: are these the assertions that would catch the failures he has seen? | The harness suite itself, green, in the crate's gate run | The estate gate discipline, in Bevy form (S8's subject, applied) |
| A16 Determinism & replay | A15, A3 | Record a full session's input stream; replay it; hash the world trace | The replay must *be* the session he played — same faces, same beats | Twin-run hash identical ×3 (the RP2 gate's shape) | RP2's deterministic-replay gate; P4's twin-run gate grown up |
| A17 Diagnostics & profiling | A15 | Tracing spans on every system; profile the crowd scene; find the expensive system and prove it with numbers | Readability of the profile story: can he see *why* the frame costs what it costs? | Written profile with measured numbers; frame-budget assertions in the harness | Performance evidence discipline in the port gates (RP3) |
| A18 Performance: archetypes, batching, executor | A17, A4 | Measure the same scene built fragmented vs packed; batching on/off; single- vs multi-threaded executor | Whether the optimized scene still *feels* like the scene — no life optimized out of it | Benchmark deltas recorded with method; no performance claim without a measurement | Engine evaluation hot loops; Strobe crowd scenes at scale |
| A19 Dev-vs-release build discipline | A15 | The same replay under dev and release profiles; dev tools behind features; a checklist audit of what differs | Nothing to play — he judges the checklist's honesty | Identical traces across profiles; audit signed off line by line | Release discipline for every estate binary (pairs with B15 packaging) |

### Capstone

**A-CAP — the club slice (CAPSTONE).** Builds on A3, A5, A12,
A16. A Strobe-like club scene slice in Bevy: a fixed-step rig
performing on stage, the haptics readout running through the
real router contract shape, audio cues on events — recorded
once, **replay-identical across runs**. Owner judges: scene
life, rig motion, sound, the room's feel. Gate judges: replay
hash ×3, frame budgets from A17 held, haptics ceiling never
exceeded in the trace. Transfer: this *is* a Strobe vertical
slice in miniature, and the rehearsal for Torogao's Playroom —
same fixed step, same router, same audio discipline.

### The reaction-audio thread (cast-wide; Owner ruling, 2026-10-26)

CAST.md makes live sound + physical reactions **one coupled
system** for every girl — the spirit of Danbooru post
**11099555**: buildup readable visually *and* by sound, eyes
rolling at climax, each physical event drawing a synced
vocal reaction (voice peaks with the event, then trails
off). The curriculum teaches that system in two halves,
and the split is the lesson:

- **Production side — atelier's audio modality, under
  AUDIO-FACTORY.** Locked voice per girl, the state matrix,
  −16 LUFS masters, manifests, the Owner's ear as admission
  gate. This is settled estate architecture (accepted
  2026-10-22, reaffirmed 2026-10-25): audio folds into
  atelier — **not a new MCP**. Arc B's registry/licensing
  discipline is what the production side inherits; no module
  here re-opens the server question.
- **Runtime side — the reaction-audio engine.** A12 carries
  it: layered loops whose parameters track the reaction
  profile's accumulator, event one-shots fired from the same
  event stream the visuals consume, one fixed-step clock for
  eyes, body, and voice. The engine consumes what the
  production side admits; it synthesizes nothing of its own
  beyond the prototype blips it starts from.

Scope, honestly: **design plus prototype slices** in this
arc — fakes and readouts first (a printed mix decision is
playable evidence before a speaker is), the production batch
and the live engine landing as later, separately gated work.
The evaluation integration below scores the thread both ways,
like everything else: his ear on the slices, the gate on the
scheduling and determinism proofs.

---

## Arc B — Port & tooling mastery

The port is real work happening on a schedule (PORT-PLAN
RP0–RP5), so this arc is keyed to it: each module names the RP
phase it tracks, and the lessons land as the port reaches them
— the student studies the phase, then watches (or lends a hand
to) the phase's actual gate. The D build is frozen as the
oracle; the core skill of the whole arc is *reading a system
you did not write, in a language you do not write, well enough
to rebuild it and prove the rebuild*.

Encounters (CORE, already in the core map): **S1** (format
parsing — the oracle's home territory), **S2** (the fail-closed
license gate), **S4** (resource ownership), **S6** (MCP/A2A at
encounter depth), **S8** (the gate suite as aftercare).

| Module | Keyed to | Builds on | Exercise (play / code) | Owner judges | Gate judges | Torogao / Strobe transfer |
|---|---|---|---|---|---|---|
| B1 Reading D I — modules, ranges, slices | RP1 | S1 | Take one small oracle CLI module; write its behavior contract in plain Rust-doc form: inputs, outputs, refusals — spec extraction, no D written | The contract's readability: could he rebuild from this description alone? | Contract checked against the oracle's actual outputs on the fixture set | Every RP1 tool port starts exactly this way |
| B2 Reading D II — UFCS, templates, contracts, GC semantics | RP1 | B1 | Mark up a module for what D hides: UFCS call chains resolved, template instantiations named, `in`/`out` contracts listed, and every place the GC quietly owns memory Rust must own explicitly | Nothing to play — he judges the markup's completeness against spot-checks | Allocation/contract inventory diffed against a reference inventory; nothing unmarked | torogao-format parser design (ownership made explicit) |
| B3 The D toolchain as tooling — LDC, dub, serve-d | RP1 | B1 | Build the frozen oracle; drive serve-d headlessly: initialize, symbols, hover on a known file | The probe transcript: does the tool answer like a colleague who has read the code? | Probe outputs match expected symbols/docs for the probe file | Reading any estate D artifact; the serve-d parity saga is the case study |
| B4 The editor gate | RP1 | S8 | Run the standing gate on a crate: headless Neovim, live config, login shell, bacon-ls **and** rust-analyzer attached, diagnostics settled to zero | The gate *report* — both servers named, zero diagnostics, evidence pasted, not summarized | Gate log clean under the estate's evidence shape | Every sandbox→primo mirror step, every worker brief's gate clause |
| B5 Differential testing against the oracle | RP1 | B1, S8 | Build a golden-file harness for one CLI verb; run it against the D binary and a Rust twin; compare | The harness's failure messages: when outputs differ, does the diff tell the story at a glance? | Byte-identical outputs where the format promises byte stability; honest typed failures elsewhere | The RP1 gate itself (90/150/141 + MCP + A2A, run unchanged) |
| B6 INP1 format parity | RP1 | B5, S1 | Round-trip the fixture corpus through a format implementation: parse, serialize, compare bytes | Nothing to play — he judges the corpus report (which fixtures, which bytes) | Byte-identical round trips across the corpus; adversarial truncations refused typed | `torogao-format` — INP1 stays the format |
| B7 The INP2 traps (ADVANCED) | RP2 | B6 | INP2 is specified and implemented on upstream main with **zero adoption**: little-endian where INP1 is big-endian, and still no version field. Classify format generation from raw bytes; refuse the wrong generation loudly | The refusal messages: a trap named is a trap disarmed | Classifier tests incl. adversarial headers (INP1 magic with INP2 body and vice versa) | Format-version judgment in the importer, forever |
| B8 Engine evaluation semantics | RP2 | B5, C2 | Replicate one fixture rig's evaluated parameter outputs in Rust, against the D engine's recorded outputs | The rig's *motion* in both engines, side by side: same numbers must read as the same movement | Outputs within recorded tolerances; deterministic replay byte-stable ×3 (the RP2 gate) | `torogao-engine` model + evaluation |
| B9 moc3 importer — byte work (ADVANCED) | RP2 | B6, S2 | Parse a real moc3 with the estate importer: header, section table at 0x40, per-version count tables, struct-of-arrays schemas; print its census | The census as a document: could a rigger read this and know the model? | Counts match the known-good census for the fixture class; malformed files refused typed, never a panic | lumen rig-import (R5a) — the importer's structures are the exporter's, inverted |
| B10 moc3 exporter & mapping judgment (ADVANCED) | RP3 | B9, P6 | P6 taught classification; here it is production: classify a construct list (EXACT / LOSSY / NO-MAP), then export a synthetic rig and read it back with **two** parsers — ours, and the Ayagami parser as a library. The §3.3 axis trap is the centerpiece: native lattices vary the last axis fastest, moc3 the first — a writer that forgets permutes silently | The export *report*: every loss named per construct, nothing silently downgraded (D-1..D-6 are the house pattern: typed refusal over quiet damage) | Readback counts exact in both parsers; determinism: two exports byte-identical by sha256 | lumen R5c exporter (its E1–E4 phases are this module's shape) |
| B11 The license gate, at production depth | RP1–RP3 | S2, B9 | S2 was the encounter; this is the gate as built: per-model license check, fail-closed, **before any model byte is read**; a doctored-header gauntlet | The refusal experience: the gate explains itself like the router's stop — what rule fired, what would pass | Every adversarial case refused typed; zero model reads on refusal (proven by instrumentation) | Importer/exporter gates everywhere; the Cubism EULA refusal is the standing lesson in *why* |
| B12 CLI design | RP1 | B5 | Add a verb to a practice CLI behind its harness: exit codes honest, machine-readable output stable, help text a contract | Help text and error messages as prose — could a stranger drive this tool from its own words? | Harness passes; help/output snapshots unchanged except the intended verb | `torogao-cli` — the harness-driven surface the gates pin |
| B13 MCP + A2A surfaces (ADVANCED) | RP1, RP5 | S6, B12 | Grow S6's encounter into production: full schemas, annotations from birth, bounded tool surfaces; the registry parity rule — the 57 tools enumerate identically | Tool *descriptions* as documents: would he trust an agent holding these tools? | `tools/list` diff empty except the intended addition; protocol harnesses pass | The `torogao-cli` registry; every estate MCP server |
| B14 Strict clippy burndown | all phases | S8 | Take a seeded crate from force-warn scan to green: one full scan, a per-lint tracker, curative fixes by lint class, one lint per commit | The tracker as a story: what was actually wrong with this code, class by class? | `clippy --all-targets -D warnings` clean; tracker shows zero open classes | Every estate Rust workspace (the strict-clippy discipline, learned as a method, not a mood) |
| B15 Packaging for crates.io (ADVANCED) | RP4–RP5 | B14, A19 | Prepare this very workspace for a dry-run publish: metadata, README, LICENSE, NOTICE; the bloch lesson taught by name — a taken package name means a package variant, the binary keeps its name | The crate page as a stranger sees it: is this a tool he would install? | `cargo publish --dry-run` clean; packaging checklist complete | NTRanch publishing: the Torogao CLI publishes as `torogao`; Hands-On under its own name |

### Capstone

**B-CAP — port a module, prove it (CAPSTONE).** Keyed to RP2;
builds on B1–B6. Port one small D module to Rust and prove it
against the frozen oracle with the full evidence stack:
behavior contract (B1), differential harness (B5), harness-suite
pattern (90/150/141 is the shape, scaled to the module), the
Editor gate (B4). Owner judges: the proof *package* — could he
sign this port on the evidence alone? Gate judges: every layer
green. Transfer: this is RP work itself, at training scale —
the graduate has done the port's core move once, completely.

---

## Arc C — Animation, rendering, GPU

This arc teaches the machinery under the other arcs: how a rig
evaluates, how a frame gets drawn, and what the hardware
underneath is actually doing. It starts from the estate's
adopted animation route, climbs the rig semantics, and ends at
the metal — with the framing kept technically honest
throughout, because backend superstition is how machines get
hung (C17 is not a metaphor).

**The bar this arc animates to** (Owner ruling, 2026-10-26):
the motion grammar of Danbooru post **12068969** and post
**9228588** — the 2D standard; 9228588 is the animation
study's named ahegao case — and, for the sprite
representation, post **6277975**. And the representation rule
that goes with the bar: a girl's representation may be 2D,
3D, or sprites per level — **the style is the constant**
(buildup readability, spring follow-through, eye behavior,
climax choreography), not the medium. Every route below is
taught as a way of holding that grammar in a different
medium; a route that cannot hold it is the wrong route for
that level, whatever its hashes say.

### Route A — deterministic code animation (adopted policy, CORE)

**C1 — codeanim as the default route.** Builds on P1, P4.
Code animation is **adopted estate policy, not a candidate**:
the spike of 2026-10-22 proved it on the corpus toolchain
(layered art + parameter-driven deformation, rendered through
ffmpeg) — **13/13 outputs byte-identical** across runs, roughly
**30× cheaper** than the Wan I2V render it was measured
against, **CPU-only**, zero license surface. Its territory, by
ruling: game **idle/ambient loops**, attract/menu loops, the
corpus ambient-loop subclass, and **Live2D-style parameter
animation as a format** — the primitives *are* the Live2D model
(parameters deforming layered art), minus the proprietary
runtime. Identity is perfect by construction: the same pixels,
remapped.

The boundary is taught verbatim, because it is a boundary of
physics, not of effort: **a warp remaps pixels that exist; it
cannot create the pixels a contact scene needs between poses.**
Explicit contact scenes are **NOT viable** in codeanim — gesture
sequences and expression/register changes beyond the authored
layers belong to other routes (rig evaluation below, or
generated video under its own lane rules).

- Exercise: reproduce a corpus ambient loop from its manifest —
  the manifest names the layers, the parameter curves, and the
  **toolchain versions, which are part of the recipe** (byte
  identity holds per toolchain, and the module teaches that
  scope honestly).
- Owner judges: the loop's life — does the ambient motion read
  as presence, per the Séraphine standard (a character idle must
  visibly change posture/expression; ambient presence has its
  own, lower bar, and he knows both)?
- Gate judges: twin-run byte-identity across the exercise set,
  in the spike's 13/13 shape; manifest completeness.
- Transfer: the default production route for idle/ambient in
  Strobe and Torogao's Playroom — and the reason P4's loops are
  authored as parameter tables in the first place.
- **Hands-On application, pending evidence:** a dedicated trial
  — codeanim on Betsy's layered House Cel art (idle, expression
  climb, the milking loop) — is running in parallel with this
  design. This module cites it as the pending evidence for the
  Hands-On application and does not duplicate it; P1/P4
  exercises adopt its verdict when it lands.

### Rig semantics

| Module | Builds on | Exercise (play / code) | Owner judges | Gate judges | Torogao / Strobe transfer |
|---|---|---|---|---|---|
| C2 Rig evaluation on the fixed step (CORE) | P1, P4, S3 | Hand-evaluate a tiny rig for one tick — parameters → bindings → deformers → drawables — then diff your frame table against the core's | The evaluated motion at three parameter poses: does the order of operations *show*? | Frame-table equality with the reference evaluation | `torogao-engine` evaluation order; the milking-core authoring twins |
| C3 Keyforms & interpolation (CORE) | C2 | Author a 2-axis keyform lattice; predict a blended form before running it. The semantics are the extracted ones: multilinear blending across axes, **no extrapolation**, the 0.001 snap | Blend *feel* at the lattice corners vs the middle — mushy middles are a design smell he can see | Predicted form within stated tolerance of the evaluated form | `torogao-format` bindings (P6's subject, at mechanism depth) |
| C4 Deformers: rotation & warp (ADVANCED) | C3 | Build both deformer kinds on a test mesh: rotation (single scale + reflect flags — anisotropic scale has no faithful target, cf. MAPPING D-3) and warp (grid deformation, barycentric inside cells) | Deformation quality: volume preserved, no candy-wrapper twist where none was authored | Vertex outputs match the reference implementation within tolerance | Engine deformers; RP2 evaluation work |
| C5 Glue (ADVANCED) | C4 | Glue two meshes and study the seam: glue is a **post-deformation fixpoint** (iterate until the seam settles), and its compatibility field is **unclamped** — importers must not clamp it and call that fidelity | The seam in motion: invisible at rest is not invisible at the extremes | Seam error measured at parameter extremes, reported as a number | Importer completeness — the R5a importer's known glue gap (3 of 4 official samples) is the case study |
| C6 SimplePhysics semantics (CORE) | C2 | Tune a hair strand (the crate's springs, grown up): pendulum model, strands and groups, input/output parameter mapping — and the warning that physics output is frame-rate-shaped in the wild while the oracle's semantics are the estate's reference | Hair *believability*: settle, overshoot, and lag as character, not noise | Simulated trace inside the predicted envelope (period, decay) | Physics in the Playroom scenes; the physics3 sidecar mapping (MAPPING §9, decisions D-5/D-6) twins this |
| C7 moc3 versions & draw rules (ADVANCED) | C3, B9 | Version law: count tables grow V3_0 → V5_3; blend modes are 3-of-18 at V3_0 with the rest gated to the V5_3 blend word; per-form multiply/screen colors need V4_2+; blend shapes are V5 parameters. Draw order is **computed from interpolated depths**, not authored — and the exporter's flat-group rescale ((d + 10000) / 20) is the bridge between the two worlds | Nothing to play — he judges the version-gate table as a reference document | Construct classification exact against the MAPPING tables | Exporter judgment (B10's law library); P6 at full depth |

### Pixels and shaders

| Module | Builds on | Exercise (play / code) | Owner judges | Gate judges | Torogao / Strobe transfer |
|---|---|---|---|---|---|
| C8 Texture atlases & color space (CORE) | P7 | Pack an atlas; find the planted faults: UV bleed from missing padding, an sRGB/linear mixup, a premultiplication error | The corrected render at arm's length — color faults are eye faults first | Faults identified by name; corrected output matches the reference render | The 8K texture target (RP3); sprite bakes (P7); every texture the factory emits |
| C9 Rendering math (CORE) | C2 | Transforms (model/view/projection), rasterization, alpha blending — including blending's order-dependence — computed by hand for a two-triangle scene | Nothing to play — he judges whether the worked example *explains* a glitch he has seen in real output | Hand computation matches the software reference, value for value | Every renderer the estate ships; the vocabulary of every later module |
| C10 Shaders I — WGSL & naga (ADVANCED) | C9 | Write the House Cel toon contract as a WGSL material: hard shadow shapes, rim, the eye treatment as texture+geometry discipline | The look on a real sprite: on-lane or not, his call | naga validation clean; golden-image diff within tolerance | Bevy custom materials (A10); Torogao stage materials; P9's material specs |
| C11 Shaders II — GLSL & SPIR-V (ADVANCED) | C10 | One shader, two roads: WGSL → SPIR-V via naga, GLSL → SPIR-V via glslang (the primo toolchain: glslangValidator, spirv-val, spirv-cross — present and verified); compare what the compiler did | Nothing to play — he judges the comparison write-up: where did the two roads diverge, and does it matter? | Both SPIR-V modules validate; rendered outputs match | Toolchain literacy for any GL/Vulkan target the estate touches |
| C12 wgpu — the abstraction and its cost (CORE) | C9 | Clear-and-triangle twice: once through raw wgpu (surface, adapter, device, queue, pipeline), once through Bevy (A10's road). wgpu is the port's chosen stack precisely because it lowers to Vulkan / Metal / D3D12 / GL per platform — the module teaches what "lowers to" means, layer by layer | The two code paths side by side: what did the abstraction buy, what did it hide? | Both paths render the reference frame | `torogao-app`'s renderer (winit + wgpu + egui) — the app's whole graphics story |

### Backend literacy — the honest map

Framing law for this whole section, stated once and applied in
every module: **an API, a platform, a vendor stack, and an
implementation are four different things, and confusing them is
how wrong code gets written.**

- **Vulkan** is an explicit API: you name your resources,
  your synchronization, your memory — and validation layers
  check your work.
- **OpenGL** is a state-machine API: a global context you
  mutate; older, looser, and still load-bearing where
  compatibility contexts and WebGL/ANGLE live.
- **CUDA** is NVIDIA's compute **platform** (language, runtime,
  drivers): memory model, kernels, streams. It runs on NVIDIA
  silicon, full stop — **CUDA does not run through Mesa, and
  does not run on Intel silicon at all.** Its interop routes
  (Vulkan external memory/semaphores, GL–CUDA interop) are how
  CUDA work *meets* graphics work, not a way to smuggle it onto
  other hardware. The estate's record carries the cautionary
  tale: ZLUDA is absent on primo and not a plan (the wiring
  report's verdict).
- **OpenCL** is a heterogeneous compute API: one kernel
  language, many vendors' devices. On primo its measured map
  is specific (C16) — "OpenCL works" is not a sentence; "NEO
  works, Rusticl enumerates zero devices" is.
- **Mesa** is an **implementation stack** — Gallium, ANV
  (Intel Vulkan), iris (Intel GL), Rusticl (OpenCL), Zink
  (GL-on-Vulkan) — that *implements* the APIs above on open
  drivers. Mesa is not an API; "runs on Mesa" means an
  implementation detail, and which detail matters is the
  module's whole point.
- **wgpu** (C12) is the Rust abstraction the estate builds
  on; this section is what it abstracts *away*, taught so the
  abstraction never becomes a superstition.

| Module | Builds on | Exercise (play / code) | Owner judges | Gate judges | Torogao / Strobe transfer |
|---|---|---|---|---|---|
| C13 Vulkan literacy (CORE) | C12 | Resources, synchronization, validation layers — then a real, **pinned** ANV compute dispatch in the wiring report's harness shape (device selected by vendor ID, readback compared) | The validation output as a text: can he read what the layer is accusing the code of? | Dispatch readback exact — 0 mismatches, the wiring report's own bar | Debugging wgpu's Vulkan path when it misbehaves; primo GPU work generally |
| C14 OpenGL literacy (CORE) | C9 | The state machine, core vs compatibility contexts, and where GL still matters: the **D oracle's engine renders OpenGL** — you cannot read RP2/RP3's subject without this. Trace one frame's state changes in that renderer | The state trace as a narrative: does the frame's story make sense in order? | State-trace exercise exact; a compatibility-context run completes | Reading the vendored inochi2d renderer during the port (B8's other half) |
| C15 CUDA literacy (ADVANCED) | C9 | Memory model, kernels, streams on the A6000 (NVIDIA, pinned by UUID); then the interop lesson: move a kernel's output toward a Vulkan-visible buffer via external memory/semaphores — or document, with measurements, why the simpler route wins for the estate's shapes | The interop write-up: is the honest route the one he would have guessed? | Kernel output parity against the CPU reference | Knowing when CUDA is the right tool vs a vendor-locked detour (the H3-parity benches run here) |
| C16 OpenCL & Mesa literacy (ADVANCED) | C13 | Enumerate primo's OpenCL and explain **every line**: NEO on the iGPU **works** (measured ≈ 358 GFLOP/s on the wiring report's FMA bench); POCL on CPU works; **Rusticl is BLOCKED — platform loads, zero devices on Mesa 26.2.5, root cause not established at user level; do not rely on it until re-verified**; clvk is broken as installed (its clspv path points into a deleted build tree). Then place each Mesa component (Gallium, ANV, iris, Rusticl, Zink) on the map | The device-selection exam: three workload briefs, he picks device + API for each and defends the pick | Selections match the measured map, with reasons cited from the wiring report, not from vibes | Never writing a consumer that assumes first-device; the pin discipline of C17 |
| C17 The hybrid-graphics safety law (CORE) | C13 | Primo is a hybrid machine: the Intel iGPU drives the displays; the NVIDIA cards are compute. In 2026-10-06 an **unpinned Vulkan workload enumerated the iGPU and hung it** — Hyprland died mid-session. The law, taught as Daddy Style for hardware: **pin the device in the environment** (`VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/nvidia_icd.json` for NVIDIA work; the Intel ICD for deliberate iGPU work), **confirm the device name from the tool's own output**, preserve interactive headroom, and check the kernel log after iGPU-touching work. The exercise runs one real workload pinned, and one *enumeration* unpinned to see the trap's shape — the dangerous run itself is never reproduced | The rule, recited back with its reason — he is the person whose session dies when this is forgotten | Pin verified in a real run log (device named by the tool); kernel log clean after | Every primo GPU job in the estate, without exception |

### Capstones

**C-CAP1 — two backends, one rig (CAPSTONE).** Builds on C2,
C12, C13/C14. Render the same rig scene through two backends
(e.g. wgpu→Vulkan and wgpu→GL, or the engine render against a
codeanim Route A render of the same parameter table) and diff
the frames: per-pixel diff statistics from the gate, the
contact sheet to the Owner. A diff is not automatically a
failure — the module's lesson is *reading* one: which
differences are backend truth (blending order, precision) and
which are bugs wearing a backend's clothes.

**C-CAP2 — one effect, two implementations (CAPSTONE).**
Builds on C6, and one of C13/C15/C16. Implement one compute
effect — a spring/particle solver, the crate's own chest-spring
family grown up — twice: a CPU reference, and one GPU route of
the student's choosing (Vulkan compute, CUDA, or OpenCL/NEO),
the choice defended in writing from the measured machine map.
Gate judges: parity tests within a stated tolerance, and the
CPU reference's determinism (the GPU route is compared *to*
something true). Owner judges: the effect in motion — the
solver serves a scene, and the scene is his to rule on.

---

## Arc D — Haptics & cross-platform Rust UI

This is the **fourth arc** (Owner ruling, 2026-10-26) — a
fourth arc rather than a thread woven through Arc B, because
its subject is one stack end to end: a stimulation signal
leaves the deterministic core, crosses the router contract,
speaks two device ecosystems' protocols, and is driven from a
Rust UI that must ship on Linux, Windows, and Android. Arc B
teaches the port's tools; Arc D teaches the *other* Rust the
estate ships — every crate and surface a Hands-On level needs
to touch real devices lawfully and present itself on real
platforms. It builds on existing assets and duplicates none
of them: the **torogao-haptics** crate (router, gate, fake
backends), the estate **lovense-mcp** (S1+S2 landed: the
8-tool contract behind its decide() gate), the PORT-PLAN app
stack (winit + wgpu + egui), and the core map's **P5 Router's
Gate** encounter.

The spiral's safety shape is stricter here than anywhere
else in the design, because the far end of this stack is a
physical device: **fakes and readouts first, always** —
device-free testing is a proven estate pattern (fake
transports in torogao-haptics; the joint-gate corrections were
proven against schemas, not hardware) — and **live hardware
stays behind the S4 rule: the Owner's explicit go, with the
Owner present.** No module below actuates anything; the one
module that even approaches a live device is a *rung*, gated
like S4 itself.

Encounters (CORE, already in the core map): **P5** (the
router contract as play), **P8** (motion becomes a haptic
track), **S5** (the same router as a Rust concurrency
problem — bounded queues, watchdog, the synchronous stop).

### Guided practice

| Module | Builds on | Exercise (play / code) | Owner judges | Gate judges | Torogao / Strobe transfer |
|---|---|---|---|---|---|
| D1 Transport thinking & fakes | P5, S5 | The `TextTransport` seam in torogao-haptics: a backend is a text channel plus a contract. Drive the fake backends through scripted scenes; write a new fake that misbehaves on cue (drops frames, answers late) and watch the router refuse it | The readout under sabotage: does the failure *read* as the failure it is? | Fake-transport gate tests: stale frames refused, malformed lines typed errors, zero real devices touched | Every device surface Torogao ever grows — the seam is where they all plug in |
| D2 Buttplug.io — protocol & the official crate | D1 | The production shape, read and exercised against fakes: the official **`buttplug` Rust crate (BSD-3-Clause) as an Intiface client** — protocol v3 `ScalarCmd` / `StopAllDevices` — living on a **bridge-worker thread** behind the crate's transport trait, because tokio never enters a fixed-step path. Why the ecosystem's dominant architecture is client-side (apps speak to Intiface Central; no BLE in app binaries) is taught as the reason, not a footnote | The architecture as a diagram he can interrogate: where can a signal be born, where can it be stopped? | Protocol-shape tests against the fake Intiface: emitted command bodies exact; a stop emits `StopAllDevices` and nothing after it | Torogao's Buttplug backend, production form (the haptics DESIGN's adopted dependency, learned as a method) |
| D3 The router & gate discipline, at depth | D2, P5 | torogao-haptics' output router as law: master switch **default OFF**, ceiling clamped in the router (never trusted to callers), generation-token refusal of stale frames, caller-clock 5 s watchdog, synchronous total stop, **no resume on reconnect** — each rule broken on purpose against the fakes, each breakage refused | The refusals as rulings: each one names the rule that fired, in words a tired person could act on | The crate's gate suite extended by the student's own adversarial cases; ceiling never exceeded in any emitted trace | The safety surface every Torogao/Strobe output crosses — P5's lesson at production depth |
| D4 The Lovense contract | D1, D3 | The client contract the estate's **lovense-mcp** implements: exactly **8 tools**, a pure **decide()** gate (wire bodies exist nowhere else), the arm latch, the watchdog that fires during idle silence, and **stop as a panic latch** — zeroing *and* latching off, recovery only by a fresh explicit arm. The LAN-route concepts underneath: the Remote app's local Game Mode route holds no credentials in the server; the joint gate's corrections are the case study (real argument names `feature` / `time_sec`; a zero frame is a strength-0 call with a bounded duration, **never** the stop tool) | The contract as a document: could he predict, from the schemas alone, what any call would do — including the ones that refuse? | Schema-conformance tests against the server's real tool definitions; disarmed calls refused typed; the clamp visible in the emitted body | Torogao's Lovense lane, client side — and the discipline of reading a contract before trusting a summary of it (the XY lesson, in haptics form) |

### Mastery modules

| Module | Builds on | Exercise (play / code) | Owner judges | Gate judges | Torogao / Strobe transfer |
|---|---|---|---|---|---|
| D5 The live rung (ADVANCED — Owner-gated) | D3, D4 | The S4 rule taught as a module, because it *is* one: what a live session requires before it may start — the Owner's explicit go, the Owner present, the stop rehearsed on fakes first, one device, ceiling pre-set, session health watched. The exercise is the *checklist and the rehearsal*, run against fakes; an actual live session is not coursework and happens only under the rule itself, outside the module's gate | The checklist's honesty: is there a line on it he would not sign? | Checklist completeness against the lovense-mcp S4 terms and the haptics DESIGN; rehearsal trace shows stop-before-start proven | The estate's live-hardware discipline, everywhere, permanently |
| D6 Rust UI shipping I — desktop (Linux + Windows) | A2, C12 | Torogao's app stack as the textbook: **winit + wgpu + egui** — window and event loop, input, the render surface, Persona/Nerd presentation over one functional skeleton — built and run on Linux, then on Windows, with the differences (windowing, paths, packaging) named as they bite | The same screen on both platforms: one app, or two apps wearing one name? | Both builds green; the cross-platform difference list complete and each item reproduced, not rumored | `torogao-app` itself (RP4/RP5's surface) — this module is the port's UI phases, learned from inside |
| D7 Rust UI shipping II — Android | D6 | The Android path for the same stack: NDK target and toolchain wiring, APK packaging, the platform differences that are real (lifecycle, touch input, backgrounding, storage) vs the ones that are folklore | The app on a phone (his validation surface of choice): does it feel like the desktop app's sibling or its apology? | APK builds from the documented recipe; lifecycle exercise (background → foreground → resume) passes with state intact | The Torogao/Strobe Android targets — itch.io-first releases include Android; the build must exist before the release does |
| D8 F-Droid discipline (ADVANCED) | D7, B15 | The constraints as design inputs, per the estate's fdroid-publish skill: **fully open build, no proprietary dependencies**, reproducible-build discipline — then the recipe itself (fdroiddata metadata, build recipe, deterministic validation gates). The exercise audits the D7 build against every constraint and writes the recipe for it | The audit's candor: which conveniences did the open build cost, and was each cost named before it was paid? | Dependency audit clean (no proprietary or non-free component in the graph); recipe validates; rebuild reproduces the artifact | Both published lanes' Android channel (ONTrack and Light Show shipped this way; the NSFW lane's Android builds inherit the same discipline) |

### Capstone

**D-CAP — one scene, three platforms, one fake device
(CAPSTONE).** Builds on D3, D6, D7. A Hands-On haptics scene
— P5's gate or P8's timeline, student's choice — driven from
the *same* Rust UI build lineage on **Linux, Windows, and
Android**, emitting through the router to a **fake** backend:
the readout traces must agree across platforms (same scene,
same lawful emission, byte-comparable traces), and the stop
must land identically on all three. Owner judges: the scene
feels like one scene on three machines, and the Android build
in his hand. Gate judges: trace agreement, ceiling never
exceeded, stop synchronous everywhere, zero live devices.
Transfer: this is the Torogao shipping shape in miniature —
one core, one router, every platform, nothing actuated that
wasn't allowed.

---

## Evaluation integration — the two judges, summarized

| | Owner judges (by playing) | Gate judges (automatically) |
|---|---|---|
| Arc A | Scene life, motion feel, sound — incl. the reaction system's buildup audibility and per-event vocal sync — control feel, tool usability | Replay hashes ×3, schedule/world-state tests, budgets, invariants |
| Arc B | Contracts, reports, and refusal messages as documents a human must trust | Differential byte comparisons, harness suites, clippy, publish dry-run |
| Arc C | Loops' life against the named bar (12068969 / 9228588; 6277975 for sprites), deformation quality, hair believability, the look on a real sprite, diff sheets | Byte-identity, frame-table equality, readback exactness, parity tolerances, censuses |
| Arc D | Refusals as rulings, the contract as a predictable document, one scene feeling like one scene on three platforms, the Android build in hand | Emitted-body exactness, ceiling/stop invariants in every trace, cross-platform trace agreement, dependency audits, reproducible rebuilds — and zero live devices outside the S4 rule |

Rules that bind all four arcs:

- A module is **mastered** only when its row passes both
  columns. Either column alone is a partial result and is
  recorded as one.
- Owner verdicts are recorded per module (`pending — Owner
  play` until played), in the cast records' Evaluation shape —
  Hands-On judges its curriculum the same way it judges its
  girls and mechanics, because it is the same surface.
- Gate evidence is exact: test counts, hashes, measured
  numbers with their method. A benchmark without its contention
  label (the good-neighbor policy's contended/exclusive rule)
  is not evidence.
- Where the Owner's verdict and the gate disagree, the
  disagreement is the finding — it is written down, not
  averaged away. (The estate has been here before: proxies that
  passed tracks his ear rejected were demoted to diagnostics.
  The ear won. It is written into this design so it does not
  have to be re-litigated per module.)

## Scope-honesty split

| Arc | CORE | ADVANCED | CAPSTONE | Total |
|---|---|---|---|---|
| A — Bevy | P3 encounter + A1–A9 guided practice, A15, A16 (11) | A10–A14, A17–A19 (8) | A-CAP (1) | 20 |
| B — Port & tooling | Encounters S1/S2/S4/S6/S8 + B1–B6, B8, B11, B12, B14 (10 modules) | B7, B9, B10, B13, B15 (5) | B-CAP (1) | 16 |
| C — Animation/render/GPU | C1 (Route A), C2, C3, C6, C8, C9, C12, C13, C14, C17 (10) | C4, C5, C7, C10, C11, C15, C16 (7) | C-CAP1, C-CAP2 (2) | 19 |
| D — Haptics & cross-platform UI | Encounters P5/P8/S5 + D1–D4 guided practice (4) | D5–D8 (4) | D-CAP (1) | 9 |
| **Total** | **35** | **24** | **5** | **64** |

(Arc A's CORE count includes the P3 encounter itself; Arc B's
and Arc D's encounters are core-map lessons, counted once, in
the CORE row's text rather than its number.)

## Where the current core map fights these arcs — flagged, not fixed

1. **P3 is one level; Arc A needs a practice block.** P3
   teaches ECS ordering by feel, but A1–A9 assume hands-on
   Bevy code from the second module on. The encounter is
   thinner than the spiral's second turn wants. → PROPOSED X1.
2. **S4 is an on-paper exercise standing where real code now
   exists.** "Hold the Frame" teaches GPU-resource ownership
   *on paper*, while C12/A10 do the same ownership with real
   wgpu/Bevy code. Keeping both is the spiral working as
   intended (encounter → production) — but CURRICULUM's S4
   text does not currently say it is the encounter for a named
   later module. Reconciliation item, not a redesign.
3. **The moc3 border is taught at three depths with no
   cross-references.** P6 (classification, by play), B9/B10
   (production byte work), C7 (version law) are one subject at
   three spiral turns — correct by design, invisible in the
   current documents. Reconciliation item.
4. **P9's designed-only boundary fights the render arcs.**
   P9 is codified DESIGNED ONLY ("no engine work in this
   pass"), while A-CAP, C10, and C12 assume real engine and
   shader work in a 3D-adjacent home. Something has to give —
   either P9 grows a buildable slice or the capstones live in
   the proposed slots below. **Owner ruling needed; this file
   does not presume it.**
5. **S2 and B11 are the same gate at two depths, and the
   Study's S5 already twins the router the Playroom's P5
   teaches.** The twins are deliberate (the spiral), but the
   current CURRICULUM presents them as unrelated lessons.
   Reconciliation item.
6. **Sound's deep home is still late — now with a thread
   running to it.** A12 remains the only module where audio
   is a production subject, but the reaction-audio thread
   (after A-CAP) now names the destination the cast records'
   sound specs point at — atelier production under
   AUDIO-FACTORY, the profile-driven runtime engine — and
   A12 carries the engine's prototype form. The ramp from
   "blips" (SPEC §2.6) to that thread is still steeper than
   any other subject's; it is now a *mapped* steepness
   rather than an unmarked one. The remaining fix is a
   curriculum decision, not this file's.
7. **Arc D's far end is a physical device, and the core map
   meets it only through fakes.** P5/P8/S5 teach the router
   with readouts and fake backends — correctly, for a
   teaching crate — while D5's live rung is deliberately
   *not* coursework: it is the S4 rule (Owner's go, Owner
   present) taught as a checklist and a rehearsal. Anyone
   merging this arc into CURRICULUM must keep that shape:
   the core ramp never actuates, and no gate in this design
   may be read as live-hardware permission.

## PROPOSED additions — beyond the 17 (Owner ruling required)

These slots are **proposals**. Nothing here renumbers, extends,
or re-casts the core map. Host/kink coverage is stated as a
*requirement for the cast pass to satisfy*, not as casting —
CAST.md owns names.

- **X1 — Stage Crew Practicum (proposed Playroom-adjacent
  block).** Home for A1–A9: a multi-sitting practice block
  attached to P3, working Vega's club scene in real Bevy code.
  Coverage needed: a stage-crew host distinct from Vega (she
  owns the encounter; the practicum wants a working tech, not
  the manager) — service/protocol flavor fits a stagehand who
  runs the booth's checklist.
- **X2 — The Oracle Reading Room (proposed Study-adjacent,
  ongoing).** Home for Arc B's module sequence and B-CAP,
  running parallel to the real RP phases — a sitting that
  reconvenes as each phase lands. Coverage needed: an
  archivist/librarian persona for the frozen oracle (she keeps
  the D build the way Vigil keeps the router: exactly);
  correction/protocol flavor, continuous with the Study's desk
  dynamic.
- **X3 — Shader Bench (proposed Playroom-adjacent).** Home for
  C10/C11 and the practical half of C8/C9. Coverage needed: a
  light-and-surface host — a painter or lighting girl whose
  kink coverage is display/exhibition flavored (her work is
  literally being looked at, correctly).
- **X4 — Backend Gauntlet (proposed, post-core).** Home for
  C13–C17's exercises and both C capstones. Coverage needed:
  an engineer/rigger host; precision-and-constraint flavor —
  her dynamic is tolerances, pins, and rules with mechanical
  consequences (C17 is her home ground).
- **X5 — codeanim Studio (proposed-or-fold).** Home for C1's
  Hands-On exercises, *if* the parallel Betsy codeanim trial's
  verdict says the format earns a slot of its own; otherwise
  C1 folds into P4 (Loop Studio), whose authoring model it
  already shares. Coverage needed only if it stands alone: a
  frame-by-frame animator host, patience-and-repetition flavor.
- **X6 — Haptics Bench & Shipping Dock (proposed, post-core).**
  Home for Arc D: D1–D4 work at a bench attached to P5/P8's
  material (the router, the fake devices, the two contracts);
  D6–D8 and D-CAP work at the dock (the student's own app,
  three platforms, the F-Droid recipe). Coverage needed: none
  new — the bench runs on Vigil's and Noor's levels, the dock
  is the student's own build; if the bench ever wants a host
  of its own, she is a device-tech: precision-and-permission
  flavor, continuous with X4's engineer.

## Reconciliation checklist — merging into CURRICULUM after the cast pass lands

- [ ] CURRICULUM gains a "Mastery arcs" section that *links*
      MASTERY.md and summarizes the four arcs in prose; the
      module tables themselves are NOT copied (one home per
      table — this file stays their home).
- [ ] P3's entry gains the "encounter" label and a pointer to
      Arc A (guided practice A1–A9, capstone A-CAP), plus the
      X1 proposal marked as proposed.
- [ ] P4's entry gains the codeanim Route A note (C1):
      adopted policy, the contact-scene boundary stated, and
      the Betsy trial's verdict recorded when it lands.
- [ ] P6's entry cross-references B9/B10 (production depth)
      and C7 (version law) as its spiral continuations.
- [ ] P9's entry: resolve the designed-only boundary against
      A-CAP / C10 / C12 with an Owner ruling; record the ruling
      in the entry either way.
- [ ] S2's entry cross-references B11; S4's entry names C12
      (and A10) as its production continuation; S5's entry
      cross-references P5 (same router, two turns); S6's entry
      cross-references B13.
- [ ] P5's and P8's entries cross-reference Arc D (D1–D4 as
      guided practice, D-CAP as the capstone), and S5's entry
      names Arc D alongside P5 — the router's third turn.
      CURRICULUM must carry the S4 shape verbatim in spirit:
      fakes/readouts in the ramp, live hardware only under
      the Owner-go + presence rule.
- [ ] The animation bar and the representation rule (Arc C's
      framing: posts 12068969 / 9228588 as the 2D standard,
      6277975 for sprites; style constant, medium variable)
      are stated in CURRICULUM's style section alongside the
      eye rule, matching CAST.md's header wording.
- [ ] A12's entry in CURRICULUM (when the arcs are linked)
      names the reaction-audio thread: production in
      atelier under AUDIO-FACTORY (not a new MCP), runtime
      engine on the shared fixed-step clock.
- [ ] The Study's framing section gains one paragraph: the
      mastery arcs as the Study's upper school — X2 named as
      proposed, Arc B keyed to RP phases.
- [ ] Check every host named or implied in MASTERY against the
      landed CAST.md: core-map hosts must match CAST.md
      exactly; X1–X5 coverage requirements must be either
      satisfied by CAST.md's cast or explicitly deferred with
      the proposal.
- [ ] Verify no renumbering crept in: every P/S id cited in
      MASTERY.md matches CURRICULUM.md's map exactly (P1–P9,
      S1–S8), and MASTERY's totals (64 modules: 35 CORE /
      24 ADVANCED / 5 CAPSTONE) still count correctly after
      any merge edits.
- [ ] Terminology sweep after merge: Daddy Style (not the
      the SFW lane's name) wherever the discipline is taught;
      House Cel / Pony Painterly spelled as in SPEC §7; the
      eye rule quoted verbatim where it appears; Mesa never
      called an API; CUDA never described as running on
      non-NVIDIA hardware.
