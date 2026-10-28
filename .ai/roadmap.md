# Roadmap — Hands-On (education) — v0.1 → v1.0
<!-- bloch:roadmap schema=1 kind=education lane=general baseline=0.1.0 target=1.0.0 -->

How to work this roadmap (any agent): run `bloch task start R-xxx --agent <you>` before working a task, `bloch task done R-xxx` when you believe it is complete — done runs every `verify:` command and flips the task green only if all pass; there is no override. `bloch task fail R-xxx --error "..."` records a failure. Fixes learned the hard way land in `.ai/memory/lessons.md` via the inbox/promote flow and are surfaced in every HANDOFF.

## What 1.0 means here (curated 2026-10-27 — bloch fleet rollout)

1.0 = the teaching game complete: all seventeen levels built
and playable — the Playroom (P1–P9) and the Study (S1–S8) —
each level twinned with the Rust module it teaches (level map:
CURRICULUM.md; cast of record: CAST.md), and released through
the studio's channels. Baseline state: P1 built, the P4 Loop
Studio slice playable, the remaining levels designed and
awaiting their builds.

## Phase 1 — Baseline truth
- [ ] R-101 Build is green from a clean checkout
  - verify: cargo build
  - evidence:
- [ ] R-102 Existing tests pass at the baseline
  - verify: cargo test
  - evidence:
- [ ] R-103 Repo identity, baseline version, and this roadmap are recorded
  - verify: test -f .ai/roadmap.md
  - evidence:

## Phase 2 — Core complete
- [ ] R-201 All seventeen levels (Playroom P1–P9, Study S1–S8) are built and playable
  - verify: cargo build
  - evidence:
- [ ] R-202 Curriculum is sequenced with prerequisites in order
  - verify: cargo build
  - evidence:
- [ ] R-203 Every level ships with the twin Rust module it teaches (play it, then read the module)
  - verify: cargo build
  - evidence:

## Phase 3 — Tests and gates
- [ ] R-301 Validation tests cover the happy path end to end (half the test split)
  - verify: cargo test
  - evidence:
- [ ] R-302 Adversarial tests cover malformed, oversized, and hostile inputs (half the test split)
  - verify: cargo test
  - evidence:
- [ ] R-303 Assessment answers are checked against the marking rubric
  - verify: cargo build
  - evidence:

## Phase 4 — Hardening
- [ ] R-401 Subject-accuracy review pass is completed by a qualified reviewer
  - verify: cargo build
  - evidence:
- [ ] R-402 Assessment items ship with a marking rubric
  - verify: cargo build
  - evidence:
- [ ] R-403 Materials meet the accessibility checklist (alt text, reading level, contrast)
  - verify: cargo build
  - evidence:

## Phase 5 — Docs
- [ ] R-501 README documents install, usage, and verification
  - verify: test -f README.md
  - evidence:
- [ ] R-502 CHANGELOG records every user-facing change
  - verify: test -f CHANGELOG.md
  - evidence:
- [ ] R-503 Instructor/maintainer notes document how to extend the material
  - verify: cargo build
  - evidence:

## Phase 6 — Release v1
- [ ] R-601 Version is bumped to 1.0.0 and the changelog records the release
  - verify: grep -q 'version = "1.0.0"' Cargo.toml
  - evidence:
- [ ] R-602 Clean-environment install and run is verified
  - verify: cargo build
  - evidence:
- [ ] R-603 Release artefacts are built from a clean checkout and checksummed
  - verify: cargo build
  - evidence:

