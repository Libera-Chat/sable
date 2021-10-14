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
    id_gen: EventIdGenerator,
    event_sender: Option<async_broadcast::Sender<Event>>,
    event_receiver: Option<RefCell<async_broadcast::Receiver<Event>>>,
}

impl EventLog {
    pub fn new(idgen: EventIdGenerator) -> Self {
        Self{
            history: HashMap::new(),
            id_gen: idgen,
            event_sender: None,
            event_receiver: None
        }
    }

    pub fn get(&self, id: &EventId) -> Option<&Event> {
        self.history.get(&id.server()).and_then(|x| x.get(&id.local()))
    }

    pub fn add(&mut self, e: Event) {
        let s = e.id.server();
        let id = e.id;
        if ! self.history.contains_key(&s) {
            self.history.insert(s, BTreeMap::new());
        }
        self.history.get_mut(&s).unwrap().insert(e.id.local(), e);
        self.broadcast(self.history.get(&s).unwrap().get(&id.local()).unwrap());
    }

    pub fn create(&self, target: impl Into<ObjectId>, details: impl Into<EventDetails>) -> Event {
        Event {
            id: self.id_gen.next(),
            timestamp: 0,
            clock: EventClock::new(),
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