use crate::*;
use crate::event::*;
use serde::{Serialize,Deserialize};

pub trait DetailType : Into<EventDetails> {
    type Target: Into<ObjectId>;
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct Event {
    pub id: EventId,
    pub timestamp: u64,
    pub clock: EventClock,
    pub target: ObjectId,

    pub details: EventDetails,
}
