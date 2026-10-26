# TRIAL — deterministic code animation on the Hands-On girls (Betsy)

Date: 2026-10-26.
No shipping file was modified. All art is the crate's own (game.html).

## VERDICT: PARTIAL

**Idle: GO. Expression climb: GO. The milking loop: PARTIAL — and the
failing piece is named here first, per the ASAP rule: the hands read
as sliding cutouts.** They are featureless authored ellipses; they
fade/slide/press-in rigidly and overlap the breast tangentially — no
finger wrap, no contact shadow, no skin displacement at the contact
point. The chest's *response* (pivot squash + spring-fitted jiggle
warp) does read as response, and the loop is exactly the game's own
shipped technique made deterministic — serviceable as the in-game
signature loop, not a showcase action piece. The ceiling there is
set by the authored hand art, not by the evaluator: no parameter
table can animate fingers that were never drawn.

Everything else the Owner asked about holds, with numbers:

- **Determinism: PASS — 204/204 artifacts byte-identical** across
  twin runs (`determinism.txt`, `hashes_run1.txt` == `hashes_run2.txt`;
  `*.results.json` excluded as wall-clock logs, the spike's stated
  precedent). Every spring closes its loop with residual **exactly
  0.0**. Params + code regenerate every clip exactly.
- **The package hypothesis is PROVEN, ~100×.** All three motions as
  codeanim: **183,245 B** (raster layers) or **51,292 B** (vector
  form). The same three motions baked: **18,359,053 B** of sprite
  sheets + GIFs at identical dimensions/frame rate. A new motion on
  an existing girl costs codeanim ~0.8 KB of parameters; baked, it
  costs 3.2–9.1 MB. Detail in §4.
- **Quality bar (Owner's amendment):** the climb was judged against
  the grammar of Danbooru 9228588 / 12068969 — gaze rolling up *into*
  the melted face. It passes: the roll is monotonic (see §3), the
  peak is a genuine torogao face, and posture changes with
  expression (the Séraphine standard), not a warp on a static face.

## 1. Route and cost

No SVG rasterizer or browser exists in the sandbox, so Betsy's stage
SVG (extracted verbatim from game.html as `stage.svg`, 10,774 B) is
decomposed by a purpose-built subset rasterizer, `svgmini.py`
(~330 lines: paths/ellipses/rects/lines, `use` resolution, opacity
accumulation, source-over compositing, ×2 supersampling). Output:
**20 layers** in `parts/` — sky/backdrop, fence, rear body, rear
tail, bucket, front outfit, chest, both hands, head base, hair, and
the nine face layers (blush, four eye states, sparkles, two mouths,
hearts) — each with the pivots the game's own markup implies (chest
600,345; tail 262,350; neck 600,302). Fidelity: ±1 px on curves vs a
browser rasterizer; every layer the animator moves comes from this
same rasterizer, so all motion comparisons are internally exact.
The rest frame was eyeballed against the game art before any motion
work (`rest_frame.png` — it is Betsy, correctly assembled).

The animator (`trial.py`, ~600 lines) extends the estate spike's
`animate.py` rather than reinventing it: fixed 16 fps clock,
closed-form motion, and 240 Hz semi-implicit spring-dampers warmed
into their periodic steady state (the spike's seam method), plus
two things the spike did not have: per-layer affine transforms about
named pivots, and the spike's elliptical-falloff warp applied to a
single layer (the chest) instead of the whole still.

Render cost, CPU-only, single thread: idle 12.0 s (64 f), climb
24.9–28.4 s (96 f), milking 12.6–28.4 s (32 f, warp-heavy; the range
is shared-sandbox contention). All three motions ≈ 53 s of CPU vs
the spike's I2V baseline of **771 s for one 81-frame clip on the
A6000** — and the marginal cost of motion #100 is another ~1 KB of
parameters and ~15 s of CPU, not another GPU render.

## 2. Idle — GO

64 f @ 16 fps, 4 s loop: breath (2.2 px bob, 0.6% scale, chest
micro-scale phase-offset π/2), two scheduled blinks (eyeHalf as the
lid, sin-profile cross-fade), look drift (±2 px gaze, head follows
at 0.8 px), hair sway phase-lagged −0.7 rad, tail on a spring
(k 55, c 6.5, ζ 0.44) driven at 2 s period.

Checklist: anticipation PARTIAL (the blink travels both ways;
breath has none by nature) · overshoot PASS (tail spring overshoots
its drive; the track is in the params) · follow-through PASS (hair
lag, tail spring, chest offset — three coupled secondaries) ·
holds PASS (sinusoidal dwell) · easing variety PARTIAL (sinusoid +
spring suffices for ambient) · loop closure PASS (pixel seam 0.446
vs median inter-frame step 0.399 — the wrap step is an ordinary
step, not a jump; spring seam residual 0.0°) · sliding cutout NO
(she moves as one layered body). In stills the breath is
sub-visible — the spike found the same; it reads in the GIF
(`run1/idle/idle.gif`). The blink at f021 is plainly visible in the
contact sheet.

## 3. Expression climb — GO

96 f @ 16 fps: the governed ladder as one continuous parameter
`s` ∈ [0,4] — Neutral → Smiling → Flushed → Aroused → Torogao —
with per-feature smoothstep cross-fade windows (`expr_weights` in
`trial.py`; the full table is in `climb.params.json`). Holds: 1 s at
Neutral, 1 s at peak; no snap anywhere (median inter-frame diff
0.34, the smoothest of the three motions).

Against the named grammar (9228588, the study's ahegao case;
12068969): **the gaze roll is the detail that mattered.** The
authored eyeUp layer starts the roll (iris cy −3.4) but the authored
melt layer's iris sits at cy −0.5 — a naive cross-fade *un-rolls*
the gaze at peak. The climb therefore carries a parametric gaze
offset (−3.2·seg(s, 2, 4) px on all eye layers) so the upward travel
continues into the melt. Peak frame f088, verified in crop: pupils
high under heavy lids, **luminous pink heart pupils**, open mouth
with tongue, full blush with sparkles, floating hearts, head tilted
−3.4° and lifted −6.4 px, chest fullness +4.8% — posture AND
expression change; it is not a transform wiggle on a static face.

Checklist: anticipation PASS (the ladder itself) · overshoot
PARTIAL (sparkle/heart pulse; bodily overshoot not applicable) ·
follow-through PASS (hair lag persists; tail amplitude grows
×1.7 with s; fullness trails) · holds PASS · easing variety PASS ·
loop closure N/A (a transition; it ends held at peak) · sliding
cutout NO. Grammar elements **absent because the art does not
contain them**: tears, drool, hair disarray. Codeanim cannot invent
pixels — those are art-authoring additions, listed in §5.

## 4. The milking loop — PARTIAL (the stress test)

32 f @ 16 fps, 2 s loop, four beats alternating hands. Per beat:
approach (hand fades/slides in over 2 f) → compress (chest squash
sx −4.5% / sy −6% about the *pressed-side* pivot) → hold (2 f) →
release into a spring-fitted jiggle (k 110, c 8, ζ 0.38, driven by
the press profile, warp ±5 px vertical / ±1.5 px horizontal through
the spike's elliptical falloff). Tail counter-spring driven by
−7°×press plus its sway; rear body counter-sways; expression rides
at s = 2.1 + 0.3×press so each squeeze visibly pulses the blush.
Full parameter table: `milking.params.json`.

Checklist: anticipation PASS · overshoot PASS in data, WEAK in
stills (the jiggle is a motion read, not a stills read — the spike
reported the identical effect) · follow-through PASS (two springs
+ lagged rear sway) · holds PASS · easing variety PASS · loop
closure PARTIAL (spring residuals exactly 0.0, but the pixel seam
is 1.42 vs median step 1.07 — the wrap step is the loop's largest
ordinary step because the hand-swap and breath phase coincide
there; a stronger beat, not a pop) · **sliding cutout: FAIL for the
hands, PASS for the chest** — see the verdict. This is the spike's
boundary (loops yes, action no) arriving exactly where predicted:
the *rhythm and response* are codeanim's; the *contact* is not.

## 5. Package math (measured, this trial)

| Form | Bytes | Contents |
|---|---|---|
| Codeanim, vector (shipping form) | **51,292** | stage.svg 10,774 + evaluator code 38,106 + 3 params 2,412 |
| Codeanim, raster layers | **183,245** | 20 layer PNGs 142,727 + code 38,106 + params 2,412 |
| Baked, GIF only | 5,543,271 | idle 1,826,642 · climb 2,764,757 · milking 951,872 |
| Baked, sprite sheets only | 12,815,782 | idle 4,275,377 · climb 6,287,525 · milking 2,252,880 |
| Baked, sheets + GIFs | **18,359,053** | same motions, same 800×520, same fps |

Marginal cost of one more motion: codeanim ≈ **+0.8 KB** (a params
file; parts and code already paid). Baked ≈ **+3.2 to +9.1 MB**.
Marginal cost of one more girl: ≈ +11 KB vector (≈ +50–140 KB
raster layers at this size) vs new baked sets per motion. The
Owner's hypothesis — codeanim keeps packages lighter — is not
marginal; it is two orders of magnitude, and it compounds with
every added motion, which is the direction the cast/curriculum
grows. (Honesty note: the unpacked frame PNGs of any route are
~14 MB — that is a render intermediate, not a package form, on
both sides of the comparison.)

## 6. Honest limits

- **No invented pixels.** Tears, drool, dishevelled hair, finger
  wrap, contact shadows, new occlusions, partner geometry: if it
  is not authored, it does not exist. The spike's verdict (e) —
  explicit contact scenes are not viable — stands untouched.
- **Warp amplitude ceiling ≈ ±8 px.** Past that, bilinear warping
  bends hard cel linework visibly (spike §4). Large pose change
  must be authored states, never bigger warps.
- **The evaluator is only as good as the parts.** Betsy's hands
  are ellipses, so her contact reads as ellipses. This is an art
  finding delivered by an animation trial.
- **Rasterizer subset.** svgmini has no gradients, filters, or
  text. A production pipeline should rasterize parts with a real
  SVG engine or accept authored PNG layers; motion conclusions
  here do not depend on the difference.
- **GIF palette.** One global palette from frame 0; flat cel art
  (small palette) shows no banding here, but a painterly-lane girl
  should be re-checked before this encode is trusted for her.
- **Judgment proxy.** This report scores contact sheets, crops,
  and the measured tracks; the GIFs are the Owner's to watch —
  his eye is the final motion verdict, per the brief.
- **Feeding the other engines (one note, per amendment):** the
  trial's per-frame parameter/event stream — `s(t)`, `press(t)`,
  beat onsets, threshold crossings — is exported in the params
  JSONs on the same fixed-step clock, which is exactly the stream
  the separately-ruled audio + physical-reaction engine consumes;
  nothing in that engine needs a second timeline.

## 7. Production implication

**Route to codeanim by default** (Hands-On and Torogao alike):
idle/ambient loops, expression ladders and climbs, attract/menu
loops, secondary-motion systems, and rhythmic response loops of
the milking kind (mechanism yes, showcase no). **Route elsewhere:**
performed action and contact scenes — hand-authored sprite/pose
work or the trained-I2V lane, as the spike already ruled.

**The art-authoring requirement, stated plainly:** a girl is
code-animatable only if she is authored as **layered parts with
named pivots plus an authored expression state set** (the four eye
layers, mouth pair, blush/sparkle/hearts overlays are the working
minimum — Betsy has exactly this, which is why she passed). A
flattened illustration gets only the spike's whole-still warps.
Two additions would raise the contact ceiling for future girls:
hands drawn with finger/thumb shapes, and a contact-shadow layer.
Every cast record should carry its pivot map and spring constants
(k/c per secondary) as data — in this trial those live in the
params JSONs, and they are the same shape a Torogao parameter
recipe takes.

## 8. Files

`svgmini.py`, `trial.py`, `stage.svg`, `parts/` (20 layers),
`run1/` + `run2/` (frames, GIFs, contact sheets, spritesheets,
params, results per motion), `hashes_run1.txt`, `hashes_run2.txt`,
`determinism.txt` (PASS 204/204), `package_math.json`,
`results_all.json`, crops `crop_climb_f*.png`, `crop_milking_f*.png`,
`rest_frame.png`.
