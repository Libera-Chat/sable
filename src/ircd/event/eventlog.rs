use crate::ircd::event::{Event,EventClock,EventDetails};
use crate::ircd::{ServerId,LocalId,Id};

use std::collections::{HashMap,BTreeMap};

pub struct EventOffset(usize);

pub struct EventLog {
    history: HashMap<ServerId, BTreeMap<LocalId, Event>>,

    log: Vec<Id>,

    my_serverid: ServerId,
    latest_localid: LocalId,
}

impl EventLog {
    pub fn new(server_id: ServerId) -> Self {
        Self{
            history: HashMap::new(),
            log: Vec::new(),
            my_serverid: server_id,
            latest_localid: 0
        }
    }

    pub fn get(&self, id: &Id) -> Option<&Event> {
        self.history.get(id.server()).and_then(|x| x.get(id.local()))
    }

    pub fn add(&mut self, e: Event) {
        let s = e.id.server();
        let id = e.id;
        if ! self.history.contains_key(s) {
            self.history.insert(*s, BTreeMap::new());
        }
        self.history.get_mut(s).unwrap().insert(*e.id.local(), e);
        self.log.push(id);
    }

    pub fn create(&mut self, target: Id, details: EventDetails) -> Event {
        self.latest_localid += 1;
        Event {
            id: Id::new(self.my_serverid, self.latest_localid),
            timestamp: 0,
            clock: EventClock::new(),
            target: target,
            details: details
        }
    }

    pub fn next_for(&self, offset: &mut EventOffset) -> Option<&Event> {
        let off = offset.0;

        if off >= self.log.len() {
            None
        } else {
            offset.0 += 1;
            self.get(&self.log[off])
        }
    }

    pub fn get_offset(&self) -> EventOffset {
        EventOffset(self.log.len())
    }
}