//! The level framework (SPEC §6): a registry of levels, each
//! hosted by an original adult character with her *own* default
//! sensitivity profile. Learning her profile — how much gain she
//! runs, where her threshold sits, how big her reservoir is — is
//! part of the play, because a reaction profile is data, and data
//! differs per character exactly the way it will in Torogao.
//!
//! Two tracks (Amendment 8): the **Playroom**, which holds the
//! play levels and the deterministic-animation levels in one
//! sequence (P1..P9), and **the Study**, the Rust-mastery track
//! (Lessons S1..S8). All seventeen are built: P1 and the P4
//! Loop Studio slice keep their bespoke loops in `main.rs`,
//! and the other fifteen are hosted by the session layer
//! (`session` / `play` / `study`), each with a scripted route
//! that reaches its success state.

use milking_core::sim::SensitivityProfile;

/// A level's host: an original adult character plus the reaction
/// profile she ships with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HostGirl {
    /// Her name.
    pub name: &'static str,
    /// One-line role/sketch.
    pub sketch: &'static str,
    /// Her default sensitivity profile.
    pub profile: SensitivityProfile,
}

/// Which curriculum track a level belongs to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Track {
    /// The Playroom track: game systems and deterministic
    /// animation, taught by playing them (P1..P9).
    Playroom,
    /// The Study: Rust mastery — you are summoned for a lesson
    /// and the compiler corrects you at the desk (S1..S8).
    /// Codified first, then built by the session layer.
    Study,
}

/// Static description of one level.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LevelDescriptor {
    /// Track this level sits on.
    pub track: Track,
    /// Level number within its track (1-based).
    pub id: u8,
    /// Display title.
    pub title: &'static str,
    /// The codified source system this level teaches.
    pub source_system: &'static str,
    /// Who hosts it.
    pub host: HostGirl,
    /// Whether an engine exists yet. All seventeen do.
    pub built: bool,
}

/// A level the framework can host. Built levels implement this;
/// designed levels are registry data until their engine lands.
pub trait Level {
    /// The level's static descriptor.
    fn descriptor(&self) -> &LevelDescriptor;
}

/// Mireille's profile (P4's host, and the Study's renderer lesson).
fn mireille_profile() -> SensitivityProfile {
    SensitivityProfile {
        gain: 1.4,
        threshold: 65.0,
        reservoir_capacity: 140.0,
        reservoir_rate: 0.45,
        refractory_ticks: 240,
    }
}

/// Vigil's profile (P5/P6's host, and the Study's safety lessons).
fn vigil_profile() -> SensitivityProfile {
    SensitivityProfile {
        gain: 0.9,
        threshold: 85.0,
        reservoir_capacity: 260.0,
        reservoir_rate: 0.6,
        refractory_ticks: 360,
    }
}

/// P1 — the milking game (proving level).
pub struct MilkingLevel {
    descriptor: LevelDescriptor,
}

impl MilkingLevel {
    /// Opal's level, with her balanced default profile.
    #[must_use]
    pub fn new() -> Self {
        Self {
            descriptor: LevelDescriptor {
                track: Track::Playroom,
                id: 1,
                title: "First Milking — Parameters & Reaction Profiles",
                source_system: "Torogao parameter model + reaction profiles",
                host: HostGirl {
                    name: "Opal",
                    sketch: "cowgirl hostess of the teaching farm; patient, teasing, hard to rush",
                    profile: SensitivityProfile::default(),
                },
                built: true,
            },
        }
    }
}

impl Default for MilkingLevel {
    fn default() -> Self {
        Self::new()
    }
}

impl Level for MilkingLevel {
    fn descriptor(&self) -> &LevelDescriptor {
        &self.descriptor
    }
}

/// The Playroom track: one sequence, P1..P9 — play levels first,
/// then the deterministic-animation levels (the animation strand
/// folded in by Amendment 8). Hosts repeat across levels; a host
/// always carries the same profile wherever she appears.
#[must_use]
pub fn playroom_levels() -> Vec<LevelDescriptor> {
    vec![
        MilkingLevel::new().descriptor,
        LevelDescriptor {
            track: Track::Playroom,
            id: 2,
            title: "Fixed-Step Rhythm — Beat Sync",
            source_system: "Strobe beat-sync on a deterministic clock",
            host: HostGirl {
                name: "Sable",
                sketch: "night-drive DJ who lives on the beat grid; twitchy, low threshold",
                profile: SensitivityProfile::high(),
            },
            built: true,
        },
        LevelDescriptor {
            track: Track::Playroom,
            id: 3,
            title: "Stage Systems — Order the Club Scene",
            source_system: "Strobe on Bevy: ECS systems, schedules & queries",
            host: HostGirl {
                name: "Vega",
                sketch: "Strobe club stage manager; runs the scene from the booth — slow build, deep reservoir, enormous payoff when the scene holds together",
                profile: SensitivityProfile::low(),
            },
            built: true,
        },
        LevelDescriptor {
            track: Track::Playroom,
            id: 4,
            title: "Loop Studio — Closure, Bake, Twin-Run Hash",
            source_system: "Torogao animation authoring · Lumen rig parameter space (INP1 generation)",
            host: HostGirl {
                name: "Mireille",
                sketch: "stop-motion animator; hosts the animation levels (the playable slice runs on Opal)",
                profile: mireille_profile(),
            },
            built: true,
        },
        LevelDescriptor {
            track: Track::Playroom,
            id: 5,
            title: "Router's Gate — Haptics Safety",
            source_system: "torogao-haptics router contract (ceiling, watchdog, stop)",
            host: HostGirl {
                name: "Vigil",
                sketch: "gatekeeper of the router shrine; exacting — spills cost her trust fast",
                profile: vigil_profile(),
            },
            built: true,
        },
        LevelDescriptor {
            track: Track::Playroom,
            id: 6,
            title: "Format Border — moc3 V3_0 Export & the Loss Report",
            source_system: "Lumen moc3 exporter: EXACT / LOSSY / NO-MAP mapping classes",
            host: HostGirl {
                name: "Vigil",
                sketch: "border inspector of formats; nothing crosses without a class",
                profile: vigil_profile(),
            },
            built: true,
        },
        LevelDescriptor {
            track: Track::Playroom,
            id: 7,
            title: "Sprite & GIF Bake — Frames That Loop",
            source_system: "Lumen sprite baker: frame selection, closure across the bake",
            host: HostGirl {
                name: "Sable",
                sketch: "gif-loop obsessive; counts frames the way she counts beats",
                profile: SensitivityProfile::high(),
            },
            built: true,
        },
        LevelDescriptor {
            track: Track::Playroom,
            id: 8,
            title: "Funscript Timeline — Motion Becomes Touch",
            source_system: "Lumen funscript writer · torogao-haptics player",
            host: HostGirl {
                name: "Vega",
                sketch: "runs the booth where motion curves leave as haptic tracks",
                profile: SensitivityProfile::low(),
            },
            built: true,
        },
        LevelDescriptor {
            track: Track::Playroom,
            id: 9,
            title: "The Third Dimension — GLB & Renders of the 3D",
            source_system: "Lumen 3D route: GLB export, deterministic renders (DESIGNED ONLY)",
            host: HostGirl {
                name: "Mireille",
                sketch: "animator again, this time with a camera that orbits",
                profile: mireille_profile(),
            },
            built: true,
        },
    ]
}

/// The current name for a host, resolving the legacy name a
/// pre-rename save or settings file may still carry. The P1
/// hostess was renamed by Owner ruling (D-195, 2026-10-27):
/// files written before the rename persist her old name;
/// they load under the new name and the next write persists
/// it. This alias is the ONLY place the old name survives in
/// shipped code — it exists so old saves keep working.
#[must_use]
pub fn canonical_host_name(raw: &str) -> &str {
    match raw {
        "Betsy" => "Opal",
        other => other,
    }
}

/// The Study (Amendment 8): Rust mastery as Lessons S1..S8,
/// codified as first-class registry entries. The designs,
/// chapter maps, and Torogao use cases live in CURRICULUM.md;
/// the playable lessons live in the session layer (`study`).
/// The framing: you are summoned to the Study for a lesson;
/// the compiler corrects you at the desk. Hosts are
/// the Playroom girls; each lesson's feedback reuses her profile.
#[must_use]
pub fn study_levels() -> Vec<LevelDescriptor> {
    let opal = HostGirl {
        name: "Opal",
        sketch: "hosts the parsing lesson in the Study: patient with first ownership mistakes",
        profile: SensitivityProfile::default(),
    };
    let sable = HostGirl {
        name: "Sable",
        sketch: "hosts the timing & async lessons in the Study: lives on the grid",
        profile: SensitivityProfile::high(),
    };
    let mireille = HostGirl {
        name: "Mireille",
        sketch: "hosts the renderer lesson in the Study: frames resources like film",
        profile: mireille_profile(),
    };
    let vega = HostGirl {
        name: "Vega",
        sketch: "hosts the live-tracking lesson in the Study: runs the booth",
        profile: SensitivityProfile::low(),
    };
    let vigil = HostGirl {
        name: "Vigil",
        sketch: "hosts the safety-critical lessons in the Study: exacting",
        profile: vigil_profile(),
    };
    vec![
        LevelDescriptor {
            track: Track::Study,
            id: 1,
            title: "Own the Puppet — Ownership, Borrowing, Lifetimes",
            source_system: "Rust Book ch. 4, 5, 6, 10 · torogao-format parsing",
            host: opal,
            built: true,
        },
        LevelDescriptor {
            track: Track::Study,
            id: 2,
            title: "Fail Closed — Typed Errors & Contracts",
            source_system: "Rust Book ch. 9, 10 · moc3 importer/exporter gates",
            host: vigil,
            built: true,
        },
        LevelDescriptor {
            track: Track::Study,
            id: 3,
            title: "The Accumulator — Fixed-Step Engine Arithmetic",
            source_system: "Rust Book ch. 3, 13 · torogao-engine fixed step",
            host: sable,
            built: true,
        },
        LevelDescriptor {
            track: Track::Study,
            id: 4,
            title: "Hold the Frame — Renderer Resource Ownership",
            source_system: "Rust Book ch. 15, 16 · winit/wgpu/egui renderer",
            host: mireille,
            built: true,
        },
        LevelDescriptor {
            track: Track::Study,
            id: 5,
            title: "Bounded Queues — The Haptics Router",
            source_system: "Rust Book ch. 10, 16, 19 · torogao-haptics router",
            host: vigil,
            built: true,
        },
        LevelDescriptor {
            track: Track::Study,
            id: 6,
            title: "Async Borders — MCP + A2A Surfaces",
            source_system: "Rust Book ch. 17 · MCP/A2A schema discipline",
            host: sable,
            built: true,
        },
        LevelDescriptor {
            track: Track::Study,
            id: 7,
            title: "Track the Performer — State Machines & Calibration",
            source_system: "Rust Book ch. 6, 19 · live mode tracking/interop",
            host: vega,
            built: true,
        },
        LevelDescriptor {
            track: Track::Study,
            id: 8,
            title: "Aftercare — Tests, Review & the Gate Suite",
            source_system: "Rust Book ch. 11 · the capstone: verification after the intensity",
            host: vigil,
            built: true,
        },
    ]
}

/// The full registry: both tracks.
#[must_use]
pub fn registry() -> Vec<LevelDescriptor> {
    let mut all = playroom_levels();
    all.extend(study_levels());
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playroom_track_is_one_sequence_of_nine_all_built() {
        let reg = playroom_levels();
        assert_eq!(reg.len(), 9);
        assert_eq!(reg.iter().filter(|l| l.built).count(), 9);
        assert_eq!(reg[0].host.name, "Opal");
        assert!(reg[0].built, "P1 is playable");
        assert!(reg[3].built, "P4 Loop Studio's slice is playable");
        assert!(reg.iter().all(|l| l.track == Track::Playroom));
        assert!(
            reg.iter().enumerate().all(|(i, l)| l.id as usize == i + 1),
            "Playroom ids are a single sequence P1..P9"
        );
    }

    #[test]
    fn study_track_is_codified_and_fully_built() {
        let study = study_levels();
        assert_eq!(study.len(), 8);
        assert!(study.iter().all(|l| l.track == Track::Study && l.built));
        assert_eq!(
            study[7].title, "Aftercare — Tests, Review & the Gate Suite",
            "S8 stays the Aftercare capstone"
        );
    }

    #[test]
    fn registry_covers_both_tracks() {
        assert_eq!(registry().len(), 17);
    }

    #[test]
    fn each_playroom_host_has_one_consistent_distinct_profile() {
        // Hosts repeat across levels after the merge; the
        // invariant is one profile per host, distinct per host.
        let reg = playroom_levels();
        let mut hosts: Vec<(&str, SensitivityProfile)> = Vec::new();
        for l in &reg {
            if let Some((_, p)) = hosts.iter().find(|(n, _)| *n == l.host.name) {
                assert_eq!(*p, l.host.profile, "{} changed profile", l.host.name);
            } else {
                assert!(
                    hosts.iter().all(|(_, p)| *p != l.host.profile),
                    "{}'s profile collides with another host's",
                    l.host.name
                );
                hosts.push((l.host.name, l.host.profile));
            }
        }
        assert_eq!(hosts.len(), 5);
    }

    #[test]
    fn level_trait_exposes_the_descriptor() {
        let level = MilkingLevel::new();
        assert_eq!(level.descriptor().id, 1);
        assert!(level.descriptor().built);
    }
}
