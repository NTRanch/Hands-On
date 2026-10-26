# Cowgirl Milking Minigame — Behavioral Design Spec

Date: 2026-10-26.

**Provenance and originality.** The mechanics in this document are
behavior-level observations of a reference gameplay recording: what
the player does, what the meters do, and how the character's state
responds. No artwork, character, name, audio, or asset from any
reference is used. All art in the prototype is original work
drawn for this build in the estate's House Cel style (§7); the
character is an original adult creation named **Betsy**. This is
an adult game for the estate's adult lane; every character is a
consenting adult.

## 1. Product shape

A Live2D-style milking minigame presented from a static camera in a
split multiple-view composition:

- **Left view — rear three-quarter:** Betsy leans on a wooden fence
  in a farm setting; a bucket sits beneath her and fills as the
  session produces.
- **Right view — front:** the same character facing the player; this
  is the view the POV hands interact with, and the view that carries
  the expression state machine.
- **Title screen first:** a title card with "press to start". No
  interaction is live until the session starts.

Betsy's costume reads at a glance: cow ears and small horns, a
cow-print outfit, a cowbell at the neck, and a tail that sways with
its own secondary motion.

## 2. Core loop

1. The player performs a **squeeze cycle**: POV hands compress and
   release her chest, alternating sides or together, in a steady
   rhythm.
2. Each well-timed squeeze drives the **squeeze parameter** (0..1)
   up its curve; flow is produced while the parameter sits in the
   productive zone.
3. Produced milk fills the **bottle gauge** (the session's win
   meter) and, in parallel, the **bucket** under the rear view.
4. Squeezes raise the **state meter** slowly; side actions raise it
   faster but produce no milk.
5. As state crosses thresholds, Betsy's **expression** advances and
   previously locked **action buttons unlock**.
6. The bottle reaching capacity is the **win state**: the session
   ends in a completion card with the final time and spill count.

### 2.1 Timing model — the squeeze is a rhythm, not a click-fest

The cycle has an **ideal interval** between squeezes
(`IDEAL_INTERVAL`, 500 ms in the prototype; 30 ticks at the core's
60 Hz fixed step). Each squeeze is graded against the time since
the previous accepted squeeze:

| Grade | Timing error | Effect |
|---|---|---|
| Perfect | ≤ 100 ms | Full flow, state +2, combo +1 |
| Good | ≤ 200 ms | Normal flow, state +1 |
| Sloppy | > 200 ms early or late | **Spill**: a small amount of milk is lost, combo resets, no flow |
| Mash | < 250 ms since last squeeze | **Spill** and the squeeze is rejected outright |

Pressure is graded the same way: hold-to-charge input maps press
duration to pressure; pressure above the safe ceiling converts a
would-be Good squeeze into a spill. The lesson the loop teaches:
**steady tempo inside the window beats speed**, which is precisely
the fixed-step timing discipline the Rust core formalizes.

### 2.2 Gauge math

- Bottle capacity `CAPACITY = 1000` units.
- Flow per tick while the squeeze parameter `s` is in the productive
  zone (`s ≥ 0.35`): `flow = s × STATE_GAIN[state] × FLOW_BASE`,
  accumulated per fixed tick, never per rendered frame — the gauge
  is a function of simulated ticks, so frame rate cannot change the
  outcome.
- `STATE_GAIN` rises with expression state (1.0 / 1.15 / 1.3 / 1.5 /
  1.75), rewarding the state-building side actions indirectly.
- Spill subtracts a fixed `SPILL_AMOUNT` (15 units), clamped at 0.
- Bucket fill is the same accumulated production shown in the rear
  view (`bucket = total_produced`), so both views agree by
  construction; the bottle is the win meter, the bucket is the
  session totalizer.

### 2.3 Expression state machine

Five states, strictly sequential — the machine may advance or
regress **one step at a time**, never skip:

```
Neutral → Smiling → Flushed → Aroused → Ecstatic
```

- Advance when the state meter crosses 20 / 40 / 60 / 80.
- Regress one step when the meter falls 10 points below the current
  state's entry threshold (hysteresis, so the face does not flicker
  at a boundary).
- The meter decays slowly (0.4/s) when no successful action has
  landed for a while, so expression is a maintained state, not a
  one-way ratchet — except that the machine still walks down one
  step at a time.
- Face rendering per state (front view) follows the arousal
  expression grammar of §7: smile deepens, blush rises, lids
  grow heavy, the gaze turns up, heart pupils brighten — and at
  the highest state the full torogao grammar lands (half-lidded
  upturned eyes, luminous heart pupils, open mouth, heavy blush,
  sparkles) — the same progression the reference shows.

### 2.4 Buttons, affordances, unlocks

Circular hand-action buttons flank the front view. Locked buttons
render dimmed with a cross-out overlay and refuse input with a
typed "locked" wobble (they are visible from the start — the player
can see what progression will earn).

| Action | Unlock condition | Effect |
|---|---|---|
| Milk (left hand) | always | Squeeze cycle, left side |
| Milk (right hand) | always | Squeeze cycle, right side |
| Caress cheek | state ≥ Smiling | State +6, no flow, no spill risk |
| Spank | state ≥ Flushed | State +10, small spill risk if spammed (< 800 ms apart = spill) |
| Both hands | state ≥ Aroused | Squeeze with both hands: +50% flow, tighter timing window |

A gear/settings control (top right) pauses the session and exposes
volume and tempo-assist toggles; it is chrome, not gameplay.

### 2.5 Secondary motion model

Secondary motion is **driven, never keyed directly**:

- **Chest jiggle** is a damped spring (`stiffness 90, damping 7`)
  whose rest position is displaced by changes in the squeeze
  parameter. A squeeze excites the spring; the visible bounce is
  the spring settling — follow-through and overlap fall out of the
  physics, exactly the model the Studio track teaches.
- **Tail sway** is a slow sine driven by session time, amplitude
  scaled by state (calmer at low state, livelier at high state).
- **Cowbell** swings on the same spring family with a phase lag
  behind the chest spring, so props feel attached, not pasted.

### 2.6 Sound

The reference has sound; the prototype uses only tiny WebAudio
oscillator blips synthesized in code (squeeze, spill, unlock, win)
or silence. No external audio files, ever.

## 3. Studio Mode (the teaching layer inside the game)

A panel beside the game hosts a **keyform timeline editor** for the
one squeeze parameter:

- Click the timeline to place keyforms (time, value); drag to move,
  click a keyform to delete it.
- Choose an easing per segment family (Linear / Smooth / EaseInOut /
  Hold) and a cycle length in ms.
- **Preview** loops the authored curve driving the same chest
  spring the game uses.
- **Scoring:** loop closure (value at cycle end must equal value at
  cycle start — a visible gap is drawn when it does not) and tempo
  match (authored cycle length vs the game's ideal cycle). The
  score is the curriculum made visible: a loop that does not close
  cannot loop seamlessly in an engine, and the player has just
  learned why.

## 4. Determinism contract (what the Rust twin guarantees)

`milking-core/` re-implements §2 as a fixed-step simulation:
identical input event sequences produce byte-identical state
traces. Input is a stream of timestamped events; rendering does not
exist in the core; all progression is a pure function of ticks and
events. The HTML prototype mirrors the same constants and grading
windows so play feel and core behavior agree.

## 5. The sensitivity system — the reaction-profile lesson (Amendment 1)

This is the owner's central interest, and the game's core
teaching object: **a character's responsiveness is a profile of
numbers, and changing the numbers changes the person.**

A `SensitivityProfile` has four fields:

| Field | Meaning |
|---|---|
| `gain` | Arousal added per unit of effective stimulation. |
| `threshold` | Arousal level at which climax triggers. |
| `reservoir_capacity` | Maximum stored volume in the milk reservoir. |
| `refractory_ticks` | Post-climax window: gain halved, re-trigger impossible. |

Plus a `reservoir_rate`: the fraction of each effective squeeze's
stimulation that is *stored* in the reservoir instead of only
passing through. Accumulated stimulation therefore does two
things at once — it raises the meter, and it **fills her**: the
reservoir level is a visible fullness parameter in both renderers
(chest scale in the HTML build, the reservoir bar in the TUI) and a
stored-volume number in the core.

### 5.1 The live dial

The game UI exposes gain and threshold as sliders, plus three
presets. The dial edits the profile *during play* — the response
changes in real time, which is the lesson: there is no code path
per personality, there is one engine and a profile.

### 5.2 The two regimes the dial demonstrates

- **High gain + low threshold (preset HIGH: gain 2.6, thr 42,
  reservoir rate 0.3).** A few on-beat squeezes send her over. The
  climax face is the governed **torogao expression** — a named
  expression (Addition G) that resolves to a parameter recipe
  (the §7 peak grammar: half-lidded upturned eyes, luminous
  heart pupils, open mouth, heavy blush, sparkles), reached
  by the state machine walking its ladder under fixed-step
  parameter interpolation. It is never a frame cut: the ladder
  moves one step per tick, and the HTML face layers cross-fade.
  But the reservoir barely filled, so the burst is small.
- **Low gain + high threshold (preset LOW: gain 0.45, thr 98,
  capacity 420, rate 0.9).** A long build. The reservoir visibly
  climbs — fullness parameter and stored volume both rise — and
  when climax finally triggers, its **intensity and milk burst
  scale with the reservoir at trigger**:
  `intensity = 0.35 + 0.65 × fullness`,
  `burst = reservoir × (0.6 + 0.4 × intensity)`, paid straight
  into the bottle. The slow build is mechanically *worth more*.

### 5.3 After climax

The reservoir drains into the bottle/bucket (the burst), the
meter drops, the expression **eases back down the ladder** one
step per tick (never a cut back to Neutral), and the refractory
window runs: gain halved, climax cannot re-trigger. The core
proves each property in tests: regime ordering (high climaxes in
strictly fewer squeezes), payoff ordering (low's intensity and
burst strictly greater), the reservoir never exceeding capacity,
and refractory blocking re-trigger.

## 6. The framework — hands-on, the learning crate (Amendment 4, IDEA-0093)

The minigame is Level 1 of something larger: **hands-on**, a
TUI/CLI learning crate that codifies the estate's game systems as
levels. Succeeding at a learning exercise drives live stimulation
feedback on that level's host girl. The girl is the reward
instrument.

### 6.1 Workspace shape

```
Cargo.toml            workspace: milking-core + hands-on
milking-core/         Level 1's deterministic core (this SPEC, §2/§5)
hands-on/             the TUI binary: level menu, framework, feedback engine
game.html             Level 1's visual prototype (HTML twin of the core)
milking.nvim.lua      Neovim launcher (separate-lane routes only)
nvim-appname/init.lua dedicated alternate-profile template
```

### 6.2 Level registry

Each level is a descriptor: `{ id, title, source system, host
girl, built? }`, and built levels implement the `Level` trait.
Each host is an original adult character with **her own default
SensitivityProfile** — Betsy (P1, balanced), Sable (P2, high),
Vega (P3, low), Mireille (P4, mid), Vigil (P5, exacting).
Learning her profile is part of the play, because profiles are
data and data differs per character.

### 6.3 The feedback engine

One shared engine (`hands-on/src/feedback.rs`) turns exercise
success/failure events into the host's stimulation, on the same
deterministic tick as the event that caused them:

- It wraps the fixed-step core, so the arousal ladder, governed
  expressions (the torogao face at peak), reservoir, climax, and
  refractory all behave exactly as §2/§5 specify.
- It renders live in the TUI: text bars, the expression **by
  name**, an event log narrating her responses, and a
  **fake-haptics readout** — the scalar the torogao-haptics
  router *would* emit for her current state, clamped at the
  router's 0.20 ceiling. Fakes only: this crate contains no
  transport and can never reach a device.
- Level 1's exercise *is* the milking loop, so its feedback is
  direct. Later levels feed the same engine from different
  exercises (beat accuracy, routing correctness, closure scores,
  gate discipline) — the engine does not know or care which
  exercise produced the event. That indifference is the design.
- The panel also carries a **style line** (§7): it names the
  House Cel style beats as they occur — heart pupils, blush
  rising, sparkles, the torogao peak — so the estate's art
  grammar is legible even in ASCII.

### 6.4 Level map (designed; P1 and the P4 slice are built)

One Playroom sequence: P1 First Milking (built) · P2 Fixed-Step
Rhythm — Strobe's beat-sync · P3 Stage Systems — Strobe on Bevy,
ECS order · P4 Loop Studio (playable slice) · P5 Router's Gate —
torogao-haptics safety contract · P6 Format Border — moc3
conversion · P7 Sprite & GIF Bake · P8 Funscript Timeline ·
P9 The Third Dimension (designed only). Plus the Study's eight
Rust lessons, all designed. Full designs in CURRICULUM.md.

### 6.5 Two tracks: the Playroom and the Study

The registry carries two tracks. The **Playroom** (§6.4)
teaches game systems and deterministic animation by playing
them — including the animation levels (P4, P6–P9), which teach
deterministic animation and format conversion the Lumen way:
pick a girl, a scene archetype, and a format; author on the
fixed-step tick; pass a fidelity gate whose pass condition is
mechanical — a named seed and a twin-run hash that must come
out byte-identical. **The Study** codifies Rust mastery as
eight *Lessons* (S1..S8): you are summoned to the Study for a
lesson, and the compiler corrects you at the desk. Its content
is unchanged from its design — the Rust Book chapter map, Rust
2024 material, the engineering discipline named **Daddy Style**
(§7.4), the Torogao use cases, the coverage table — and
contracts/limits survive inside it as a *topic*: Lesson S2
teaches typed errors and contracts themselves (B-01, the
estate's contract/limits type). The
capstone lesson S8 stays **Aftercare** (B-80) — tests, review,
and verification after the intensity. In this pass only P1 and
the P4 Loop Studio slice (loop closure + bake + twin-run hash,
on Betsy) are playable; no Study lesson is built.

## 7. Art style — the house style (Amendment 9)

The estate's established NSFW anime style, codified for this
crate. The look is **TV-anime / key-visual**: flat cel shading
with hard-edged shadow shapes, clean bold lineart, saturated
color, detailed glossy eyes, expressive flustered faces. It is
the Owner's ruled target from the atelier work — *much more
anime* — never western/comic rendering.

### 7.1 Two style lanes, selectable per spec

- **House Cel** — flat cel shading, hard shadow shapes, bold
  lineart. The Iris lane; the default. `game.html` applies
  House Cel.
- **Pony Painterly** — painterly/glossy rendering: soft
  gradients, glossier skin and eyes. The alternate lane,
  codified here; a spec names its lane, and a build does not
  drift between lanes mid-asset.

### 7.2 The eye rule (a law)

**All girls have heart-shaped PUPILS; the iris and the rest of
the eye keep her natural/canonical eye color — heart is pupil
shape only.**

The rule binds every renderer, every lane, every state. A face
whose pupils lose the heart is off-model, the same way a loop
that does not close is broken.

### 7.3 The arousal expression grammar

As state rises, the style itself responds — by smooth
interpolation, never a cut:

- **Blush deepens** state by state (absent → edge blush → full).
- **Catchlights and sparkles appear** and multiply in the eyes.
- **The gaze turns up**; lids grow heavy.
- **Heart pupils become luminous** — same shape (the eye rule),
  brighter presence.
- **The peak torogao face** is the melted pleasure expression:
  half-lidded upturned eyes, luminous heart pupils, open mouth,
  heavy blush, sparkles — reached by the ladder walking one
  step per tick and the layers cross-fading.

The TUI carries the same grammar in text: the feedback panel's
style line names the beats as they occur (§6.3).

### 7.4 Daddy Style — the engineering discipline (a law)

The crate's code is written under **Daddy Style**: explicit
contracts and assertions, bounded work with named bounds,
typed errors, determinism, no hidden allocation or behavior,
safety first. The principles are the estate's engineering law;
the name is this lane's own.

Daddy Style evokes the positive side of a daddy-dom dynamic:
**strict requirements set by a firm, caring authority, for
the submissive's own benefit.** The rules are known in advance,
applied consistently, never arbitrary. Follow them and the
work thrives — deterministic, safe, predictable, gates pass.
Break them and poor outcomes follow mechanically —
nondeterminism, gate failures, defects — as *consequence, not
punishment*. Discipline as care; the strictness is protective.

That is the Study's shape exactly: the host sets the
requirements, the compiler enforces them at the desk, and the
correction is the care working as intended.
