//! Contains the event log

use irc_network::event::*;
use irc_network::*;

use std::{
    collections::{HashMap,BTreeMap},
    ops::Bound::*
};
use tokio::sync::mpsc::{
    Sender,
};
use log;
use chrono::prelude::*;

/// An event log.
/// 
/// The event log contains the history of all events that have been seen and
/// processed by the server.
#[derive(Debug)]
pub struct EventLog {
    history: BTreeMap<ServerId, BTreeMap<EventId, Event>>,
    pending: HashMap<EventId, Event>,
    id_gen: EventIdGenerator,
    event_sender: Option<Sender<Event>>,
    last_event_clock: EventClock,
}

impl EventLog {
    /// Construct a new `EventLog`. `event_sender`, if `Some`, will receive
    /// notification of all new events as they are added to the log and become
    /// ready for processing by the [`Network`](irc_network::Network).
    pub fn new(idgen: EventIdGenerator, event_sender: Option<Sender<Event>>) -> Self {
        Self{
            history: BTreeMap::new(),
            pending: HashMap::new(),
            id_gen: idgen,
            event_sender: event_sender,
            last_event_clock: EventClock::new(),
        }
    }

    /// Return a single event.
    pub fn get(&self, id: &EventId) -> Option<&Event> {
        self.history.get(&id.server()).and_then(|x| x.get(&id))
    }

    /// Iterate over all events in the current log which do not precede the
    /// given event clock.
    /// 
    /// When provided with the current event clock from a remote server, this
    /// will produce a list of all events that this server knows about and the
    /// remote one does not.
    pub fn get_since(&self, id: EventClock) -> impl Iterator<Item=&Event>
    {
        self.history.iter().flat_map(move |(server,list)| {
            let begin = if let Some(got) = id.get(*server) {
                Excluded(got)
            } else {
                Unbounded
            };
            list.range((begin, Unbounded)).map(|(_,v)| v)
        })
    }

    /// The log's current event clock
    pub fn clock(&self) -> &EventClock {
        &self.last_event_clock
    }

    /// Set the clock for this log.
    /// 
    /// This should only be used when importing a serialized 
    /// [Network](irc_network::Network) state, to sync the event log's view of
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
    pub fn add(&mut self, e: Event)
    {
        if self.get(&e.id).is_some()
        {
            return;
        }

        if self.has_dependencies_for(&e)
        {
            self.do_add(e);
            self.check_pending();
        }
        else
        {
            log::info!("Deferring event {:?}; event clock={:?} my clock={:?}", e.id, e.clock, self.last_event_clock);
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
            details: details.into()
        }
    }

    /// Update the epoch ID used when creating new event IDs
    pub fn set_epoch(&mut self, new_epoch: EpochId)
    {
        self.id_gen.set_epoch(new_epoch);
        self.id_gen.update_to(1);
    }

    fn do_add(&mut self, e: Event)
    {
        let s = e.id.server();
        let id = e.id;

        if ! self.history.contains_key(&s) {
            self.history.insert(s, BTreeMap::new());
        }

        let server_map = self.history.get_mut(&s).unwrap();
        server_map.insert(e.id, e);

        self.last_event_clock.update_with_id(id);

        self.broadcast(self.history.get(&s).unwrap().get(&id).unwrap());
    }

    pub(crate) fn has_dependencies_for(&self, e: &Event) -> bool
    {
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

    pub(crate) fn missing_ids_for(&self, clock: &EventClock) -> Vec<EventId>
    {
        let mut ret = Vec::new();

        for (_,v) in clock.0.iter()
        {
            if self.get(v).is_none()
            {
                ret.push(v.clone());
            }
        }
        ret
    }

    fn next_satisified_pending(&self) -> Option<EventId>
    {
        for (id, event) in &self.pending
        {
            if self.has_dependencies_for(&event)
            {
                return Some(*id);
            }
        }
        None
    }

    fn check_pending(&mut self)
    {
        while let Some(id) = self.next_satisified_pending()
        {
            if let Some(event) = self.pending.remove(&id)
            {
                log::info!("Adding satisfied deferred event {:?}", event);
                self.do_add(event);
            }
        }
    }

    fn broadcast(&self, event: &Event)
    {
        if let Some(send) = &self.event_sender
        {
            send.try_send(event.clone()).expect("failed to broadcast event");
        }
    }
}