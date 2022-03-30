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
pub use eventlog::EventLogState;
pub use network::GossipNetwork;
pub use network::GossipNetworkState;
pub use network::NetworkError;
pub use message::Message;
pub use message::MessageDetail;
pub use message::Request;

pub use replicated_log::ReplicatedEventLog;
pub use replicated_log::ReplicatedEventLogState;

#[cfg(test)]
mod tests;