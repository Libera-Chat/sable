use crate::prelude::*;
use crate::network::event::*;
use serde::{Serialize,Deserialize};

/// Used by the `event_details!` and `dispatch_event!` macros to determine the
/// expected type of the target object for a given event detail type.
pub trait DetailType : Into<EventDetails> {
    type Target: Into<ObjectId>;
}

/// A network state event.
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct Event {
    /// The event ID. This identifies the server which originated this event,
    /// as well as an increasing local identifier.
    pub id: EventId,

    /// The Unix timestamp at which this event was created.
    pub timestamp: i64,

    /// Other event IDs on which this event depends.
    pub clock: EventClock,

    /// The target object being updated by this event.
    pub target: ObjectId,

    /// The actual type and content of the event.
    pub details: EventDetails,
}
