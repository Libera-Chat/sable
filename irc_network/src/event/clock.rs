use std::collections::HashMap;
use std::hash::Hash;
use std::cmp::Ordering;
use serde::{Serialize,Deserialize};

use crate::{EventId,ServerId};

/// A vector clock defining, for each server, the most recent event from that
/// server which has been processed.
/// 
/// This is primarily used in event dependency resolution - each server's event
/// log maintains a clock of its current state, and each event contains a clock
/// which reflects its origin server's clock when it was emitted. By comparing
/// these, a server receiving a remote event can determine whether the incoming
/// event can be applied immediately, or whether missing dependencies need to
/// be requested.
#[derive(Clone,Eq,PartialEq,Debug,Serialize,Deserialize)]
pub struct EventClock (pub HashMap<ServerId, EventId>);

impl EventClock {
    /// Create a new empty clock.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Get the most recent event contained in this clock for the given
    /// originating server.
    pub fn get(&self, server: ServerId) -> Option<EventId>
    {
        self.0.get(&server).map(|x| *x)
    }

    /// Update this clock to reflect receipt of a given event ID.
    pub fn update_with_id(&mut self, id: EventId) {
        let s = id.server();
        // If we already have a value for this ServerId, and the value we already have is greater
        // than the provided one, do nothing. Else update.
        if let Some(current) = self.0.get(&s)
        {
            if *current > id {
                return;
            }
        }
        self.0.insert(s, id);
    }

    pub fn update_with_clock(&mut self, other: &EventClock) {
        for (_, id) in &other.0 {
            self.update_with_id(*id);
        }
    }

    /// Determine whether the given event ID has been processed.
    /// 
    /// Returns true if the server portion of the provided event ID is present
    /// in the clock and the associated event ID is lexicographically greater
    /// than or equal to that provided.
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
            if self.0.iter().all(|(k,v)| v <= &other.0[&k])
            {
                return Some(Ordering::Less);
            } else {
                return None;
            }
        }
        else if keys_ge
        {
            if other.0.iter().all(|(k,v)| v <= &self.0[&k])
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