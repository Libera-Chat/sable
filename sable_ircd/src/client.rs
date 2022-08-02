use messages::MessageSink;
use sable_network::prelude::*;
use super::*;
use client_listener::*;
use crate::movable::Movable;
use crate::throttled_queue::*;
use crate::capability::*;
use crate::utils::WrapOption;

use std::net::IpAddr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use once_cell::sync::OnceCell;
use serde::*;
use serde_with::serde_as;

/// A client protocol connection
pub struct ClientConnection
{
    /// The underlying network connection
    pub connection: Movable<Connection>,

    /// The user ID, if this connection has completed registration
    pub user_id: Option<UserId>,

    /// The registration information received so far, if this connection has not
    /// yet completed registration
    pub pre_client: Option<PreClient>,

    // Pending lines to be processed
    receive_queue: Movable<ThrottledQueue<String>>,

    /// Capability flags
    pub capabilities: ClientCapabilitySet
}

/// Serialised state of a [`ClientConnection`], for later resumption
#[derive(serde::Serialize,serde::Deserialize)]
pub(super) struct ClientConnectionState
{
    connection_data: ConnectionData,
    user_id: Option<UserId>,
    pre_client: Option<PreClient>,
    receive_queue: ThrottledQueue<String>,
    capabilities: ClientCapabilitySet,
}

/// Information received from a client connection that has not yet completed registration
#[serde_as]
#[derive(Debug,Serialize,Deserialize)]
pub struct PreClient
{
    #[serde_as (as = "WrapOption<Username>")]
    pub user: OnceCell<Username>,
    #[serde_as (as = "WrapOption<Nickname>")]
    pub nick: OnceCell<Nickname>,
    #[serde_as (as = "WrapOption<String>")]
    pub realname: OnceCell<String>,
    #[serde_as (as = "WrapOption<Hostname>")]
    pub hostname: OnceCell<Hostname>,
    pub cap_in_progress: AtomicBool,
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
            pre_client: Some(PreClient::new()),
            receive_queue: Movable::new(ThrottledQueue::new(throttle_settings, 16)),
            capabilities: ClientCapabilitySet::new(),
        }
    }

    /// Save the connection's data
    pub(crate) fn save(mut self) -> ClientConnectionState
    {
        ClientConnectionState {
            connection_data: self.connection.unwrap().save(),
            user_id: self.user_id,
            pre_client: self.pre_client.take(), // Take because we can't move out of ClientConnection which is Drop
            receive_queue: self.receive_queue.unwrap(),
            capabilities: self.capabilities,
        }
    }

    /// Restore a connection from a previously saved state, associating
    /// it with the provided [`ListenerCollection`]
    pub(crate) fn restore(state: ClientConnectionState, listener_collection: &ListenerCollection) -> Self
    {
        Self {
            connection: Movable::new(listener_collection.restore_connection(state.connection_data)),
            user_id: state.user_id,
            pre_client: state.pre_client,
            receive_queue: Movable::new(state.receive_queue),
            capabilities: state.capabilities,
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
    pub fn poll_messages(&mut self) -> impl Iterator<Item=String> + '_
    {
        self.receive_queue.iter_mut()
    }
}

impl MessageSink for ClientConnection
{
    fn send(&self, msg: &impl messages::MessageTypeFormat)
    {
        if let Some(formatted) = msg.format_for_client_caps(&self.capabilities)
        {
            self.connection.send(formatted)
        }
    }

    fn user_id(&self) -> Option<UserId>
    {
        self.user_id
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
            user: OnceCell::new(),
            nick: OnceCell::new(),
            realname: OnceCell::new(),
            hostname: OnceCell::new(),
            cap_in_progress: AtomicBool::new(false),
        }
    }

    /// Determine whether this connection is ready to complete registration
    pub fn can_register(&self) -> bool
    {
        let result = self.user.get().is_some()
                    && self.nick.get().is_some()
                    && self.hostname.get().is_some()
                    && !self.cap_in_progress.load(Ordering::Relaxed);

        tracing::trace!(?self, result, "PreClient::can_register");
        result
    }
}