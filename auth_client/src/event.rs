use client_listener::ConnectionId;
use irc_network::Hostname;

/// The result of a DNS lookup
#[derive(Debug,serde::Serialize,serde::Deserialize)]
pub struct DnsResult
{
    /// The connection ID provided when initiating the request
    pub conn: ConnectionId,
    /// The hostname, or None if no suitable name was found
    pub hostname: Option<Hostname>,
}

/// A notification that something has completed
#[derive(Debug,serde::Serialize,serde::Deserialize)]
pub enum AuthEvent
{
    DnsResult(DnsResult)
}