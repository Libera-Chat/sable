use crate::*;
use capability::*;
use messages::*;

use sable_network::prelude::*;
use event::*;
use rpc::*;

use auth_client::*;
use client_listener::*;

use tokio::{
    sync::mpsc::{
        Receiver,
        UnboundedSender,
        UnboundedReceiver,
        unbounded_channel,
    },
    sync::oneshot,
    select,
};

use std::{
    sync::Arc,
};

use futures::future::OptionFuture;

use strum::IntoEnumIterator;

mod management;
pub use management::ServerManagementCommand;
pub use management::ServerManagementCommandType;

mod upgrade;
pub use upgrade::ClientServerState;

mod command_action;
mod update_handler;
mod user_access;

/// A client server.
///
/// This type uses the [`Server`](sable_network::server::Server) struct to link to the network
/// and process state. It consumes the stream of history output by `Server`, and speaks
/// IRC client protocol.
pub struct ClientServer
{
    action_receiver: UnboundedReceiver<CommandAction>,
    action_submitter: UnboundedSender<CommandAction>,
    connection_events: UnboundedReceiver<ConnectionEvent>,
    command_dispatcher: command::CommandDispatcher,
    connections: ConnectionCollection,
    auth_client: AuthClient,
    auth_events: UnboundedReceiver<AuthEvent>,
    isupport: ISupportBuilder,
    client_caps: CapabilityRepository,

    server: Arc<Server>,
    history_receiver: UnboundedReceiver<NetworkHistoryUpdate>,
}

impl ClientServer
{
    /// Create a new `ClientServer`
    pub fn new(id: ServerId,
               epoch: EpochId,
               name: ServerName,
               net: Network,
               event_log: Arc<ReplicatedEventLog>,
               rpc_receiver: UnboundedReceiver<NetworkMessage>,
               connection_events: UnboundedReceiver<ConnectionEvent>,
            ) -> Self
    {
        let (history_sender, history_receiver) = unbounded_channel();
        let (action_submitter, action_receiver) = unbounded_channel();
        let (auth_sender, auth_events) = unbounded_channel();

        let policy = policy::StandardPolicyService::new();

        let server = Arc::new(Server::new(id, epoch, name, net, event_log, rpc_receiver, history_sender, policy));

        Self {
            action_receiver,
            action_submitter: action_submitter,
            connection_events,
            command_dispatcher: CommandDispatcher::new(),
            connections: ConnectionCollection::new(),
            auth_client: AuthClient::new(auth_sender).unwrap(),
            auth_events,
            isupport: Self::build_basic_isupport(),
            client_caps: CapabilityRepository::new(),
            server,
            history_receiver,
        }
    }

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

    /// The underlying network server
    pub fn server(&self) -> &Server
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
    pub fn find_connection(&self, id: ConnectionId) -> Option<&ClientConnection>
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
    async fn process_connection_event(&mut self, msg: ConnectionEvent)
    {
        match msg.detail {
            ConnectionEventDetail::NewConnection(conn) => {
                tracing::trace!("Got new connection");
                let conn = ClientConnection::new(conn);

                conn.send(&message::Notice::new(self, &UnknownTarget,
                            "*** Looking up your hostname"));
                self.auth_client.start_dns_lookup(conn.id(), conn.remote_addr());
                self.connections.add(msg.source, conn);
            },
            ConnectionEventDetail::Message(m) => {
                tracing::trace!(msg=?m, "Got message");

                self.connections.new_message(msg.source, m);
            },
            ConnectionEventDetail::Error(e) => {
                if let Ok(conn) = self.connections.get(msg.source) {
                    if let Some(userid) = conn.user_id {
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
                self.connections.remove(msg.source);
            }
        }
    }

    async fn process_pending_client_messages(&mut self)
    {
        for (conn_id, message) in self.connections.poll_messages().collect::<Vec<_>>()
        {
            if let Some(parsed) = ClientMessage::parse(conn_id, &message)
            {
                let processor = CommandProcessor::new(self, &self.command_dispatcher);
                processor.process_message(parsed);
            }
            else
            {
                tracing::info!(?message, "Failed parsing")
            }
        }

        for flooded in self.connections.flooded_connections()
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
    pub async fn run(&mut self, mut management_channel: Receiver<ServerManagementCommand>, shutdown_channel: oneshot::Receiver<ShutdownAction>) -> ShutdownAction
    {
        let (server_shutdown, server_shutdown_recv) = oneshot::channel();
        let mut server_shutdown = Some(server_shutdown);

        let mut server_task = tokio::spawn(Arc::clone(&self.server).run(server_shutdown_recv));

        let mut shutdown_channel = OptionFuture::from(Some(shutdown_channel));

        loop
        {
            // Before looking for an I/O event, do our internal bookkeeping.
            // First, take inbound client messages and process them
            self.process_pending_client_messages().await;

            // Then, see whether there are any actions we need to process synchronously
            while let Ok(act) = self.action_receiver.try_recv()
            {
                tracing::trace!(?act, "Got pending CommandAction");
                self.apply_action(act).await;
            }

            let timeout = tokio::time::sleep(tokio::time::Duration::from_millis(250));
            tokio::pin!(timeout);

            select! {
                _ = &mut timeout =>
                {
                    // Make sure we don't block waiting for i/o for too long, in case there are
                    // queued client messages to be processed or other housekeeping
                    continue;
                },
                res = &mut server_task =>
                {
                    match res
                    {
                        Ok(action) => break action,
                        Err(e) => panic!("Server task exited abnormally ({})", e)
                    };
                },
                res = self.history_receiver.recv() =>
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
                res = self.connection_events.recv() =>
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
                res = self.auth_events.recv() =>
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
                                if let Some(pc) = &conn.pre_client {
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
                res = management_channel.recv() =>
                {
                    tracing::trace!("...from management_channel");
                    match res {
                        Some(cmd) =>
                        {
                            self.handle_management_command(cmd).await;
                        }
                        None =>
                        {
                            panic!("Lost management service");
                        }
                    }
                },
                shutdown = &mut shutdown_channel =>
                {
                    shutdown_channel = None.into();
                    match shutdown
                    {
                        Some(Err(e)) =>
                        {
                            tracing::error!("Got error ({}) from shutdown channel; exiting", e);
                            break ShutdownAction::Shutdown;
                        }
                        Some(Ok(action)) =>
                        {
                            // Signal the underlying network server to shut down, but keep going
                            // until it does so that we can process any state changes it emits
                            if let Some(server_shutdown) = server_shutdown.take()
                            {
                                server_shutdown.send(action).expect("Failed to signal server shutdown");
                            }
                        }
                        None => ()
                    }
                },
            }
        }
    }
}