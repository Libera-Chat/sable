//! The client protocol listener process, and interface thereto.
//!
//! This package contains the `client_listener` executable, which maintains all
//! persistent TCP connections for the IRC client protocol. Keeping these in a
//! dedicated process allows the main server process to restart for upgrades
//! without interrupting client connections.
//!
//! Listeners are managed using the [`ListenerCollection`], which spawns both
//! the child worker process and an asynchronous task to manage communications.
//! Once a listener has been created, new connections and any events on existing
//! connections will be sent via the provided event channel.


pub mod id;
pub use id::*;

pub mod error;
pub use error::*;

mod protocols;
pub use protocols::*;

mod connection;
pub use connection::*;

mod listener_collection;
pub use listener_collection::*;

mod listener_process;
pub use listener_process::*;

mod internal
{
    pub mod protocols;
    pub use protocols::*;
    pub mod connection;
    pub(crate) use connection::*;
    pub mod connection_task;
    pub(crate) use connection_task::*;
    pub mod listener;
    pub(crate) use listener::*;
}

pub use internal::InternalConnectionEvent;
pub use internal::ControlMessage;