//! Fail Closed (Study S2's twin): the license gate of the
//! estate's moc3 importer/exporter, reduced to its decision.
//!
//! A header arrives with claims: a license string, an
//! attribution flag, a redistribution flag. The gate either
//! admits the asset or refuses it with a **typed** error
//! naming exactly which clause failed. The lesson is in what
//! the gate never does: it never admits on a technicality, it
//! never fails with a string, and an unknown license is a
//! refusal — fail *closed* means the default answer to
//! anything unrecognized is no.

/// A license header as it arrives at the gate.
#[derive(Clone, Debug, PartialEq)]
pub struct LicenseHeader {
    /// The license identifier claimed (e.g. "Apache-2.0").
    pub license: String,
    /// Whether attribution to the author is present.
    pub attribution_present: bool,
    /// Whether the asset's terms allow redistribution.
    pub redistribution_allowed: bool,
    /// Whether AI-training use is allowed by the terms.
    pub ai_training_allowed: bool,
}

/// The licenses the gate knows. Anything else is unrecognized
/// — and unrecognized is refused.
#[must_use]
pub fn is_known_license(id: &str) -> bool {
    matches!(
        id,
        "Apache-2.0" | "BSD-2-Clause" | "CC0-1.0" | "CC-BY-4.0" | "MIT"
    )
}

/// Licenses that require attribution to travel with the asset.
#[must_use]
pub fn requires_attribution(id: &str) -> bool {
    matches!(id, "Apache-2.0" | "BSD-2-Clause" | "CC-BY-4.0" | "MIT")
}

/// Why the gate refused. One variant per clause — the variant
/// *is* the diagnosis, which is the lesson (contracts are
/// types, B-01 style).
#[derive(Clone, Debug, PartialEq)]
pub enum LicenseError {
    /// The license identifier is not one the gate knows.
    UnknownLicense {
        /// The identifier offered.
        license: String,
    },
    /// The license requires attribution and none is present.
    MissingAttribution {
        /// The license that requires it.
        license: String,
    },
    /// The intended use is redistribution and the terms
    /// forbid it.
    RedistributionForbidden,
    /// The intended use is AI training and the terms forbid
    /// it.
    TrainingForbidden,
}

/// What the asset is wanted for — the gate judges the header
/// against a *use*, not in the abstract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Use {
    /// Ship the asset in a build.
    Ship,
    /// Redistribute the asset itself.
    Redistribute,
    /// Train on the asset.
    Train,
}

/// Judge one header for one use.
///
/// # Errors
/// The [`LicenseError`] for the first clause that fails, in
/// contract order: known license → attribution → use terms.
pub fn check(header: &LicenseHeader, use_: Use) -> Result<(), LicenseError> {
    if !is_known_license(&header.license) {
        return Err(LicenseError::UnknownLicense {
            license: header.license.clone(),
        });
    }
    if requires_attribution(&header.license) && !header.attribution_present {
        return Err(LicenseError::MissingAttribution {
            license: header.license.clone(),
        });
    }
    match use_ {
        Use::Redistribute if !header.redistribution_allowed => {
            Err(LicenseError::RedistributionForbidden)
        }
        Use::Train if !header.ai_training_allowed => Err(LicenseError::TrainingForbidden),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(license: &str) -> LicenseHeader {
        LicenseHeader {
            license: license.to_string(),
            attribution_present: true,
            redistribution_allowed: true,
            ai_training_allowed: true,
        }
    }

    #[test]
    fn clean_headers_are_admitted() {
        assert_eq!(check(&header("Apache-2.0"), Use::Ship), Ok(()));
        assert_eq!(check(&header("CC0-1.0"), Use::Redistribute), Ok(()));
        assert_eq!(check(&header("BSD-2-Clause"), Use::Train), Ok(()));
    }

    #[test]
    fn cc0_needs_no_attribution() {
        let mut h = header("CC0-1.0");
        h.attribution_present = false;
        assert_eq!(check(&h, Use::Ship), Ok(()));
    }

    // ---- adversarial ----

    #[test]
    fn unknown_licenses_fail_closed() {
        for id in ["Proprietary", "GPL-3.0", "", "apache", "Apache 2.0"] {
            assert_eq!(
                check(&header(id), Use::Ship),
                Err(LicenseError::UnknownLicense {
                    license: id.to_string()
                }),
                "{id:?} must be refused, not approximated"
            );
        }
    }

    #[test]
    fn missing_attribution_is_its_own_variant() {
        let mut h = header("MIT");
        h.attribution_present = false;
        assert_eq!(
            check(&h, Use::Ship),
            Err(LicenseError::MissingAttribution {
                license: "MIT".to_string()
            })
        );
    }

    #[test]
    fn use_terms_are_judged_per_use() {
        let mut h = header("CC-BY-4.0");
        h.redistribution_allowed = false;
        assert_eq!(
            check(&h, Use::Ship),
            Ok(()),
            "shipping is not redistributing"
        );
        assert_eq!(
            check(&h, Use::Redistribute),
            Err(LicenseError::RedistributionForbidden)
        );
        let mut h = header("CC-BY-4.0");
        h.ai_training_allowed = false;
        assert_eq!(check(&h, Use::Train), Err(LicenseError::TrainingForbidden));
    }

    #[test]
    fn the_first_failed_clause_wins() {
        // Unknown license AND missing attribution: the answer
        // names the license clause — order is part of the
        // contract.
        let mut h = header("Proprietary");
        h.attribution_present = false;
        assert!(matches!(
            check(&h, Use::Ship),
            Err(LicenseError::UnknownLicense { .. })
        ));
    }
}
