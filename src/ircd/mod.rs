pub mod state {
    mod user;
    mod channel;
    mod message;

    pub use user::*;
    pub use channel::*;
    pub use message::*;
}

pub mod wrapper {
    mod wrapper;
    mod user;
    mod user_mode;
    mod channel;
    mod channel_mode;
    mod membership;

    pub use wrapper::ObjectWrapper;
    pub use wrapper::WrappedObjectIterator;
    pub use wrapper::WrapOption;
    pub use wrapper::WrapResult;
    pub use wrapper::WrapIterator;

    pub use user::User;
    pub use user_mode::UserMode;
    pub use channel::Channel;
    pub use channel_mode::ChannelMode;
    pub use membership::Membership;
}

pub mod event;

mod id;
pub use id::*;

mod validated;
pub use validated::*;

mod errors;
pub use errors::*;

mod flags;
pub use flags::*;

mod network;
pub use network::Network;

pub mod irc {
    pub mod utils {
        mod channel_modes;
        pub use channel_modes::*;

        mod user_modes;
        pub use user_modes::*;

        mod channel_names;
        pub use channel_names::*;

        mod numeric_utils;
        pub use numeric_utils::*;
    }
    pub use utils::numeric_error;

    pub mod policy;

    mod dns;
    pub use dns::DnsClient;

    pub mod server;
    pub use server::Server;

    pub mod connection;

    pub mod client;
    use client::ClientConnection;
    use client::PreClient;

    mod listener;
    use listener::ListenerCollection;

    mod client_message;
    use client_message::ClientMessage;

    mod command;
    pub use command::CommandError;
    pub use command::CommandResult;

    mod messages;
    pub use messages::message;
    pub use messages::numeric;
}