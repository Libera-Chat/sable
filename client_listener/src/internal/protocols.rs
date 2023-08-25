use crate::error::*;
use crate::id::*;
use crate::protocols::*;

use rustls::ServerConfig;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
pub enum InternalConnectionType {
    Clear,
    Tls(Arc<ServerConfig>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConnectionControlDetail {
    Send(String),
    Close,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ListenerControlDetail {
    Add(SocketAddr, ConnectionType),
    Close,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ControlMessage {
    Connection(ConnectionId, ConnectionControlDetail),
    Listener(ListenerId, ListenerControlDetail),
    LoadTlsSettings(TlsSettings),
    Shutdown,
    SaveForUpgrade,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListenerData {
    pub id: ListenerId,
    pub addr: SocketAddr,
    pub conn_type: ConnectionType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InternalConnectionEvent {
    NewConnection(ConnectionData),
    Message(ConnectionId, String),
    ConnectionError(ConnectionId, ConnectionError),
    ConnectionClosed(ConnectionId),
    NewListener(ListenerData),
    ListenerError(ListenerId, ListenerError),
    ListenerClosed(ListenerId),
    BadTlsConfig,
    CommunicationError,
}

pub(crate) enum InternalConnectionEventType {
    New(super::InternalConnection),
    Event(InternalConnectionEvent),
}
