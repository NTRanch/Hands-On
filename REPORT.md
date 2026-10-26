# hands-on (IDEA-0093) — Build Report

Date: 2026-10-26.
Nothing committed or pushed. No external assets, downloads, or
audio files anywhere in the package.

## What was built

A Cargo workspace codifying the estate's game-system learning
as levels, where exercise success drives live stimulation
feedback on each level's host girl:

| Piece | State |
|---|---|
| `milking-core/` — Level 1's deterministic core (fixed-step squeeze sim, bottle gauge, expression ladder, unlock affordances, keyform authoring, sensitivity/climax/reservoir system) | **Built, gated** |
| `hands-on/` — TUI binary: two-track level menu (the Playroom P1–P9, the Study S1–S8), Level trait/registry, shared feedback engine (bars, named expressions, style line, event log, fake-haptics readout clamped at 0.20) | **Built, gated** |
| Level P1 (First Milking, host Betsy) in the TUI | **Built, playable** |
| Playroom P4 Loop Studio slice (loop closure + bake + twin-run hash on Betsy, seed `betsy-loop-a1`) | **Built, playable** |
| `game.html` — Level 1 visual prototype (original SVG Betsy in the House Cel style, dual view, sensitivity dial, Studio Mode) | **Built** (opens standalone; JS syntax-checked with `node --check`; zero external references) |
| `milking.nvim.lua` + `nvim-appname/init.lua` — Neovim routes | **Built** (Lua parse-checked with luac) |
| Playroom P2, P3, P5–P9 · all 8 Study Lessons | **Designed only** (CURRICULUM.md; registry entries locked) |
| SPEC.md (design + §5 sensitivity + §6 framework/tracks + §7 style law), CURRICULUM.md (both lesson tracks, the Study, the animation levels, the style section, chapter table) | **Written** |

## Gates (nightly-2026-09-25 pin, `RUSTC_WRAPPER=` cleared)

- `cargo test` (workspace): **38 passed, 0 failed** —
  18 milking-core unit + 11 milking-core integration
  (`tests/gates.rs`) + 9 hands-on unit. Coverage includes every
  briefed adversarial case: out-of-window squeeze spills (no
  flow), mashing and over-pressure spill, stale-tick rejection,
  locked-action typed refusals, a smuggled non-side action,
  mismatched end keyform failing closure, tick-by-tick proof the
  expression ladder never skips (including the post-climax
  descent), bottle-win session closure.
- Amendment 1 regime proofs, all in the test suite: high
  sensitivity climaxes in **strictly fewer** squeezes than low;
  low regime's climax **intensity and burst strictly greater**;
  reservoir never exceeds capacity; refractory blocks re-trigger
  (climax count pinned at 1 through the window); twin-run
  `trace_bytes` byte-identical per profile and distinct across
  profiles.
- `cargo clippy --all-targets -- -D warnings`: **clean**.
- `cargo fmt --check`: **clean**.
- TUI smoke, headless and proven in this session:
  `hands-on --demo` exits 0 printing the two-track menu, a
  14-squeeze session (1 climax, 0 spills, bottle 38.3), the
  House Cel style line, a studio closure PASS, and the P4
  Loop Studio gate — *twin-run hashes `88dd763631bc07f9` /
  `88dd763631bc07f9`, identical: true*. Scripted stdin sessions
  exercised: Level 1 play, P4 PASS (`run 1000 smooth
  0=0,400=1,1000=0`) and FAIL (`…1000=0.4` → `LoopOpen { start:
  0.0, end: 0.4 }`, GATE: FAIL), Lesson S8 and Playroom P7
  locked messages, menu return, quit.
- **Editor LSP gate: not run** — standalone workdir; it is
  owed at any repo landing, per studio rule.

## Amendment log (all twelve, as applied)

1. **Sensitivity system** — `SensitivityProfile { gain,
   threshold, reservoir_capacity, reservoir_rate,
   refractory_ticks }` in the core; live dial + HIGH/Balanced/
   LOW presets in HTML; `+/-/T/t` dial keys in the TUI; climax
   intensity/burst = f(reservoir at trigger); refractory halves
   gain and blocks re-trigger; expression descends the ladder
   one step per tick (the sim was changed from settle-all-at-once
   to one-step-per-tick when a gate caught a Flushed→Neutral
   jump — the fix is the teaching point: no frame cuts).
2. **Playable from Neovim** — the TUI is the `hands-on` binary
   (Amendment 4 superseded the `milking-tui` name before it ever
   shipped). I/O choice: **plain line-buffered stdin/stdout**,
   zero added dependencies; wall-clock time between commands is
   folded into fixed ticks (capped) so tempo grading stays the
   core's. Consequence, stated honestly: piped instant input
   reads as mashing and spills — scripted play should use `w`
   (wait-one-beat) between squeezes, or `--demo`.
3. **Lane correction** — the launcher is never installed into
   the main editor config or any SFW-lane repo. Documented routes:
   (a) per-session `:luafile`, commands vanish on exit;
   (b) dedicated `NVIM_APPNAME` profile from the shipped
   `nvim-appname/init.lua`, a local-only directory. The lane
   rule is stated in the launcher header and CURRICULUM.md.
4. **Framework restructure** — workspace + level registry +
   feedback engine as SPEC §6. Level 1 built; framework built;
   L2–L5 (now L2/L3/L5 + Lab) designed.
5. **Lane scrub** — every Light Show reference removed from
   spec, curriculum, code, comments, and HTML (name, puzzle
   framing, cast, product names). The Bevy teaching is reframed
   entirely through Strobe: L3 is now **Stage Systems** (order
   the club scene's systems; host Vega). Final grep over all
   deliverables for Light Show / fiber / the old host name:
   **zero hits**.
6. **Animation Lab** — third strand in registry and menu,
   fully designed in CURRICULUM.md (girl × scene × format,
   determinism gate mechanics, EXACT/LOSSY/NO-MAP teaching
   points for the moc3 border level). Built: only the **A1
   slice** (closure + bake + twin-run hash). Playroom's old L4
   Loop Studio folded into A1; the Playroom id 4 is retired,
   not reused. Designed, not built: A2–A5 (A5 GLB/3D is
   designed-only by its own terms).
7. **Forge → the Contract** — the Rust-mastery track renamed
   everywhere user-visible: TUI menu, registry (`Track::Contract`,
   `contract_levels()`, Clause codes C1–C8), SPEC, CURRICULUM.
   Levels are Clauses; the framing is B-01 (contract/limits) —
   negotiated limits = bounded work, safeword = synchronous
   stop, explicit contracts = signatures/typed errors — and
   Clause 8 is the **Aftercare** capstone (B-80: tests, review,
   verification). Substance (Book chapter map, Rust 2024
   material, house principles, Torogao mapping, coverage table)
   unchanged by the rename. Final grep for "Forge": **zero
   user-visible occurrences**.
8. **Two tracks: the Study + the Playroom absorbs the Lab** —
   the Contract track is renamed **the Study** (Owner's pick):
   Clauses C1–C8 are now Lessons S1..S8 (`Track::Study`,
   `study_levels()`), framed as being summoned to the Study
   while the compiler corrects you at the desk; S8 stays the
   **Aftercare** capstone, and contracts/limits survive as
   Lesson S2's *topic* (typed errors & contracts), not the
   track name. The Animation Lab strand is folded into the
   Playroom: one sequence **P1..P9** — Loop Studio is P4 (its
   playable slice re-homed; the internal seed `betsy-loop-a1`
   and the twin-run hash `88dd763631bc07f9` are unchanged), and
   the former A2–A5 designs are P6–P9. `Track::Lab` and
   `lab_levels()` are gone; the registry still totals 17.
   The one test whose invariant the merge changed
   (per-level → per-host profile distinctness) was rewritten
   to the honest invariant: one consistent profile per host,
   distinct across hosts. Final greps: no "Clause"/"C1..C8"
   ids, no "Lab" strand, no "Contract" as a track name.
9. **Art style — House Cel / Pony Painterly + the eye rule** —
   the estate's NSFW anime style codified (SPEC §7, CURRICULUM
   style section): TV-anime/key-visual, flat cel with
   hard-edged shadows, bold lineart, saturated color, glossy
   detailed eyes. Two lanes selectable per spec: **House Cel**
   (flat cel; the Iris lane; applied by game.html) and **Pony
   Painterly** (painterly/glossy; codified alternate). The
   **eye rule** is stated as a law: all girls have heart-shaped
   PUPILS; the iris and the rest of the eye keep her
   natural/canonical eye color — heart is pupil shape only.
   game.html's Betsy restyled to House Cel in both views: four
   authored eye layers (open / heavy-lidded / gaze-up / melted
   torogao) built from one `<defs>` eye with heart pupils and
   catchlights in every state, sparkles from Flushed up, blush
   deepening from Smiling, the peak layer's pupils luminous
   pink with a pale core; hard cel shadow shapes on hair, skin,
   chest, outfit. Mechanics untouched. The TUI feedback panel
   gained a **style line** naming the beats per state (plus a
   test pinning the eye rule's wording at every state), and
   the climax log line now spells the torogao grammar out.
   CURRICULUM teaches style in the animation levels: lane as a
   per-spec choice, the eye rule as a named law, style
   consistency as a gate concept.
10. **Daddy Style** — the crate's engineering discipline is
    named **Daddy Style** (SPEC §7.4; CURRICULUM's Study
    section): the principles are unchanged (explicit
    contracts/assertions, bounded work, typed errors,
    determinism, no hidden behavior, safety first); the framing
    is the positive side of a daddy-dom dynamic — strict
    requirements from a firm, caring authority, for the
    submissive's own benefit; rules known in advance, applied
    consistently, never arbitrary; breaking them yields poor
    outcomes mechanically, as consequence not punishment.
    (Historical note: the discipline was renamed to Daddy
    Style by Owner's ruling, 2026-10-26 — the old name belongs
    to the SFW lane and appears nowhere else in this crate.)
11. **Cast of record — CAST.md** — the Owner approved a
    distinct girl per level ("records first"), with the
    sequence starting at the basic basics and ramping to the
    complex. `CAST.md` now holds seventeen records, one per
    level: the five existing hosts keep exactly one level
    each at their strongest association (Betsy P1, Sable P2,
    Vega P3, Mireille P4, Vigil P5) with their profiles
    quoted exactly from the code (`SensitivityProfile`
    default/high/low in `milking-core/src/sim.rs`;
    `mireille_profile()` / `vigil_profile()` in
    `hands-on/src/level.rs`), and twelve new original adult
    girls are designed as records to the same depth (Petra
    P6, Wren P7, Noor P8, Ines P9; Alba S1, Ruth S2, Cleo
    S3, Yara S4, Mina S5, Sana S6, Odette S7, June S8).
    CAST.md states the crate's dual purpose (the Owner's
    Rust learning at the Daddy Style standard + his
    evaluation surface for Torogao), the mastery threads,
    the eye law, the two style lanes, an explicit ramp per
    track with per-level rungs and prerequisites, and three
    ramp flags (RF-1..RF-3) as proposals — nothing was
    renumbered; P1/P4 ids are load-bearing. Every record
    carries an Evaluation block (animation / sound /
    mechanic to judge, the competence named where one
    applies) with the verdict `pending — Owner play`,
    verbatim, on all seventeen. Gates run on the file:
    17 records; all girl names unique and clear of both
    forbidden cast lists; every field present on every
    record; every Playroom record names a kink (P1 states
    plainly that the canon has no milking entry rather than
    forcing one) and every Study record names its desk
    dynamic. **No code changed, no art generated** — wiring
    the twelve new hosts into `level.rs` is the follow-on
    pass, after the Owner reviews the cast table.
12. **Animation bar, representation rule, reaction system,
    and the haptics/UI thread** — four Owner rulings of
    2026-10-26 (13:05–13:06) folded into CAST.md and
    MASTERY.md; **no code changed**. (a) *Animation bar:*
    CAST.md's header now names the bar references — the
    motion grammar of Danbooru posts **12068969** and
    **9228588** (the 2D standard; 9228588 is the animation
    study's named ahegao case) and post **6277975** for the
    sprite representation; MASTERY.md states the same bar
    where Arc C frames its routes. (b) *Representation:*
    CAST.md states the rule — a girl's representation may be
    2D, 3D, or sprites per level; the **style** (buildup
    readability, spring follow-through, eye behavior,
    climax choreography) is the constant, and the curriculum
    reinforces it in every representation. No record had a
    representation forced on it; records state one only
    where the design already implied it. (c) *Reaction
    system, cast-wide:* every girl gets **live sound +
    physical reactions as one coupled system** (the spirit
    of post **11099555**) — buildup readable visually *and*
    by sound, eyes rolling at climax, each physical event
    drawing a synced vocal reaction (peak with the event,
    then trailing off). CAST.md's header states the system,
    its no-audio note now says the records spec that coupled
    system, and **all seventeen** Evaluation blocks were
    amended in place: *Animation to judge* now includes eye
    behavior through the climax, *Sound to judge* now covers
    buildup audibility and per-event vocal sync. MASTERY.md
    gains the taught thread: production side = atelier's
    audio modality under AUDIO-FACTORY (locked voice per
    girl, state matrix, −16 LUFS masters — settled
    architecture, **not a new MCP**); runtime side = the
    profile-driven reaction-audio engine (layered loops
    tracking the accumulator, event one-shots on the same
    fixed-step clock as the visuals), placed across Arc A's
    A12 (rewritten to carry it) and the evaluation
    integration, scoped honestly as design + prototype
    slices, fakes/readouts first. (d) *Haptics +
    cross-platform Rust UI:* MASTERY.md gains **Arc D** —
    a fourth arc (the doc states why: one stack, core →
    router → two device protocols → shipped UI) covering
    **all** the Rust for Buttplug.io (official `buttplug`
    crate as Intiface client, protocol v3, torogao-haptics
    router/gate discipline, fakes-first), the Lovense API
    (the lovense-mcp 8-tool contract, decide() gate,
    watchdog, panic-latch stop, LAN-route concepts), and
    Rust UI shipping (winit + wgpu + egui on Linux +
    Windows; the Android NDK/APK path; F-Droid's fully-open,
    reproducible-build discipline). Spiral is fakes-first
    throughout; live hardware stays behind the S4 rule
    (Owner's go + presence), taught as a gated rung (D5),
    never as coursework. CAST.md names the fourth mastery
    thread and, where a level genuinely exercises the
    integration, its Mechanic field now names that
    competence (P5 Router's Gate, P8 Funscript Timeline —
    nowhere forced). MASTERY.md's counts moved honestly with
    the arc: **64 modules (35 CORE / 24 ADVANCED /
    5 CAPSTONE)**, its evaluation table gained the Arc D
    row, its flags gained the live-hardware shape (flag 7)
    and the updated sound flag (flag 6), and its
    reconciliation checklist gained the Arc D, bar, and
    reaction-audio merge items; proposed slot X6 (Haptics
    Bench & Shipping Dock) was added for Arc D's home.

## Prototype-grade vs port-ready

- **Port-ready:** `milking-core` — it *is* the port twin:
  fixed-step, typed errors, no rendering, no clock, no RNG;
  byte-deterministic traces. `hands-on`'s framework (registry,
  feedback engine) is production-shaped but terminal-bound.
- **Prototype-grade:** `game.html` mirrors the core's constants
  and grading by hand (duplicated numbers, kept in sync by
  comment discipline, not by construction); its Studio Mode is a
  teaching sketch of `authoring.rs`, not a second implementation
  of record. SVG art is original work in the House Cel style
  (SPEC §7) — cel-shaded, eye rule applied — and remains
  prototype-grade as *art* until the Owner's eye rules on it.

## Spec ambiguities resolved (and how)

- **Bucket vs bottle:** the reference shows both filling. Resolved:
  bottle = win meter (capacity-bound), bucket = session
  totalizer (all production, including bursts) — both views agree
  by construction (SPEC §2.2).
- **"Side actions raise state but don't fill":** they feed the
  meter at half gain and store **no** reservoir — otherwise the
  low-gain lesson (reservoir = stored squeezes) would leak.
- **Climax threshold vs the 0–100 meter:** profiles may set
  thresholds conceptually beyond the meter; the core clamps the
  trigger to the meter's cap, and the dial's range (30–100)
  matches what the meter can reach.
- **First squeeze grading:** with no previous beat there is
  nothing to grade against — it is Good by definition and sets
  the beat (mirrored in both implementations).
- **Bake step divisibility:** `bake(step)` refuses a step that
  does not divide the cycle (typed `EmptyCycle`-family refusal)
  rather than silently rounding the loop point — the Lab's
  10 ms bake inherits this.
- **Expression descent:** one ladder step per tick everywhere
  (events included) — see Amendment 1 note; the gate that caught
  the jump is `state_machine_cannot_skip_states`, which observes
  tick-by-tick through a full session and its descent.
