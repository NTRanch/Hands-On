//! Action buttons and their unlock rules (SPEC §2.4).
//!
//! Locked buttons are *visible* from the start — the player sees
//! what progression will earn — but using one refuses with a
//! typed [`AffordanceError`]. A refusal is data, not a panic and
//! not a silent no-op: the UI animates the cross-out wobble from
//! exactly this value.

use crate::expression::Expression;

/// Every player action the minigame exposes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    /// Primary squeeze, left hand. Always available.
    MilkLeft,
    /// Primary squeeze, right hand. Always available.
    MilkRight,
    /// Squeeze with both hands: more flow, tighter window.
    MilkBoth,
    /// Cheek caress: builds state, produces nothing.
    Caress,
    /// Spank: builds state faster, spills if spammed.
    Spank,
}

impl Action {
    /// All actions, in button order.
    pub const ALL: [Action; 5] = [
        Action::MilkLeft,
        Action::MilkRight,
        Action::MilkBoth,
        Action::Caress,
        Action::Spank,
    ];

    /// The expression state at which this action unlocks.
    #[must_use]
    pub fn unlock_state(self) -> Expression {
        match self {
            Action::MilkLeft | Action::MilkRight => Expression::Neutral,
            Action::Caress => Expression::Smiling,
            Action::Spank => Expression::Flushed,
            Action::MilkBoth => Expression::Aroused,
        }
    }

    /// Is this action usable at `state`?
    #[must_use]
    pub fn is_unlocked(self, state: Expression) -> bool {
        state >= self.unlock_state()
    }
}

/// Why an action was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AffordanceError {
    /// The action exists but its unlock state has not been reached.
    Locked {
        /// The action that was attempted.
        action: Action,
        /// The state it requires.
        required: Expression,
        /// The state the session is actually in.
        current: Expression,
    },
}

/// Guard used by the simulation before applying any action.
///
/// # Errors
/// Returns [`AffordanceError::Locked`] when `action` is not yet
/// unlocked at `state`.
pub fn require_unlocked(action: Action, state: Expression) -> Result<(), AffordanceError> {
    if action.is_unlocked(state) {
        Ok(())
    } else {
        Err(AffordanceError::Locked {
            action,
            required: action.unlock_state(),
            current: state,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn milk_is_always_available() {
        for state in Expression::ALL {
            assert!(Action::MilkLeft.is_unlocked(state));
            assert!(Action::MilkRight.is_unlocked(state));
        }
    }

    #[test]
    fn side_actions_unlock_in_progression_order() {
        assert!(!Action::Caress.is_unlocked(Expression::Neutral));
        assert!(Action::Caress.is_unlocked(Expression::Smiling));
        assert!(!Action::Spank.is_unlocked(Expression::Smiling));
        assert!(Action::Spank.is_unlocked(Expression::Flushed));
        assert!(!Action::MilkBoth.is_unlocked(Expression::Flushed));
        assert!(Action::MilkBoth.is_unlocked(Expression::Aroused));
    }

    #[test]
    fn locked_action_refuses_with_a_typed_error() {
        let err = require_unlocked(Action::Spank, Expression::Neutral).unwrap_err();
        assert_eq!(
            err,
            AffordanceError::Locked {
                action: Action::Spank,
                required: Expression::Flushed,
                current: Expression::Neutral,
            }
        );
    }
}
