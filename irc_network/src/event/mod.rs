mod clock;
mod event;
mod eventlog;

pub mod details;

pub use clock::EventClock;

pub use event::Event;
pub use event::DetailType;

pub use details::*;

pub use eventlog::EventLog;

#[cfg(test)]
mod tests;