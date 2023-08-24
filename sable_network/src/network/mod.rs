#![allow(clippy::module_inception)]
#![allow(clippy::new_without_default)]

/// Defines the various state objects making up a network
pub mod state {
    mod access_flag;
    mod account;
    mod audit_log;
    mod bans;
    mod channel;
    mod message;
    mod server;
    mod services;
    mod user;

    pub use access_flag::*;
    pub use account::*;
    pub use audit_log::*;
    pub use bans::*;
    pub use channel::*;
    pub use message::*;
    pub use server::*;
    pub use services::*;
    pub use user::*;
}

/// Defines wrapper objects which provide accessor methods and basic
/// application logic for objects in [`state`]
pub mod wrapper {

    mod account;
    mod bans;
    mod channel;
    mod channel_access;
    mod channel_invite;
    mod channel_mode;
    mod channel_registration;
    mod channel_role;
    mod channel_topic;
    mod list_mode;
    mod list_mode_entry;
    mod membership;
    mod message;
    mod nick_binding;
    mod nick_registration;
    mod server;
    mod services;
    mod user;
    mod user_mode;
    mod wrapper;

    pub use wrapper::ObjectWrapper;
    pub use wrapper::WrapIterator;
    pub use wrapper::WrapOption;
    pub use wrapper::WrapResult;
    pub use wrapper::WrappedObjectIterator;

    pub use account::*;
    pub use bans::*;
    pub use channel::Channel;
    pub use channel_access::*;
    pub use channel_invite::ChannelInvite;
    pub use channel_mode::ChannelMode;
    pub use channel_registration::*;
    pub use channel_role::*;
    pub use channel_topic::ChannelTopic;
    pub use list_mode::ListMode;
    pub use list_mode_entry::ListModeEntry;
    pub use membership::Membership;
    pub use message::Message;
    pub use message::MessageTarget;
    pub use nick_binding::NickBinding;
    pub use nick_registration::*;
    pub use server::Server;
    pub use services::*;
    pub use user::User;
    pub use user_mode::UserMode;
}

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
