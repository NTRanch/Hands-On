//! Bounded Queues (Study S5's twin): the haptics router's
//! queue discipline — every queue names its bound before it
//! runs, and the synchronous stop works *mid-stream*.
//!
//! A bounded queue refuses what it cannot hold, typed; it
//! never grows silently and never drops silently. The study
//! session configures a router from these queues: name the
//! bound, set the watchdog (the router's own module), and
//! rehearse the stop — which drains nothing and delivers
//! nothing further: after the stop, the queue is a wall.

/// A FIFO queue with a named capacity bound.
#[derive(Clone, Debug)]
pub struct BoundedQueue<T> {
    bound: usize,
    items: std::collections::VecDeque<T>,
}

/// Why a push was refused.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QueueError {
    /// The queue is at its bound — backpressure is a typed
    /// refusal, not a silent drop or a silent grow.
    Full {
        /// The queue's bound.
        bound: usize,
    },
    /// The queue has been stopped: the stop is synchronous
    /// and total, so nothing further is accepted, ever, until
    /// the queue is rebuilt.
    Stopped,
}

impl<T> BoundedQueue<T> {
    /// A queue holding at most `bound` items. A bound of 0 is
    /// lawful: a queue that holds nothing and refuses
    /// everything (the null router).
    #[must_use]
    pub fn new(bound: usize) -> Self {
        Self {
            bound,
            items: std::collections::VecDeque::new(),
        }
    }

    /// The named bound.
    #[must_use]
    pub fn bound(&self) -> usize {
        self.bound
    }

    /// Items currently held.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the queue holds nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Push one item.
    ///
    /// # Errors
    /// [`QueueError::Full`] at the bound.
    pub fn push(&mut self, item: T) -> Result<(), QueueError> {
        if self.items.len() >= self.bound {
            return Err(QueueError::Full { bound: self.bound });
        }
        self.items.push_back(item);
        Ok(())
    }

    /// Pop the oldest item, if any.
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop_front()
    }

    /// Drain everything currently held, oldest first — the
    /// orderly shutdown path (contrast with the stop, which
    /// discards).
    pub fn drain(&mut self) -> Vec<T> {
        self.items.drain(..).collect()
    }
}

/// A stream of emissions passing through a bounded queue with
/// a stop latch: the S5 exercise's router rehearsal.
#[derive(Clone, Debug)]
pub struct StreamRouter {
    queue: BoundedQueue<f64>,
    stopped: bool,
    delivered: Vec<f64>,
}

impl StreamRouter {
    /// A router whose queue holds at most `bound` pending
    /// emissions.
    #[must_use]
    pub fn new(bound: usize) -> Self {
        Self {
            queue: BoundedQueue::new(bound),
            stopped: false,
            delivered: Vec::new(),
        }
    }

    /// Offer one emission to the stream.
    ///
    /// # Errors
    /// [`QueueError::Stopped`] after the stop;
    /// [`QueueError::Full`] when the queue is at its bound
    /// (the consumer is behind — slow down, don't buffer
    /// forever).
    pub fn offer(&mut self, value: f64) -> Result<(), QueueError> {
        if self.stopped {
            return Err(QueueError::Stopped);
        }
        self.queue.push(value)
    }

    /// Deliver the next queued emission to the consumer.
    /// After the stop this returns `None` forever: pending
    /// items were discarded by the stop, not delivered late.
    pub fn deliver(&mut self) -> Option<f64> {
        if self.stopped {
            return None;
        }
        let v = self.queue.pop()?;
        self.delivered.push(v);
        Some(v)
    }

    /// The synchronous stop: latches now, discards the pending
    /// queue, and freezes delivery. Returns how many pending
    /// emissions were discarded — the stop's cost, stated.
    pub fn stop(&mut self) -> usize {
        self.stopped = true;
        let discarded = self.queue.len();
        self.queue.drain();
        discarded
    }

    /// Whether the stop has latched.
    #[must_use]
    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    /// Everything delivered so far, in order.
    #[must_use]
    pub fn delivered(&self) -> &[f64] {
        &self.delivered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_queue_holds_to_its_bound() {
        let mut q = BoundedQueue::new(2);
        q.push(1).unwrap();
        q.push(2).unwrap();
        assert_eq!(q.push(3), Err(QueueError::Full { bound: 2 }));
        assert_eq!(q.pop(), Some(1));
        q.push(3).unwrap();
        assert_eq!(q.drain(), vec![2, 3]);
        assert!(q.is_empty());
    }

    #[test]
    fn fifo_order_is_kept() {
        let mut q = BoundedQueue::new(4);
        for v in [0.1, 0.2, 0.3] {
            q.push(v).unwrap();
        }
        assert_eq!(q.pop(), Some(0.1));
        assert_eq!(q.pop(), Some(0.2));
        assert_eq!(q.pop(), Some(0.3));
        assert_eq!(q.pop(), None);
    }

    #[test]
    fn the_stop_discards_pending_and_freezes_delivery() {
        let mut r = StreamRouter::new(8);
        r.offer(0.1).unwrap();
        r.offer(0.2).unwrap();
        assert_eq!(r.deliver(), Some(0.1));
        let discarded = r.stop();
        assert_eq!(discarded, 1, "the 0.2 was pending when the stop landed");
        assert!(r.is_stopped());
        assert_eq!(r.deliver(), None, "nothing is delivered after the stop");
        assert_eq!(r.delivered(), &[0.1]);
    }

    // ---- adversarial ----

    #[test]
    fn offers_after_the_stop_are_refused() {
        let mut r = StreamRouter::new(4);
        r.stop();
        assert_eq!(r.offer(0.1), Err(QueueError::Stopped));
    }

    #[test]
    fn a_zero_bound_queue_refuses_everything() {
        let mut q: BoundedQueue<u8> = BoundedQueue::new(0);
        assert_eq!(q.push(1), Err(QueueError::Full { bound: 0 }));
        assert_eq!(q.pop(), None);
    }

    #[test]
    fn backpressure_is_typed_not_silent() {
        let mut r = StreamRouter::new(1);
        r.offer(0.5).unwrap();
        assert_eq!(r.offer(0.6), Err(QueueError::Full { bound: 1 }));
        assert_eq!(r.deliver(), Some(0.5), "the held item is unharmed");
    }
}
