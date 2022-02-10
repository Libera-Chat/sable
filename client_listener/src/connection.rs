use crate::*;
use internal::*;

use std::net::IpAddr;

use tokio::sync::mpsc::UnboundedSender;

/// A connection being managed by the worker process.
pub struct Connection
{
    pub id: ConnectionId,
    pub conn_type: ConnectionType,
    pub remote_addr: IpAddr,
    send_channel: UnboundedSender<ControlMessage>
}

impl Connection
{
    pub(crate) fn new(id: ConnectionId, conn_type: ConnectionType, remote_addr: IpAddr, send_channel: UnboundedSender<ControlMessage>) -> Self
    {
        Self {
            id: id,
            conn_type: conn_type,
            remote_addr: remote_addr,
            send_channel: send_channel,
        }
    }

    /// Is this a TLS connection?
    pub fn is_tls(&self) -> bool
    {
        match self.conn_type {
            ConnectionType::Clear => false,
            ConnectionType::Tls => true
        }
    }

    fn send_control(&self, msg: ConnectionControlDetail)
    {
        if let Err(e) = self.send_channel.send(ControlMessage::Connection(self.id, msg))
        {
            tracing::error!("Error sending connection control message: {}", e);
        }
    }

    /// Close the connection.
    pub fn close(&self)
    {
        self.send_control(ConnectionControlDetail::Close);
    }

    /// Send the provided text to the connection
    pub fn send(&self, msg: String)
    {
        self.send_control(ConnectionControlDetail::Send(msg));
    }

    /// Save the connection state for later restoration.
    ///
    /// See [`ListenerCollection::restore_connection`] for the counterpart to restore
    /// the saved state to a new `Connection`.
    pub fn save(self) -> ConnectionData
    {
        ConnectionData {
            id: self.id,
            remote_addr: self.remote_addr,
            conn_type: self.conn_type
        }
    }
}