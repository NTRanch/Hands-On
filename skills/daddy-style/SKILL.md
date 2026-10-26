---
name: "daddy-style"
description: "The engineering discipline for the Hands-On lane (Nice Thoughts Ranch): explicit contracts and assertions, bounded work with named bounds, typed errors, determinism, no hidden behavior, safety first. Use when writing, regenerating, refactoring, or reviewing any code in this repository — the Rust crates, the Lua launchers, the trial tooling. The canonical statement is SPEC.md §7.4; the source text is quoted in references/spec-source.md."
---

# Daddy Style

## Purpose

Apply this lane's engineering discipline to every line of
code in this repository. The discipline evokes the positive
side of a daddy-dom dynamic: **strict requirements set by a
firm, caring authority, for the student's own benefit.** The
rules are known in advance, applied consistently, never
arbitrary. Follow them and the work thrives — deterministic,
safe, predictable, gates pass. Break them and poor outcomes
follow mechanically — nondeterminism, gate failures,
defects — as *consequence, not punishment*. Discipline as
care; the strictness is protective.

Priority order, always: **Safety > Performance > Developer
Experience.**

## The rules

1. **Explicit contracts.** Every function states what it
   requires and what it guarantees. Preconditions are
   checked, not assumed; invariants are named where they
   live. A caller should never have to read a body to know
   its contract.
2. **Assertions stay in.** Contract violations are defects,
   and defects fail loudly at the point of violation — in
   tests and in debug runs alike — never as silent
   fallbacks, defaulted values, or swallowed errors.
3. **Bounded work, named bounds.** Every loop, queue,
   buffer, retry, and accumulation has a bound, and the
   bound is a named constant with a reason. Unbounded growth
   is a defect even when it has not bitten yet.
4. **Typed errors.** Failures travel as typed error values
   that name the refusal (the crate's `LoopOpen { start,
   end }` is the house shape). No stringly-typed errors, no
   panics across module borders, no `unwrap` on a path a
   user action can reach.
5. **Determinism.** Progression is a pure function of ticks
   and events. Fixed-step simulation; rendering and I/O live
   outside the core. A behavior that cannot be reproduced
   from a named seed and an event log is not finished.
6. **No hidden behavior.** No hidden allocation on hot
   paths, no ambient state, no side effects a signature
   does not declare, no configuration that changes behavior
   from outside the record. What the code does is what its
   contracts say it does.
7. **Safety first.** When a rule here conflicts with speed
   or convenience, safety wins and the tradeoff is written
   down where the next reader will find it.

## Gates

Work in this repository is done when all of these pass, per
change:

- `cargo test` — the full suite, including the refusal-path
  cases. Tests here treat locked doors as first-class
  behavior: an affordance that must refuse, refuses with a
  typed error, and a test says so.
- `cargo clippy --all-targets -- -D warnings` — zero
  warnings.
- `cargo fmt --check` — clean.
- Any animation or bake claim carries its determinism
  evidence: a named seed and a twin-run hash that came out
  byte-identical.

## Review checklist

- [ ] Every new loop/queue/buffer names its bound.
- [ ] Every refusal is a typed error value, tested.
- [ ] No new ambient state; the core stays a pure function
      of ticks and events.
- [ ] No hidden allocation or behavior a signature hides.
- [ ] Gate numbers are reported as run — actuals, not
      expectations.
