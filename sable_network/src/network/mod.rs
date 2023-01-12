#![allow(clippy::module_inception)]
#![allow(clippy::new_without_default)]

/// Defines the various state objects making up a network
pub mod state {
    mod user;
    mod channel;
    mod message;
    mod server;
    mod bans;
    mod audit_log;
    mod account;
    mod access_flag;
    mod services;

    pub use user::*;
    pub use channel::*;
    pub use message::*;
    pub use server::*;
    pub use bans::*;
    pub use audit_log::*;
    pub use account::*;
    pub use access_flag::*;
    pub use services::*;
}

/// Defines wrapper objects which provide accessor methods and basic
/// application logic for objects in [`state`]
pub mod wrapper {

    mod wrapper;
    mod nick_binding;
    mod user;
    mod user_mode;
    mod channel;
    mod channel_mode;
    mod list_mode;
    mod list_mode_entry;
    mod channel_topic;
    mod channel_invite;
    mod membership;
    mod server;
    mod message;
    mod bans;
    mod account;
    mod nick_registration;
    mod channel_registration;
    mod channel_access;
    mod channel_role;
    mod services;

    pub use wrapper::ObjectWrapper;
    pub use wrapper::WrappedObjectIterator;
    pub use wrapper::WrapOption;
    pub use wrapper::WrapResult;
    pub use wrapper::WrapIterator;

    pub use nick_binding::NickBinding;
    pub use user::User;
    pub use user_mode::UserMode;
    pub use channel::Channel;
    pub use channel_mode::ChannelMode;
    pub use list_mode::ListMode;
    pub use list_mode_entry::ListModeEntry;
    pub use channel_topic::ChannelTopic;
    pub use channel_invite::ChannelInvite;
    pub use membership::Membership;
    pub use server::Server;
    pub use message::Message;
    pub use message::MessageTarget;
    pub use bans::*;
    pub use account::*;
    pub use nick_registration::*;
    pub use channel_registration::*;
    pub use channel_access::*;
    pub use channel_role::*;
    pub use services::*;
}

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
    pub mod fixtures;
    mod serialize;
    mod event_application;
}