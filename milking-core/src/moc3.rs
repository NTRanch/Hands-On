//! The moc3 format border (Playroom P6's twin): Lumen's moc3
//! exporter and its MAPPING tables, reduced to the judgment
//! the level teaches — for each construct in a rig, does it
//! cross to moc3 V3_0 **EXACT**, **LOSSY**, or **NO-MAP**?
//!
//! The classes are the estate's exporter findings, codified:
//!
//! - Most parameter and binding maps are EXACT: what you
//!   authored is what plays.
//! - Per-form multiply/screen colors are **version-gated**:
//!   before V4_2 they cross LOSSY (the color is approximated),
//!   at V4_2 and later they are EXACT.
//! - The blend modes with no V3_0 target (15 of the estate's
//!   18) and part opacity (no binary home in V3_0) are NO-MAP:
//!   refused typed by the exporter rather than silently
//!   downgraded. A converter that approximates what it cannot
//!   represent is lying in a file format's voice.
//!
//! The border also checks the *style* contract: an export spec
//! names its style lane, and a construct that survives as EXACT
//! must still render on-lane — above all the eye rule: pupils
//! are heart-shaped in every lane, iris color is canonical.

/// The two style lanes an export spec may name (SPEC §7.1).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StyleLane {
    /// Flat cel, hard shadow shapes, bold lineart (default).
    HouseCel,
    /// Painterly/glossy: soft gradients, glossier skin/eyes.
    PonyPainterly,
}

/// The constructs a rig can contain, at the granularity the
/// mapping tables judge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConstructKind {
    /// A scalar parameter with a keyform track.
    Parameter,
    /// A parameter→property binding.
    Binding,
    /// A rotation or warp deformer.
    Deformer,
    /// Per-form multiply color on a drawable.
    MultiplyColor,
    /// Per-form screen color on a drawable.
    ScreenColor,
    /// A blend mode on a drawable. `has_v3_target` records
    /// whether this mode is one of the 3 of 18 with a V3_0
    /// home.
    BlendMode {
        /// True for the 3 blend modes V3_0 can represent.
        has_v3_target: bool,
    },
    /// Part opacity (no binary home in V3_0).
    PartOpacity,
    /// A glue pair between drawables.
    Glue,
    /// A clipping mask.
    ClipMask,
}

/// A moc3 target version. The border's home version is V3_0;
/// V4_2 is where per-form colors become EXACT.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MocVersion {
    /// Major version.
    pub major: u32,
    /// Minor version.
    pub minor: u32,
}

impl MocVersion {
    /// The border's home target.
    pub const V3_0: Self = Self { major: 3, minor: 0 };
    /// Where multiply/screen colors stop being LOSSY.
    pub const V4_2: Self = Self { major: 4, minor: 2 };
}

/// How a construct crosses the border.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MappingClass {
    /// Crosses unchanged.
    Exact,
    /// Crosses approximated; the loss is named in the report.
    Lossy,
    /// Does not cross; the exporter refuses it typed.
    NoMap,
}

/// Classify one construct for one target version — the whole
/// mapping table as one function.
#[must_use]
pub fn classify(kind: ConstructKind, target: MocVersion) -> MappingClass {
    match kind {
        ConstructKind::Parameter
        | ConstructKind::Binding
        | ConstructKind::Deformer
        | ConstructKind::Glue
        | ConstructKind::ClipMask => MappingClass::Exact,
        ConstructKind::MultiplyColor | ConstructKind::ScreenColor => {
            if target >= MocVersion::V4_2 {
                MappingClass::Exact
            } else {
                MappingClass::Lossy
            }
        }
        ConstructKind::BlendMode { has_v3_target } => {
            if has_v3_target {
                MappingClass::Exact
            } else {
                MappingClass::NoMap
            }
        }
        ConstructKind::PartOpacity => MappingClass::NoMap,
    }
}

/// One construct instance in a rig being exported.
#[derive(Clone, Debug, PartialEq)]
pub struct Construct {
    /// The construct's name in the rig.
    pub name: String,
    /// What kind of construct it is.
    pub kind: ConstructKind,
    /// Style facts the border checks when the construct
    /// carries face/eye material: `Some` only for eye-bearing
    /// constructs.
    pub eye: Option<EyeStyle>,
}

/// The style facts the border verifies on eye material.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EyeStyle {
    /// Pupil shape is the heart (the eye rule, SPEC §7.2).
    pub heart_pupil: bool,
    /// The iris renders in the character's canonical color.
    pub iris_canonical: bool,
    /// The lane this construct was authored in.
    pub lane: StyleLane,
}

/// Why the border refuses a whole export.
#[derive(Clone, Debug, PartialEq)]
pub enum BorderError {
    /// A construct maps NO-MAP: the export is refused rather
    /// than silently downgraded.
    Refused {
        /// The construct that cannot cross.
        construct: String,
    },
    /// A construct crosses EXACT but breaks the style
    /// contract: off-lane, or the eye rule violated.
    StyleViolation {
        /// The offending construct.
        construct: String,
        /// What rule it broke, in words.
        rule: &'static str,
    },
}

/// The loss report for one export: every construct, its
/// class, and the counts the grader reads.
#[derive(Clone, Debug, PartialEq)]
pub struct LossReport {
    /// (construct name, its class) in rig order.
    pub entries: Vec<(String, MappingClass)>,
    /// How many crossed EXACT.
    pub exact: usize,
    /// How many crossed LOSSY.
    pub lossy: usize,
    /// How many were NO-MAP (always 0 in a returned report —
    /// a NO-MAP construct is an error, not an entry).
    pub no_map: usize,
}

/// Run the border: classify every construct, check the style
/// contract on anything eye-bearing, and produce the loss
/// report. The first NO-MAP or style violation refuses the
/// whole export typed — borders do not wave problems through.
///
/// # Errors
/// [`BorderError::Refused`] on the first NO-MAP construct;
/// [`BorderError::StyleViolation`] on the first eye construct
/// that is off-lane or breaks the eye rule.
pub fn run_border(
    constructs: &[Construct],
    target: MocVersion,
    spec_lane: StyleLane,
) -> Result<LossReport, BorderError> {
    let mut report = LossReport {
        entries: Vec::with_capacity(constructs.len()),
        exact: 0,
        lossy: 0,
        no_map: 0,
    };
    for c in constructs {
        let class = classify(c.kind, target);
        if class == MappingClass::NoMap {
            return Err(BorderError::Refused {
                construct: c.name.clone(),
            });
        }
        if let Some(eye) = c.eye {
            if eye.lane != spec_lane {
                return Err(BorderError::StyleViolation {
                    construct: c.name.clone(),
                    rule: "construct authored off the spec's style lane",
                });
            }
            if !eye.heart_pupil {
                return Err(BorderError::StyleViolation {
                    construct: c.name.clone(),
                    rule: "the eye rule: pupils are heart-shaped in every lane",
                });
            }
            if !eye.iris_canonical {
                return Err(BorderError::StyleViolation {
                    construct: c.name.clone(),
                    rule: "the eye rule: iris keeps the canonical color",
                });
            }
        }
        match class {
            MappingClass::Exact => report.exact += 1,
            MappingClass::Lossy => report.lossy += 1,
            MappingClass::NoMap => unreachable!("refused above"),
        }
        report.entries.push((c.name.clone(), class));
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(name: &str, kind: ConstructKind) -> Construct {
        Construct {
            name: name.to_string(),
            kind,
            eye: None,
        }
    }

    #[test]
    fn the_core_maps_are_exact() {
        for kind in [
            ConstructKind::Parameter,
            ConstructKind::Binding,
            ConstructKind::Deformer,
            ConstructKind::Glue,
            ConstructKind::ClipMask,
        ] {
            assert_eq!(classify(kind, MocVersion::V3_0), MappingClass::Exact);
        }
    }

    #[test]
    fn per_form_colors_are_version_gated() {
        assert_eq!(
            classify(ConstructKind::MultiplyColor, MocVersion::V3_0),
            MappingClass::Lossy
        );
        assert_eq!(
            classify(ConstructKind::ScreenColor, MocVersion::V3_0),
            MappingClass::Lossy
        );
        assert_eq!(
            classify(ConstructKind::MultiplyColor, MocVersion::V4_2),
            MappingClass::Exact
        );
    }

    #[test]
    fn a_full_rig_reports_its_losses() {
        let rig = vec![
            plain("angle_x", ConstructKind::Parameter),
            plain("body_mul", ConstructKind::MultiplyColor),
            plain("cheek_screen", ConstructKind::ScreenColor),
            plain(
                "blend_normal",
                ConstructKind::BlendMode {
                    has_v3_target: true,
                },
            ),
        ];
        let report = run_border(&rig, MocVersion::V3_0, StyleLane::HouseCel).unwrap();
        assert_eq!(report.exact, 2);
        assert_eq!(report.lossy, 2);
        assert_eq!(report.no_map, 0);
        assert_eq!(report.entries.len(), 4);
    }

    // ---- adversarial ----

    #[test]
    fn no_map_constructs_refuse_the_export() {
        let rig = vec![
            plain("angle_x", ConstructKind::Parameter),
            plain("part_opacity", ConstructKind::PartOpacity),
        ];
        assert_eq!(
            run_border(&rig, MocVersion::V3_0, StyleLane::HouseCel),
            Err(BorderError::Refused {
                construct: "part_opacity".to_string()
            })
        );
        let rig = vec![plain(
            "blend_overlay_plus",
            ConstructKind::BlendMode {
                has_v3_target: false,
            },
        )];
        assert!(matches!(
            run_border(&rig, MocVersion::V3_0, StyleLane::HouseCel),
            Err(BorderError::Refused { .. })
        ));
    }

    #[test]
    fn the_eye_rule_is_checked_at_the_border() {
        let mut eye_construct = plain("iris", ConstructKind::Parameter);
        eye_construct.eye = Some(EyeStyle {
            heart_pupil: false,
            iris_canonical: true,
            lane: StyleLane::HouseCel,
        });
        assert!(matches!(
            run_border(&[eye_construct], MocVersion::V3_0, StyleLane::HouseCel),
            Err(BorderError::StyleViolation { .. })
        ));
    }

    #[test]
    fn off_lane_and_off_color_eyes_are_violations() {
        let mut off_lane = plain("iris", ConstructKind::Parameter);
        off_lane.eye = Some(EyeStyle {
            heart_pupil: true,
            iris_canonical: true,
            lane: StyleLane::PonyPainterly,
        });
        assert!(matches!(
            run_border(&[off_lane], MocVersion::V3_0, StyleLane::HouseCel),
            Err(BorderError::StyleViolation {
                rule: "construct authored off the spec's style lane",
                ..
            })
        ));
        let mut off_color = plain("iris", ConstructKind::Parameter);
        off_color.eye = Some(EyeStyle {
            heart_pupil: true,
            iris_canonical: false,
            lane: StyleLane::HouseCel,
        });
        assert!(matches!(
            run_border(&[off_color], MocVersion::V3_0, StyleLane::HouseCel),
            Err(BorderError::StyleViolation { .. })
        ));
    }

    #[test]
    fn lawful_eyes_cross() {
        let mut eye_construct = plain("iris", ConstructKind::Parameter);
        eye_construct.eye = Some(EyeStyle {
            heart_pupil: true,
            iris_canonical: true,
            lane: StyleLane::PonyPainterly,
        });
        let report =
            run_border(&[eye_construct], MocVersion::V3_0, StyleLane::PonyPainterly).unwrap();
        assert_eq!(report.exact, 1);
    }
}
