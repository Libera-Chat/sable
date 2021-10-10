use crate::ircd::event::*;
use crate::ircd::*;

use std::{
    collections::{HashMap,BTreeMap},
    sync::Arc,
};

pub struct EventOffset(usize);

#[derive(Debug)]
pub struct EventLog {
    history: HashMap<ServerId, BTreeMap<LocalId, Event>>,

    log: Vec<Id>,

    id_gen: Arc<IdGenerator>
}

impl EventLog {
    pub fn new(idgen: Arc<IdGenerator>) -> Self {
        Self{
            history: HashMap::new(),
            log: Vec::new(),
            id_gen: idgen,
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

    pub fn create(&self, target: Id, details: EventDetails) -> Event {
        Event {
            id: self.id_gen.next(),
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