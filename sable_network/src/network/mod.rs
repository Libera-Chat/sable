#![allow(clippy::module_inception)]
#![allow(clippy::new_without_default)]

/// Defines the various state objects making up a network
pub mod state {
    mod access_flag;
    mod account;
    mod audit_log;
    mod bans;
    mod channel;
    mod historic;
    mod message;
    mod server;
    mod services;
    mod user;

    pub use access_flag::*;
    pub use account::*;
    pub use audit_log::*;
    pub use bans::*;
    pub use channel::*;
    pub use historic::*;
    pub use message::*;
    pub use server::*;
    pub use services::*;
    pub use user::*;
}

/// Defines wrapper objects which provide accessor methods and basic
/// application logic for objects in [`state`]
pub mod wrapper;

pub mod ban;
pub mod config;
pub mod event;

pub mod errors;
pub use errors::*;

mod network;
pub use network::*;

pub mod update;
pub use update::NetworkStateChange;
pub use update::NetworkUpdateReceiver;

mod update_receiver;
pub use update_receiver::SavedUpdateReceiver;

mod option_change;
pub use option_change::OptionChange;

mod state_utils;

#[cfg(test)]
pub mod tests {
    mod event_application;
    pub mod fixtures;
    mod serialize;
}
