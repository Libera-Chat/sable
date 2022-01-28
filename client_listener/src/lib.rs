//! The client protocol listener process, and interface thereto.
//!
//! This crate contains the `client_listener` executable, which maintains all
//! persistent TCP connections for the IRC client protocol. Keeping these in a
//! dedicated process allows the main server process to restart for upgrades
//! without interrupting client connections.


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
    pub use connection::*;
    pub mod connection_task;
    pub use connection_task::*;
    pub mod listener;
    pub use listener::*;
}

pub use internal::InternalConnectionEvent;
pub use internal::ControlMessage;