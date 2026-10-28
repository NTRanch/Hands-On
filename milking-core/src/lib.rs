//! milking-core — the deterministic Torogao port twin of the
//! cowgirl milking teaching minigame (see `../SPEC.md`).
//!
//! Architecture, in teaching order:
//!
//! - [`sim`] — the fixed-step squeeze-cycle simulation: input
//!   events land on ticks, the squeeze parameter follows its
//!   envelope, flow accumulates into the bottle gauge, spills
//!   subtract. Rendering does not exist here; the same event
//!   stream always produces the same state trace.
//! - [`expression`] — the five-state expression machine. It moves
//!   one step at a time, with hysteresis, so a face can never
//!   teleport between expressions.
//! - [`affordance`] — which action buttons exist and when they
//!   unlock. Locked actions refuse with a typed error, never a
//!   silent no-op.
//! - [`authoring`] — the Studio Mode model: keyform tracks,
//!   easing, loop-closure checking, and baking a track to a
//!   fixed-step frame table an engine can play verbatim.
//!
//! Constants live in [`sim::Config`] and mirror the HTML
//! prototype's numbers, so play feel and core behavior agree.
//!
//! The remaining levels' twins live here too (SPEC §6: later
//! levels feed the same feedback engine from different
//! exercises, and the engine's indifference starts with the
//! exercises sharing one pure core):
//!
//! - [`beat`] — P2: Strobe beat-sync on a deterministic grid.
//! - [`ecs`] — P3: ECS schedule discipline (the schedule is
//!   the program).
//! - [`router`] — P5: the torogao-haptics router contract.
//! - [`moc3`] — P6: the moc3 mapping border (EXACT/LOSSY/
//!   NO-MAP) plus the style/eye-rule border check.
//! - [`sprite`] — P7: sprite/GIF bake frame selection and
//!   closure across the bake.
//! - [`funscript`] — P8: the funscript writer and its router
//!   clamp.
//! - [`render3d`] — P9: the GLB emitter and the deterministic
//!   software render (byte-identical twin runs).
//! - [`puppet`] (S1), [`license`] (S2), [`accumulator`] (S3),
//!   [`frame`] (S4), [`queues`] (S5), [`border`] (S6),
//!   [`tracking`] (S7), [`aftercare`] (S8) — the Study's
//!   lesson twins.
//!
//! One nightly feature is used: `gen_blocks`, for the frame
//! stream generator the S3 lesson teaches (Rust 2024 material,
//! on the estate's pinned nightly).
#![forbid(unsafe_code)]
#![feature(gen_blocks)]

pub mod accumulator;
pub mod affordance;
pub mod aftercare;
pub mod authoring;
pub mod beat;
pub mod border;
pub mod ecs;
pub mod expression;
pub mod frame;
pub mod funscript;
pub mod license;
pub mod moc3;
pub mod puppet;
pub mod queues;
pub mod render3d;
pub mod router;
pub mod sim;
pub mod sprite;
pub mod tracking;
