# Hands-On

A teaching game from Nice Thoughts Ranch.

Hands-On codifies a working studio's own game systems as
levels. Each level takes one real system — a fixed-step
simulation, a reaction profile, a safety router, a format
border — turns it into an exercise, and pays success out as
live feedback on the level's host: her reactions run on the
same deterministic tick as your inputs. Learn the system by
playing it; then read the Rust module it twins, already
knowing what the module *does* because you have felt its
numbers.

Two tracks:

- **The Playroom** (P1–P9) — game systems and deterministic
  animation, learned by playing them.
- **The Study** (S1–S8) — Rust mastery as lessons, under the
  lane's engineering discipline, **Daddy Style** (SPEC §7.4).

Seventeen levels, seventeen hosts — `CAST.md` is the cast of
record. Level P1 and the P4 Loop Studio slice are playable;
everything else is designed and awaiting its build.

## Play

```sh
cargo build --release
./target/release/hands-on          # interactive TUI
./target/release/hands-on --demo   # scripted smoke session
```

Or open `game.html` in a browser — Level 1's visual prototype,
no build step, no assets.

## Read

- `SPEC.md` — the behavioral design spec.
- `CURRICULUM.md` — the level designs and setup routes.
- `CAST.md` — the cast of record, one record per level.
- `MASTERY.md` — the mastery arcs above the core map.
- `REPORT.md` — the build report and amendment log.
- The same documents as a book: `mdbook build` (output in
  `book/dist/`).
- `code-intel/` — a SCIP index of the Rust workspace.
- `skills/` — the lane's working skills for agents.

## Layout

- `milking-core/` — the deterministic fixed-step core:
  simulation, reaction profiles, the expression ladder,
  authoring gates. No rendering, no I/O — pure functions of
  ticks and events.
- `hands-on/` — the TUI: level registry, feedback engine,
  menus.
- `codeanim-trial/` — the deterministic code-animation trial
  record: scripts, parameters, hashes, contact evidence.

## License

BSD 2-Clause. See `LICENSE` and `NOTICE`.
