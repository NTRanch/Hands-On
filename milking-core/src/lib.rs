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
#![forbid(unsafe_code)]

pub mod affordance;
pub mod authoring;
pub mod expression;
pub mod sim;
