//! hands-on — the framework library (SPEC §6): the level
//! registry and the shared feedback engine, shared by every
//! front-end. The TUI binary (`src/main.rs`) is one front-end;
//! the egui app (`hands-on-app/`) is another. All game logic
//! lives here and in `milking-core`; front-ends only render
//! and forward input.
//!
//! Nothing in this library does I/O, reads a clock, or renders:
//! front-ends fold their own wall-clock time into the core's
//! fixed ticks and hand the engine timestamped events, exactly
//! as the determinism contract (SPEC §4) requires. The session
//! layer (`session`, `play`, `study`) turns each level's pure
//! `milking-core` twin into a playable sitting on that basis.
//!
//! The one deliberate exception is the persistence layer
//! (`save`, `settings`): the store owns the game's save
//! slots, campaign file, and settings file — atomically
//! written, version-tagged, refused-when-corrupt — because
//! both front-ends must share exactly one implementation of
//! it. It still reads no clock: front-ends supply every
//! timestamp it writes.
#![forbid(unsafe_code)]

pub mod feedback;
pub mod level;
pub mod play;
pub mod save;
pub mod session;
pub mod settings;
pub mod study;
