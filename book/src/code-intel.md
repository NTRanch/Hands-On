# Code intelligence

This repository ships a SCIP index of its Rust workspace at
`code-intel/hands-on-index.scip`, so agents and tooling can
resolve definitions and references precisely instead of
guessing from text search.

- **Coverage:** all 9 Rust source documents in the workspace
  — `milking-core` (lib, sim, expression, affordance,
  authoring, plus the gates test) and `hands-on` (main,
  level, feedback) — with 619 unique symbols across both
  crates at founding.
- **Generated with:** rust-analyzer's SCIP export
  (`rust-analyzer scip`), run against the workspace at its
  pinned toolchain.
- **Regenerate:** run `rust-analyzer scip <repo root>` from a
  directory *outside* the repo (the repo's toolchain pin
  otherwise redirects the rust-analyzer binary lookup); it
  writes `index.scip` into the working directory — move it
  over `code-intel/hands-on-index.scip` and record the new
  coverage counts here.

The index is a generated artifact with a source of truth (the
code). When it and the code disagree, the code wins and the
index is stale: regenerate it.
