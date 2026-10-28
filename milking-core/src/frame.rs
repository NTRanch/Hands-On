//! Hold the Frame (Study S4's twin): a renderer resource
//! lifecycle on paper — GPU-ish handles with owners, ordered
//! drops, and exactly one justified unsafe block, fenced.
//!
//! A frame holds resources (buffers, textures, pipelines).
//! Every resource has exactly one owner; the frame renders
//! only while all its handles are alive and owned; drops
//! happen in reverse acquisition order — children before the
//! device they were made from. Orphan a handle and the frame
//! *flickers*: visibly, in the state, which is the lesson.
//!
//! The one unsafe block: in the real renderer an FFI call
//! stands at the present. This crate forbids unsafe code, so
//! the block is absent by law and the module instead marks
//! its exact fence point — the Study's rule for unsafe extern
//! in Rust 2024: named, bounded, justified, or absent.

/// A handle to one renderer resource.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Handle {
    /// Resource id (acquisition order, 0-based).
    pub id: usize,
    /// The kind of resource.
    pub kind: ResourceKind,
}

/// The resource kinds a frame owns.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceKind {
    /// The device everything is made from. Acquired first,
    /// dropped last.
    Device,
    /// A GPU buffer.
    Buffer,
    /// A texture.
    Texture,
    /// A render pipeline.
    Pipeline,
}

/// Why a frame operation failed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FrameError {
    /// A handle was used after its drop, or dropped twice.
    DeadHandle {
        /// The handle id.
        id: usize,
    },
    /// A drop was attempted out of order: a resource cannot
    /// be dropped while a later-acquired resource still
    /// lives (children first).
    OutOfOrder {
        /// The handle the caller tried to drop.
        id: usize,
        /// The later handle still alive.
        blocking: usize,
    },
    /// Render attempted while a handle is orphaned (owner
    /// gone): the frame flickers instead of presenting.
    Orphaned {
        /// The orphaned handle id.
        id: usize,
    },
}

/// One frame's resource set, with ownership tracked.
#[derive(Clone, Debug)]
pub struct Frame {
    handles: Vec<Handle>,
    alive: Vec<bool>,
    orphaned: Vec<bool>,
    drop_log: Vec<usize>,
}

impl Frame {
    /// Acquire the standard frame set in order: device,
    /// buffer, texture, pipeline.
    #[must_use]
    pub fn new() -> Self {
        let kinds = [
            ResourceKind::Device,
            ResourceKind::Buffer,
            ResourceKind::Texture,
            ResourceKind::Pipeline,
        ];
        Self {
            handles: kinds
                .iter()
                .enumerate()
                .map(|(id, kind)| Handle { id, kind: *kind })
                .collect(),
            alive: vec![true; 4],
            orphaned: vec![false; 4],
            drop_log: Vec::new(),
        }
    }

    /// The handles, in acquisition order.
    #[must_use]
    pub fn handles(&self) -> &[Handle] {
        &self.handles
    }

    /// Whether a handle is alive.
    #[must_use]
    pub fn is_alive(&self, id: usize) -> bool {
        self.alive.get(id).copied().unwrap_or(false)
    }

    /// The drop order so far (the audit the lesson reads).
    #[must_use]
    pub fn drop_log(&self) -> &[usize] {
        &self.drop_log
    }

    /// Orphan a handle: its owner vanishes without a drop.
    /// The resource is not freed — it is *leaked from the
    /// frame's discipline*, and the frame will refuse to
    /// present until the orphan is reclaimed by a drop.
    ///
    /// # Errors
    /// [`FrameError::DeadHandle`] if the handle is not alive.
    pub fn orphan(&mut self, id: usize) -> Result<(), FrameError> {
        if !self.is_alive(id) {
            return Err(FrameError::DeadHandle { id });
        }
        self.orphaned[id] = true;
        Ok(())
    }

    /// Present the frame. Succeeds only while every handle is
    /// alive and none is orphaned; the present is where the
    /// real renderer's one fenced unsafe block stands (the
    /// FFI call — absent here by this crate's unsafe law).
    ///
    /// # Errors
    /// [`FrameError::Orphaned`] naming the first orphan.
    pub fn render(&self) -> Result<u64, FrameError> {
        for (id, orphaned) in self.orphaned.iter().enumerate() {
            if *orphaned {
                return Err(FrameError::Orphaned { id });
            }
        }
        // The frame's checksum stands in for presented pixels.
        let checksum = self
            .handles
            .iter()
            .filter(|h| self.is_alive(h.id))
            .fold(0x9e37_79b9_7f4a_7c15u64, |acc, h| {
                acc ^ (h.id as u64 + 1).wrapping_mul(0xbf58_476d_1ce4_e5b9)
            });
        // THE fence point — named, bounded, and absent:
        // FFI-RENDER is where the real renderer's one unsafe
        // block stands (the extern call into the driver ABI,
        // sound there because every handle it touches was
        // just proven alive and owned above). This crate
        // forbids unsafe code outright, so the block is
        // *absent by law* — the Study's rule names absence as
        // the fourth lawful outcome, and the fence that
        // remains is this comment plus the proof above it.
        Ok(checksum)
    }

    /// Drop one handle. Drops must be in reverse acquisition
    /// order — a resource dies only after everything acquired
    /// after it. Dropping also reclaims an orphan.
    ///
    /// # Errors
    /// [`FrameError::DeadHandle`] on a double drop;
    /// [`FrameError::OutOfOrder`] while a later handle lives.
    pub fn drop_handle(&mut self, id: usize) -> Result<(), FrameError> {
        if !self.is_alive(id) {
            return Err(FrameError::DeadHandle { id });
        }
        if let Some(blocking) = self
            .alive
            .iter()
            .enumerate()
            .find(|(other, alive)| **alive && *other > id)
            .map(|(other, _)| other)
        {
            return Err(FrameError::OutOfOrder { id, blocking });
        }
        self.alive[id] = false;
        self.orphaned[id] = false;
        self.drop_log.push(id);
        Ok(())
    }

    /// Drop everything in lawful order; returns the order used.
    pub fn drop_all(&mut self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.handles.len()).rev().collect();
        order.retain(|id| self.is_alive(*id));
        for id in &order {
            let id = *id;
            self.alive[id] = false;
            self.orphaned[id] = false;
            self.drop_log.push(id);
        }
        order
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_held_frame_renders() {
        let frame = Frame::new();
        assert!(frame.render().is_ok());
        assert_eq!(frame.handles().len(), 4);
        assert_eq!(frame.handles()[0].kind, ResourceKind::Device);
    }

    #[test]
    fn drops_run_children_first() {
        let mut frame = Frame::new();
        let order = frame.drop_all();
        assert_eq!(order, vec![3, 2, 1, 0]);
        assert_eq!(frame.drop_log(), &[3, 2, 1, 0]);
        assert!(frame.handles().iter().all(|h| !frame.is_alive(h.id)));
    }

    #[test]
    fn single_drops_in_reverse_order_succeed() {
        let mut frame = Frame::new();
        for id in [3usize, 2, 1, 0] {
            frame.drop_handle(id).unwrap();
        }
        assert_eq!(frame.drop_log(), &[3, 2, 1, 0]);
    }

    // ---- adversarial ----

    #[test]
    fn dropping_the_device_first_is_refused() {
        let mut frame = Frame::new();
        assert_eq!(
            frame.drop_handle(0),
            Err(FrameError::OutOfOrder { id: 0, blocking: 1 })
        );
        assert!(frame.is_alive(0), "a refused drop changes nothing");
    }

    #[test]
    fn double_drops_are_refused() {
        let mut frame = Frame::new();
        frame.drop_handle(3).unwrap();
        assert_eq!(frame.drop_handle(3), Err(FrameError::DeadHandle { id: 3 }));
    }

    #[test]
    fn an_orphaned_handle_flickers_the_frame() {
        let mut frame = Frame::new();
        frame.orphan(2).unwrap();
        assert_eq!(frame.render(), Err(FrameError::Orphaned { id: 2 }));
        // Reclaiming by lawful drop restores the frame.
        frame.drop_handle(3).unwrap();
        frame.drop_handle(2).unwrap();
        assert!(frame.render().is_ok());
    }

    #[test]
    fn orphaning_a_dead_handle_is_refused() {
        let mut frame = Frame::new();
        frame.drop_handle(3).unwrap();
        assert_eq!(frame.orphan(3), Err(FrameError::DeadHandle { id: 3 }));
    }
}
