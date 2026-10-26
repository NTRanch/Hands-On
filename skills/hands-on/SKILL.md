---
name: "hands-on"
description: "Project conventions for Hands-On (Nice Thoughts Ranch): reaction profiles, the expression ladder and eye rule, cast Evaluation blocks and the Owner-play verdict workflow, the animation quality bar, the audio silence rule, fakes-only haptics, determinism evidence, and which document owns what. Use when adding a level, a host, an animation, a sound spec, or any content to this repository. Engineering discipline lives in the sibling skill, skills/daddy-style/."
---

# Hands-On — project conventions

## What this project is

A teaching game with a dual purpose: it teaches Rust at the
Daddy Style standard, and it is the evaluation surface where
each girl, mechanic, animation, and sound is prototyped
small and judged by playing before it graduates to Torogao
production. Every convention below serves one of those two
purposes.

## Documents of record

- `SPEC.md` owns behavior — the loop, the gauges, the state
  machines, the laws (§7).
- `CURRICULUM.md` owns the systems — level designs, setup
  routes, build status.
- `CAST.md` owns the girls — one record per level: identity,
  palette, kink/dynamic, sensitivity sketch, Evaluation
  block. Art and wiring follow the records, never the other
  way around.
- `MASTERY.md` owns the arcs above the 17-level core map.
- `.ai/memory/` owns state — current, decisions, handoffs.
  Findings go to the inbox first and are promoted when
  validated.

## Reaction profiles

A character's responsiveness is a profile of numbers, and
changing the numbers changes the person (`SensitivityProfile`
in `milking-core/src/sim.rs`):

| Field | Meaning |
|---|---|
| `gain` | Arousal added per unit of effective stimulation. |
| `threshold` | Arousal level at which climax triggers. |
| `reservoir_capacity` | Maximum stored volume in the reservoir. |
| `reservoir_rate` | Fraction of each stimulation stored instead of passing through. |
| `refractory_ticks` | Post-climax window: gain halved, re-trigger impossible. |

Profiles are data, not code paths. A new host gets a profile
in this vocabulary (presets exist: default, high, low); a
level never branches on *who* the host is.

## The expression ladder and the eye rule

Expression is a ladder — Neutral, Smiling, Flushed, Aroused,
Torogao — walked **one step per tick**, never jumped, with
layers cross-fading (SPEC §7.3). The arousal grammar
interpolates: blush deepens, catchlights and sparkles
multiply, the gaze turns up, lids grow heavy.

**Eye rule (a law, SPEC §7.2):** all girls have heart-shaped
PUPILS; the iris and the rest of the eye keep her natural eye
color. Heart is pupil shape only — every lane, every
renderer, every state. A face whose pupils lose the heart is
off-model, the same way a loop that does not close is broken.

## Evaluation blocks and verdicts

Every cast record carries an Evaluation block (template in
references/evaluation-block.md): Animation to judge, Sound to
judge, Mechanic to judge, Verdict. Rules:

- A new record's verdict reads exactly `pending — Owner
  play`. No other verdict exists until the Owner plays the
  piece himself.
- Only what passes his play verdict graduates to Torogao
  production. Silence is not a keep.
- Prototypes must be representative: real animation loops
  and real sounds at prototype grade. A text proxy cannot be
  judged for animation or sound.

## The animation bar

The quality bar is a motion grammar, named in CAST.md
(reference works are cited there by post id): buildup
readable at a glance, spring follow-through on secondary
motion, eye behavior that carries the state, climax
choreography that lands. The medium may vary per level —
2D, 3D, sprites — while the style stays constant. A motion
that reads as a sliding cutout fails the bar no matter how
cheap it was to produce.

## Audio: the silence rule

No audio testing is ever audible unless the Owner expressly
presses play himself. Generation, QC, and verification are
silent — file metrics, loudness, waveform and spectrogram
inspection only. Nothing autoplays in any build, page, or
deliverable. Audio reaches the Owner as files only, and he
presses play. (The current build has no recorded audio at
all; CAST.md's sound records are the spec that future audio
prototypes are built against.)

## Haptics: fakes only

Haptics work runs on simulated backends and readouts under
the router contract: master switch default OFF, ceiling
clamped in the router, generation-stamped frames, a caller
clock watchdog, synchronous total stop. Live hardware is
actuated only on the Owner's explicit go, in his presence —
never from a test, a demo, or a batch run.

## Determinism evidence

Any animation, bake, or loop claim carries: a named seed, a
fixed-step tick, and a twin-run hash that came out
byte-identical. `codeanim-trial/` is the worked example —
204/204 artifacts byte-identical across twin runs, hashes
shipped with the record.

## Levels and hosts

Level ids P1–P9 and S1–S8 are stable and never renumber.
Each level has exactly one host, recorded in CAST.md; the
five founding hosts each keep exactly one level. Wiring a
new host into `hands-on/src/level.rs` follows the Owner's
cast review — records first, code second.

## Public-artifact hygiene

Public artifacts carry studio attribution (Nice Thoughts
Ranch) only. Other lanes' names, internal tooling brands,
personal names, and machine-local paths never ship — where a
document needs the concept, it says "the SFW lane", "the main
editor config", or a repo-relative path.
