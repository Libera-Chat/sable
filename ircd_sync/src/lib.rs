//! This crate contains the code required to manage the ircd's event log and
//! synchronise it with other servers.

mod eventlog;
mod network;
mod message;
mod config;

mod replicated_log;

pub use config::ConfigError;
pub use config::PeerConfig;
pub use config::NetworkConfig;
pub use config::NodeConfig;
pub use eventlog::EventLog;
pub use network::Network;
use message::Message;
use message::Request;

pub use replicated_log::ReplicatedEventLog;

#[cfg(test)]
mod tests;