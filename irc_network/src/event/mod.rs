//! Defines the `Event` type and associated objects.

mod clock;
mod event;

pub mod details;

pub use clock::EventClock;

pub use event::Event;
pub use event::DetailType;

pub use details::*;

#[cfg(test)]
mod tests;