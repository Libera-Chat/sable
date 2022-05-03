use serde::{Serialize,Deserialize};
use crate::utils::now;

use std::collections::VecDeque;

/// Parameters for the token bucket algorithm used by [`ThrottledQueue`]
#[derive(Debug,Clone,Copy,Serialize,Deserialize)]
pub struct ThrottleSettings
{
    /// Number of messages allowed per `time` seconds
    pub num: i64,
    /// Time permitted to send `num` messages
    pub time: i64,
    /// Number of messages allowed to temporarily exceed the normal rate
    pub burst: i64,
}

/// A message queue that implements token bucket throttling on read, as well as
/// enforcing a maximum number of pending messages.
#[derive(Debug,Serialize,Deserialize)]
pub struct ThrottledQueue<T>
{
    settings: ThrottleSettings,
    counter: i64,

    max_len: usize,
    pending: VecDeque<T>,
}

/// An iterator over those messages in a `ThrottledQueue` that the throttle permits to
/// be processed
#[derive(Debug)]
pub struct ThrottledQueueIterator<'a, T> ( &'a mut ThrottledQueue<T> );

impl<T: std::fmt::Debug> ThrottledQueue<T>
{
    /// Construct a `ThrottledQueue` with the given throttle settings and maximum queue size
    pub fn new(settings: ThrottleSettings, max_len: usize) -> Self
    {
        Self {
            settings,
            counter: 0,
            max_len,
            pending: VecDeque::with_capacity(max_len)
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
    pub fn add(&mut self, item: T) -> Result<(), T>
    {
        if self.pending.len() < self.max_len
        {
            self.pending.push_back(item);
            Ok(())
        }
        else
        {
            Err(item)
        }
    }

    /// Retrieve an item from the queue, if one is available and the throttle permits it.
    ///
    /// If there is no pending item in the queue, then no tokens are consumed by the call.
    pub fn next(&mut self) -> Option<T>
    {
        if self.pending.is_empty()
        {
            None
        }
        else
        {
            let adjusted_now = now() * self.settings.num;

            if self.counter < adjusted_now
            {
                self.counter = adjusted_now
            }

            if self.counter + self.settings.time > adjusted_now + self.settings.burst * self.settings.num
            {
                None
            }
            else
            {
                self.counter += self.settings.time;
                self.pending.pop_front()
            }
        }
    }

    /// Return an iterator that repeatedly calls `next()` on this queue to drain
    /// elements that are ready for processing.
    pub fn iter_mut(&mut self) -> ThrottledQueueIterator<T>
    {
        ThrottledQueueIterator(self)
    }
}

impl<'a, T: std::fmt::Debug> Iterator for ThrottledQueueIterator<'a, T>
{
    type Item = T;

    fn next(&mut self) -> Option<T>
    {
        self.0.next()
    }
}