use irc_network::*;
use super::*;
use client_listener::*;

use std::cell::RefCell;
use std::net::IpAddr;

/// A client protocol connection
pub struct ClientConnection
{
    /// The underlying network connection
    pub connection: Connection,
    /// The user ID, if this connection has completed registration
    pub user_id: Option<UserId>,
    /// The registration information received so far, if this connection has not
    /// yet completed registration
    pub pre_client: Option<RefCell<PreClient>>
}

/// Information received from a client connection that has not yet completed registration
#[derive(serde::Serialize,serde::Deserialize)]
pub struct PreClient
{
    pub user: Option<Username>,
    pub nick: Option<Nickname>,
    pub realname: Option<String>,
    pub hostname: Option<Hostname>,
    pub cap_in_progress: bool,
}

impl ClientConnection
{
    /// Construct a `ClientConnection` from an underlying [`Connection`]
    pub fn new(conn: Connection) -> Self
    {
        Self {
            connection: conn,
            user_id: None,
            pre_client: Some(RefCell::new(PreClient::new()))
        }
    }

    /// The connection ID
    pub fn id(&self) -> ConnectionId
    {
        self.connection.id
    }

    /// The remote IP address from which this client connected
    pub fn remote_addr(&self) -> IpAddr
    {
        self.connection.remote_addr
    }

    /// Send a protocol message to this connection
    pub fn send(&self, msg: &dyn messages::MessageType)
    {
        self.connection.send(msg.to_string())
    }

    /// Close this connection with an error message
    pub fn error(&self, msg: &str)
    {
        // Can't do much if this does fail, because we're already tearing down the connection
        let _res = self.connection.send(format!("ERROR :{}", msg));
        let _res = self.connection.close();
    }
}

impl PreClient {
    /// Construct a `PreClient`
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self
    {
        Self {
            user: None,
            nick: None,
            realname: None,
            hostname: None,
            cap_in_progress: false
        }
    }

    /// Determine whether this connection is ready to complete registration
    pub fn can_register(&self) -> bool
    {
        self.user.is_some() && self.nick.is_some() && self.hostname.is_some() && !self.cap_in_progress
    }
}