use crate::id::*;
use crate::error::*;
use crate::Connection;

use std::net::IpAddr;
use serde::{Serialize,Deserialize};

/// Information about a client connection's TLS status
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct TlsInfo
{
    pub fingerprint: Option<String>,
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub enum ConnectionType
{
    Clear,
    Tls
}

/// The saved state of a [`Connection`]
#[derive(Debug,Serialize,Deserialize)]
pub struct ConnectionData
{
    pub(crate) id: ConnectionId,
    pub(crate) remote_addr: IpAddr,
    pub(crate) tls_info: Option<TlsInfo>,
}

/// The certificate chain and private key required to create a TLS listener.
///
/// Should be provided to
/// [`ListenerCollection::load_tls_certificates`](crate::ListenerCollection::load_tls_certificates).
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct TlsSettings
{
    pub cert_chain: Vec<Vec<u8>>,
    pub key: Vec<u8>,
}

/// Possible types of event that might occur on a given connection.
pub enum ConnectionEventDetail
{
    /// A new connection was accepted
    NewConnection(Connection),
    /// A new line was received
    Message(String),
    /// An error occurred
    Error(ConnectionError),
}

/// An event be notified via a `ListenerCollection`'s event channel.
pub struct ConnectionEvent
{
    /// The connection ID to which this event relates
    pub source: ConnectionId,
    /// The type of event and its content
    pub detail: ConnectionEventDetail
}

impl ConnectionEvent
{
    pub(crate) fn message(id: ConnectionId, message: String) -> Self
    {
        Self { source: id, detail: ConnectionEventDetail::Message(message) }
    }

    pub(crate) fn error(id: ConnectionId, error: ConnectionError) -> Self
    {
        Self { source: id, detail: ConnectionEventDetail::Error(error) }
    }

    pub(crate) fn new(id: ConnectionId, conn: Connection) -> Self
    {
        Self { source: id, detail: ConnectionEventDetail::NewConnection(conn) }
    }
}