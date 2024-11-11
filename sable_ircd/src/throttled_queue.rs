use std::sync::atomic::{AtomicI64, Ordering};

use sable_network::utils::now;
use serde::{Deserialize, Serialize};

use concurrent_queue::{ConcurrentQueue, PushError};

/// Parameters for the token bucket algorithm used by [`ThrottledQueue`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ThrottleSettings {
    /// Number of messages allowed per `time` seconds
    pub num: i64,
    /// Time permitted to send `num` messages
    pub time: i64,
    /// Number of messages allowed to temporarily exceed the normal rate
    pub burst: i64,
}

/// A message queue that implements token bucket throttling on read, as well as
/// enforcing a maximum number of pending messages.
#[derive(Debug)]
pub struct ThrottledQueue<T> {
    settings: ThrottleSettings,
    counter: AtomicI64,

    pending: ConcurrentQueue<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedThrottledQueue<T> {
    settings: ThrottleSettings,
    counter: i64,
    capacity: usize,
    pending: Vec<T>,
}

/// An iterator over those messages in a `ThrottledQueue` that the throttle permits to
/// be processed
#[derive(Debug)]
pub struct ThrottledQueueIterator<'a, T>(&'a ThrottledQueue<T>);

impl<T: std::fmt::Debug> ThrottledQueue<T> {
    /// Construct a `ThrottledQueue` with the given throttle settings and maximum queue size
    pub fn new(settings: ThrottleSettings, max_len: usize) -> Self {
        Self {
            settings,
            counter: AtomicI64::new(0),
            pending: ConcurrentQueue::bounded(max_len),
        }
    }

    /*
        /// Replace the current throttle settings with the provided new settings
        pub fn change_settings(&mut self, new_settings: ThrottleSettings)
        {
            self.settings = new_settings;

            // `self.counter` loses meaning if the multiplier in settings changes, so wipe it out
            // and start again. The possible side effect here is that if the queue had previously used
            // up its burst capacity, it will be reset and allowed to immediately burst again. Changing
            // settings should be an infrequent enough operation that this doesn't matter.
            self.counter = 0;
        }
    */

    /// Add an item to the queue, if doing so does not exceed the maximum capacity
    ///
    /// Returns `Ok(())` on success, and `Err(_)` containing the provided item if the queue is full
    pub fn add(&self, item: T) -> Result<(), T> {
        self.pending.push(item).map_err(PushError::into_inner)
    }

    /// Retrieve an item from the queue, if one is available and the throttle permits it.
    ///
    /// If there is no pending item in the queue, then no tokens are consumed by the call.
    pub fn next(&self) -> Option<T> {
        if self.pending.is_empty() {
            None
        } else {
            let adjusted_now = now() * self.settings.num;

            // If the counter has fallen behind the adjusted 'now' value, update it to match
            self.counter.fetch_max(adjusted_now, Ordering::Relaxed);

            if self.counter.load(Ordering::Relaxed) + self.settings.time
                > adjusted_now + self.settings.burst * self.settings.num
            {
                None
            } else {
                self.counter
                    .fetch_add(self.settings.time, Ordering::Relaxed);
                self.pending.pop().ok()
            }
        }
    }

    /// Return an iterator that repeatedly calls `next()` on this queue to drain
    /// elements that are ready for processing.
    pub fn iter(&self) -> ThrottledQueueIterator<T> {
        ThrottledQueueIterator(self)
    }

    pub fn save(self) -> SavedThrottledQueue<T> {
        let mut pending = Vec::new();

        while let Ok(item) = self.pending.pop() {
            pending.push(item);
        }

        SavedThrottledQueue {
            settings: self.settings,
            counter: self.counter.load(Ordering::Relaxed),
            capacity: self.pending.capacity().unwrap(), // We only construct this type with a bounded queue, so capacity will always be Some
            pending,
        }
    }

    pub fn restore_from(saved: SavedThrottledQueue<T>) -> Self {
        let ret = Self {
            settings: saved.settings,
            counter: AtomicI64::new(saved.counter),
            pending: ConcurrentQueue::bounded(saved.capacity),
        };

        for item in saved.pending {
            ret.pending
                .push(item)
                .expect("SavedThrottledQueue exceeded its own capacity?");
        }

        ret
    }
}

impl<T: std::fmt::Debug> Iterator for ThrottledQueueIterator<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.0.next()
    }
}
