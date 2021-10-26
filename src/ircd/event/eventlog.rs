use crate::ircd::event::*;
use crate::ircd::*;
use async_broadcast;
use std::cell::RefCell;

use std::{
    collections::{HashMap,BTreeMap},
};

static BROADCAST_QUEUE_CAP: usize = 500;

#[derive(Debug)]
pub struct EventLog {
    history: HashMap<ServerId, BTreeMap<LocalId, Event>>,
    pending: HashMap<EventId, Event>,
    id_gen: EventIdGenerator,
    event_sender: Option<async_broadcast::Sender<Event>>,
    event_receiver: Option<RefCell<async_broadcast::Receiver<Event>>>,
    last_event_clock: EventClock,
}

impl EventLog {
    pub fn new(idgen: EventIdGenerator) -> Self {
        Self{
            history: HashMap::new(),
            pending: HashMap::new(),
            id_gen: idgen,
            event_sender: None,
            event_receiver: None,
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

    fn do_add(&mut self, e: Event)
    {
        let s = e.id.server();
        let id = e.id;

        if ! self.history.contains_key(&s) {
            self.history.insert(s, BTreeMap::new());
        }

        self.last_event_clock.update_with_clock(&e.clock);
        self.last_event_clock.update_with_id(id);

        let server_map = self.history.get_mut(&s).unwrap();
        server_map.insert(e.id.local(), e);

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

    pub fn create(&self, target: impl Into<ObjectId>, details: impl Into<EventDetails>) -> Event {
        Event {
            id: self.id_gen.next(),
            timestamp: 0,
            clock: self.last_event_clock.clone(),
            target: target.into(),
            details: details.into()
        }
    }

    pub fn attach(&mut self) -> async_broadcast::Receiver<Event>
    {
        if self.event_receiver.is_none()
        {
            let (send, recv) = async_broadcast::broadcast::<Event>(BROADCAST_QUEUE_CAP);

            self.event_sender = Some(send);
            self.event_receiver = Some(RefCell::new(recv));
        }
        let rc = self.event_receiver.as_ref().unwrap();
        let receiver = rc.borrow();
        receiver.clone()
    }

    fn broadcast(&self, event: &Event)
    {
        if let Some(send) = &self.event_sender
        {
            send.try_broadcast(event.clone()).expect("failed to broadcast event");
        }
        // We need to keep a receiver around to be able to clone it, but need to keep it empty
        // so the queue doesn't back up
        if let Some(recv) = &self.event_receiver
        {
            while let Ok(_event) = recv.borrow_mut().try_recv()
            { }
        }
    }
}