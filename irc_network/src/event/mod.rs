mod clock;
mod event;

pub mod details;

pub use clock::EventClock;

pub use event::Event;
pub use event::DetailType;

pub use details::*;

#[derive(Debug)]
pub enum EventLogUpdate
{
    NewEvent(crate::ObjectId, EventDetails),
    EpochUpdate(crate::EpochId)
}

#[cfg(test)]
mod tests;