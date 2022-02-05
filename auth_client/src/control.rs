use client_listener::ConnectionId;
use std::net::IpAddr;

#[derive(Debug,serde::Serialize,serde::Deserialize)]
pub enum ControlMessage
{
    Shutdown,
    StartDnsLookup(ConnectionId, IpAddr)
}