use crate::movable::Movable;
use crate::*;
use capability::*;
use messages::*;

use event::*;
use rpc::*;
use sable_network::{config::TlsData, network::ban::NetworkBanAction, prelude::*};

use auth_client::*;
use client_listener::*;

use tokio::{
    select,
    sync::{
        broadcast,
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Mutex,
    },
    time,
};

use std::{
    collections::VecDeque,
    sync::{Arc, Weak},
    time::Duration,
};

use parking_lot::RwLock;

use strum::IntoEnumIterator;

mod async_handler_collection;
pub use async_handler_collection::*;

mod upgrade;

use self::{
    config::{ClientServerConfig, RawClientServerConfig, ServerInfoStrings},
    message_sink_repository::MessageSinkRepository,
};
use crate::monitor::MonitorSet;

pub mod config;

mod command_action;
mod message_sink_repository;
mod server_type;
mod update_handler;
mod user_access;

const PREREG_TIMEOUT: time::Duration = time::Duration::from_secs(120);

/// Last parameters of the RPL_MYINFO (004) numeric
struct MyInfo {
    user_modes: String,
    chan_modes: String,
    chan_modes_with_a_parameter: String,
}

/// A client server.
///
/// This type uses the [`NetworkNode`] struct to link to the network
/// and process state. It consumes the stream of history output by `NetworkNode`, and speaks
/// IRC client protocol.
pub struct ClientServer {
    // These must be tokio Mutexes so that we can hold on to them across await points
    action_receiver: Mutex<UnboundedReceiver<CommandAction>>,
    connection_events: Mutex<UnboundedReceiver<ConnectionEvent>>,
    auth_events: Mutex<UnboundedReceiver<AuthEvent>>,
    history_receiver: Mutex<UnboundedReceiver<NetworkHistoryUpdate>>,

    // Stored `MessageSink`s for deferred labeled-response
    stored_response_sinks: RwLock<MessageSinkRepository>,

    action_submitter: UnboundedSender<CommandAction>,
    command_dispatcher: command::CommandDispatcher,

    connections: RwLock<ConnectionCollection>,
    /// Connections which either did not complete registration or completed it recently,
    /// in increasing order of when they were open.
    prereg_connections: Mutex<VecDeque<Weak<ClientConnection>>>,

    auth_client: AuthClient,
    myinfo: MyInfo,
    pub isupport: ISupportBuilder,
    client_caps: CapabilityRepository,

    node: Arc<NetworkNode>,
    listeners: Movable<ListenerCollection>,

    // Any general static info (responses for MOTD, ADMIN, and so on)
    pub info_strings: ServerInfoStrings,

    pub monitors: RwLock<MonitorSet>,
}

impl ClientServer {
    /// Access the network state
    pub fn network(&self) -> Arc<Network> {
        self.node.network()
    }

    /// The ID generator used to identify objects created by this server
    pub fn ids(&self) -> &ObjectIdGenerator {
        self.node.ids()
    }

    /// The underlying network node
    pub fn node(&self) -> &NetworkNode {
        &self.node
    }

    /// This server's name
    pub fn name(&self) -> &ServerName {
        self.node.name()
    }

    /// Submit a command action to process in the next loop iteration.
    #[tracing::instrument(skip(self))]
    pub fn add_action(&self, act: CommandAction) {
        self.action_submitter.send(act).unwrap();
    }

    /// Access the currently used [`PolicyService`](sable_network::policy::PolicyService)
    pub(crate) fn policy(&self) -> &dyn sable_network::policy::PolicyService {
        self.node.policy()
    }

    /// Find a client connection
    pub fn find_connection(&self, id: ConnectionId) -> Option<Arc<ClientConnection>> {
        let ret = self.connections.get(id).ok();
        tracing::trace!(
            "Looking up connection id {:?}, {}",
            id,
            if ret.is_some() { "found" } else { "not found" }
        );
        ret
    }

    /// The [`CapabilityRepository`] describing the server's supported client capability flags
    pub(crate) fn client_capabilities(&self) -> &CapabilityRepository {
        &self.client_caps
    }

    /// Store a [`MessageSink`] to use when processing updates caused by the given event
    pub(crate) fn store_response_sink(
        &self,
        event_id: EventId,
        connection_id: ConnectionId,
        sink: Arc<dyn MessageSink + 'static>,
    ) {
        self.stored_response_sinks
            .write()
            .store(event_id, connection_id, sink);
    }

    #[tracing::instrument]
    fn build_myinfo() -> MyInfo {
        MyInfo {
            user_modes: UserModeSet::all().map(|m| m.mode_char()).iter().collect(),
            chan_modes: ChannelModeSet::all()
                .map(|m| m.mode_char())
                .iter()
                .collect(),
            chan_modes_with_a_parameter: ListModeType::iter()
                .map(|t| t.mode_char())
                .chain(KeyModeType::iter().map(|t| t.mode_char()))
                .chain(MembershipFlagSet::all().map(|m| m.mode_char()).into_iter())
                .collect(),
        }
    }

    #[tracing::instrument]
    fn build_basic_isupport(config: &ClientServerConfig) -> ISupportBuilder {
        let mut ret = ISupportBuilder::new();
        ret.add(ISupportEntry::simple("EXCEPTS"));
        ret.add(ISupportEntry::simple("INVEX"));
        ret.add(ISupportEntry::simple("FNC"));

        // https://ircv3.net/specs/extensions/utf8-only
        ret.add(ISupportEntry::simple("UTF8ONLY"));

        ret.add(ISupportEntry::int(
            "MONITOR",
            config.monitor.max_per_connection.into(),
        ));

        ret.add(ISupportEntry::string("CASEMAPPING", "ascii"));

        // https://ircv3.net/specs/extensions/message-tags#rpl_isupport-tokens
        // Tell clients all client tags are rejected, so conforming clients won't
        // even try to send TAGMSG (which we don't support yet).
        ret.add(ISupportEntry::string("CLIENTTAGDENY", "*"));

        ret.add(ISupportEntry::int(
            "HOSTLEN",
            Hostname::LENGTH.try_into().unwrap(),
        ));
        ret.add(ISupportEntry::int(
            "NICKLEN",
            Nickname::LENGTH.try_into().unwrap(),
        ));
        ret.add(ISupportEntry::int(
            "USERLEN",
            Username::LENGTH.try_into().unwrap(),
        ));

        let list_modes: String = ListModeType::iter().map(|t| t.mode_char()).collect();
        let key_modes: String = KeyModeType::iter().map(|t| t.mode_char()).collect();
        let param_modes = "";
        let simple_modes: String = ChannelModeSet::all()
            .map(|m| m.mode_char())
            .iter()
            .collect();
        let chanmodes = format!(
            "{},{},{},{}",
            list_modes, key_modes, param_modes, simple_modes
        );

        ret.add(ISupportEntry::string("CHANMODES", &chanmodes));

        // https://ircv3.net/specs/extensions/chathistory#isupport-tokens
        // 'msgid' not supported yet
        ret.add(ISupportEntry::string("MSGREFTYPES", "timestamp"));

        let prefix_modes: String = MembershipFlagSet::all()
            .map(|m| m.mode_char())
            .iter()
            .collect();
        let prefix_chars: String = MembershipFlagSet::all()
            .map(|m| m.prefix_char())
            .iter()
            .collect();

        let prefix = format!("({}){}", prefix_modes, prefix_chars);
        ret.add(ISupportEntry::string("PREFIX", &prefix));

        ret
    }

    /// Disconnects `PreClient`s that have been connected for too long (ie. connections
    /// which did not complete registration)
    #[tracing::instrument(skip_all)]
    async fn reap_preclients(self: Arc<Self>) {
        match self.prereg_connections.try_lock() {
            Err(_) => {
                tracing::warn!("Previous reap_preclients task is still running, skipping.")
            }
            Ok(mut prereg_connections) => {
                let threshold = time::Instant::now() - PREREG_TIMEOUT;
                while let Some(conn) = prereg_connections.pop_front() {
                    if let Some(conn) = conn.upgrade() {
                        // If not already disconnected
                        if let Some(pre_client) = conn.pre_client() {
                            // If not done registering
                            if pre_client.connected_at < threshold {
                                tracing::debug!("{:?} registration timed out", conn.id());
                                conn.send(message::Error::new("Registration timed out"));
                                self.add_action(CommandAction::CloseConnection(conn.id()));
                            } else {
                                // Client didn't time out yet, put it back in the queue
                                prereg_connections.push_front(Arc::downgrade(&conn));
                                // stop iteration (as the queue is sorted)
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    #[tracing::instrument(skip_all, fields(source = ?msg.source))]
    async fn process_connection_event(&self, msg: ConnectionEvent) {
        match msg.detail {
            ConnectionEventDetail::NewConnection(conn) => {
                tracing::trace!("Got new connection");

                let conn_details = ban::NewConnectionBanSettings {
                    ip: conn.remote_addr,
                    tls: conn.is_tls(),
                };
                for ban in self
                    .network()
                    .network_bans()
                    .find_new_connection(&conn_details)
                {
                    if let NetworkBanAction::RefuseConnection(_) = ban.action {
                        conn.send(format!("ERROR :*** Banned: {}\r\n", ban.reason));
                        conn.close();
                        return;
                    }
                }

                let conn = ClientConnection::new(conn);

                conn.send(message::Notice::new(
                    self,
                    &UnknownTarget,
                    "*** Looking up your hostname",
                ));
                self.auth_client
                    .start_dns_lookup(conn.id(), conn.remote_addr());
                let conn = self.connections.write().add(msg.source, conn);
                self.prereg_connections.lock().await.push_back(conn);
            }
            ConnectionEventDetail::Message(m) => {
                tracing::trace!(msg=?m, "Got message");

                self.connections.write().new_message(msg.source, m);
            }
            ConnectionEventDetail::Error(e) => {
                if let Ok(conn) = self.connections.get(msg.source) {
                    if let Some((userid, user_conn_id)) = conn.user_ids() {
                        // Tell the network that this connection has gone away, regardless of whether the
                        // user itself is sticking around
                        self.node
                            .submit_event(user_conn_id, details::UserDisconnect {});

                        // If the user has a session key set, then they're in persistent session mode
                        // and shouldn't be quit just because one of their connections closed
                        let should_quit = if let Ok(user) = self.network().user(userid) {
                            user.session_key().is_none()
                        } else {
                            true
                        };

                        if should_quit {
                            self.apply_action(CommandAction::state_change(
                                userid,
                                details::UserQuit {
                                    message: e.to_string(),
                                },
                            ))
                            .await;
                        }
                    }
                    match e {
                        ConnectionError::InputLineTooLong => {
                            conn.send(numeric::InputTooLong::new_for(
                                &self.node.name().to_string(),
                                &"*".to_string(),
                            ))
                        }
                        _ => conn.send(message::Error::new(&e.to_string())),
                    }
                }
                self.connections.write().remove(msg.source);
            }
        }
    }

    fn process_pending_client_messages(self: &Arc<Self>, async_handlers: &AsyncHandlerCollection) {
        let connections = self.connections.read();
        for (conn_id, message) in connections.poll_messages().collect::<Vec<_>>() {
            if let Some(parsed) = ClientMessage::parse(conn_id, &message) {
                if let Ok(connection) = connections.get(conn_id) {
                    if let Ok(command) = ClientCommand::new(Arc::clone(self), connection, parsed) {
                        if let Some(async_handler) =
                            self.command_dispatcher.dispatch_command(command)
                        {
                            async_handlers.add(async_handler);
                        }
                    }
                }
            } else {
                tracing::info!(?message, "Failed parsing")
            }
        }
        drop(connections);

        for flooded in self.connections.write().flooded_connections() {
            if let Some(user_id) = flooded.user_id() {
                if let Ok(user) = self.node.network().user(user_id) {
                    if user.session_key().is_some() {
                        // Don't kill a multi-connection or persistent user because one connection flooded off
                        continue;
                    }

                    self.node.submit_event(
                        user_id,
                        event::details::UserQuit {
                            message: "Excess Flood".to_string(),
                        },
                    );
                    flooded.error("Excess Flood");
                }
            }
        }
    }

    /// Run the server
    ///
    /// Arguments:
    /// - `management_channel`: receives management commands from the management service
    /// - `shutdown_channel`: used to signal the server to shut down
    #[tracing::instrument(skip_all)]
    async fn do_run(
        self: Arc<Self>,
        mut shutdown_channel: broadcast::Receiver<ShutdownAction>,
    ) -> ShutdownAction {
        // Take ownership of these receivers here, so that we no longer need a mut borrow of `self` once the
        // run loop starts
        let mut action_receiver = self.action_receiver.lock().await;
        let mut history_receiver = self.history_receiver.lock().await;
        let mut connection_events = self.connection_events.lock().await;
        let mut auth_events = self.auth_events.lock().await;

        let mut async_handlers = AsyncHandlerCollection::new();

        let mut reap_preclients_timer = time::interval(Duration::from_secs(60));

        loop {
            // tracing::trace!("ClientServer run loop");
            // Before looking for an I/O event, do our internal bookkeeping.
            // First, take inbound client messages and process them
            self.process_pending_client_messages(&async_handlers);

            // Then, see whether there are any actions we need to process synchronously
            while let Ok(act) = action_receiver.try_recv() {
                tracing::trace!(?act, "Got pending CommandAction");
                self.apply_action(act).await;
            }

            let timeout = tokio::time::sleep(tokio::time::Duration::from_millis(250));
            tokio::pin!(timeout);

            select! {
                _ = &mut timeout =>
                {
                    // tracing::trace!("...from timeout");
                    // Make sure we don't block waiting for i/o for too long, in case there are
                    // queued client messages to be processed or other housekeeping
                    continue;
                },
                _ = reap_preclients_timer.tick() =>
                {
                    // Spawning a sub-task in order not to block all events
                    tracing::trace!("...from reap_preclients_timer");
                    tokio::spawn(self.clone().reap_preclients());
                },
                _ = async_handlers.poll(), if !async_handlers.is_empty() =>
                {
                    tracing::trace!("...from async_handlers");
                    // No need to do anything here; just polling the collection is enough to
                    // drive execution of any async handlers that are running
                },
                res = history_receiver.recv() =>
                {
                    tracing::trace!(?res, "...from history_receiver");
                    match res
                    {
                        Some(update) =>
                        {
                            if let Err(e) = self.handle_history_update(update)
                            {
                                tracing::error!("Error handing history update: {}", e);
                            }
                        }
                        None => panic!("Lost server"),
                    };
                },
                res = connection_events.recv() =>
                {
                    tracing::trace!("...from connection_events");
                    match res {
                        Some(msg) => {
                            self.process_connection_event(msg).await;
                        },
                        None => {
                            panic!("what to do here?");
                        }
                    }
                },
                res = auth_events.recv() =>
                {
                    tracing::trace!("...from auth_events");
                    match res
                    {
                        Some(AuthEvent::DnsResult(msg)) =>
                        {
                            if let Ok(conn) = self.connections.get(msg.conn) {
                                tracing::trace!("DNS lookup finished for {:?}: {}/{:?}", msg.conn,
                                                                                conn.remote_addr(),
                                                                                msg.hostname
                                                                                );
                                if let Some(pc) = conn.pre_client() {
                                    if let Some(hostname) = msg.hostname {
                                        conn.send(message::Notice::new(&self, &UnknownTarget,
                                                        &format!("*** Found your hostname: {}", hostname)));

                                        pc.hostname.set(hostname).ok();
                                    } else {
                                        conn.send(message::Notice::new(&self, &UnknownTarget,
                                                        "*** Couldn't look up your hostname"));
                                        let no_hostname = Hostname::convert(conn.remote_addr());
                                        match no_hostname {
                                            Ok(n) => { pc.hostname.set(n).ok(); }
                                            Err(e) => { conn.error(&e.to_string()); }
                                        }
                                    }
                                    if pc.can_register() {
                                        let res = self.action_submitter.send(CommandAction::RegisterClient(conn.id()));
                                        if let Err(e) = res {
                                            conn.error(&e.to_string());
                                        }
                                    }
                                }
                            }
                        },
                        None =>
                        {
                            panic!("Lost auth client task");
                        }
                    }
                },
                shutdown = shutdown_channel.recv() =>
                {
                    tracing::trace!("...from shutdown_channel");

                    match shutdown
                    {
                        Err(e) =>
                        {
                            tracing::error!("Got error ({}) from shutdown channel; exiting", e);
                            break ShutdownAction::Shutdown;
                        }
                        Ok(action) =>
                        {
                            break action;
                        }
                    }
                },
            }
        }
    }
}
