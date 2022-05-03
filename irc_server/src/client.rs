use irc_network::*;
use super::*;
use client_listener::*;
use crate::movable::Movable;
use crate::throttled_queue::*;

use std::cell::RefCell;
use std::net::IpAddr;

/// A client protocol connection
pub struct ClientConnection
{
    /// The underlying network connection
    pub connection: Movable<Connection>,

    /// The user ID, if this connection has completed registration
    pub user_id: Option<UserId>,

    /// The registration information received so far, if this connection has not
    /// yet completed registration
    pub pre_client: Option<RefCell<PreClient>>,

    // Pending lines to be processed
    receive_queue: Movable<ThrottledQueue<String>>,
}

/// Serialised state of a [`ClientConnection`], for later resumption
#[derive(serde::Serialize,serde::Deserialize)]
pub(super) struct ClientConnectionState
{
    connection_data: ConnectionData,
    user_id: Option<UserId>,
    pre_client: Option<PreClient>,
    receive_queue: ThrottledQueue<String>,
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
        let throttle_settings = ThrottleSettings {
            num: 1,
            time: 1,
            burst: 4
        };

        Self {
            connection: Movable::new(conn),
            user_id: None,
            pre_client: Some(RefCell::new(PreClient::new())),
            receive_queue: Movable::new(ThrottledQueue::new(throttle_settings, 16)),
        }
    }

    /// Save the connection's data
    pub(crate) fn save(mut self) -> ClientConnectionState
    {
        ClientConnectionState {
            connection_data: self.connection.unwrap().save(),
            user_id: self.user_id,
            pre_client: self.pre_client.take().map(|c| c.into_inner()),
            receive_queue: self.receive_queue.unwrap(),
        }
    }

    /// Restore a connection from a previously saved state, associating
    /// it with the provided [`ListenerCollection`]
    pub(crate) fn restore(state: ClientConnectionState, listener_collection: &ListenerCollection) -> Self
    {
        Self {
            connection: Movable::new(listener_collection.restore_connection(state.connection_data)),
            user_id: state.user_id,
            pre_client: state.pre_client.map(RefCell::new),
            receive_queue: Movable::new(state.receive_queue),
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
        self.connection.send(format!("ERROR :{}", msg));
        self.connection.close();
    }

    /// Notify that a new message has been received on this connection
    ///
    /// Returns `Ok(())` on success, `Err(message)` if the connection's receive queue is full
    pub fn new_message(&mut self, message: String) -> Result<(), String>
    {
        self.receive_queue.add(message)
    }

    /// Poll for messages that the throttle permits to be processed
    pub fn poll_messages<'a>(&'a mut self) -> impl Iterator<Item=String> + 'a
    {
        self.receive_queue.iter_mut()
    }
}

impl Drop for ClientConnection
{
    fn drop(&mut self)
    {
        if let Some(conn) = self.connection.take()
        {
            conn.close();
        }
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