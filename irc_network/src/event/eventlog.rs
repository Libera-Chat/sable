use crate::event::*;
use crate::*;

use std::{
    collections::{HashMap,BTreeMap},
};
use async_std::channel;

#[derive(Debug)]
pub struct EventLog {
    history: HashMap<ServerId, BTreeMap<LocalId, Event>>,
    pending: HashMap<EventId, Event>,
    id_gen: EventIdGenerator,
    event_sender: Option<channel::Sender<Event>>,
    last_event_clock: EventClock,
}

impl EventLog {
    pub fn new(idgen: EventIdGenerator, event_sender: Option<channel::Sender<Event>>) -> Self {
        Self{
            history: HashMap::new(),
            pending: HashMap::new(),
            id_gen: idgen,
            event_sender: event_sender,
            last_event_clock: EventClock::new(),
        }
    }

    pub fn get(&self, id: &EventId) -> Option<&Event> {
        self.history.get(&id.server()).and_then(|x| x.get(&id.local()))
    }

    pub fn add(&mut self, e: Event)
    {
        if self.has_dependencies_for(&e)
        {
            self.do_add(e);
            self.check_pending();
        }
        else
        {
            self.pending.insert(e.id, e);
        }
    }

    pub fn create(&self, target: impl Into<ObjectId>, details: impl Into<EventDetails>) -> Event {
        Event {
            id: self.id_gen.next(),
            timestamp: 0,
            clock: self.last_event_clock.clone(),
            target: target.into(),
            details: details.into()
        }
    }

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
        server_map.insert(e.id.local(), e);

        self.last_event_clock.update_with_id(id);

        self.broadcast(self.history.get(&s).unwrap().get(&id.local()).unwrap());
    }

    fn has_dependencies_for(&self, e: &Event) -> bool
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