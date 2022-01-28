use crate::id::*;
use crate::error::*;
use crate::protocols::*;

use serde::{Serialize,Deserialize};
use std::net::{
    IpAddr,
    SocketAddr
};
use std::sync::Arc;
use rustls::ServerConfig;

#[derive(Clone)]
pub enum InternalConnectionType
{
    Clear,
    Tls(Arc<ServerConfig>)
}

impl InternalConnectionType
{
    pub fn to_pub(&self) -> ConnectionType {
        match self {
            InternalConnectionType::Clear => ConnectionType::Clear,
            InternalConnectionType::Tls(_) => ConnectionType::Tls
        }
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub enum ConnectionControlDetail
{
    Send(String),
    Close,
}

#[derive(Debug,Serialize,Deserialize)]
pub enum ListenerControlDetail
{
    Add(SocketAddr,ConnectionType),
    Close,
}

#[derive(Debug,Serialize,Deserialize)]
pub enum ControlMessage
{
    Connection(ConnectionId, ConnectionControlDetail),
    Listener(ListenerId, ListenerControlDetail),
    LoadTlsSettings(TlsSettings),
    Shutdown
}

#[derive(Debug,Serialize,Deserialize)]
pub struct ConnectionData
{
    pub id: ConnectionId,
    pub endpoint: IpAddr,
    pub conn_type: ConnectionType,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct ListenerData
{
    pub id: ListenerId,
    pub addr: SocketAddr,
    pub conn_type: ConnectionType,
}

#[derive(Debug,Serialize,Deserialize)]
pub enum InternalConnectionEvent
{
    NewConnection(ConnectionData),
    Message(ConnectionId, String),
    ConnectionError(ConnectionId, ConnectionError),
    ConnectionClosed(ConnectionId),
    NewListener(ListenerData),
    ListenerError(ListenerId, ListenerError),
    ListenerClosed(ListenerId),
    BadTlsConfig,
    CommunicationError
}