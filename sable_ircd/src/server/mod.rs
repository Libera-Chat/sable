use crate::*;
use crate::movable::Movable;
use capability::*;
use messages::*;

use sable_network::{prelude::*, config::TlsData};
use event::*;
use rpc::*;

use auth_client::*;
use client_listener::*;

use tokio::{
    sync::{
        mpsc::{
            UnboundedSender,
            UnboundedReceiver,
            unbounded_channel,
        },
        broadcast,
        Mutex,
    },
    select,
};

use std::{
    sync::Arc,
};

use parking_lot::RwLock;

use strum::IntoEnumIterator;

mod async_handler_collection;
pub use async_handler_collection::*;

mod upgrade;

use self::config::ClientServerConfig;

pub mod config;

mod command_action;
mod update_handler;
mod user_access;
mod server_type;

/// A client server.
///
/// This type uses the [`NetworkNode`](sable_network::node::NetworkNode) struct to link to the network
/// and process state. It consumes the stream of history output by `NetworkNode`, and speaks
/// IRC client protocol.
pub struct ClientServer
{
    // These must be tokio Mutexes so that we can hold on to them across await points
    action_receiver: Mutex<UnboundedReceiver<CommandAction>>,
    connection_events: Mutex<UnboundedReceiver<ConnectionEvent>>,
    auth_events: Mutex<UnboundedReceiver<AuthEvent>>,
    history_receiver: Mutex<UnboundedReceiver<NetworkHistoryUpdate>>,

    action_submitter: UnboundedSender<CommandAction>,
    command_dispatcher: command::CommandDispatcher,
    connections: RwLock<ConnectionCollection>,
    auth_client: AuthClient,
    isupport: ISupportBuilder,
    client_caps: CapabilityRepository,

    server: Arc<NetworkNode>,
    listeners: Movable<ListenerCollection>,
}

impl ClientServer
{
    /// Access the network state
    pub fn network(&self) -> Arc<Network>
    {
        self.server.network()
    }

    /// The ID generator used to identify objects created by this server
    pub fn ids(&self) -> &ObjectIdGenerator
    {
        self.server.ids()
    }

    /// The underlying network node
    pub fn server(&self) -> &NetworkNode
    {
        &self.server
    }

    /// This server's name
    pub fn name(&self) -> &ServerName
    {
        self.server.name()
    }

    /// Submit a command action to process in the next loop iteration.
    #[tracing::instrument(skip(self))]
    pub fn add_action(&self, act: CommandAction)
    {
        self.action_submitter.send(act).unwrap();
    }

    /// Access the currently used [`PolicyService`](sable_network::policy::PolicyService)
    pub(crate) fn policy(&self) -> &dyn sable_network::policy::PolicyService
    {
        self.server.policy()
    }

    /// Find a client connection
    pub fn find_connection(&self, id: ConnectionId) -> Option<Arc<ClientConnection>>
    {
        let ret = self.connections.get(id).ok();
        tracing::trace!("Looking up connection id {:?}, {}", id, if ret.is_some() {"found"}else{"not found"});
        ret
    }

    /// The [`CapabilityRepository`] describing the server's supported client capability flags
    pub(crate) fn client_capabilities(&self) -> &CapabilityRepository
    {
        &self.client_caps
    }

    #[tracing::instrument]
    fn build_basic_isupport() -> ISupportBuilder
    {
        let mut ret = ISupportBuilder::new();
        ret.add(ISupportEntry::simple("EXCEPTS"));
        ret.add(ISupportEntry::simple("INVEX"));
        ret.add(ISupportEntry::simple("FNC"));

        ret.add(ISupportEntry::string("CASEMAPPING", "ascii"));

        let list_modes: String = ListModeType::iter().map(|t| t.mode_letter()).collect();
        let key_modes: String = KeyModeType::iter().map(|t| t.mode_letter()).collect();
        let param_modes = "";
        let simple_modes: String = ChannelModeSet::all().map(|m| m.1).iter().collect();
        let chanmodes = format!("{},{},{},{}", list_modes, key_modes, param_modes, simple_modes);

        ret.add(ISupportEntry::string("CHANMODES", &chanmodes));

        let prefix_modes: String = MembershipFlagSet::all().map(|m| m.1).iter().collect();
        let prefix_chars: String = MembershipFlagSet::all().map(|m| m.2).iter().collect();

        let prefix = format!("({}){}", prefix_modes, prefix_chars);
        ret.add(ISupportEntry::string("PREFIX", &prefix));

        ret
    }


    #[tracing::instrument(skip_all, fields(source = ?msg.source))]
    async fn process_connection_event(&self, msg: ConnectionEvent)
    {
        match msg.detail {
            ConnectionEventDetail::NewConnection(conn) => {
                tracing::trace!("Got new connection");
                let conn = ClientConnection::new(conn);

                conn.send(&message::Notice::new(self, &UnknownTarget,
                            "*** Looking up your hostname"));
                self.auth_client.start_dns_lookup(conn.id(), conn.remote_addr());
                self.connections.write().add(msg.source, conn);
            },
            ConnectionEventDetail::Message(m) => {
                tracing::trace!(msg=?m, "Got message");

                self.connections.write().new_message(msg.source, m);
            },
            ConnectionEventDetail::Error(e) => {
                if let Ok(conn) = self.connections.get(msg.source) {
                    if let Some(userid) = conn.user_id() {
                        // If the user has a session key set, then they're in persistent session mode
                        // and shouldn't be quit just because one of their connections closed
                        let should_quit = if let Ok(user) = self.network().user(userid) {
                            user.session_key().is_none()
                        } else {
                            true
                        };

                        if should_quit
                        {
                            self.apply_action(CommandAction::state_change(
                                userid,
                                details::UserQuit {
                                    message: format!("I/O error: {}", e)
                                }
                            )).await;
                        }
                    }
                }
                self.connections.write().remove(msg.source);
            }
        }
    }

    fn process_pending_client_messages<'a, 'b>(&'b self, async_handlers: &AsyncHandlerCollection<'a>)
        where Self: 'a, 'b: 'a
    {
        let connections = self.connections.read();
        for (conn_id, message) in connections.poll_messages().collect::<Vec<_>>()
        {
            if let Some(parsed) = ClientMessage::parse(conn_id, &message)
            {
                let processor = CommandProcessor::new(self, &self.command_dispatcher);
                processor.process_message(parsed, async_handlers);
            }
            else
            {
                tracing::info!(?message, "Failed parsing")
            }
        }
        drop(connections);

        for flooded in self.connections.write().flooded_connections()
        {
            if let Some(user_id) = flooded.user_id()
            {
                if let Ok(user) = self.server.network().user(user_id)
                {
                    if user.session_key().is_some()
                    {
                        // Don't kill a multi-connection or persistent user because one connection flooded off
                        continue;
                    }

                    self.server.submit_event(user_id, event::details::UserQuit { message: "Excess Flood".to_string() });
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
    async fn do_run(&self, mut shutdown_channel: broadcast::Receiver<ShutdownAction>) -> ShutdownAction
    {
        // Take ownership of these receivers here, so that we no longer need a mut borrow of `self` once the
        // run loop starts
        let mut action_receiver = self.action_receiver.lock().await;
        let mut history_receiver = self.history_receiver.lock().await;
        let mut connection_events = self.connection_events.lock().await;
        let mut auth_events = self.auth_events.lock().await;

        let mut async_handlers = AsyncHandlerCollection::new();

        let shutdown_action = loop
        {
            tracing::trace!("ClientServer run loop");
            // Before looking for an I/O event, do our internal bookkeeping.
            // First, take inbound client messages and process them
            self.process_pending_client_messages(&async_handlers);

            // Then, see whether there are any actions we need to process synchronously
            while let Ok(act) = action_receiver.try_recv()
            {
                tracing::trace!(?act, "Got pending CommandAction");
                self.apply_action(act).await;
            }

            let timeout = tokio::time::sleep(tokio::time::Duration::from_millis(250));
            tokio::pin!(timeout);

            select! {
                _ = &mut timeout =>
                {
                    tracing::trace!("...from timeout");
                    // Make sure we don't block waiting for i/o for too long, in case there are
                    // queued client messages to be processed or other housekeeping
                    continue;
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
                                        conn.send(&message::Notice::new(self, &UnknownTarget,
                                                        &format!("*** Found your hostname: {}", hostname)));

                                        pc.hostname.set(hostname).ok();
                                    } else {
                                        conn.send(&message::Notice::new(self, &UnknownTarget,
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
        };

        shutdown_action
    }
}