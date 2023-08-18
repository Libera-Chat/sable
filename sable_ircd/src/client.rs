use messages::MessageSink;
use sable_network::prelude::*;
use super::*;
use client_listener::*;
use crate::movable::Movable;
use crate::throttled_queue::*;
use crate::capability::*;
use crate::utils::WrapOption;

use std::{
    net::IpAddr,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

use once_cell::sync::OnceCell;
use arc_swap::ArcSwapOption;
use serde::*;
use serde_with::serde_as;

/// A client protocol connection
pub struct ClientConnection
{
    /// The underlying network connection
    pub connection: Movable<Connection>,

    /// The user ID, if this connection has completed registration
    user_id: OnceCell<UserId>,

    /// The registration information received so far, if this connection has not
    /// yet completed registration
    pre_client: ArcSwapOption<PreClient>,

    // Pending lines to be processed
    receive_queue: Movable<ThrottledQueue<String>>,

    /// Capability flags
    pub capabilities: AtomicCapabilitySet
}

/// Serialised state of a [`ClientConnection`], for later resumption
#[derive(serde::Serialize,serde::Deserialize)]
pub(super) struct ClientConnectionState
{
    connection_data: ConnectionData,
    user_id: Option<UserId>,
    pre_client: Option<PreClient>,
    receive_queue: SavedThrottledQueue<String>,
    capabilities: ClientCapabilitySet,
}

/// Operations that, while ongoing, will block a client from registering
#[derive(Debug,Clone,Copy)]
#[repr(u32)]
pub enum ProgressFlag
{
    CapNegotiation,
    SaslAuthentication,
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
    #[serde_as (as = "WrapOption<SaslSessionId>")]
    pub sasl_session: OnceCell<SaslSessionId>,
    #[serde_as (as = "WrapOption<AccountId>")]
    pub sasl_account: OnceCell<AccountId>,

    progress_flags: AtomicU32,
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
            user_id: OnceCell::new(),
            pre_client: ArcSwapOption::new(Some(Arc::new(PreClient::new()))),
            receive_queue: Movable::new(ThrottledQueue::new(throttle_settings, 16)),
            capabilities: AtomicCapabilitySet::new(),
        }
    }

    /// Save the connection's data
    pub(crate) fn save(mut self) -> ClientConnectionState
    {
        ClientConnectionState {
            connection_data: self.connection.unwrap().save(),
            user_id: self.user_id.get().copied(),
            pre_client: self.pre_client.load_full()
                            .map(|a| Arc::try_unwrap(a)
                                        .unwrap_or_else(|_| panic!("Outstanding reference to preclient while upgrading?"))),
            receive_queue: self.receive_queue.unwrap().save(),
            capabilities: (&self.capabilities).into(),
        }
    }

    /// Restore a connection from a previously saved state, associating
    /// it with the provided [`ListenerCollection`]
    pub(crate) fn restore(state: ClientConnectionState, listener_collection: &ListenerCollection) -> Self
    {
        Self {
            connection: Movable::new(listener_collection.restore_connection(state.connection_data)),
            user_id: match state.user_id { Some(v) => OnceCell::with_value(v), None => OnceCell::new() },
            pre_client: ArcSwapOption::new(state.pre_client.map(Arc::new)),
            receive_queue: Movable::new(ThrottledQueue::restore_from(state.receive_queue)),
            capabilities: state.capabilities.into(),
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

    /// The TLS info for this connection, if any
    pub fn tls_info(&self) -> Option<&TlsInfo>
    {
        self.connection.tls_info.as_ref()
    }

    /// Close this connection with an error message
    pub fn error(&self, msg: &str)
    {
        self.connection.send(format!("ERROR :{}", msg));
        self.connection.close();
    }

    /// Return the associated user ID, if any
    pub fn user_id(&self) -> Option<UserId>
    {
        self.user_id.get().copied()
    }

    /// Return the associated pre-client data, if any
    pub fn pre_client(&self) -> Option<Arc<PreClient>>
    {
        self.pre_client.load_full()
    }

    /// Set the associated user ID
    pub fn set_user_id(&self, user_id: UserId)
    {
        let _ = self.user_id.set(user_id);
        self.pre_client.swap(None);
    }

    /// Notify that a new message has been received on this connection
    ///
    /// Returns `Ok(())` on success, `Err(message)` if the connection's receive queue is full
    pub fn new_message(&self, message: String) -> Result<(), String>
    {
        self.receive_queue.add(message)
    }

    /// Poll for messages that the throttle permits to be processed
    pub fn poll_messages(&self) -> impl Iterator<Item=String> + '_
    {
        self.receive_queue.iter()
    }
}

impl MessageSink for ClientConnection
{
    fn send(&self, msg: &messages::OutboundClientMessage)
    {
        if let Some(formatted) = msg.format_for_client_caps(&(&self.capabilities).into())
        {
            tracing::trace!("Sending to {:?}: {}", self.id(), formatted);
            self.connection.send(formatted)
        }
    }

    fn user_id(&self) -> Option<UserId>
    {
        ClientConnection::user_id(self)
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
            sasl_session: OnceCell::new(),
            sasl_account: OnceCell::new(),
            progress_flags: AtomicU32::new(0),
        }
    }

    /// Determine whether this connection is ready to complete registration
    pub fn can_register(&self) -> bool
    {
        let result = self.user.get().is_some()
                    && self.nick.get().is_some()
                    && self.hostname.get().is_some()
                    && self.progress_flags.load(Ordering::Relaxed) == 0;

        tracing::trace!(?self, result, "PreClient::can_register");
        result
    }

    /// Set a progress flag, indicating that the given operation is beginning
    pub fn start_progress(&self, flag: ProgressFlag)
    {
        self.progress_flags.fetch_or(flag as u32, Ordering::Relaxed);
    }

    /// Unset a progress flag, indicating that the given operation has completed
    ///
    /// Return true if the client is ready to register, i.e. if `can_register`
    /// would return true immediately after this call
    pub fn complete_progress(&self, flag: ProgressFlag) -> bool
    {
        let prev_flags = self.progress_flags.fetch_and(!(flag as u32), Ordering::Relaxed);

        let result = self.user.get().is_some()
                    && self.nick.get().is_some()
                    && self.hostname.get().is_some()
                    && prev_flags == flag as u32;

        tracing::trace!(?self, ?flag, result, "PreClient::complete_progress");
        result
    }
}