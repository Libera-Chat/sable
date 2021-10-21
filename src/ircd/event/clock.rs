use std::collections::HashMap;
use std::hash::Hash;
use std::cmp::Ordering;
use serde::{Serialize,Deserialize};

use crate::ircd::{EventId,ServerId,LocalId};

#[derive(Clone,Eq,PartialEq,Debug,Serialize,Deserialize)]
pub struct EventClock (HashMap<ServerId, LocalId>);

impl EventClock {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn update_with(&mut self, s: ServerId, i: LocalId) {
        // If we already have a value for this ServerId, and the value we already have is greater
        // than the provided one, do nothing. Else update.
        if ! matches!(self.0.get(&s), Some(current) if *current > i) {
            self.0.insert(s, i);
        }
    }

    pub fn update_with_id(&mut self, id: EventId) {
        self.update_with(id.server(), id.local());
    }

    pub fn update_with_clock(&mut self, other: &EventClock) {
        for (s, l) in &other.0 {
            self.update_with(*s, *l);
        }
    }
}

fn keys_match<T: Eq + Hash, U, V>(
    map1: &HashMap<T, U>, 
    map2: &HashMap<T, V>,
) -> bool {
    map1.len() == map2.len() && map1.keys().all(|k| map2.contains_key(k))
}

impl PartialOrd for EventClock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if ! keys_match(&self.0, &other.0) {
            return None;
        }
        let mut some_less = false;
        let mut some_more = false;

        for (key, mine) in self.0.iter() {
            match other.0.get(key) {
                Some(theirs) => {
                    if mine < theirs { some_less = true; }
                    if mine > theirs { some_more = true; }
                },
                None => { }
            }
        }
        if some_less && some_more { return None; }
        if some_less { return Some(Ordering::Less); }
        if some_more { return Some(Ordering::Greater); }
        return Some(Ordering::Equal);

    }
}