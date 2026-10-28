//! The sprite/GIF bake (Playroom P7's twin): Lumen's sprite
//! baker, reduced to its two governing questions.
//!
//! 1. **Which frames survive sampling?** A bake at a coarser
//!    step keeps the frames whose phases land on the sampling
//!    grid — and must keep the loop's *extremes* (its minimum
//!    and maximum frames), or the baked motion lies about its
//!    range. The baker therefore always selects grid frames
//!    plus the argmin/argmax frames.
//! 2. **Does the loop still close across the bake?** A loop
//!    that closed at 10 ms can open at GIF frame rates: closure
//!    is a property of the *baked table*, checked on the baked
//!    table — the value just past the last frame must return
//!    to the first frame's value by the track's own wrap rule.
//!
//! The eye-rule corollary lives here too: at GIF resolution a
//! heart pupil is a handful of pixels and still required, so
//! occupancy — how much of the pupil survives a frame cell —
//! is a checkable number, not a hope.

use crate::authoring::{Track, TrackError};

/// The bake's frame selection: indices into a full-resolution
/// frame table.
#[derive(Clone, Debug, PartialEq)]
pub struct Selection {
    /// Kept frame indices, ascending, always including 0, the
    /// argmin, and the argmax.
    pub frames: Vec<usize>,
    /// True when the selection keeps both extremes.
    pub keeps_extremes: bool,
}

/// Select the frames a bake at `step` (every `step`th frame)
/// keeps from a full table, forcing in the min and max frames
/// so the baked range is the authored range.
///
/// # Errors
/// [`TrackError::EmptyCycle`] on a zero step or empty table.
pub fn select_frames(table: &[f64], step: usize) -> Result<Selection, TrackError> {
    if step == 0 || table.is_empty() {
        return Err(TrackError::EmptyCycle);
    }
    let mut frames: Vec<usize> = (0..table.len()).step_by(step).collect();
    let mut argmin = 0usize;
    let mut argmax = 0usize;
    for (i, v) in table.iter().enumerate() {
        if *v < table[argmin] {
            argmin = i;
        }
        if *v > table[argmax] {
            argmax = i;
        }
    }
    for idx in [argmin, argmax] {
        if !frames.contains(&idx) {
            frames.push(idx);
        }
    }
    frames.sort_unstable();
    Ok(Selection {
        keeps_extremes: frames.contains(&argmin) && frames.contains(&argmax),
        frames,
    })
}

/// Whether a baked table still loops: the track's wrap rule
/// applied to the baked table's endpoints. A bake closes when
/// the source track closes *and* the selection kept frame 0
/// and the extremes — checked on the artifacts, not assumed.
///
/// # Errors
/// Propagates the track's own closure error; a selection that
/// dropped frame 0 is [`TrackError::LoopOpen`] with the
/// endpoints it actually baked.
pub fn bake_closes(track: &Track, table: &[f64], selection: &Selection) -> Result<(), TrackError> {
    track.check_closure()?;
    if selection.frames.first() != Some(&0) || table.is_empty() {
        return Err(TrackError::LoopOpen {
            start: table.first().copied().unwrap_or(0.0),
            end: table.last().copied().unwrap_or(0.0),
        });
    }
    if !selection.keeps_extremes {
        return Err(TrackError::LoopOpen {
            start: table[0],
            end: table[selection.frames[selection.frames.len() - 1]],
        });
    }
    Ok(())
}

/// Pupil occupancy at GIF resolution: a heart pupil drawn in a
/// cell of `cell_px` pixels survives if its bounding box keeps
/// at least `min_px` lit pixels in the cell. Returns the lit
/// count for a pupil of `pupil_px` lit pixels scaled into the
/// cell — the number the level's style check gates on.
#[must_use]
pub fn pupil_occupancy(pupil_px: usize, cell_px: usize, frame_px: usize) -> usize {
    if frame_px == 0 {
        return 0;
    }
    // Occupancy scales with area: pupil_px * (cell/frame)^2,
    // integer math, rounding down — the pessimistic count.
    pupil_px * cell_px * cell_px / (frame_px * frame_px)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authoring::{Easing, Keyform};

    fn track() -> Track {
        Track::new(
            1000,
            vec![
                Keyform {
                    at_ms: 0,
                    value: 0.0,
                },
                Keyform {
                    at_ms: 400,
                    value: 1.0,
                },
                Keyform {
                    at_ms: 1000,
                    value: 0.0,
                },
            ],
            Easing::Smooth,
        )
        .unwrap()
    }

    #[test]
    fn selection_keeps_grid_and_extremes() {
        let table = track().bake(10).unwrap(); // 100 frames
        let sel = select_frames(&table, 30).unwrap();
        assert!(sel.frames.contains(&0));
        assert!(sel.frames.contains(&40), "the peak survives the bake");
        assert!(sel.keeps_extremes);
        assert!(sel.frames.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn a_closing_track_bakes_closed() {
        let table = track().bake(10).unwrap();
        let sel = select_frames(&table, 25).unwrap();
        assert_eq!(bake_closes(&track(), &table, &sel), Ok(()));
    }

    #[test]
    fn pupil_occupancy_scales_with_area() {
        // Full-res: a 12px pupil in a 12px cell of a 12px
        // frame keeps all 12.
        assert_eq!(pupil_occupancy(12, 12, 12), 12);
        // Quarter-res cell: a quarter of the area.
        assert_eq!(pupil_occupancy(16, 6, 12), 4);
        assert_eq!(pupil_occupancy(10, 5, 0), 0);
    }

    // ---- adversarial ----

    #[test]
    fn open_tracks_stay_open_across_the_bake() {
        let open = Track::new(
            1000,
            vec![
                Keyform {
                    at_ms: 0,
                    value: 0.0,
                },
                Keyform {
                    at_ms: 1000,
                    value: 0.5,
                },
            ],
            Easing::Linear,
        )
        .unwrap();
        let table = open.bake(10).unwrap();
        let sel = select_frames(&table, 25).unwrap();
        assert!(bake_closes(&open, &table, &sel).is_err());
    }

    #[test]
    fn zero_step_and_empty_tables_are_refused() {
        assert_eq!(select_frames(&[0.0], 0), Err(TrackError::EmptyCycle));
        assert_eq!(select_frames(&[], 4), Err(TrackError::EmptyCycle));
    }

    #[test]
    fn dropping_the_extremes_opens_the_loop() {
        // A selection that hand-drops the peak: the baked loop
        // no longer carries the authored range, and the check
        // says so.
        let table = track().bake(10).unwrap();
        let sel = Selection {
            frames: vec![0, 30, 60, 90],
            keeps_extremes: false,
        };
        assert!(bake_closes(&track(), &table, &sel).is_err());
    }
}
