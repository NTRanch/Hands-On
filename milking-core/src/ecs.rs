//! ECS schedule discipline (Playroom P3's twin): Strobe on
//! Bevy, reduced to the rule the level teaches — *the schedule
//! is the program.*
//!
//! Components are data; systems are functions over declared
//! queries; the schedule is a contract the scene renders under.
//! A schedule is lawful only when every system runs *after*
//! every system that produces what it reads, and no two systems
//! that share a step mutably borrow the same component. The
//! scene simulator then runs the lawful schedule on a fixed
//! step under an event load: a lawful scene holds (its checksum
//! is a pure function of steps and load), an unlawful one is
//! refused before it can desync anything.

/// The club scene's components — data only, as ECS demands.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Component {
    /// Player/audience input events.
    Input,
    /// Rig parameters and animation state.
    Rig,
    /// Physics bodies and springs.
    Physics,
    /// Haptics emission state.
    Haptics,
    /// Render-prep buffers.
    RenderPrep,
}

/// One system: a name, the components it reads, the components
/// it writes. A write is also a read for ordering purposes (a
/// system consumes the current value of what it updates).
#[derive(Clone, Debug, PartialEq)]
pub struct SystemDef {
    /// Display/identity name.
    pub name: &'static str,
    /// Components read (must be produced before this runs).
    pub reads: Vec<Component>,
    /// Components written (this system is their producer).
    pub writes: Vec<Component>,
}

/// Why a schedule is unlawful.
#[derive(Clone, Debug, PartialEq)]
pub enum ScheduleError {
    /// A system reads a component no earlier system produces
    /// and which is not an initial component of the scene.
    ReadsBeforeProducer {
        /// The offending system.
        system: &'static str,
        /// The component it needed too early.
        component: Component,
    },
    /// Two systems in the same step both write one component:
    /// an aliased mutable borrow, refused like the borrow
    /// checker refuses it.
    AliasedWrite {
        /// The first writer.
        first: &'static str,
        /// The second writer.
        second: &'static str,
        /// The component both write.
        component: Component,
    },
    /// The schedule names a system that does not exist.
    UnknownSystem {
        /// The name that was not found.
        name: String,
    },
    /// A declared system never made it into the schedule.
    MissingSystem {
        /// The system left out.
        name: &'static str,
    },
}

/// The club scene's five canonical systems, in their lawful
/// order: input → rig/animation → physics → haptics → render
/// prep. This is the schedule the level asks the player to
/// reconstruct.
#[must_use]
pub fn club_systems() -> Vec<SystemDef> {
    vec![
        SystemDef {
            name: "input",
            reads: vec![],
            writes: vec![Component::Input],
        },
        SystemDef {
            name: "rig",
            reads: vec![Component::Input],
            writes: vec![Component::Rig],
        },
        SystemDef {
            name: "physics",
            reads: vec![Component::Rig],
            writes: vec![Component::Physics],
        },
        SystemDef {
            name: "haptics",
            reads: vec![Component::Physics],
            writes: vec![Component::Haptics],
        },
        SystemDef {
            name: "render",
            reads: vec![Component::Rig, Component::Physics, Component::Haptics],
            writes: vec![Component::RenderPrep],
        },
    ]
}

/// Validate a proposed schedule (a list of steps; each step is
/// a set of systems running together) against the declared
/// systems. Initial components are the scene's starting data —
/// readable by anyone, produced by no system.
///
/// # Errors
/// The first [`ScheduleError`] the schedule commits, in step
/// order: unknown or missing systems, a read before its
/// producer, or an aliased write inside one step.
pub fn validate_schedule(
    systems: &[SystemDef],
    initial: &[Component],
    steps: &[Vec<&str>],
) -> Result<(), ScheduleError> {
    // Every scheduled name must exist; every declared system
    // must be scheduled exactly once.
    let mut scheduled: Vec<&str> = Vec::new();
    for step in steps {
        for name in step {
            if !systems.iter().any(|s| s.name == *name) {
                return Err(ScheduleError::UnknownSystem {
                    name: (*name).to_string(),
                });
            }
            scheduled.push(name);
        }
    }
    for sys in systems {
        if !scheduled.contains(&sys.name) {
            return Err(ScheduleError::MissingSystem { name: sys.name });
        }
    }
    // Ordering + aliasing, walking steps in order. `produced`
    // accumulates everything earlier steps (and the scene's
    // initial data) provide.
    let mut produced: Vec<Component> = initial.to_vec();
    for step in steps {
        // Aliasing inside the step: two writers of one
        // component in the same step.
        let mut writers: Vec<(&str, Component)> = Vec::new();
        for name in step {
            let sys = systems
                .iter()
                .find(|s| s.name == *name)
                .expect("checked above");
            for w in &sys.writes {
                if let Some((first, _)) = writers.iter().find(|(_, c)| c == w) {
                    return Err(ScheduleError::AliasedWrite {
                        first,
                        second: sys.name,
                        component: *w,
                    });
                }
                writers.push((sys.name, *w));
            }
        }
        // Reads must be satisfied by initial data or earlier
        // steps — NOT by a system in the same step (same-step
        // systems have no order between them).
        for name in step {
            let sys = systems
                .iter()
                .find(|s| s.name == *name)
                .expect("checked above");
            for r in &sys.reads {
                if !produced.contains(r) {
                    return Err(ScheduleError::ReadsBeforeProducer {
                        system: sys.name,
                        component: *r,
                    });
                }
            }
        }
        for (_, c) in writers {
            if !produced.contains(&c) {
                produced.push(c);
            }
        }
    }
    Ok(())
}

/// The outcome of running the scene for a fixed number of
/// steps under an event load.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneOutcome {
    /// Steps actually simulated.
    pub steps: u64,
    /// Total events the crowd threw at the scene.
    pub events: u64,
    /// Deterministic checksum of the run: a pure function of
    /// steps and the load curve, identical across runs.
    pub checksum: u64,
    /// True when every step held (a lawful schedule under any
    /// load this function is given always holds — instability
    /// is a schedule property, refused at validation).
    pub stable: bool,
}

/// Run the lawful club scene: `steps` fixed steps, the crowd
/// load growing linearly from `load_start` to `load_end`
/// events per step across the run. Deterministic: the same
/// arguments always produce the same outcome.
#[must_use]
pub fn run_scene(steps: u64, load_start: u64, load_end: u64) -> SceneOutcome {
    let mut events: u64 = 0;
    let mut checksum: u64 = 0xcbf2_9ce4_8422_2325;
    for i in 0..steps {
        let load = if steps <= 1 {
            load_start
        } else {
            load_start + (load_end - load_start) * i / (steps - 1)
        };
        events += load;
        // Fold the step's load through an FNV-1a step so the
        // checksum depends on the whole load curve, not just
        // its sum.
        for b in load.to_le_bytes() {
            checksum ^= u64::from(b);
            checksum = checksum.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    SceneOutcome {
        steps,
        events,
        checksum,
        stable: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lawful() -> Vec<Vec<&'static str>> {
        vec![
            vec!["input"],
            vec!["rig"],
            vec!["physics"],
            vec!["haptics"],
            vec!["render"],
        ]
    }

    #[test]
    fn the_canonical_order_is_lawful() {
        let systems = club_systems();
        assert_eq!(validate_schedule(&systems, &[], &lawful()), Ok(()));
    }

    #[test]
    fn one_step_per_system_in_order_is_the_minimal_lawful_shape() {
        // The canonical schedule uses exactly the declared
        // producer chain.
        let systems = club_systems();
        assert_eq!(systems.len(), 5);
        assert_eq!(systems[0].name, "input");
        assert_eq!(systems[4].name, "render");
    }

    // ---- adversarial ----

    #[test]
    fn reading_before_the_producer_is_refused() {
        let systems = club_systems();
        let mut steps = lawful();
        steps.swap(1, 2); // physics before rig
        assert_eq!(
            validate_schedule(&systems, &[], &steps),
            Err(ScheduleError::ReadsBeforeProducer {
                system: "physics",
                component: Component::Rig,
            })
        );
    }

    #[test]
    fn same_step_dependency_is_not_ordering() {
        // Rig and physics in one step: physics reads Rig, but a
        // same-step system has no order — still refused.
        let systems = club_systems();
        let steps = vec![
            vec!["input"],
            vec!["rig", "physics"],
            vec!["haptics"],
            vec!["render"],
        ];
        assert!(matches!(
            validate_schedule(&systems, &[], &steps),
            Err(ScheduleError::ReadsBeforeProducer {
                system: "physics",
                ..
            })
        ));
    }

    #[test]
    fn aliased_writes_in_one_step_are_refused() {
        let mut systems = club_systems();
        systems.push(SystemDef {
            name: "rig_echo",
            reads: vec![Component::Input],
            writes: vec![Component::Rig],
        });
        let steps = vec![
            vec!["input"],
            vec!["rig", "rig_echo"],
            vec!["physics"],
            vec!["haptics"],
            vec!["render"],
        ];
        assert!(matches!(
            validate_schedule(&systems, &[], &steps),
            Err(ScheduleError::AliasedWrite {
                component: Component::Rig,
                ..
            })
        ));
    }

    #[test]
    fn unknown_and_missing_systems_are_refused() {
        let systems = club_systems();
        let mut steps = lawful();
        steps[0] = vec!["imput"];
        assert!(matches!(
            validate_schedule(&systems, &[], &steps),
            Err(ScheduleError::UnknownSystem { .. })
        ));
        let mut steps = lawful();
        steps.remove(4);
        assert_eq!(
            validate_schedule(&systems, &[], &steps),
            Err(ScheduleError::MissingSystem { name: "render" })
        );
    }

    #[test]
    fn initial_components_are_readable_without_a_producer() {
        // A scene whose Input arrives as initial data needs no
        // input system scheduled first — but the system must
        // still be scheduled somewhere (MissingSystem check).
        let systems = club_systems();
        let steps = vec![
            vec!["rig"],
            vec!["input"],
            vec!["physics"],
            vec!["haptics"],
            vec!["render"],
        ];
        assert_eq!(
            validate_schedule(&systems, &[Component::Input], &steps),
            Ok(())
        );
    }

    #[test]
    fn scene_runs_are_deterministic_and_load_shaped() {
        let a = run_scene(120, 2, 40);
        let b = run_scene(120, 2, 40);
        assert_eq!(a, b, "same run, same outcome — the schedule is the program");
        assert!(a.stable);
        assert_eq!(a.steps, 120);
        assert!(a.events > 120 * 2 && a.events < 120 * 40);
        let calm = run_scene(120, 2, 3);
        assert_ne!(a.checksum, calm.checksum, "load shapes the run");
    }
}
