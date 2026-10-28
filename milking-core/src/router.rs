//! The haptics router contract (Playroom P5's twin):
//! `torogao-haptics`' safety router, rule for rule.
//!
//! The contract has four clauses, and this module is those
//! clauses as types:
//!
//! 1. **Ceiling.** No emission exceeds the configured ceiling,
//!    ever. Over-ceiling input is a contract violation — the
//!    router refuses it typed rather than silently clamping a
//!    signal it was told to route faithfully.
//! 2. **Watchdog.** The source must heartbeat inside the
//!    watchdog window. A dead source means the router emits
//!    nothing: silence is the safe state.
//! 3. **Synchronous stop.** [`Router::stop`] latches *now*, in
//!    the call, and zeroes the output. There is no "finish the
//!    current frame first."
//! 4. **No resume on reconnect.** A stopped or faulted router
//!    never resumes on its own. Only an explicit
//!    [`Router::reset`] — a human decision — clears a latch,
//!    and a reset router starts silent, not mid-signal.
//!
//! Fakes only, as everywhere in this crate: the router routes
//! *values*, and there is no transport here to route them to.

/// Router configuration: the negotiated terms, named before
/// anything runs (Daddy Style: bounds in advance).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RouterConfig {
    /// Maximum lawful emission, 0..=1.
    pub ceiling: f64,
    /// Ticks a heartbeat stays valid: the source must beat at
    /// least this often or the watchdog declares it dead.
    pub watchdog_ticks: u64,
}

impl Default for RouterConfig {
    /// The estate's fake-readout terms: ceiling 0.20 (the same
    /// ceiling the feedback panel clamps to), watchdog 90
    /// ticks (1.5 s at 60 Hz).
    fn default() -> Self {
        Self {
            ceiling: 0.20,
            watchdog_ticks: 90,
        }
    }
}

/// Why the router refused or faulted. Each variant names the
/// clause of the contract that fired.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RouterError {
    /// Clause 1: a value above the ceiling was offered.
    OverCeiling {
        /// The value offered.
        value: f64,
        /// The configured ceiling.
        ceiling: f64,
    },
    /// Clause 2: no heartbeat inside the watchdog window —
    /// the source is declared dead and the round faults.
    WatchdogDead {
        /// Ticks since the last heartbeat.
        silent_ticks: u64,
        /// The configured window.
        window: u64,
    },
    /// Clause 3/4: the router is stopped (latched) and routes
    /// nothing until an explicit reset.
    Stopped,
    /// A heartbeat or emission arrived stamped with a tick
    /// behind the router's clock — time travel is refused.
    StaleTick {
        /// The tick offered.
        event_tick: u64,
        /// The router's current tick.
        current_tick: u64,
    },
}

/// What the router emitted for one offered value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Emission {
    /// The value was lawful and is routed through unchanged.
    Routed(f64),
    /// The watchdog is dead or the router is stopped: the
    /// lawful emission is silence, and that is what this tick
    /// carries. (A dead watchdog faults the *round* via
    /// [`Router::poll`]; per-tick emission stays total.)
    Silence,
}

/// The router: a ceiling, a watchdog, a stop latch, and an
/// audit trail of every fault it has declared.
#[derive(Clone, Debug)]
pub struct Router {
    cfg: RouterConfig,
    now: u64,
    last_heartbeat: u64,
    stopped: bool,
    faults: Vec<RouterError>,
}

impl Router {
    /// A fresh router at tick 0. The watchdog starts *unfed*:
    /// until the first heartbeat, the source is not yet
    /// proven alive — but the window runs from tick 0, so a
    /// source that never beats faults at the window's edge.
    #[must_use]
    pub fn new(cfg: RouterConfig) -> Self {
        Self {
            cfg,
            now: 0,
            last_heartbeat: 0,
            stopped: false,
            faults: Vec::new(),
        }
    }

    /// The router's configuration.
    #[must_use]
    pub fn config(&self) -> RouterConfig {
        self.cfg
    }

    /// Whether the stop latch is set.
    #[must_use]
    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    /// Every fault declared so far, in order.
    #[must_use]
    pub fn faults(&self) -> &[RouterError] {
        &self.faults
    }

    /// Whether the watchdog currently considers the source
    /// alive: a heartbeat landed within the window.
    #[must_use]
    pub fn watchdog_alive(&self) -> bool {
        self.now - self.last_heartbeat <= self.cfg.watchdog_ticks
    }

    /// Feed the watchdog: the source proves it is alive at
    /// `tick`. Also advances the router's clock.
    ///
    /// # Errors
    /// [`RouterError::StaleTick`] if `tick` is behind the
    /// clock; [`RouterError::Stopped`] while latched (a stopped
    /// router does not accept heartbeats — reset is the only
    /// way back, clause 4).
    pub fn heartbeat(&mut self, tick: u64) -> Result<(), RouterError> {
        if self.stopped {
            return Err(RouterError::Stopped);
        }
        if tick < self.now {
            return Err(RouterError::StaleTick {
                event_tick: tick,
                current_tick: self.now,
            });
        }
        self.now = tick;
        self.last_heartbeat = tick;
        Ok(())
    }

    /// Advance the router's clock without a heartbeat (time
    /// passing is how watchdogs die).
    ///
    /// # Errors
    /// [`RouterError::StaleTick`] on a backwards clock.
    pub fn advance_to(&mut self, tick: u64) -> Result<(), RouterError> {
        if tick < self.now {
            return Err(RouterError::StaleTick {
                event_tick: tick,
                current_tick: self.now,
            });
        }
        self.now = tick;
        Ok(())
    }

    /// Clause 3: the synchronous stop. Latches immediately and
    /// returns the lawful output from this instant: zero.
    /// Idempotent — stopping a stopped router is still zero.
    pub fn stop(&mut self) -> f64 {
        self.stopped = true;
        0.0
    }

    /// Clause 4: the explicit human reset. Clears the stop
    /// latch and re-arms the watchdog from the current tick.
    /// The router starts silent: the next emission still needs
    /// a lawful offer inside a live window.
    pub fn reset(&mut self) {
        self.stopped = false;
        self.last_heartbeat = self.now;
    }

    /// Poll the watchdog: if the source has been silent past
    /// the window, declare the fault (once per death) and
    /// latch the router stopped — a dead source ends the
    /// round, it does not merely mute it.
    ///
    /// # Errors
    /// [`RouterError::WatchdogDead`] on the poll that finds
    /// the window exceeded.
    pub fn poll(&mut self) -> Result<(), RouterError> {
        if self.stopped {
            return Ok(());
        }
        let silent = self.now - self.last_heartbeat;
        if silent > self.cfg.watchdog_ticks {
            let err = RouterError::WatchdogDead {
                silent_ticks: silent,
                window: self.cfg.watchdog_ticks,
            };
            self.faults.push(err);
            self.stopped = true;
            return Err(err);
        }
        Ok(())
    }

    /// Offer one value at the router's current tick. Returns
    /// what the router emits: the value routed, or silence.
    ///
    /// # Errors
    /// [`RouterError::OverCeiling`] if the value exceeds the
    /// ceiling — refused typed and recorded as a fault, never
    /// clamped into compliance. [`RouterError::Stopped`]
    /// while the latch is set.
    pub fn emit(&mut self, value: f64) -> Result<Emission, RouterError> {
        if self.stopped {
            return Err(RouterError::Stopped);
        }
        if value > self.cfg.ceiling {
            let err = RouterError::OverCeiling {
                value,
                ceiling: self.cfg.ceiling,
            };
            self.faults.push(err);
            return Err(err);
        }
        if !self.watchdog_alive() {
            return Ok(Emission::Silence);
        }
        Ok(Emission::Routed(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn router() -> Router {
        Router::new(RouterConfig::default())
    }

    #[test]
    fn lawful_signal_routes_unchanged() {
        let mut r = router();
        r.heartbeat(0).unwrap();
        assert_eq!(r.emit(0.20), Ok(Emission::Routed(0.20)));
        assert_eq!(r.emit(0.05), Ok(Emission::Routed(0.05)));
        assert!(r.faults().is_empty());
    }

    #[test]
    fn stop_is_synchronous_and_total() {
        let mut r = router();
        r.heartbeat(0).unwrap();
        assert_eq!(r.stop(), 0.0);
        assert!(r.is_stopped());
        assert_eq!(r.emit(0.1), Err(RouterError::Stopped));
        // Even a heartbeat cannot revive a stopped router.
        assert_eq!(r.heartbeat(1), Err(RouterError::Stopped));
    }

    #[test]
    fn reset_is_the_only_way_back_and_starts_silent() {
        let mut r = router();
        r.heartbeat(0).unwrap();
        r.stop();
        r.advance_to(500).unwrap();
        r.reset();
        assert!(!r.is_stopped());
        // Re-armed watchdog: emission routes again inside the
        // fresh window.
        assert_eq!(r.emit(0.1), Ok(Emission::Routed(0.1)));
    }

    // ---- adversarial ----

    #[test]
    fn over_ceiling_is_refused_typed_never_clamped() {
        let mut r = router();
        r.heartbeat(0).unwrap();
        let err = r.emit(0.21).unwrap_err();
        assert_eq!(
            err,
            RouterError::OverCeiling {
                value: 0.21,
                ceiling: 0.20
            }
        );
        assert_eq!(r.faults().len(), 1, "the violation is on the audit trail");
    }

    #[test]
    fn dead_watchdog_faults_and_latches() {
        let mut r = router();
        r.heartbeat(0).unwrap();
        r.advance_to(90).unwrap();
        assert_eq!(
            r.poll(),
            Ok(()),
            "at the window edge the source still lives"
        );
        r.advance_to(91).unwrap();
        let err = r.poll().unwrap_err();
        assert_eq!(
            err,
            RouterError::WatchdogDead {
                silent_ticks: 91,
                window: 90
            }
        );
        assert!(r.is_stopped(), "a dead source ends the round");
        assert_eq!(r.emit(0.1), Err(RouterError::Stopped));
    }

    #[test]
    fn silent_window_emits_silence_not_signal() {
        // Watchdog window passed but poll not yet called: the
        // per-tick emission lawfully degrades to silence rather
        // than routing a signal from an unproven source.
        let mut r = Router::new(RouterConfig {
            ceiling: 0.20,
            watchdog_ticks: 10,
        });
        r.heartbeat(0).unwrap();
        r.advance_to(50).unwrap();
        assert_eq!(r.emit(0.1), Ok(Emission::Silence));
    }

    #[test]
    fn backwards_time_is_refused() {
        let mut r = router();
        r.heartbeat(100).unwrap();
        assert_eq!(
            r.heartbeat(99),
            Err(RouterError::StaleTick {
                event_tick: 99,
                current_tick: 100
            })
        );
        assert_eq!(
            r.advance_to(50),
            Err(RouterError::StaleTick {
                event_tick: 50,
                current_tick: 100
            })
        );
    }

    #[test]
    fn no_resume_on_reconnect_without_reset() {
        // Simulate a reconnect storm: heartbeats and offers
        // after a watchdog death change nothing until reset.
        let mut r = router();
        r.heartbeat(0).unwrap();
        r.advance_to(200).unwrap();
        let _ = r.poll();
        for t in 200..210 {
            assert_eq!(r.heartbeat(t), Err(RouterError::Stopped));
            assert_eq!(r.emit(0.1), Err(RouterError::Stopped));
        }
        r.reset();
        assert_eq!(r.emit(0.1), Ok(Emission::Routed(0.1)));
    }
}
