use super::*;
use crate::capability::*;
use crate::movable::Movable;
use crate::throttled_queue::*;
use crate::utils::WrapOption;
use client_listener::*;
use messages::MessageSink;
use sable_network::prelude::*;

use std::{
    net::IpAddr,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

use arc_swap::ArcSwapOption;
use serde::*;
use serde_with::serde_as;
use std::sync::OnceLock;
use tokio::time::Instant;

/// A client protocol connection
#[derive(Debug)]
pub struct ClientConnection {
    /// The underlying network connection
    pub connection: Movable<Connection>,

    /// The user and user connection IDs, if this connection has completed registration
    user: OnceLock<(UserId, UserConnectionId)>,

    /// The registration information received so far, if this connection has not
    /// yet completed registration
    pre_client: ArcSwapOption<PreClient>,

    // Pending lines to be processed
    receive_queue: Movable<ThrottledQueue<String>>,

    /// Capability flags
    pub capabilities: AtomicCapabilitySet,
}

/// Serialised state of a [`ClientConnection`], for later resumption
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct ClientConnectionState {
    connection_data: ConnectionData,
    user: Option<(UserId, UserConnectionId)>,
    pre_client: Option<PreClient>,
    receive_queue: SavedThrottledQueue<String>,
    capabilities: ClientCapabilitySet,
}

/// Operations that, while ongoing, will block a client from registering
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum ProgressFlag {
    CapNegotiation = 0x1,
    SaslAuthentication = 0x2,
}

/// Information received from a client connection that has not yet completed registration
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct PreClient {
    #[serde(skip, default = "Instant::now")]
    /// Reset on deserialization, to avoid timing out clients if the server took a long
    /// time to restart.
    pub connected_at: Instant,

    #[serde_as(as = "WrapOption<UserId>")]
    pub attach_user_id: OnceLock<UserId>,
    #[serde_as(as = "WrapOption<Username>")]
    pub user: OnceLock<Username>,
    #[serde_as(as = "WrapOption<(String,String)>")]
    pub extra_user_params: OnceLock<(String, String)>,
    #[serde_as(as = "WrapOption<Nickname>")]
    pub nick: OnceLock<Nickname>,
    #[serde_as(as = "WrapOption<Realname>")]
    pub realname: OnceLock<Realname>,
    #[serde_as(as = "WrapOption<Hostname>")]
    pub hostname: OnceLock<Hostname>,
    #[serde_as(as = "WrapOption<SaslSessionId>")]
    pub sasl_session: OnceLock<SaslSessionId>,
    #[serde_as(as = "WrapOption<AccountId>")]
    pub sasl_account: OnceLock<AccountId>,

    progress_flags: AtomicU32,
}

impl ClientConnection {
    /// Construct a `ClientConnection` from an underlying [`Connection`]
    pub fn new(conn: Connection) -> Self {
        let throttle_settings = ThrottleSettings {
            num: 1,
            time: 1,
            burst: 4,
        };

        Self {
            connection: Movable::new(conn),
            user: OnceLock::new(),
            pre_client: ArcSwapOption::new(Some(Arc::new(PreClient::new()))),
            receive_queue: Movable::new(ThrottledQueue::new(throttle_settings, 16)),
            capabilities: AtomicCapabilitySet::new(),
        }
    }

    /// Save the connection's data
    pub(crate) fn save(mut self) -> ClientConnectionState {
        ClientConnectionState {
            connection_data: self.connection.unwrap().save(),
            user: self.user.get().copied(),
            pre_client: self.pre_client.load_full().map(|a| {
                Arc::try_unwrap(a).unwrap_or_else(|_| {
                    panic!("Outstanding reference to preclient while upgrading?")
                })
            }),
            receive_queue: self.receive_queue.unwrap().save(),
            capabilities: (&self.capabilities).into(),
        }
    }

    /// Restore a connection from a previously saved state, associating
    /// it with the provided [`ListenerCollection`]
    pub(crate) fn restore(
        state: ClientConnectionState,
        listener_collection: &ListenerCollection,
    ) -> Self {
        Self {
            connection: Movable::new(listener_collection.restore_connection(state.connection_data)),
            user: match state.user {
                Some(v) => OnceLock::from(v),
                None => OnceLock::new(),
            },
            pre_client: ArcSwapOption::new(state.pre_client.map(Arc::new)),
            receive_queue: Movable::new(ThrottledQueue::restore_from(state.receive_queue)),
            capabilities: state.capabilities.into(),
        }
    }

    /// The connection ID
    pub fn id(&self) -> ConnectionId {
        self.connection.id
    }

    /// The remote IP address from which this client connected
    pub fn remote_addr(&self) -> IpAddr {
        self.connection.remote_addr
    }

    /// The TLS info for this connection, if any
    pub fn tls_info(&self) -> Option<&TlsInfo> {
        self.connection.tls_info.as_ref()
    }

    /// Close this connection with an error message
    pub fn error(&self, msg: &str) {
        self.connection.send(format!("ERROR :{msg}"));
        self.connection.close();
    }

    /// Return the associated user ID, if any
    pub fn user_id(&self) -> Option<UserId> {
        self.user.get().map(|v| v.0)
    }

    /// Return the associated UserConnection ID, if any
    pub fn user_connection_id(&self) -> Option<UserConnectionId> {
        self.user.get().map(|v| v.1)
    }

    /// Return the associated User and UserConnection IDs, if present
    pub(super) fn user_ids(&self) -> Option<(UserId, UserConnectionId)> {
        self.user.get().copied()
    }

    /// Return the associated pre-client data, if any
    pub fn pre_client(&self) -> Option<Arc<PreClient>> {
        self.pre_client.load_full()
    }

    /// Set the associated user ID
    pub fn set_user(&self, user_id: UserId, user_connection_id: UserConnectionId) {
        let _ = self.user.set((user_id, user_connection_id));
        self.pre_client.swap(None);
    }

    /// Notify that a new message has been received on this connection
    ///
    /// Returns `Ok(())` on success, `Err(message)` if the connection's receive queue is full
    pub fn new_message(&self, message: String) -> Result<(), String> {
        self.receive_queue.add(message)
    }

    /// Poll for messages that the throttle permits to be processed
    pub fn poll_messages(&self) -> impl Iterator<Item = String> + '_ {
        self.receive_queue.iter()
    }
}

impl MessageSink for ClientConnection {
    fn send(&self, msg: messages::OutboundClientMessage) {
        if let Some(formatted) = msg.format_for_client_caps((&self.capabilities).into()) {
            tracing::trace!(
                "Sending to {:?}: {}",
                self.id(),
                formatted.strip_suffix("\r\n").unwrap_or(&formatted)
            );
            self.connection.send(formatted)
        }
    }

    fn user_id(&self) -> Option<UserId> {
        ClientConnection::user_id(self)
    }

    fn capabilities(&self) -> ClientCapabilitySet {
        (&self.capabilities).into()
    }
}

impl Drop for ClientConnection {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            conn.close();
        }
    }
}

impl PreClient {
    /// Construct a `PreClient`
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            connected_at: Instant::now(),
            attach_user_id: OnceLock::new(),
            user: OnceLock::new(),
            extra_user_params: OnceLock::new(),
            nick: OnceLock::new(),
            realname: OnceLock::new(),
            hostname: OnceLock::new(),
            sasl_session: OnceLock::new(),
            sasl_account: OnceLock::new(),
            progress_flags: AtomicU32::new(0),
        }
    }

    /// Determine whether this connection is ready to complete registration.
    ///
    /// This will return true if the connection is ready to either register as a new user,
    /// or attach as a new connection to an existing user.
    pub fn can_register(&self) -> bool {
        let can_register_new = self.can_register_new_user();
        let can_attach = self.can_attach_to_user().is_some();

        tracing::trace!(
            ?self,
            can_register_new,
            can_attach,
            "PreClient::can_register"
        );
        can_register_new || can_attach
    }

    /// Determine whether this connection is ready to register as a new user
    pub fn can_register_new_user(&self) -> bool {
        self.user.get().is_some()
            && self.nick.get().is_some()
            && self.hostname.get().is_some()
            && self.progress_flags.load(Ordering::Relaxed) == 0
    }

    /// Determine whether this connection is ready to attach to an existing user
    pub fn can_attach_to_user(&self) -> Option<UserId> {
        self.hostname.get().and(self.attach_user_id.get()).copied()
    }

    /// Set a progress flag, indicating that the given operation is beginning
    pub fn start_progress(&self, flag: ProgressFlag) {
        self.progress_flags.fetch_or(flag as u32, Ordering::Relaxed);
    }

    /// Unset a progress flag, indicating that the given operation has completed
    ///
    /// Return true if the client is ready to register, i.e. if `can_register`
    /// would return true immediately after this call
    pub fn complete_progress(&self, flag: ProgressFlag) -> bool {
        let prev_flags = self
            .progress_flags
            .fetch_and(!(flag as u32), Ordering::Relaxed);

        let result = self.user.get().is_some()
            && self.nick.get().is_some()
            && self.hostname.get().is_some()
            && prev_flags == flag as u32;

        tracing::trace!(?self, ?flag, result, "PreClient::complete_progress");
        result
    }
}
