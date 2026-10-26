# hands-on — Curriculum (IDEA-0093)

The estate's game systems, codified as levels. Each level takes
one real system, turns it into an exercise, and pays success out
as live stimulation feedback on the level's host girl — she is
the reward instrument, and her reactions run on the same
deterministic tick as your inputs. Learn the system by playing
it; then go read the Rust module it twins, and you will already
know what the module *does* because you have felt its numbers.

**Build status: Level P1 is built and playable, and the P4
Loop Studio slice is playable. The framework (level registry +
feedback engine) is built. The menu carries two tracks — the
**Playroom** (P1–P9: play levels + the deterministic-animation
levels in one sequence; P2, P3, P5–P9 designed) and **the
Study** (Rust mastery; all eight Lessons designed). Everything
marked "designed" in this document is not built.**

**The cast of record is `CAST.md`** — one record per level (identity, palette, kink, sensitivity profile, and the Evaluation block the Owner rules on by playing); this curriculum owns the systems, CAST.md owns the girls.

## Setup

### Build

```sh
cd <this repo's root>
cargo build --release        # workspace: milking-core + hands-on
./target/release/hands-on --demo   # scripted smoke session
./target/release/hands-on          # interactive TUI (menu first)
```

Or open `game.html` in any browser — Level 1's visual prototype,
no build step, no assets.

### Play it from Neovim (the Addition V pattern)

> **Lane rule, plainly:** this game is NSFW-lane material. Its
> launcher must **never** be installed into your main editor config or
> any SFW-lane configuration or repository. NSFW-lane
> tooling stays out of SFW-lane configs and repos. The two routes
> below are the only sanctioned ones, and neither touches a
> standing config.

**Route (a) — on demand, per session.** Inside a running Neovim:

```vim
:luafile <repo root>/milking.nvim.lua
:MilkingGame        " TUI in a floating terminal window
:MilkingGameHtml    " game.html beside Neovim (vim.ui.open)
```

The `:MilkingGame*` commands exist only for that session and
vanish when Neovim exits. Nothing is installed, nothing persists.

**Route (b) — a dedicated alternate profile.** The workdir ships
`nvim-appname/init.lua`. Copy it to a local-only directory that
is not in any repository:

```sh
mkdir -p ~/.config/hands-on
cp nvim-appname/init.lua ~/.config/hands-on/init.lua
# edit GAME_WORKDIR inside it if your workdir path differs
NVIM_APPNAME=hands-on nvim
```

Your main config never loads this profile. Inside it,
`<leader>mg` opens the TUI and `<leader>mh` opens the HTML.

**The honest boundary:** the HTML/canvas version cannot render
inside a text buffer — no 60 fps stage lives in a terminal grid.
So in-Neovim play is the **TUI twin on the same milking-core**
(same constants, same windows, same outcomes), and the HTML
opens *beside* Neovim, launched from it. What is shared is the
core; what differs is only the renderer. That is the Torogao
Addition V shape exactly.

## Track 1 — Play (the milking game teaches the engine)

Host: **Betsy** — cowgirl hostess of the teaching farm; cow ears
and horns, cow-print outfit, cowbell, tail. Patient and teasing;
her balanced default profile (gain 1.0, threshold 96, reservoir
220) forgives a beginner's tempo while they learn the windows.

| # | Lesson | The mechanic that teaches it | Torogao twin |
|---|---|---|---|
| 1 | **Parameters** | The squeeze is a value 0..1 with an envelope, not an animation. Flow happens only while it sits in the productive zone. | `torogao-format` parameter model |
| 2 | **Curves** | Pressure and timing shape the parameter's rise; Studio Mode (Track 2) makes you author the curve yourself. | `torogao-format` bindings & interpolation |
| 3 | **Fixed-step timing** | The beat window: on-beat fills, off-beat spills, mashing spills. The sim runs 60 ticks/sec regardless of frame rate — mash the keys and the *core* refuses you, not the UI. | `torogao-engine` fixed-step evaluation |
| 4 | **Reaction & state profiles** | The sensitivity dial + the expression ladder (below). | Torogao reaction profiles; Addition G governed expressions |

### The reaction-profile lesson (Play 4, the centerpiece)

A reaction profile is four numbers — **gain, threshold,
reservoir, refractory** — and Betsy is a different girl under
each setting:

- **HIGH gain / low threshold:** a few on-beat squeezes trigger
  climax. The face she makes is the governed **torogao
  expression**: a *named* expression that resolves to a parameter
  recipe, arrived at by the ladder walking up one step per tick
  and cross-fading — smooth parameter travel, never a frame cut.
  Small reservoir → small burst.
- **LOW gain / high threshold:** the long build. Every effective
  squeeze stores volume; her fullness parameter climbs visibly
  (chest scale in HTML, reservoir bar in TUI). When she finally
  goes over, intensity *and* burst scale with what you stored.
- **Refractory:** afterwards her gain halves and she cannot
  re-trigger; her expression eases back down the ladder. Profiles
  have *recovery*, not just response.

Gain/threshold/accumulator are exactly a Torogao reaction
profile's shape. The module twins: the profile type lives with
the parameter data (`torogao-format`), the accumulation and
state machine evaluate fixed-step (`torogao-engine`), and the
fake-haptics readout in the TUI is the profile's output as the
**torogao-haptics router** would clamp it (ceiling 0.20).

## Track 2 — Studio (authoring animations like the game's own)

Host for the studio lessons: the same Betsy in Level 1's Studio
Mode panel; the full Loop Studio level is Playroom **P4**,
hosted by Mireille. The Play and Studio lesson tracks together
form the **Playroom track** of the menu.

| # | Lesson | The exercise | Torogao twin |
|---|---|---|---|
| 1 | **Keyforms** | Click the timeline to place (time, value) keyforms for the squeeze parameter; drag to move. | `torogao-format` keyform tracks |
| 2 | **Cycle phase & seamless closure** | Your loop is scored on closure: end value must equal start value. The gap is drawn in red when it doesn't — a non-closing loop pops every iteration in a real engine. | `torogao-engine` loop evaluation |
| 3 | **Easing** | Switch Linear / Smooth / EaseInOut / Hold and watch the same keyforms become different motion. | interpolation in `torogao-format` |
| 4 | **Secondary motion** | Preview drives the game's chest spring: the bounce is a damped spring *driven by* your curve, never keyed. Follow-through is physics with two constants. | engine secondary-motion solvers |
| 5 | **Bake to a scene track** | The curve is sampled to a fixed-step frame table (the TUI's studio exercise prints the bake) — after the bake, playback is a table lookup, identical everywhere. | scene tracks; deterministic playback |
| 6 | **Haptics / funscript export** | A baked track is one rename away from a funscript timeline; the feedback engine's readout shows the same values clamped through the router contract. | `torogao-haptics` player + router |

Studio exercise in the TUI (inside Level 1):
`studio 1000 smooth 0=0,400=1,1000=0` → closure PASS, tempo
100/100, and the baked sparkline. Break closure on purpose —
`studio 1000 linear 0=0,500=1,1000=0.4` — and read the failure:
it names both endpoint values. That failure message *is* the
lesson.

## The house style — House Cel & Pony Painterly (taught, not just applied)

The crate renders in the estate's established NSFW anime style,
and the Playroom's animation levels (P4, P6–P9) teach it as a
subject, not just a coat of paint. The look is **TV-anime /
key-visual**: flat cel shading with hard-edged shadow shapes,
clean bold lineart, saturated color, detailed glossy eyes,
expressive flustered faces — the Owner's ruled atelier target,
*much more anime*, never western/comic rendering.

**Two style lanes, selectable per spec:**

- **House Cel** — flat cel, hard shadow shapes, bold lineart.
  The Iris lane; the default. `game.html` applies House Cel,
  and the TUI names its beats in text.
- **Pony Painterly** — painterly/glossy: soft gradients,
  glossier skin and eyes. The alternate lane. A spec names its
  lane up front; drifting between lanes mid-asset is a defect.

**The eye rule — a named law of the estate:**

> **All girls have heart-shaped PUPILS; the iris and the rest
> of the eye keep her natural/canonical eye color — heart is
> pupil shape only.**

**The arousal expression grammar:** as state rises, blush
deepens, catchlights/sparkles appear and multiply, the gaze
turns up, lids grow heavy, and heart pupils become luminous.
The peak torogao face is the melted pleasure expression —
half-lidded upturned eyes, luminous heart pupils, open mouth,
heavy blush, sparkles — reached by smooth interpolation up the
ladder, never a cut.

**Style as a gate concept in the animation levels:** a level
spec names its style lane the way it names its seed. A bake or
conversion that drifts off-lane — pupils that lose the heart,
an iris that changes color, a face that jumps instead of
interpolating — fails the level's style-consistency check the
same way a non-closing loop fails closure. Style is data with
a contract, exactly like the parameters it renders.

## The codified level map

### P1 — First Milking · host Betsy · **BUILT**
- **Source system:** Torogao's parameter model + reaction
  profiles.
- **Exercise:** the milking game of SPEC §2/§5 (this package).
- **Success → host:** direct — squeezes are stimulation; her
  ladder, reservoir, climax, and refractory are the feedback.
- **Twins:** `milking-core` ↔ `torogao-format` (parameters,
  profiles, keyform tracks) + `torogao-engine` (fixed-step
  evaluation) + `torogao-haptics` (readout clamp).

### P2 — Fixed-Step Rhythm · host Sable · designed, not built
- **Source system:** Strobe's beat-sync — judgments made against
  a deterministic clock, not wall time.
- **Host:** **Sable**, a night-drive DJ who lives on the beat
  grid; oversized headphones, metronome pendant. Twitchy high
  profile (gain 2.6, threshold 42): she rewards precision
  instantly and punishes drift just as fast.
- **Exercise:** hit ticks on a scrolling deterministic beat grid
  at rising tempos; the exercise engine counts a hit only if it
  lands inside the tick window the *core* defines — rendering
  frame rate is provably irrelevant, and a "frame-perfect vs
  tick-perfect" round makes the difference felt.
- **Success → host:** every on-tick hit is a stimulation event
  shaped like a haptics envelope (attack/sustain/decay drawn on
  the grid); combo streaks raise her gain temporarily.
- **Twins:** Strobe's sync core; `torogao-haptics` envelope
  shapes in the player.

### P3 — Stage Systems · host Vega · designed, not built
- **Source system:** Strobe on Bevy — ECS discipline: components
  are data, systems are functions, the schedule is a contract.
- **Host:** **Vega**, stage manager of the Strobe club, who runs
  the show from the booth and notices every dropped frame. Deep
  low profile (gain 0.45, threshold 98, reservoir 420): a scene
  that holds together all night pays out enormously.
- **Exercise:** order and tune the systems of a live club scene —
  input → rig/animation → physics → haptics → render prep — on a
  text schedule board. Declare each system's queries and change
  detection, place it in the fixed-timestep schedule, and keep
  the scene stable while the crowd (event load) rises. A system
  scheduled before its producer, or a query that aliases a
  mutable borrow, visibly destabilizes the scene: props desync,
  the rig stutters, her expression cools.
- **Success → host:** every stable fixed step under load is a
  stimulation tick through the feedback engine; a full set with
  zero instability is a stored-charge climax. You learn Bevy's
  rule in your hands: *the schedule is the program.*
- **Twins:** Bevy schedules/queries/change detection;
  `torogao-engine`'s fixed-step evaluation order.

### P4 — Loop Studio · host Mireille · **PLAYABLE SLICE (on Betsy)**
- **Source system:** Torogao animation authoring — keyform
  tracks, loop closure, the bake.
- **Exercise:** author a keyform loop (Studio Mode grown up),
  bake it at 10 ms, and pass the twin-run gate. Seed
  `betsy-loop-a1` is shown; the frame table prints as it bakes;
  two hashes must match and closure must hold. The playable
  slice runs on Betsy; Mireille hosts the level's full design.
- **Success → host:** a loop that closes and twin-runs
  identical plays on her seamlessly, forever — the hash match
  *is* the stimulation event.
- **Style:** House Cel per spec (the style section above); the
  loop's face travel must interpolate, never cut.
- **Twins:** `milking-core::authoring` ↔ Lumen rig parameter
  space.

### P5 — Router's Gate · host Vigil · designed, not built
- **Source system:** the torogao-haptics safety contract —
  ceiling clamp, heartbeat watchdog, synchronous stop, no resume
  on reconnect.
- **Host:** **Vigil**, gatekeeper of the router shrine; braided
  cable hair, a brass stop-key on a chain. Exacting profile
  (gain 0.9, threshold 85, reservoir 260, long refractory):
  sloppy routing doesn't just fail, it cools her down.
- **Exercise:** route stimulation signals from sources to her
  through a router you configure per round — set ceilings,
  keep the heartbeat alive, hit STOP inside the window when a
  fault injects. Tripping the gate (over-ceiling, dead watchdog,
  late stop) ends the round safely and explains which rule of
  the router contract fired.
- **Success → host:** clean routed signal is stimulation through
  the *real* contract shape — the fake readout becomes the
  exercise itself.
- **Twins:** `torogao-haptics` router + `decide()` gate, rule
  for rule.

### P6 — Format Border · host Vigil · designed, not built
- **Source system:** the Lumen moc3 exporter and its MAPPING
  tables.
- **Exercise:** predict, per construct, whether it crosses to
  moc3 V3_0 as EXACT, LOSSY, or NO-MAP — then the loss report
  grades you. Teaching points from the estate's exporter work:
  most parameter/bindings maps are EXACT; multiply/screen
  per-form colors are version-gated (LOSSY before V4_2); the
  15-of-18 blend modes with no V3_0 target and part opacity (no
  binary home in V3_0) are NO-MAP, refused typed rather than
  silently downgraded.
- **Style:** the export spec names its lane (House Cel or Pony
  Painterly); a construct that survives as EXACT must still
  render on-lane — the eye rule crosses the border or the level
  fails.

### P7 — Sprite & GIF Bake · host Sable · designed, not built
- **Source system:** the Lumen sprite baker.
- **Exercise:** choose frames for a spritesheet/GIF bake: which
  phases survive sampling, and does the loop still *close*
  across the bake — a loop that closed at 10 ms can open at GIF
  frame rates, and the twin-run hash on the baked artifact is
  the judge.
- **Style:** frame selection is style selection — the baked
  frames must hold the lane and the eye rule at GIF resolution,
  where a heart pupil is a handful of pixels and still required.

### P8 — Funscript Timeline · host Vega · designed, not built
- **Source system:** the Lumen funscript writer;
  `torogao-haptics` player.
- **Exercise:** turn a motion curve into a haptic track: sample
  the squeeze parameter to a funscript timeline, clamp it
  through the router contract, and hear the difference between
  the authored curve and the lawful emission.

### P9 — The Third Dimension · host Mireille · **DESIGNED ONLY**
- **Source system:** Lumen's GLB/3D emitter.
- **Exercise (codified; no engine work in this pass):** GLB
  export and the renders-of-the-3D route (the estate's ruled
  2D-from-3D path): the same scene, the same seed, renders that
  must twin-run byte-identical. In 3D the style lanes become
  material/shading specs — House Cel is a toon-shading contract,
  and the eye rule survives as geometry + texture, not pixels.

## Track 2 — The Study (Rust mastery, codified — designed, not built)

**You are summoned to the Study for a lesson.** The Study is
where the house's Rust is taught: eight **Lessons**, S1..S8,
each one a sitting at the desk where the compiler corrects you
— a borrow error is not a failure state, it is the lesson
arriving. The framing is the pedagogy:

- **Bounded work, negotiated in advance.** Every loop, queue,
  buffer, and retry names its bound before it runs — the terms
  are agreed before anything starts, never discovered mid-task.
- **The stop is synchronous and total.** Any participant can
  end everything *now* — the torogao-haptics router's stop (and
  Rust's panic-latch discipline) is the teaching example: no
  partial stops, no "after this one last frame."
- **Explicit contracts = signatures, assertions, typed
  errors.** What a function takes, returns, and refuses is
  agreed in its types before the call starts. Failures arrive
  as typed errors, never as stringly surprises. (Contracts
  themselves are a *topic* here — Lesson S2 teaches them,
  B-01-style: limits named, terms explicit.)
- **Aftercare = the capstone (B-80).** After the intensity:
  tests, review, verification. The gate suite is aftercare —
  the deliberate work that makes the next sitting safe.

**Daddy Style** (the crate's engineering discipline): safety
over performance over DX · explicit contracts and assertions ·
bounded work with named bounds · no hidden allocation or
behavior · typed errors · determinism (same input, same
output) · alphabetical order. The name is the framing: strict
requirements set by a firm, caring authority, for the
submissive's own benefit — rules known in advance, applied
consistently, never arbitrary. Follow them and the work
thrives: deterministic, safe, predictable, gates pass. Break
them and poor outcomes follow mechanically — nondeterminism,
gate failures, defects — as consequence, not punishment.
Discipline as care; the strictness is protective. In the
Study, the host sets the requirements and the compiler
enforces them.

**Rust 2024 edition material**, woven where genuinely relevant:
let chains (Lessons 2, 7 — this crate's own code uses them),
gen blocks (Lesson 3, frame-table generation), async closures
(Lesson 6), lifetime capture in `impl Trait` (Lesson 1),
unsafe extern blocks (Lesson 4 — named, bounded, justified or
absent), never-type fallback (Lesson 2, error paths).

### The Lessons

| Lesson | Teaches (Rust Book) | Exercise sketch | Her feedback shows | Torogao twin |
|---|---|---|---|---|
| S1 Own the Puppet | ch. 4 ownership, ch. 5 structs, ch. 6 enums, ch. 10 lifetimes | Repair a puppet-file parser's ownership: arrange moves/borrows so the parse holds; a borrow error is staged as the sitting stopping safely | Betsy holds a pose while you hold the borrow — drop it wrong and she resets, unruffled, and the log names the rule | `torogao-format` parsing (INP1 model space) |
| S2 Fail Closed | ch. 9 error handling, ch. 10 traits/generics | Build a license-gate check that must refuse typed: given doctored headers, choose the error variant each deserves; a stringly failure trips the gate | Vigil's regard is the error budget: exact variants warm her, vague ones cool her on the spot | moc3 importer/exporter fail-closed license gate |
| S3 The Accumulator | ch. 3 types, ch. 13 iterators/closures, gen blocks (2024) | Implement the fixed-step accumulator with integer tick arithmetic — no floats in the contract — and prove frame counts against a drifting naive version | Sable dances to your clock: drift shows on her face before it shows in your numbers | `torogao-engine` fixed-step evaluation |
| S4 Hold the Frame | ch. 15 smart pointers, ch. 16 concurrency, unsafe extern (2024) | Own a GPU-ish resource lifecycle on paper: order the drops, place the one justified unsafe block, bound it, fence it | Mireille keeps the frame only while every resource has an owner; an orphaned handle visibly flickers her | renderer on winit/wgpu/egui |
| S5 Bounded Queues | ch. 10 traits, ch. 16 concurrency, ch. 19 patterns | Configure the haptics router: name every queue's bound, set the watchdog, rehearse the synchronous stop mid-stream | Vigil receives only what the router lawfully emits — the fake readout *is* her state, clamped | `torogao-haptics` router + watchdog |
| S6 Async Borders | ch. 17 async (async closures, 2024), ch. 14 Cargo/workspaces | Wire an MCP tool surface: schemas first, async only at the border, the core stays synchronous and deterministic | Sable answers tool calls on the beat; a schema lie makes her skip — visibly | MCP + A2A surfaces over the Torogao registry |
| S7 Track the Performer | ch. 6 enums, ch. 19 patterns, let chains (2024) | Write the tracking state machine: states, transitions, calibration states; an unhandled transition is a pattern the compiler rejects | Vega's booth view is your state diagram, live; exhaustiveness reads as her composure | live mode tracking/interop (HOST-07) |
| S8 Aftercare | ch. 11 testing, ch. 14 Cargo | The capstone: given the crate of Lessons 1–7, write the adversarial half of the gate suite — tests that think like the person who wants the terms broken | All five hosts attend; their combined feedback is the review board's verdict | the estate gate discipline itself |

### Chapter coverage (Rust Book → Lesson → Torogao use case)

| Rust Book chapter | Lesson | Torogao use case |
|---|---|---|
| 3 — Common concepts (types) | S3 | tick arithmetic, no floats in contracts |
| 4 — Ownership | S1 | format parsing: buffers with owners |
| 5 — Structs | S1 | model/record data shapes |
| 6 — Enums & matching | S1, S7 | node kinds; tracking state machines |
| 9 — Error handling | S2 | fail-closed importer/exporter gates |
| 10 — Generics, traits, lifetimes | S1, S2, S5 | parser generics; backend traits; router bounds |
| 11 — Testing | S8 | the gate suite as aftercare |
| 13 — Iterators & closures | S3 | frame-table generation, gen blocks |
| 14 — Cargo & workspaces | S6, S8 | the workspace you are standing in |
| 15 — Smart pointers | S4 | renderer resource ownership |
| 16 — Concurrency | S4, S5 | bridge threads; watchdog |
| 17 — Async fundamentals | S6 | MCP/A2A async borders |
| 19 — Patterns | S5, S7 | exhaustive routing; transition tables |

## The Playroom's animation levels — girl × scene × format (P4, P6–P9)

The Playroom's back half teaches animation the way Lumen
actually works: one rig, many scenes, many formats — and
**determinism as the pass gate, mechanically**. Every
animation exercise runs from a *named seed*; the TUI shows the
seed, the per-frame parameter table as it plays, and a twin-run
hash. A level passes only when two runs produce byte-identical
frame tables (conversion levels: byte-identical output
artifacts). You don't get told determinism matters. You watch
a hash match — or not.

### The choice set: girl × scene × format

Pick the host girl, pick a scene archetype, pick a format. The
scene changes the motion curves and parameter sets **on the same
rig** — the lesson is that one rig plus a reaction profile
expresses many scenes:

| Scene archetype | Motion/parameter character |
|---|---|
| Milking (P1's scene, reusable) | rhythmic squeeze cycle; reservoir/fullness parameters |
| Rhythm / strobe sync | parameters quantized to the beat grid; phase is the parameter |
| Bondage restraint sway | physics secondary motion *under constraint*: springs with clamped ranges, struggle impulses decaying against the bounds |
| Tease-and-denial pacing | slow curves, long plateaus, threshold play — the sensitivity system as a scene (gain low, threshold near, never over until the authored moment) |
| Lap / club dance (Strobe stage) | looping full-body cycles, accent keyforms on the phrase boundaries |

### The format chain (Lumen's real map)

```
native rig parameters (INP1-generation model space)
  → moc3 V3_0 export        (mapping classes: EXACT / LOSSY / NO-MAP)
  → spritesheet bake + GIF  (frame selection, closure across the bake)
  → funscript timeline      (motion curve → haptic track)
  → GLB / 3D                (designed only: renders-of-the-3D route)
```

The levels that walk this chain are in the level map above:
**P4** (authoring + closure + the twin-run gate), **P6** (the
moc3 border), **P7** (the sprite/GIF bake), **P8** (the
funscript timeline), **P9** (the 3D route, designed only).
Each spec also names its **style lane** (House Cel or Pony
Painterly — see the style section): the format changes, the
lane and the eye rule do not.

## How to read the code after playing

1. `milking-core/src/sim.rs` — the game you just played, minus
   pictures. Find the window constants you learned by feel.
2. `milking-core/src/expression.rs` — the ladder. Five states,
   one step at a time, hysteresis. Twenty lines of rule behind
   every face she made.
3. `milking-core/src/authoring.rs` — your Studio loops as data,
   and `bake()` — the moment authoring becomes playback.
4. `hands-on/src/feedback.rs` — where exercise events become
   her responses, and where the fake-haptics readout clamps.
5. `hands-on/src/level.rs` — the registry: seventeen levels
   across two tracks, five hosts. The whole curriculum is a
   data table.
