/// Defines the various state objects making up a network
pub mod state {
    mod user;
    mod channel;
    mod message;
    mod server;

    pub use user::*;
    pub use channel::*;
    pub use message::*;
    pub use server::*;
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
    mod membership;
    mod server;
    mod message;

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
    pub use membership::Membership;
    pub use server::Server;
    pub use message::Message;
    pub use message::MessageTarget;
}

pub mod event;

pub mod id;
pub use id::*;

pub mod validated;
pub use validated::*;

pub mod errors;
pub use errors::*;

pub mod modes;
pub use modes::*;

mod network;
pub use network::*;

pub mod update;
pub use update::NetworkStateChange;
pub use update::NetworkUpdateReceiver;

mod option_change;
pub use option_change::OptionChange;

mod state_utils;

#[cfg(test)]
pub mod tests {
    pub mod fixtures;
    mod serialize;
    mod event_application;
}