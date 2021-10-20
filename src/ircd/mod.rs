pub mod state {
    mod user;
    mod channel;
    mod message;

    pub use user::User;
    pub use channel::Channel;
    pub use channel::Membership;
    pub use message::Message;
}

pub mod wrapper {
    mod wrapper;
    mod user;
    mod channel;
    mod membership;

    pub use wrapper::ObjectWrapper;
    pub use wrapper::WrappedObjectIterator;
    pub use wrapper::WrapOption;
    pub use wrapper::WrapResult;
    pub use wrapper::WrapIterator;

    pub use user::User;
    pub use channel::Channel;
    pub use membership::Membership;
}

pub mod event {
    mod clock;
    mod event;
    mod eventlog;

    pub mod details;
    
    pub use clock::EventClock;

    pub use event::Event;
    pub use event::DetailType;
    
    pub use details::*;

    pub use eventlog::EventLog;
}

mod id;
pub use id::*;

mod errors;
pub use errors::LookupError;
pub use errors::LookupResult;

mod network;
pub use network::Network;

pub mod irc {
    pub mod server;
    pub use server::Server;

    pub mod connection;
//    use connection::Connection;
    use connection::ConnectionError;
//    use connection::ConnectionEvent;

    pub mod client;
    use client::ClientConnection;
    use client::PreClient;

    mod listener;
    use listener::ListenerCollection;

    mod client_message;
    use client_message::ClientMessage;

    mod command;

    mod messages;
    pub use messages::message;
    pub use messages::numeric;
}