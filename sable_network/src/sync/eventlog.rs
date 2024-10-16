//! Contains the event log

use crate::network::event::*;
use crate::prelude::*;

use chrono::prelude::*;
use std::{
    collections::{BTreeMap, HashMap},
    ops::Bound::*,
};
use tokio::sync::mpsc::UnboundedSender;

/// An event log.
///
/// The event log contains the history of all events that have been seen and
/// processed by the server.
#[derive(Debug)]
pub struct EventLog {
    history: BTreeMap<ServerId, BTreeMap<EventId, Event>>,
    pending: HashMap<EventId, Event>,
    id_gen: ObjectIdGenerator,
    event_sender: Option<UnboundedSender<Event>>,
    last_event_clock: EventClock,
}

/// Saved state for an [EventLog], used to save and restore across an upgrade
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EventLogState {
    id_gen: ObjectIdGenerator,
    clock: EventClock,
}

#[derive(Debug, serde::Serialize)]
pub struct EventLogStats {
    pub pending_events: usize,
    pub current_clock: EventClock,
}

impl EventLog {
    /// Construct a new `EventLog`. `event_sender`, if `Some`, will receive
    /// notification of all new events as they are added to the log and become
    /// ready for processing by the [`Network`].
    pub fn new(idgen: ObjectIdGenerator, event_sender: Option<UnboundedSender<Event>>) -> Self {
        Self {
            history: BTreeMap::new(),
            pending: HashMap::new(),
            id_gen: idgen,
            event_sender,
            last_event_clock: EventClock::new(),
        }
    }

    /// Restore an `EventLog` from a previously saved state
    pub fn restore(state: EventLogState, event_sender: Option<UnboundedSender<Event>>) -> Self {
        Self {
            history: BTreeMap::new(),
            pending: HashMap::new(),
            id_gen: state.id_gen,
            event_sender,
            last_event_clock: state.clock,
        }
    }

    /// Save the state required to resume operation later
    ///
    /// The log is consumed
    pub fn save_state(self) -> EventLogState {
        EventLogState {
            id_gen: self.id_gen,
            clock: self.last_event_clock,
        }
    }

    /// Return a single event.
    pub fn get(&self, id: &EventId) -> Option<&Event> {
        self.history.get(&id.server()).and_then(|x| x.get(id))
    }

    /// Iterate over all events in the current log which do not precede the
    /// given event clock.
    ///
    /// When provided with the current event clock from a remote server, this
    /// will produce a list of all events that this server knows about and the
    /// remote one does not.
    pub fn get_since(&self, id: EventClock) -> impl Iterator<Item = &Event> {
        self.history.iter().flat_map(move |(server, list)| {
            let begin = if let Some(got) = id.get(*server) {
                Excluded(got)
            } else {
                Unbounded
            };
            list.range((begin, Unbounded)).map(|(_, v)| v)
        })
    }

    /// The log's current event clock
    pub fn clock(&self) -> &EventClock {
        &self.last_event_clock
    }

    /// Return some statistics about the event log
    pub fn get_stats(&self) -> EventLogStats {
        EventLogStats {
            pending_events: self.pending.len(),
            current_clock: self.last_event_clock.clone(),
        }
    }

    #[cfg(feature = "debug")]
    /// Access the full list of events stored in the log
    pub fn all_events(&self) -> impl Iterator<Item = &Event> {
        self.history.values().map(|m| m.values()).flatten()
    }

    #[cfg(feature = "debug")]
    /// Access the list of pending events
    pub fn pending_events(&self) -> impl Iterator<Item = &Event> {
        self.pending.values()
    }

    /// Set the clock for this log.
    ///
    /// This should only be used when importing a serialized
    /// [Network] state, to sync the event log's view of
    /// 'current' with that from the imported network state.
    pub fn set_clock(&mut self, new_clock: EventClock) {
        self.last_event_clock = new_clock;
    }

    /// Add an event to the log.
    ///
    /// - If the event ID already exists within the log, do nothing.
    /// - If the event's dependencies (as denoted by the embedded event clock)
    ///   are all already present in the log, then immediately add it to the
    ///   log, update the log's event clock to reflect the newly added event,
    ///   and notify the configured event channel of a new event.
    /// - If the event's dependencies are not already satisfied, then hold the
    ///   event in a pending queue until they are. Once the dependencies become
    ///   satisfied, then it will be added and notified.
    pub fn add(&mut self, e: Event) {
        if self.get(&e.id).is_some() {
            return;
        }

        if self.has_dependencies_for(&e) {
            tracing::debug!(
                "Adding event {:?}; event clock={:?} my clock={:?}",
                e,
                e.clock,
                self.last_event_clock
            );

            self.do_add(e);
            self.check_pending();
        } else {
            tracing::debug!(
                "Deferring event {:?}; event clock={:?} my clock={:?}",
                e,
                e.clock,
                self.last_event_clock
            );
            self.pending.insert(e.id, e);
        }
    }

    /// Create an [`Event`] with the provided details. The resulting ID,
    /// timestamp and dependency clock will be generated based on the current
    /// state of the log.
    pub fn create(&self, target: impl Into<ObjectId>, details: impl Into<EventDetails>) -> Event {
        Event {
            id: self.id_gen.next(),
            timestamp: Utc::now().timestamp(),
            clock: self.last_event_clock.clone(),
            target: target.into(),
            details: details.into(),
        }
    }

    /// Remove events older than the provided timestamp
    pub fn prune_events_before(&mut self, threshold_timestamp: i64) {
        for (_server_id, server_events) in self.history.iter_mut() {
            server_events.retain(|_id, event| event.timestamp < threshold_timestamp);
        }
    }

    fn do_add(&mut self, e: Event) {
        let s = e.id.server();
        let id = e.id;

        self.history.entry(s).or_default();

        let server_map = self.history.get_mut(&s).unwrap();
        server_map.insert(e.id, e);

        self.last_event_clock.update_with_id(id);

        self.broadcast(self.history.get(&s).unwrap().get(&id).unwrap());
    }

    pub(crate) fn has_dependencies_for(&self, e: &Event) -> bool {
        // <= returns true iff, for every key in e.clock, we have the same
        // key and the value is the same or higher.
        //
        // Local clock being higher is OK because it means we've processed an
        // event that the originator hadn't at the time this was emitted. Local
        // clock having a key that the incoming one doesn't means that at the time
        // it was emitted, the originating server hadn't received any events from
        // that server.
        //
        // Local clock being lower, or missing, for a key that's in the remote one
        // means remote had already processed an event that we haven't.
        //
        // N.B. the incoming clock does *not* include the event's own ID - it's all
        // the events seen by the originating server *before* this event was emitted.
        e.clock <= self.last_event_clock
    }

    pub(crate) fn missing_ids_for(&self, clock: &EventClock) -> Vec<EventId> {
        let mut ret = Vec::new();

        for (_, v) in clock.0.iter() {
            if self.get(v).is_none() {
                ret.push(*v);
            }
        }
        ret
    }

    fn next_satisified_pending(&self) -> Option<EventId> {
        for (id, event) in &self.pending {
            if self.has_dependencies_for(event) {
                return Some(*id);
            }
        }
        None
    }

    fn check_pending(&mut self) {
        while let Some(id) = self.next_satisified_pending() {
            if let Some(event) = self.pending.remove(&id) {
                tracing::debug!("Adding satisfied deferred event {:?}", event);
                self.do_add(event);
            }
        }
    }

    fn broadcast(&self, event: &Event) {
        if let Some(send) = &self.event_sender {
            send.send(event.clone()).expect("failed to broadcast event");
        }
    }
}
