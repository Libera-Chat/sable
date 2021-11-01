use std::collections::HashMap;
use std::hash::Hash;
use std::cmp::Ordering;
use serde::{Serialize,Deserialize};

use crate::{EventId,ServerId};

#[derive(Clone,Eq,PartialEq,Debug,Serialize,Deserialize)]
pub struct EventClock (HashMap<ServerId, EventId>);

impl EventClock {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn update_with_id(&mut self, id: EventId) {
        let s = id.server();
        // If we already have a value for this ServerId, and the value we already have is greater
        // than the provided one, do nothing. Else update.
        if ! matches!(self.0.get(&s), Some(current) if *current > id) {
            self.0.insert(s, id);
        }
    }

    pub fn update_with_clock(&mut self, other: &EventClock) {
        for (_, id) in &other.0 {
            self.update_with_id(*id);
        }
    }

    pub fn contains(&self, id: EventId) -> bool
    {
        if let Some(local) = self.0.get(&id.server())
        {
            local >= &id
        }
        else
        {
            false
        }
    }
}

fn keys_subset<T: Eq + Hash, U, V>(
    map1: &HashMap<T, U>, 
    map2: &HashMap<T, V>,
) -> bool {
    map1.keys().all(|k| map2.contains_key(k))
}

impl PartialOrd for EventClock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let keys_le = keys_subset(&self.0, &other.0);
        let keys_ge = keys_subset(&other.0, &self.0);

        if keys_le && keys_ge
        {
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
        else if keys_le
        {
            if self.0.iter().all(|(k,v)| v < &other.0[&k])
            {
                return Some(Ordering::Less);
            } else {
                return None;
            }
        }
        else if keys_ge
        {
            if other.0.iter().all(|(k,v)| v < &self.0[&k])
            {
                return Some(Ordering::Greater);
            } else {
                return None;
            }
       } else {
           return None;
       }
    }
}