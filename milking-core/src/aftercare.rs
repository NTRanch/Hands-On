//! Aftercare (Study S8's twin — the capstone): the gate
//! suite as a discipline. After the intensity: tests, review,
//! verification.
//!
//! A gate suite has two halves by estate law: **validation**
//! tests, which prove the system does what it claims, and
//! **adversarial** tests, which think like the person who
//! wants the terms broken. A suite with only the first half
//! is not half a suite — it is an unreviewed claim. This
//! module is the suite-runner's model: challenges arrive
//! typed by class, the runner checks both halves ran and
//! both halves passed, and the verdict is the conjunction —
//! one failure anywhere is a failed gate, stated with the
//! failing challenge's name.

/// Which half of the suite a challenge belongs to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChallengeClass {
    /// Proves the system does what it claims.
    Validation,
    /// Thinks like the person who wants the terms broken.
    Adversarial,
}

/// One challenge in the suite: a name, its class, and the
/// check itself (a pure predicate over the system under
/// review — here modeled as a bool the session computes).
#[derive(Clone, Debug, PartialEq)]
pub struct Challenge {
    /// Challenge name.
    pub name: &'static str,
    /// Which half of the suite it belongs to.
    pub class: ChallengeClass,
}

/// The outcome of one challenge.
#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    /// The challenge's name.
    pub name: &'static str,
    /// Its class.
    pub class: ChallengeClass,
    /// Whether it passed.
    pub passed: bool,
}

/// The suite's verdict.
#[derive(Clone, Debug, PartialEq)]
pub struct Verdict {
    /// Every challenge's outcome, in suite order.
    pub outcomes: Vec<Outcome>,
    /// Validation challenges run.
    pub validation_run: usize,
    /// Adversarial challenges run.
    pub adversarial_run: usize,
    /// Challenges failed.
    pub failures: usize,
}

impl Verdict {
    /// The gate passes only when both halves ran, neither
    /// half is empty, and nothing failed.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.failures == 0 && self.validation_run > 0 && self.adversarial_run > 0
    }
}

/// Why a suite cannot even be judged.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SuiteError {
    /// The suite has no validation half.
    MissingValidation,
    /// The suite has no adversarial half: an unreviewed claim.
    MissingAdversarial,
    /// Fewer results than challenges — someone stopped early.
    Incomplete {
        /// Challenges declared.
        expected: usize,
        /// Results supplied.
        got: usize,
    },
}

/// Judge a suite: the declared challenges against the results
/// the session produced (one bool per challenge, in order).
///
/// # Errors
/// [`SuiteError`] when the suite is structurally unjudgeable:
/// a missing half, or a result count that does not match the
/// challenge count.
pub fn judge(challenges: &[Challenge], results: &[bool]) -> Result<Verdict, SuiteError> {
    let validation = challenges
        .iter()
        .filter(|c| c.class == ChallengeClass::Validation)
        .count();
    let adversarial = challenges.len() - validation;
    if validation == 0 {
        return Err(SuiteError::MissingValidation);
    }
    if adversarial == 0 {
        return Err(SuiteError::MissingAdversarial);
    }
    if results.len() != challenges.len() {
        return Err(SuiteError::Incomplete {
            expected: challenges.len(),
            got: results.len(),
        });
    }
    let outcomes: Vec<Outcome> = challenges
        .iter()
        .zip(results)
        .map(|(c, passed)| Outcome {
            name: c.name,
            class: c.class,
            passed: *passed,
        })
        .collect();
    let failures = outcomes.iter().filter(|o| !o.passed).count();
    Ok(Verdict {
        outcomes,
        validation_run: validation,
        adversarial_run: adversarial,
        failures,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suite() -> Vec<Challenge> {
        vec![
            Challenge {
                name: "closed loops close",
                class: ChallengeClass::Validation,
            },
            Challenge {
                name: "router clamps emission",
                class: ChallengeClass::Validation,
            },
            Challenge {
                name: "open loops are refused",
                class: ChallengeClass::Adversarial,
            },
            Challenge {
                name: "over-ceiling is refused typed",
                class: ChallengeClass::Adversarial,
            },
        ]
    }

    #[test]
    fn a_full_passing_suite_passes_the_gate() {
        let verdict = judge(&suite(), &[true, true, true, true]).unwrap();
        assert!(verdict.passed());
        assert_eq!(verdict.validation_run, 2);
        assert_eq!(verdict.adversarial_run, 2);
        assert_eq!(verdict.failures, 0);
    }

    #[test]
    fn one_failure_fails_the_gate_and_names_itself() {
        let verdict = judge(&suite(), &[true, false, true, true]).unwrap();
        assert!(!verdict.passed());
        assert_eq!(verdict.failures, 1);
        assert_eq!(verdict.outcomes[1].name, "router clamps emission");
    }

    // ---- adversarial ----

    #[test]
    fn a_suite_without_an_adversarial_half_is_unjudgeable() {
        let half: Vec<Challenge> = suite()
            .into_iter()
            .filter(|c| c.class == ChallengeClass::Validation)
            .collect();
        assert_eq!(
            judge(&half, &[true, true]),
            Err(SuiteError::MissingAdversarial)
        );
    }

    #[test]
    fn a_suite_without_a_validation_half_is_unjudgeable() {
        let half: Vec<Challenge> = suite()
            .into_iter()
            .filter(|c| c.class == ChallengeClass::Adversarial)
            .collect();
        assert_eq!(
            judge(&half, &[true, true]),
            Err(SuiteError::MissingValidation)
        );
    }

    #[test]
    fn stopping_early_is_incomplete_not_passing() {
        assert_eq!(
            judge(&suite(), &[true, true]),
            Err(SuiteError::Incomplete {
                expected: 4,
                got: 2
            })
        );
    }

    #[test]
    fn adversarial_failures_count_the_same() {
        // The adversarial half is not decoration: its failure
        // is a gate failure.
        let verdict = judge(&suite(), &[true, true, false, true]).unwrap();
        assert!(!verdict.passed());
        assert_eq!(verdict.outcomes[2].class, ChallengeClass::Adversarial);
    }
}
