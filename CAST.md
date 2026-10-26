# hands-on — Cast of Record (IDEA-0093)

Date: 2026-10-26.
This file is the **cast source of truth** for the crate: one
record per level, seventeen records, P1..P9 and S1..S8. Level
ids and titles are the registry's (`hands-on/src/level.rs`);
they do not change here. Wiring the twelve new hosts below
into `level.rs` is a follow-on pass — this file is records
only: no code was changed, no art was generated.

## Why this file exists — the dual purpose

Hands-On serves two purposes (Owner ruling, 2026-10-26):

1. **It is the Owner's way to learn Rust at the Daddy Style
   standard** — the Study's lessons, the Playroom's systems,
   the gate discipline.
2. **It is the Owner's evaluation surface for Torogao.** Each
   girl, her animations, her sounds, and her level's mechanic
   are prototyped here, in the small, where they can be
   *played*. By playing it himself, the Owner judges each
   piece before it graduates into Torogao production.

That is why every record below carries an **Evaluation**
block: what animation, what sound, and what mechanic this
level exists to put in front of the Owner's hands. A record
is a prototype contract. **Only what passes his play verdict
graduates to Torogao production** — and every verdict in this
file currently reads `pending — Owner play`, because no other
verdict exists yet and no record may carry one until he plays
it.

## Mastery threads

The seventeen records are the **foundation ramp** (§2). Four
mastery threads run through them and beyond them:

- **(a) Bevy / stage systems** — seeded at P3 (ECS ordering on
  a live scene) and exercised wherever a scene must hold
  together under load.
- **(b) The D→Rust port and its tooling** — oracle reading,
  differential gates, moc3 IO: seeded at S1/S2 (format
  parsing, fail-closed gates), P6 (the moc3 border), and
  proven at S8 (the gate suite as a discipline).
- **(c) Animation / rendering / GPU backends** — P4's
  fixed-step rig evaluation, P7/P9's emission routes, S4's
  renderer resource ownership, S7's tracking.
- **(d) Haptics + cross-platform Rust UI** — seeded at P5
  (the router contract) and P8 (timeline-to-haptics), with
  S5 as its Study twin: the Buttplug and Lovense
  integrations, and the Rust UI stack that ships them on
  Linux, Windows, and Android. Developed as Arc D in
  MASTERY.md.

A separate MASTERY.md design develops these threads; this
file is the foundation they stand on and does not contradict
it. Each record's Evaluation names the Torogao-pipeline
competence its mechanic exercises, where one genuinely
applies — and says `core only` where none does.

## The eye law (stated once, binding on every record)

> **Every girl has heart-shaped PUPILS; her iris and the rest
> of the eye keep her own natural color — the heart is pupil
> shape only.** (SPEC §7.2.)

Each record states her natural iris color. A produced face
whose pupils lose the heart, or whose iris changes color, is
off-model — a defect of the same class as a loop that does
not close.

## Style lanes

- **House Cel** — flat cel shading, hard-edged shadow shapes,
  bold lineart, saturated color. The default lane; a record
  is House Cel unless it says otherwise.
- **Pony Painterly** — painterly/glossy: soft gradients,
  glossier skin and eyes. Named only where a record states a
  design reason for it. A build never drifts between lanes
  mid-asset.

## The animation bar (Owner ruling, 2026-10-26)

Every girl's animation is judged against one bar, named
here so no record has to guess at it: the **motion grammar**
of Danbooru post **12068969** and post **9228588** — the 2D
standard (9228588 is the animation study's named ahegao
case) — and, for the sprite representation, post **6277975**.
The bar is the grammar, not the footage: buildup you can
read, spring follow-through, eye behavior, climax
choreography. A loop that hits its hashes but does not move
like the bar does not pass.

## Representation — the medium varies, the style does not

A girl's representation may be **2D, 3D, or sprites** per
level. The constant is the style above — buildup
readability, spring follow-through, eye behavior, climax
choreography — not the medium, and the curriculum
reinforces that style in every representation it teaches.
Records below state a representation only where the level's
design already implies one (P1's built SVG, P7's bake, P9's
3D); where none is implied, none is forced here — the
choice stays open until the level is built.

## The reaction system — live sound + physical reactions (cast-wide)

Every girl gets **live sound and physical reactions as one
coupled system** — the spirit of Danbooru post **11099555**,
where the buildup can be told both visually and by sound,
her eyes roll as she climaxes, and she reacts to each
physical event with her voice peaking, then trailing off.
Cast-wide, that means:

- **Buildup is readable two ways** — in the body and face,
  and in the sound (breath, voice, wetness tracking the
  accumulator).
- **Eyes roll at climax** — the eye behavior runs *through*
  the peak, not up to it.
- **Every physical event draws a synced vocal reaction** —
  the voice peaks with the event, then trails off; sound
  and body are one event, not two tracks that drift.

That is why every Evaluation block below judges sound on
**buildup audibility and per-event vocal sync**, and
animation on **eye behavior through the climax** — those
lines are part of every record, not a garnish on some.

## Records first

**A girl is a record first.** Art exists only by later,
separately gated production (anchor → expression ladder →
the Owner's eye), exactly as the estate's cast work is gated
elsewhere. Today exactly one girl has produced art: Betsy
(P1, the `game.html` SVG). Every other record in this file is
**record only**, and says so.

**A note on sound:** the current build has no recorded audio
or voice — the HTML prototype's only sounds are tiny
synthesized UI blips (SPEC §2.6). The *Sound to judge* fields
below are therefore a spec: the voice character and sound
design each audio prototype will be built against when it is
time to build them — and, per the reaction system above,
they spec a coupled live system (buildup audible, reactions
synced to events), not a folder of disconnected clips.

**Names and designs:** every girl below is an original adult
woman, designed for this crate. No Strobe cast name and no
Light Show cast name is used, and no design is school-coded
in outfit, setting, or framing — anywhere, at any rung.

## The cast at a glance

| Level | Girl | Kink / dynamic | Lane | Rung |
|---|---|---|---|---|
| P1 First Milking | Betsy | milking (no canon entry — the crate's own) | House Cel | Playroom 1 |
| P2 Fixed-Step Rhythm | Sable | B-45 gooning (beat-grid trance) | House Cel | Playroom 2 |
| P3 Stage Systems | Vega | B-12 exhibitionism (the club floor) | Pony Painterly | Playroom 3 |
| P4 Loop Studio | Mireille | B-54 deep subspace (the seamless loop) | House Cel | Playroom 4 |
| P5 Router's Gate | Vigil | B-34 orgasm denial (denial as contract) | House Cel | Playroom 5 |
| P6 Format Border | Petra | B-62 interrogation play (the inspection) | House Cel | Playroom 6 |
| P7 Sprite & GIF Bake | Wren | B-44 edging (held at 99%) | House Cel | Playroom 7 |
| P8 Funscript Timeline | Noor | B-51 remote control devices (the scripted remote) | House Cel | Playroom 8 |
| P9 The Third Dimension | Ines | B-76 voyeur-room performance (the orbit) | House Cel | Playroom 9 |
| S1 Own the Puppet | Alba | B-24 obedience training, praise register (B-74/B-67) | House Cel | Study 1 |
| S2 Fail Closed | Ruth | B-01 total power exchange (the contract ritual) | House Cel | Study 2 |
| S3 The Accumulator | Cleo | B-53 service-oriented submission (keeping the tick) | House Cel | Study 3 |
| S4 Hold the Frame | Yara | B-47 high protocol (positions, held) | House Cel | Study 4 |
| S5 Bounded Queues | Mina | B-35 consensual punishment (correction) | House Cel | Study 5 |
| S6 Async Borders | Sana | B-63 pleasure from pleasing (the answered request) | Pony Painterly | Study 6 |
| S7 Track the Performer | Odette | B-75 voyeurism, watching side (the booth glass) | Pony Painterly | Study 7 |
| S8 Aftercare | June | B-80 aftercare as its own scene | Pony Painterly | Study 8 |

## 2. The ramp — basics first, complex last

The Owner's requirement: the sequence starts with the basic
basics and ramps to the complex stuff. Each track's ramp is
stated explicitly; every record in §3–§4 repeats its own rung
and prerequisites.

### The Playroom ramp

| Rung | Level | What it teaches | Assumes |
|---|---|---|---|
| 1 | P1 First Milking | The fixed-step loop and the squeeze: a parameter (0..1), a tick (60 Hz), a timing window, a reaction profile. The basic basics of the whole crate. | Nothing. This is the door. |
| 2 | P2 Fixed-Step Rhythm | The same tick, judged against a beat grid: timing as a scored discipline. | P1's tick and window discipline. |
| 3 | P3 Stage Systems | Many systems on the one clock: ordering, queries, change detection — the schedule is a contract. | P1's tick; P2's grid discipline under load. |
| 4 | P4 Loop Studio | Authoring: keyforms, easing, loop closure, the bake, the twin-run hash. | P1's parameters and curves; P2's timing. |
| 5 | P5 Router's Gate | The safety contract: routing stimulation lawfully — ceiling, watchdog, synchronous stop. | P1's reaction profiles (what is being routed); P3's ordering (a router is a schedule with teeth). |
| 6 | P6 Format Border | Conversion: the authored loop crosses to moc3 V3_0; mapping judgment (EXACT / LOSSY / NO-MAP). | P4 (something authored, worth converting). |
| 7 | P7 Sprite & GIF Bake | Emission by sampling: frame selection, closure across the bake at hostile frame rates. | P4's closure; P2's grid (sampling is a grid decision); P6's artifact thinking. |
| 8 | P8 Funscript Timeline | Timelines: a motion curve becomes a haptic track, clamped through the router contract. | P5's contract (the track is lawful only through it); P7's sampling. |
| 9 | P9 The Third Dimension | 3D: GLB export and deterministic renders of the 3D — the same determinism in a new medium. The summit. | P4's authoring; P6's conversion judgment; P7's bake discipline. |

### The Study ramp

| Rung | Lesson | What it teaches | Assumes |
|---|---|---|---|
| 1 | S1 Own the Puppet | Ownership, borrowing, lifetimes — the basic basics of Rust (Book ch. 4, 5, 6, 10). | Nothing. This is the desk's first sitting. |
| 2 | S2 Fail Closed | Typed errors and contracts: what failure looks like once you own things, and how to refuse lawfully (ch. 9, 10). | S1's ownership. |
| 3 | S3 The Accumulator | Fixed-step engine arithmetic: integer tick math, iterators, gen blocks (ch. 3, 13). | S1's buffers with owners; S2's no-hidden-behavior contract discipline. |
| 4 | S4 Hold the Frame | Renderer resource ownership: smart pointers, concurrency, one bounded and justified unsafe (ch. 15, 16). | S1's ownership at scale; S3's bounded arithmetic. |
| 5 | S5 Bounded Queues | The haptics router as a Rust problem: named bounds, watchdog, the synchronous stop (ch. 10, 16, 19). | S2's contracts; S4's concurrency. |
| 6 | S6 Async Borders | MCP + A2A surfaces: schemas first, async only at the border, a synchronous deterministic core (ch. 17, 14). | S2's schema discipline; S5's bounded thinking. |
| 7 | S7 Track the Performer | State machines and calibration: enums, exhaustive patterns, let chains (ch. 6, 19). | S1's enums; S6's border events arriving as transitions. |
| 8 | S8 Aftercare | The capstone: tests, review, and the gate suite over everything before (ch. 11, 14). Verification after the intensity. | S1–S7, all of it. |

### Ramp flags — proposals for the Owner (nothing renumbered)

P1 and P4's ids are load-bearing in code and tests, and no
flag below proposes renumbering anything. These are tensions
the Owner should see, with the cheap fix named:

- **RF-1 — Two basic Rust topics arrive late in the Study.**
  Book ch. 3 (types) rides inside S3 and ch. 14 (Cargo /
  workspaces) inside S6/S8, because the Study's order is by
  Torogao use-case complexity, not Book order. The ramp is
  honest, but a beginner meets lifetimes (S1) before plain
  types (S3). *Proposal:* a short "Lesson 0" reading note in
  CURRICULUM covering ch. 3/14 basics — an annotation, not a
  renumber.
- **RF-2 — Cross-track dependencies are implicit.** S5
  (Bounded Queues) reads best *after* P5 (Router's Gate) has
  been played — same router, felt first, formalized second —
  and S7's live-mode material leans on P3's stage thinking.
  The menu presents the tracks as independent. *Proposal:*
  menu/curriculum annotations ("pairs with P5") rather than
  any reordering.
- **RF-3 — P3 is an abstraction spike, accepted knowingly.**
  ECS ordering (P3) is conceptually heavier than the
  hands-on authoring that follows it (P4). The ramp accepts
  this because P4's *gate* is the stricter one (twin-run
  byte-identity), so difficulty still rises overall — concept
  first, then rigor. Flagged so the trade is on record.

## 3. The Playroom records (P1..P9)

### P1 — First Milking — Parameters & Reaction Profiles

- **Rung / prerequisites:** Playroom rung 1 — the fixed-step
  loop and the squeeze. Assumes nothing; this is the door.
- **Host:** Betsy.
- **Identity:** an adult woman; the cowgirl hostess of the
  teaching farm. Warm round face, easy grin; long warm-brown
  hair with caramel highlights (the `game.html` art: base
  `#9c5220`, highlight `#d98a44`); eyes amber-brown
  (`#a34d16`), heart pupils per the eye law; sturdy, curvy
  build; cow ears and small horns, a tail of her own.
- **Palette:** warm browns and caramel, cow-print black and
  white, brass bell-gold, milk white.
- **Outfit & props:** cow-print outfit, cowbell at her neck;
  the fence she leans on, the bottle gauge (the win meter),
  the bucket (the session totalizer).
- **Kink:** milking — the crate's own archetype (SPEC §2/§5).
  The BDSM canon (B-01..B-80) has **no milking/lactation
  entry**; none is forced. Nearest canon neighbor, named as
  neighbor only: B-48 (nipple play).
- **Sensitivity profile:** the balanced default, exactly as
  coded — `SensitivityProfile::default()` in
  `milking-core/src/sim.rs`:
  `gain: 1.0`, `threshold: 96.0`,
  `reservoir_capacity: 220.0`, `reservoir_rate: 0.55`,
  `refractory_ticks: 300`. Forgiving of a beginner's tempo
  while the windows are learned; the reservoir fills at a
  middling rate and the refractory is a patient five seconds
  of ticks.
- **Style lane:** House Cel.
- **Art status:** **built** — original SVG Betsy in
  `game.html`, both views, four authored eye layers; the only
  produced art in this cast.
- **Evaluation:**
  - *Animation to judge:* her idle (fence-lean breathing,
    tail sway scaled by state, cowbell swinging on its lagged
    spring) and the signature motion: the squeeze itself —
    chest-spring follow-through excited by the parameter,
    never keyed — plus the five-state expression ladder
    interpolating one step per tick into the torogao peak.
    Eye behavior is judged through the climax — gaze,
    pupils, and the roll at peak, per the bar above.
  - *Sound to judge:* voice — warm alto, unhurried drawl,
    teasing patience; lines are tempo coaching ("steady, not
    fast") and farm-warm encouragement. Signature effects:
    the cowbell's swing, milk trickling into the bucket, the
    wooden fence's creak under her lean.
    Buildup must be audible as well as visible, and every
    physical event draws a synced vocal reaction — the
    voice peaks with the event, then trails off.
  - *Mechanic to judge:* the reaction profile itself — gain /
    threshold / reservoir / refractory as a *person*: the
    live dial, the HIGH/LOW regimes, burst scaling with the
    reservoir at trigger. The competence exercised is
    **fixed-step reaction-profile evaluation** — the
    `torogao-engine` shape every later level assumes.
  - *Verdict:* pending — Owner play

### P2 — Fixed-Step Rhythm — Beat Sync

- **Rung / prerequisites:** Playroom rung 2 — timing and
  rhythm on the tick. Assumes P1's tick and windows.
- **Host:** Sable.
- **Identity:** an adult woman; a night-drive DJ who lives on
  the beat grid. Sharp, bright face that moves before she
  speaks; jet-black hair in a blunt cut; eyes gold, heart
  pupils per the eye law; lean, quick build; a metronome
  pendant she taps without noticing.
- **Palette:** black and charcoal, booth-glow cyan, gold
  accents.
- **Outfit & props:** oversized headphones (one cup usually
  off an ear), booth jacket; her decks and the scrolling
  beat grid; the metronome pendant.
- **Kink:** rhythm / strobe sync — the level's established
  archetype. Canon: **B-45 (gooning)** — trance edging,
  screen glow, looped arousal: she locks onto the grid the
  way the canon describes the trance, and drift breaks it.
- **Sensitivity profile:** the twitchy regime, exactly as
  coded — `SensitivityProfile::high()` in
  `milking-core/src/sim.rs`: `gain: 2.6`, `threshold: 42.0`,
  `reservoir_capacity: 220.0` (inherited from the default),
  `reservoir_rate: 0.3`, `refractory_ticks: 300`
  (inherited). A few on-beat hits send her over; the
  reservoir barely stores, so the burst is small and the
  lesson is precision, not accumulation.
- **Style lane:** House Cel.
- **Art status:** record only.
- **Evaluation:**
  - *Animation to judge:* her idle is metered — head-bob and
    shoulder ticks exactly on the grid, hair sway quantized
    to the beat; the signature motion is the hit envelope
    itself (attack / sustain / decay drawn on the grid)
    landing on her body, and the visible *stutter* when a
    hit lands frame-perfect but tick-late.
    Eye behavior is judged through the climax — gaze,
    pupils, and the roll at peak, per the bar above.
  - *Sound to judge:* voice — bright, quick soprano, clipped
    DJ patter; she counts beats aloud ("and — four") and
    calls drift the instant it happens. Signature effects:
    the metronome tick, a vinyl-scratch accent on a miss,
    low booth hum under everything.
    Buildup must be audible as well as visible, and every
    physical event draws a synced vocal reaction — the
    voice peaks with the event, then trails off.
  - *Mechanic to judge:* beat-sync judgment against a
    deterministic clock — hit windows the *core* defines,
    provably independent of frame rate. The competence
    exercised is **deterministic beat-grid evaluation**: the
    shared engine clock Torogao's fixed step and Strobe's
    sync both stand on.
  - *Verdict:* pending — Owner play

### P3 — Stage Systems — Order the Club Scene

- **Rung / prerequisites:** Playroom rung 3 — system
  ordering. Assumes P1's tick; P2's grid discipline under
  load.
- **Host:** Vega.
- **Identity:** an adult woman; stage manager of the Strobe
  club, who runs the show from the booth and notices every
  dropped frame. Composed, striking face; long violet hair;
  eyes emerald green, heart pupils per the eye law; tall,
  statuesque build; a cue-light bracelet she checks like a
  watch.
- **Palette:** deep violet and black, emerald cue-light
  green, sequin silver.
- **Outfit & props:** tailored stage blacks with a talkback
  headset; the booth, the text schedule board, the crowd
  (event load) below; a sequined wrap she dons for the
  closing number.
- **Kink:** the club-dance archetype — the scene is hers to
  order, and hers to be seen in once it holds together.
  Canon: **B-12 (exhibitionism)** — the thrill of the floor
  watching, earned by a scene that never stumbles.
- **Sensitivity profile:** the slow-build regime, exactly as
  coded — `SensitivityProfile::low()` in
  `milking-core/src/sim.rs`: `gain: 0.45`,
  `threshold: 98.0`, `reservoir_capacity: 420.0`,
  `reservoir_rate: 0.9`, `refractory_ticks: 300`
  (inherited). A scene that holds together all night fills
  the deepest reservoir in the cast and pays out enormously;
  instability cools her on the spot.
- **Style lane:** **Pony Painterly** — stated reason: the
  club's neon-gloss register is the one Playroom subject
  whose light is painterly by nature; her gloss is the
  stage's gloss.
- **Art status:** record only.
- **Evaluation:**
  - *Animation to judge:* her idle (booth sway, eyes
    scanning the floor, bracelet checks); the signature
    motion is the closing number — a looping full-body
    club-dance cycle with accent keyforms on the phrase
    boundaries — and its failure form: the desync stutter
    when a system is scheduled before its producer.
    Eye behavior is judged through the climax — gaze,
    pupils, and the roll at peak, per the bar above.
  - *Sound to judge:* voice — low contralto, calm command
    register; lines are stage-manager cues ("places",
    "hold", "take it again") that warm as the scene holds.
    Signature effects: club room tone, a distant bass pulse
    that steadies as the schedule stabilizes, the talkback
    click.
    Buildup must be audible as well as visible, and every
    physical event draws a synced vocal reaction — the
    voice peaks with the event, then trails off.
  - *Mechanic to judge:* ECS ordering as play — input → rig
    / animation → physics → haptics → render prep, declared
    queries, change detection, fixed-timestep stability
    under rising event load. The competence exercised is
    **Bevy ECS ordering** (mastery thread a) — the schedule
    *is* the program.
  - *Verdict:* pending — Owner play

### P4 — Loop Studio — Closure, Bake, Twin-Run Hash

- **Rung / prerequisites:** Playroom rung 4 — loop
  authoring. Assumes P1's parameters and curves; P2's
  timing.
- **Host:** Mireille.
- **Identity:** an adult woman; a stop-motion animator.
  Thoughtful face, quick precise eyes; chestnut-brown hair
  pinned up with pencils; eyes hazel, heart pupils per the
  eye law; slight, deft build; ink and pencil marks on her
  fingers that never quite wash off.
- **Palette:** chestnut and paper cream, lightbox white,
  pencil graphite, splice-tape red.
- **Outfit & props:** animator's smock with pencil pockets;
  her lightbox, keyform pegs, and a garland of film strip
  she is always mid-way through inspecting.
- **Kink:** the loop as trance — she performs loops she has
  authored until the seam disappears and she drops into
  them. Canon: **B-54 (deep subspace)** — dissociated
  bliss, lost in sensation; here the sensation is a motion
  with no seam to wake her.
- **Sensitivity profile:** exactly as coded —
  `mireille_profile()` in `hands-on/src/level.rs`:
  `gain: 1.4`, `threshold: 65.0`,
  `reservoir_capacity: 140.0`, `reservoir_rate: 0.45`,
  `refractory_ticks: 240`. Mid gain, the cast's smallest
  reservoir and shortest refractory: she cycles — author,
  peak, recover, author again — faster than anyone.
- **Style lane:** House Cel.
- **Art status:** record only. **Teaching-material note:**
  the playable slice's material is **Betsy's** loop (seed
  `betsy-loop-a1`, twin-run hash `88dd763631bc07f9`) even
  though Mireille hosts the level — the slice runs on Betsy;
  Mireille hosts the level's full design.
- **Evaluation:**
  - *Animation to judge:* her idle (hands turning an
    invisible strip, eyes flicking frame to frame); the
    signature motion is the authored loop itself, playing on
    her seamlessly and forever — and its failure form, the
    seam-pop, which must be *visible* when closure fails.
    Eye behavior is judged through the climax — gaze,
    pupils, and the roll at peak, per the bar above.
  - *Sound to judge:* voice — soft mezzo, precise; she
    mutters frame numbers and closure verdicts like a
    animator's notebook spoken aloud ("…forty-one, forty-two
    — closes"). Signature effects: film-strip clicks, pencil
    ticks on the pegboard, the soft splice blip when a loop
    closes.
    Buildup must be audible as well as visible, and every
    physical event draws a synced vocal reaction — the
    voice peaks with the event, then trails off.
  - *Mechanic to judge:* the authoring feel — placing
    keyforms, choosing easing, chasing closure, and the
    twin-run hash as the pass gate. The competence exercised
    is **fixed-step rig evaluation** (mastery thread c): the
    same seed producing byte-identical frame tables, twice.
  - *Verdict:* pending — Owner play

### P5 — Router's Gate — Haptics Safety

- **Rung / prerequisites:** Playroom rung 5 — the safety
  contract. Assumes P1's reaction profiles; P3's ordering.
- **Host:** Vigil.
- **Identity:** an adult woman; gatekeeper of the router
  shrine. Severe, beautiful face that gives nothing away
  early; silver-white hair worn as braided cables; eyes
  crimson, heart pupils per the eye law; straight, exact
  posture; a brass stop-key on a chain at her throat, which
  her hand rests on when she is deciding.
- **Palette:** silver-white and gunmetal, brass, signal red.
- **Outfit & props:** a high-collared gatekeeper's coat with
  cable-braid trim; the shrine gate itself, the ceiling
  dial, the heartbeat lamp; the stop-key.
- **Kink:** the router contract as denial — nothing reaches
  her except through the gate she controls, on the terms
  agreed in advance; an unlawful signal is not punished, it
  is *refused*, and she cools. Canon: **B-34 (orgasm
  denial)** — held back, denied release, by a rule she set
  and keeps.
- **Sensitivity profile:** exactly as coded —
  `vigil_profile()` in `hands-on/src/level.rs`:
  `gain: 0.9`, `threshold: 85.0`,
  `reservoir_capacity: 260.0`, `reservoir_rate: 0.6`,
  `refractory_ticks: 360`. Exacting: spills cost her trust
  fast, and the cast's longest refractory means a climax,
  once earned, closes her gate for a long while.
- **Style lane:** House Cel.
- **Art status:** record only.
- **Evaluation:**
  - *Animation to judge:* her idle is stillness — breath,
    the stop-key turning slowly in her fingers, the
    heartbeat lamp's pulse; the signature motion is a lawful
    pulse traveling visibly through the gate to her — and
    its opposite: the hard, total freeze of a synchronous
    stop, mid-motion, with nothing half-delivered.
    Eye behavior is judged through the climax — gaze,
    pupils, and the roll at peak, per the bar above.
  - *Sound to judge:* voice — measured mezzo, formal; she
    enunciates the contract's terms as rulings ("over
    ceiling. denied."). Signature effects: the brass key's
    turn, the gate latch, the watchdog heartbeat tick that
    must never skip.
    Buildup must be audible as well as visible, and every
    physical event draws a synced vocal reaction — the
    voice peaks with the event, then trails off.
  - *Mechanic to judge:* the torogao-haptics router
    contract in play — ceiling clamp, heartbeat watchdog,
    synchronous stop, no resume on reconnect — felt as a
    game before it is read as a spec. The competence
    exercised is the **haptics router contract** itself —
    the torogao-haptics integration competence (router and
    gate discipline over the Buttplug and Lovense backends),
    the safety surface every Torogao output crosses.
  - *Verdict:* pending — Owner play

### P6 — Format Border — moc3 V3_0 Export & the Loss Report

- **Rung / prerequisites:** Playroom rung 6 — format
  conversion. Assumes P4 (something authored to convert).
- **Host:** Petra. *(New girl.)*
- **Identity:** an adult woman; the border inspector of
  formats — nothing crosses without a class. Level,
  appraising face; ash-blonde hair in a severe low bun;
  eyes steel gray, heart pupils per the eye law; upright,
  economical build; a loupe on a cord she raises to inspect
  whatever is presented to her.
- **Palette:** ash blonde and slate, customs navy, stamp
  inks in three colors (exact green, lossy amber, no-map
  red).
- **Outfit & props:** a tailored inspector's coat (adult
  professional dress, worn like a uniform of office); her
  stamp belt with three stamps — EXACT, LOSSY, NO-MAP — the
  loupe, and a manifest folder that is never out of order.
- **Kink:** the inspection — every construct is questioned
  until it confesses its class, and she enjoys the
  questioning. Canon: **B-62 (interrogation play)** —
  questioning, teased answers; here the answers are mapping
  classes and the refusals are typed.
- **Sensitivity profile (sketch):** mid gain, high
  threshold — correctness warms her slowly and sloppiness
  not at all; a small, slow reservoir (she stores little;
  she *files* it); refractory long and procedural: after a
  climax she re-checks every stamp before she can be
  approached again.
- **Style lane:** House Cel.
- **Art status:** record only.
- **Evaluation:**
  - *Animation to judge:* her idle (stamp practice motions,
    the loupe raising and lowering, manifest pages turned
    in order); the signature motion is a crossing: a
    construct rendered in native form walks the line and
    emerges re-rendered in the target format — unchanged
    (EXACT), visibly compromised (LOSSY), or turned back at
    the line (NO-MAP), the class legible in the motion
    itself.
    Eye behavior is judged through the climax — gaze,
    pupils, and the roll at peak, per the bar above.
  - *Sound to judge:* voice — crisp alto, customs-officer
    formality; lines are inspection questions and verdicts
    ("state your mapping"). Signature effects: three stamp
    thunks at three pitches, manifest paper sliding, the
    loupe's click.
    Buildup must be audible as well as visible, and every
    physical event draws a synced vocal reaction — the
    voice peaks with the event, then trails off.
  - *Mechanic to judge:* mapping judgment — predicting
    EXACT / LOSSY / NO-MAP per construct before the loss
    report grades you, and the discipline of typed refusal
    over silent downgrade. The competence exercised is
    **moc3 mapping judgment** (mastery thread b), the
    exporter's whole art.
  - *Verdict:* pending — Owner play

### P7 — Sprite & GIF Bake — Frames That Loop

- **Rung / prerequisites:** Playroom rung 7 — bakes and
  sampling. Assumes P4's closure; P2's grid; P6's artifact
  thinking.
- **Host:** Wren. *(New girl.)*
- **Identity:** an adult woman; the frame tender of the
  bakehouse-darkroom where loops are baked. Bright,
  impatient face; short choppy copper-red hair; eyes warm
  brown, heart pupils per the eye law; small, quick build;
  she counts on her fingers constantly, including in her
  sleep, colleagues claim.
- **Palette:** copper and cream, darkroom red, timer brass.
- **Outfit & props:** a work apron over rolled sleeves, one
  heat glove; the bake oven with its frame window, the
  shutter counter, a timer pendant that ticks audibly.
- **Kink:** the bake's anticipation — a loop held at 99%
  while the frames finish, again and again, by her own
  schedule. Canon: **B-44 (edging)** — sustained edging,
  held at the edge, trembling, denied release until the
  authored moment: closure.
- **Sensitivity profile (sketch):** high gain and a low-mid
  threshold — she heats fast; a reservoir that fills in
  spurts, the way sampling drops frames; refractory short,
  which is her problem and her charm: she is ready to bake
  again almost at once, and the level keeps holding her at
  the edge anyway.
- **Style lane:** House Cel.
- **Art status:** record only.
- **Evaluation:**
  - *Animation to judge:* her idle (counting frames on her
    fingers, glancing at the oven window); the signature
    motion is the bake itself — frames visibly dropping out
    under sampling while the loop on her body must still
    close — motion that survives GIF frame rates with the
    eye rule intact at a handful of pixels per pupil.
    Eye behavior is judged through the climax — gaze,
    pupils, and the roll at peak, per the bar above.
  - *Sound to judge:* voice — light soprano; she counts
    under her breath and laughs when a bake hits 99%.
    Signature effects: oven-timer ticks, shutter clicks in
    volleys, and the bake-ding that must never come early.
    Buildup must be audible as well as visible, and every
    physical event draws a synced vocal reaction — the
    voice peaks with the event, then trails off.
  - *Mechanic to judge:* frame selection under sampling —
    which phases survive, and whether closure survives
    them; the twin-run hash on the *baked artifact* as
    judge. The competence exercised is **deterministic
    artifact emission** (mastery thread c): the Lumen
    sprite baker's discipline of a loop that closes at
    every rate it ships at.
  - *Verdict:* pending — Owner play

### P8 — Funscript Timeline — Motion Becomes Touch

- **Rung / prerequisites:** Playroom rung 8 — timelines.
  Assumes P5's router contract; P7's sampling.
- **Host:** Noor. *(New girl.)*
- **Identity:** an adult woman; the booth's timeline
  percussionist, who turns motion curves into touch. Calm,
  hooded-eyed face that listens more than it watches;
  espresso-brown hair worn long and loose; eyes deep brown,
  heart pupils per the eye law; graceful, unhurried build;
  faint callus shine on her fingertips from the pulse pads.
- **Palette:** espresso and amber, dusk purple, pad-light
  rose.
- **Outfit & props:** fingertip percussion caps, a scarf
  printed with a waveform; the timeline desk, the pulse
  pads, the clamped readout.
- **Kink:** she performs to a script someone else wrote —
  a timeline of touch authored in advance, arriving on
  schedule whether she is ready or not, and she adores the
  appointment. Canon: **B-51 (remote control devices)** —
  the funscript *is* a remote control written ahead of
  time; the level makes the writing of it the play.
- **Sensitivity profile (sketch):** mid gain, mid
  threshold; a steady, deep reservoir that fills evenly —
  no spurts, no shortcuts; refractory musical: she
  recovers in phrases, sensitivity returning on a count of
  bars rather than all at once.
- **Style lane:** House Cel.
- **Art status:** record only.
- **Evaluation:**
  - *Animation to judge:* her idle (fingertips drumming
    patterns on the desk, head tilted to a track only she
    hears); the signature motion is translation made
    visible: an authored curve on the desk becoming pulses
    her body answers, beat for beat — the difference
    between the authored curve and the lawful, clamped
    emission readable in how she moves.
    Eye behavior is judged through the climax — gaze,
    pupils, and the roll at peak, per the bar above.
  - *Sound to judge:* voice — husky mezzo; she speaks in
    rhythm and taps as she talks, narrating the track
    ("rising — hold — there"). Signature effects:
    fingertip drum patterns, pulse thumps sampled from the
    timeline itself, the router clamp's soft ceiling tone.
    Buildup must be audible as well as visible, and every
    physical event draws a synced vocal reaction — the
    voice peaks with the event, then trails off.
  - *Mechanic to judge:* curve-to-track conversion —
    sampling the squeeze parameter to a funscript timeline
    and hearing/feeling what the router contract lawfully
    emits versus what was authored. The competence
    exercised is **timeline-to-haptics conversion**: the
    Lumen funscript writer feeding the torogao-haptics
    player, end to end — the haptics integration competence
    in its timeline form (authored curve in, lawful device
    emission out, through the router).
  - *Verdict:* pending — Owner play

### P9 — The Third Dimension — GLB & Renders of the 3D

- **Rung / prerequisites:** Playroom rung 9 — 3D, the
  summit. Assumes P4's authoring; P6's conversion judgment;
  P7's bake discipline.
- **Host:** Ines. *(New girl.)*
- **Identity:** an adult woman; a sculptor's model who
  holds poses for the orbiting camera. Serene, classical
  face; lilac-silver hair in a long fall; eyes violet,
  heart pupils per the eye law; statuesque build, trained
  stillness; a sculptor's caliper mark — a small ink dot —
  at one shoulder, her registration point.
- **Palette:** lilac and silver, marble white, studio
  charcoal.
- **Outfit & props:** a classical draped wrap (a studio
  garment, worn as sculptors' models have always worn it);
  the pose stand, the orbit camera rail, the calipers.
- **Kink:** she performs to be rendered — watched from
  every angle by an audience of cameras, every render a
  sitting she holds herself still for. Canon: **B-76
  (voyeur-room performance)** — performing behind the
  glass, watched; here the glass is the viewport and the
  watchers are deterministic.
- **Sensitivity profile (sketch):** low gain, high
  threshold; a vast, slow reservoir — a long posed build
  with almost nothing stored per minute and everything
  stored by the end; refractory statuesque: a long, still
  recovery in which she does not so much cool as settle.
- **Style lane:** House Cel — in 3D the lane is a
  toon-shading material contract (CURRICULUM, P9), and the
  eye rule survives as geometry + texture, not pixels.
- **Art status:** record only. (The level itself is
  designed only, by its own terms.)
- **Evaluation:**
  - *Animation to judge:* her idle (pose-holding with
    micro-shifts, breath disciplined to near-invisibility);
    the signature motion is the orbit itself — the camera
    circling her render while she holds, five angle renders
    that must twin-run byte-identical.
    Eye behavior is judged through the climax — gaze,
    pupils, and the roll at peak, per the bar above.
  - *Sound to judge:* voice — serene alto, few words,
    breath-led; lines are pose calls and quiet confirmations
    ("holding"). Signature effects: orbit shutter clicks,
    the turntable's hum, studio hush.
    Buildup must be audible as well as visible, and every
    physical event draws a synced vocal reaction — the
    voice peaks with the event, then trails off.
  - *Mechanic to judge:* the 3D route's determinism — GLB
    export and renders-of-the-3D from one record, same
    seed, byte-identical renders. The competence exercised
    is **3D emission and render determinism** (mastery
    thread c): the factory's hardest promise, judged at the
    top of the ramp.
  - *Verdict:* pending — Owner play

## 4. The Study records (S1..S8)

The Study's frame is one dynamic throughout: you are
summoned, the host sets the requirements, the compiler
corrects you at the desk (SPEC §6.5, §7.4 — Daddy Style).
Each lesson's host rides that dynamic in her own flavor.

### S1 — Own the Puppet — Ownership, Borrowing, Lifetimes

- **Rung / prerequisites:** Study rung 1 — ownership, the
  basic basics of Rust. Assumes nothing; the first sitting.
- **Host:** Alba. *(New girl.)*
- **Identity:** an adult woman; the Study's puppet-keeper,
  who tends the jointed puppet the parsing lessons are
  practiced on. Open, eager face; strawberry-blonde hair in
  a loose braid; eyes green, heart pupils per the eye law;
  soft, sturdy build; a ribbon at her throat she adjusts
  when she has been praised and is trying not to show it.
- **Palette:** strawberry blonde and cream, ribbon scarlet,
  desk oak.
- **Outfit & props:** a simple desk dress with the ribbon;
  the puppet itself — jointed, patient, endlessly
  re-poseable — and its stand.
- **Kink / dynamic:** **B-24 (obedience training)** —
  kneeling-at-the-desk register translated to the sitting:
  posture, commands, devotion to doing it *as instructed*.
  Her reward register is praise — **B-74** (praise kink)
  working inside B-24's frame, paid as **B-67** (rewarded
  good girl): follow the ownership rule exactly, and be
  told so, warmly, by name.
- **Sensitivity profile (sketch):** mid-high gain with a
  threshold that drops for praise — words land on her at
  full gain, code merely well; a small reservoir (she does
  not store much; she *responds*); refractory short: she
  resets eager, ready to be instructed again.
- **Style lane:** House Cel.
- **Art status:** record only.
- **Evaluation:**
  - *Animation to judge:* her idle (holding the puppet's
    pose, glancing up for instruction); the signature
    motion is the held pose itself — she holds perfectly
    while your borrow lives, and when you drop it wrong she
    resets *cleanly*, unruffled, never collapsing: the
    reset is the lesson made visible.
    Eye behavior is judged through the climax — gaze,
    pupils, and the roll at peak, per the bar above.
  - *Sound to judge:* voice — gentle soprano, eager to
    please; she asks "like this?" and audibly melts at
    exact praise. Signature effects: the desk bell she
    rings for a correct hold, soft page turns, the puppet's
    wooden joints.
    Buildup must be audible as well as visible, and every
    physical event draws a synced vocal reaction — the
    voice peaks with the event, then trails off.
  - *Mechanic to judge:* ownership repair as play —
    arranging moves and borrows so the parse holds, with
    every borrow error staged as the sitting stopping
    safely. The competence exercised is **torogao-format
    parsing discipline** (mastery thread b): buffers with
    owners, the port's first habit.
  - *Verdict:* pending — Owner play

### S2 — Fail Closed — Typed Errors & Contracts

- **Rung / prerequisites:** Study rung 2 — typed errors and
  contracts. Assumes S1's ownership.
- **Host:** Ruth. *(New girl.)*
- **Identity:** an adult woman; the Study's terms-keeper,
  who drafts the contracts the desk runs on. Unsentimental,
  handsome face; dark auburn hair drawn back plainly; eyes
  gray-green, heart pupils per the eye law; tall, spare
  build; reading glasses on a chain, taken off when a
  clause displeases her.
- **Palette:** auburn and charcoal, contract cream, seal-wax
  red.
- **Outfit & props:** a severe tailored suit; the clause
  book, the seal press, a pen that is only ever used for
  signatures and refusals.
- **Kink / dynamic:** **B-01 (total power exchange)** —
  the contract ritual itself: limits named, terms explicit,
  devotion entered into *in writing*, willingly. This is
  the lesson where contracts/limits live as a topic (SPEC
  §6.5): she does not merely keep the terms; the terms are
  the scene.
- **Sensitivity profile (sketch):** low gain, very high
  threshold; a nearly sealed reservoir — almost nothing
  stores except exact correctness, filed clause by clause;
  refractory absolute: after the rare climax, a full
  re-reading of the terms before anything resumes.
- **Style lane:** House Cel.
- **Art status:** record only.
- **Evaluation:**
  - *Animation to judge:* her idle (reading, pen tapping a
    clause into rhythm); the signature motion is the
    refusal made visible — a doctored header presented, her
    glasses coming off, and the contract book closing on
    the exact clause that condemns it. Typed, specific,
    never vague.
    Eye behavior is judged through the climax — gaze,
    pupils, and the roll at peak, per the bar above.
  - *Sound to judge:* voice — dry alto; she reads clauses
    verbatim and ornaments nothing. Signature effects: pen
    strokes, the seal press, the book's close — the
    refusal's full stop.
    Buildup must be audible as well as visible, and every
    physical event draws a synced vocal reaction — the
    voice peaks with the event, then trails off.
  - *Mechanic to judge:* the fail-closed gate as play —
    choosing the error variant each doctored input
    deserves, and learning that a stringly failure is
    itself a breach. The competence exercised is **moc3 IO
    gate discipline** (mastery thread b): the
    importer/exporter license gate, refusing typed, before
    any model is read.
  - *Verdict:* pending — Owner play

### S3 — The Accumulator — Fixed-Step Engine Arithmetic

- **Rung / prerequisites:** Study rung 3 — engine
  arithmetic. Assumes S1's owned buffers; S2's contract
  discipline.
- **Host:** Cleo. *(New girl.)*
- **Identity:** an adult woman; the Study's timekeeper, who
  keeps the tick the whole house runs on. Precise, amused
  face; rich mahogany hair in a neat bob; eyes topaz, heart
  pupils per the eye law; compact, exact build; a
  metronome brooch at her collar that she winds like
  jewelry.
- **Palette:** mahogany and brass, topaz, ledger green.
- **Outfit & props:** a tailored waistcoat dress; the
  pocket watch on its chain, the Study clock, the tick
  ledger in which every beat is accounted.
- **Kink / dynamic:** **B-53 (service-oriented
  submission)** — devotion through service, and her service
  is *time itself*: she keeps the tick for the desk,
  perfectly, as an act of devotion, and takes her pleasure
  in its never slipping.
- **Sensitivity profile (sketch):** exact mid gain, mid
  threshold; a reservoir that fills *only on-tick* — in
  character, off-beat stimulation simply does not store;
  refractory counted: a fixed number of beats, announced
  in advance, not negotiable mid-window.
- **Style lane:** House Cel.
- **Art status:** record only.
- **Evaluation:**
  - *Animation to judge:* her idle (metronomic sway, watch
    consulted on the beat); the signature motion is
    phase-lock as dance — she stays locked to your clock
    across thousands of ticks, and drift, when your naive
    version introduces it, shows on her face as a gradual
    phase slip before it shows in your numbers.
    Eye behavior is judged through the climax — gaze,
    pupils, and the roll at peak, per the bar above.
  - *Sound to judge:* voice — precise mezzo; she speaks in
    counted phrases and finishes sentences on beats.
    Signature effects: the metronome, tick layers settling
    into a groove, and the drift alarm — two ticks audibly
    slipping apart.
    Buildup must be audible as well as visible, and every
    physical event draws a synced vocal reaction — the
    voice peaks with the event, then trails off.
  - *Mechanic to judge:* the accumulator itself — integer
    tick arithmetic, no floats in the contract, frame
    counts proven against a drifting naive implementation.
    The competence exercised is **fixed-step engine
    arithmetic**: the torogao-engine evaluation order,
    learned in the hands.
  - *Verdict:* pending — Owner play

### S4 — Hold the Frame — Renderer Resource Ownership

- **Rung / prerequisites:** Study rung 4 — renderer
  resources. Assumes S1's ownership at scale; S3's bounded
  arithmetic.
- **Host:** Yara. *(New girl.)*
- **Identity:** an adult woman; the Study's frame-keeper,
  who carries the gilt frame — the render target — and
  every resource in it. Composed, formal face; midnight-
  blue hair in a smooth fall; eyes black-brown, heart
  pupils per the eye law; statuesque, deliberate build;
  white gloves, because protocol is not a metaphor to her.
- **Palette:** midnight blue and gilt, glove white, GPU
  teal.
- **Outfit & props:** a formal keeper's dress with the
  gloves; the gilt frame; resource trays balanced like
  glassware.
- **Kink / dynamic:** **B-47 (high protocol)** — formal
  protocol, positions, address, ritualized service: she
  *holds positions* — the frame, the pose, the resource —
  exactly as protocol specifies, for exactly as long as
  specified, and the holding is the devotion.
- **Sensitivity profile (sketch):** mid gain, mid-high
  threshold; a moderate reservoir that fills in protocol
  order — presentation first, then warmth; refractory
  formal: recovery proceeds through its steps, resources
  re-owned one by one, never skipped.
- **Style lane:** House Cel.
- **Art status:** record only.
- **Evaluation:**
  - *Animation to judge:* her idle (protocol stillness,
    the frame held level, breath minimal); the signature
    motion is ownership made visible — each resource in
    the frame lit while it has an owner, and an orphaned
    handle *flickering her* until it is re-owned or dropped
    in order.
    Eye behavior is judged through the climax — gaze,
    pupils, and the roll at peak, per the bar above.
  - *Sound to judge:* voice — composed alto, protocol
    address ("as agreed", "in order"); she names each
    resource as she takes it up. Signature effects: tray
    chimes like glassware, the frame's set-down thunk, a
    low hush that swells like a fan under load.
    Buildup must be audible as well as visible, and every
    physical event draws a synced vocal reaction — the
    voice peaks with the event, then trails off.
  - *Mechanic to judge:* resource lifecycle as play —
    ordering the drops, placing the one justified unsafe
    block, bounding it, fencing it. The competence
    exercised is **renderer resource ownership** (mastery
    thread c): the winit/wgpu/egui discipline Torogao's
    renderer stands on.
  - *Verdict:* pending — Owner play

### S5 — Bounded Queues — The Haptics Router

- **Rung / prerequisites:** Study rung 5 — bounded queues.
  Assumes S2's contracts; S4's concurrency. (Pairs with P5 —
  see ramp flag RF-2.)
- **Host:** Mina. *(New girl.)*
- **Identity:** an adult woman; the queue mistress of the
  Study's router annex. Firm, teasing face that goes exact
  the moment a bound is tested; wine-red hair in a high
  tail; eyes green, heart pupils per the eye law; athletic
  build; a belt of queue tokens she moves hand to hand at
  a measured pace.
- **Palette:** wine red and black, token brass, watchdog
  amber.
- **Outfit & props:** a fitted annex uniform of her own
  design; the token belt, the bound ledger, the watchdog
  lamp.
- **Kink / dynamic:** **B-35 (consensual punishment)** —
  correction, chosen in advance and agreed in the terms: a
  bound breached earns exactly the discipline the contract
  names, delivered without anger and without surprise.
  Discipline as care, enforced at the desk.
- **Sensitivity profile (sketch):** firm mid gain, high
  threshold; a reservoir bounded *hard* — small capacity,
  insisted on, because unbounded storage is how routers
  and girls both get hurt; refractory is the stop itself:
  after a climax, everything halts for a long, total
  window, and she keeps it.
- **Style lane:** House Cel.
- **Art status:** record only.
- **Evaluation:**
  - *Animation to judge:* her idle (tokens moving hand to
    hand, the watchdog lamp's pulse checked); the
    signature motion is the mid-stream stop — a flowing
    sequence halting on the same tick, everywhere, with
    nothing half-delivered and nothing spilled.
    Eye behavior is judged through the climax — gaze,
    pupils, and the roll at peak, per the bar above.
  - *Sound to judge:* voice — firm mezzo, teasing first
    and exact when a bound breaks; she announces bounds
    before she enforces them. Signature effects: token
    clicks in measured volleys, the watchdog heartbeat,
    and the stop's sudden total silence.
    Buildup must be audible as well as visible, and every
    physical event draws a synced vocal reaction — the
    voice peaks with the event, then trails off.
  - *Mechanic to judge:* naming every queue's bound before
    it runs, setting the watchdog, rehearsing the
    synchronous stop mid-stream. The competence exercised
    is **bounded-queue router discipline**: the
    torogao-haptics router as a concurrency problem with a
    body attached.
  - *Verdict:* pending — Owner play

### S6 — Async Borders — MCP + A2A Surfaces

- **Rung / prerequisites:** Study rung 6 — async borders.
  Assumes S2's schema discipline; S5's bounded thinking.
- **Host:** Sana. *(New girl.)*
- **Identity:** an adult woman; the correspondence keeper
  of the Study, through whose desk every request crosses
  to the world outside and back. Warm, delighted face;
  deep-teal hair in soft waves; eyes honey-brown, heart
  pupils per the eye law; rounded, welcoming build; ink on
  one fingertip, always, from other people's letters.
- **Palette:** deep teal and cream, honey, request-bell
  silver.
- **Outfit & props:** a correspondence keeper's dress with
  rolled sleeves; the message trays (in / out, never
  mixed), the schema seals, the request bells.
- **Kink / dynamic:** **B-63 (pleasure from pleasing)** —
  the answered request is the act: a well-formed request,
  answered well, on the far side of the border, is her
  pleasure taken in the other side's satisfaction.
- **Sensitivity profile (sketch):** responsive gain, mid
  threshold; a reservoir that fills per *well-formed*
  request — malformed ones store nothing, and she feels
  the difference instantly; refractory asynchronous in
  flavor: she returns in her own time, never on a blocking
  call, and resents being awaited badly.
- **Style lane:** **Pony Painterly** — stated reason: hers
  is the lesson of surfaces and correspondence, and the
  painterly gloss is the crate's *presentation* register —
  the face the house shows the world across the border.
- **Art status:** record only.
- **Evaluation:**
  - *Animation to judge:* her idle (sorting the trays,
    small nods as requests arrive); the signature motion
    is cadence — request and response moving across her
    desk like a rally, and the visible *skip* when a
    schema lie reaches her: one beat simply not taken.
    Eye behavior is judged through the climax — gaze,
    pupils, and the roll at peak, per the bar above.
  - *Sound to judge:* voice — warm soprano, a
    letter-reader's lilt; she reads requests aloud with
    genuine delight when they are well-formed. Signature
    effects: message chimes at the border, letter slides,
    a soft buzz — never harsh — for a schema error.
    Buildup must be audible as well as visible, and every
    physical event draws a synced vocal reaction — the
    voice peaks with the event, then trails off.
  - *Mechanic to judge:* wiring a tool surface — schemas
    first, async closures only at the border, the core
    synchronous and deterministic underneath. The
    competence exercised is **MCP + A2A surface
    discipline**: the registry's outer face, where Torogao
    meets other agents.
  - *Verdict:* pending — Owner play

### S7 — Track the Performer — State Machines & Calibration

- **Rung / prerequisites:** Study rung 7 — state machines
  and tracking. Assumes S1's enums; S6's border events.
- **Host:** Odette. *(New girl.)*
- **Identity:** an adult woman; the watcher in the booth,
  who tracks the performer through the glass and calls
  every state. Knowing, unhurried face; rose-pink hair in
  a long fall; eyes deep blue, heart pupils per the eye
  law; willowy build; opera glasses she raises to *name*
  a transition, not to magnify it.
- **Palette:** rose pink and booth black, glass teal,
  calibration-chart primaries.
- **Outfit & props:** a watcher's evening dress (the booth
  is a place of occasion to her); the opera glasses, the
  state-call cards, the calibration chart.
- **Kink / dynamic:** **B-75 (voyeurism, watching side)**
  — aroused by watching, her attention a form of guidance:
  the performer performs *because* Odette's gaze is on the
  glass, and her called states are how the gaze touches.
- **Sensitivity profile (sketch):** a watcher's profile —
  gain rises with *observed* accuracy (she responds to
  what she can see being done right); mid threshold; a
  reservoir that mirrors the performer's state more than
  her own; refractory observational: she must watch the
  full descent, one step at a time, before she will
  re-engage.
- **Style lane:** **Pony Painterly** — stated reason: the
  watcher is seen through glass and gloss; the painterly
  lane is the register of being *looked at*, and she is
  the cast's professional looker.
- **Art status:** record only.
- **Evaluation:**
  - *Animation to judge:* her idle (head tracking an
    unseen performer, glasses rising at moments only she
    perceives); the signature motion is composure as a
    state diagram — every transition the performer makes
    is named on her face and cards in the same tick, and
    an unhandled transition simply never appears: the
    compiler refused it upstream.
    Eye behavior is judged through the climax — gaze,
    pupils, and the roll at peak, per the bar above.
  - *Sound to judge:* voice — breathy alto; she narrates
    what she watches, state names spoken like calls
    ("calibrating — locked — drifting"). Signature
    effects: a tap on the booth glass, calibration beeps,
    a soft chime per transition.
    Buildup must be audible as well as visible, and every
    physical event draws a synced vocal reaction — the
    voice peaks with the event, then trails off.
  - *Mechanic to judge:* the tracking state machine —
    states, transitions, calibration states, written so an
    unhandled transition is a pattern the compiler
    rejects. The competence exercised is **live-mode
    tracking/interop state modeling**: HOST-07's discipline
    (BRIEF-P6 Item 2), the performer's state owned
    exhaustively.
  - *Verdict:* pending — Owner play

### S8 — Aftercare — Tests, Review & the Gate Suite

- **Rung / prerequisites:** Study rung 8 — the capstone.
  Assumes S1–S7, all of it; pairs with the whole Playroom.
- **Host:** June. *(New girl.)*
- **Identity:** an adult woman; the Study's aftercare
  keeper, in whose room every sitting ends. Gentle,
  unlined face; golden-brown hair worn soft and loose;
  eyes warm brown, heart pupils per the eye law; a
  comforting build; a blanket over one arm that is never
  folded away, because someone always needs it next.
- **Palette:** golden brown and cream, blanket rose, tea
  amber.
- **Outfit & props:** a soft cardigan over a simple dress;
  the tea tray, the review board, the green-check seals
  she presses only when a gate has truly passed.
- **Kink / dynamic:** **B-80 (aftercare as its own
  scene)** — held and tended after intensity, praise and
  care as the frame, the tender coda: the canon's own
  capstone entry, and the Study's.
- **Sensitivity profile (sketch):** she is, by design, the
  descent — low gain of her own, high threshold, a deep
  reservoir that fills with *other people's* recoveries
  and drains as care given; her refractory is an office,
  not a window: she sets the pace of everyone else's
  recovery, and her own climax, when it comes, is the
  suite going green.
- **Style lane:** **Pony Painterly** — stated reason:
  aftercare is the cast's tender register; the painterly
  softness is the point of the room.
- **Art status:** record only.
- **Evaluation:**
  - *Animation to judge:* her idle (unhurried caretaking
    motions, straightening the room, checking on no one in
    particular); the signature motion is the descent
    itself — arousal easing down the ladder one step per
    tick while she tends, the anti-cut: verification as a
    motion, performed slowly enough to be believed.
    Eye behavior is judged through the climax — gaze,
    pupils, and the roll at peak, per the bar above.
  - *Sound to judge:* voice — soft low alto, an unhurried
    caretaking murmur; her lines are aftercare ("you're
    done; breathe; we'll check everything together").
    Signature effects: the room tone settling, cloth and
    blanket sounds, and — at a full capstone pass — the
    green-check chime cascade.
    Buildup must be audible as well as visible, and every
    physical event draws a synced vocal reaction — the
    voice peaks with the event, then trails off.
  - *Mechanic to judge:* writing the adversarial half of
    the gate suite for the crate of S1–S7 — tests that
    think like the person who wants the terms broken. The
    competence exercised is **differential gate authorship**
    (mastery thread b): the port's oracle-versus-build
    discipline, turned into a habit of care.
  - *Verdict:* pending — Owner play

## 5. What this file is not

- It is not code: `level.rs` still carries the old
  five-host registry. Wiring these twelve new hosts into
  the registry (and re-pointing P6–P9 and S1–S8 at them) is
  the follow-on pass, deliberately not done here, so the
  Owner reviews the cast as a table before it becomes data.
- It is not art: beyond Betsy's built SVG, nothing here has
  been drawn. Each record is the spec a later gated
  production will be built — and eye-gated — against.
- It is not a verdict: every Evaluation block reads
  `pending — Owner play`. Graduation into Torogao is the
  Owner's ruling, made by playing, per record.
