use crate::ircd::*;
use crate::ircd::event::*;

pub trait DetailType : Into<EventDetails> {
    type Target: Into<ObjectId>;
}

#[derive(Clone,Debug)]
pub struct Event {
    pub id: EventId,
    pub timestamp: i64,
    pub clock: EventClock,
    pub target: ObjectId,

    pub details: EventDetails,
}


