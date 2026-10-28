# Source text — SPEC.md §6.5 and §7.4

Quoted verbatim from this repository's `SPEC.md`, the
canonical statement the `daddy-style` skill operationalizes.
When the two disagree, the SPEC wins.

## §6.5 Two tracks: the Playroom and the Study

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
on Opal) are playable; no Study lesson is built.

## §7.4 Daddy Style — the engineering discipline (a law)

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
