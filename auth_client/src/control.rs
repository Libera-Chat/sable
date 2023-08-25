use client_listener::ConnectionId;
use std::net::IpAddr;

/// A message sent from the consumer process to the worker
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ControlMessage {
    /// Shut down the worker process
    Shutdown,

    /// Start a reverse DNS lookup for the given connection ID and IP address
    StartDnsLookup(ConnectionId, IpAddr),
}
