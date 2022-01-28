use crate::id::*;
use crate::error::*;
use crate::Connection;

use serde::{Serialize,Deserialize};

#[derive(Clone,Debug,Serialize,Deserialize)]
pub enum ConnectionType
{
    Clear,
    Tls
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct TlsSettings
{
    pub cert_chain: Vec<Vec<u8>>,
    pub key: Vec<u8>,
}

pub enum ConnectionEventDetail
{
    NewConnection(Connection),
    Message(String),
    Error(ConnectionError),
}

pub struct ConnectionEvent
{
    pub source: ConnectionId,
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