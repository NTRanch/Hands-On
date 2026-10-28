# Current state — 2026-10-26 (founding)

- Hands-On is the studio's teaching game (IDEA-0093): the
  working systems of the studio's game stack, codified as
  levels whose success pays out as live feedback on each
  level's host, whose reactions run on the same deterministic
  tick as the player's inputs. Dual purpose: it teaches Rust
  at the Daddy Style standard, and it is the evaluation
  surface where each girl, mechanic, animation, and sound is
  prototyped small and judged by playing before it graduates
  to Torogao production.
- Workspace: two crates. `milking-core` — the deterministic
  fixed-step core (simulation, reaction profiles, expression
  ladder, authoring gates; no rendering, no I/O). `hands-on` —
  the TUI (level registry, feedback engine, menus). Zero
  external dependencies; toolchain pinned in
  `rust-toolchain.toml`.
- Built and playable: Level P1 (First Milking) and the P4
  Loop Studio slice (loop closure + bake + twin-run hash, on
  Opal). `game.html` is P1's visual prototype. The other
  fifteen levels are designed in CURRICULUM.md, not built.
- Cast: CAST.md carries seventeen records (P1–P9, S1–S8),
  one distinct girl per level, each with identity, palette,
  kink/dynamic, sensitivity sketch, and an Evaluation block.
  Every verdict reads `pending — Owner play`. The twelve new
  hosts are not yet wired into `hands-on/src/level.rs` — that
  follows the Owner's cast review.
- Mastery: MASTERY.md designs four arcs (Bevy/stage systems;
  the D→Rust port and its tooling; animation/rendering/GPU;
  haptics + cross-platform Rust UI) — 64 modules over the
  17-level core. Its reconciliation checklist merges the arcs
  into CURRICULUM.md as a later pass.
- Animation: the codeanim trial record lives in
  `codeanim-trial/` — deterministic code animation on the
  project's own layered art, twin-run byte-identical
  (204/204 artifacts). Which motion classes route to codeanim
  by default awaits the Owner's eye on that evidence.
- Gates at founding: cargo test 38/38, clippy `-D warnings`
  clean, rustfmt clean, both Lua launchers parse. The repo
  ships with a bloch memory tree (`.ai/memory/`), a SCIP
  index (`code-intel/`), the lane's skills (`skills/`), and
  an mdBook (`book.toml`, `book/`) over the canonical docs.
