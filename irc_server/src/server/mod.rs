use super::*;
use irc_network::*;
use event::*;
use crate::policy::*;
use utils::OrLog;
use rpc_protocols::*;
use auth_client::*;
use ircd_sync::ReplicatedEventLog;

use client_listener::{
    ConnectionEvent,
    ConnectionEventDetail,
    ConnectionId,
};

use tokio::{
    sync::mpsc::{
        Receiver,
        channel,
        UnboundedSender,
        UnboundedReceiver,
        unbounded_channel
    },
    sync::oneshot,
    time,
    select,
};
use strum::IntoEnumIterator;

use std::{
    collections::HashMap,
    time::Duration,
    sync::Arc,
};

pub(crate) mod command_processor;
use command_processor::*;

mod connection_collection;
use connection_collection::ConnectionCollection;
use command::*;

mod management;
pub use management::ServerManagementCommand;
pub use management::ServerManagementCommandType;

mod state_change_receiver;

mod isupport;
pub use isupport::*;

/// An IRC client server.
pub struct Server
{
    my_id: ServerId,
    name: ServerName,
    version: String,
    net: Network,
    event_log: Arc<ReplicatedEventLog>,
    epoch: EpochId,
    id_generator: ObjectIdGenerator,
    rpc_receiver: Receiver<NetworkMessage>,
    action_receiver: UnboundedReceiver<CommandAction>,
    action_submitter: UnboundedSender<CommandAction>,
    connection_events: Receiver<ConnectionEvent>,
    command_dispatcher: command::CommandDispatcher,
    connections: ConnectionCollection,
    policy_service: StandardPolicyService,
    auth_client: AuthClient,
    auth_events: Receiver<AuthEvent>,
    isupport: ISupportBuilder,
}

impl Server
{
    /// Construct a server.
    ///
    /// Arguments:
    /// - `id`: This server's ID
    /// - `epoch`: The epoch ID to be used for ID generation. This must be unique
    ///   across all invocations with the same server ID, and should be the same
    ///   as the epoch ID provided to the event log
    /// - `name`: The server's name
    /// - `connection_events`: a channel to receive connection events. In normal
    ///   use this should be shared with a
    ///   [`ListenerCollection`](client_listener::ListenerCollection)
    /// - `rpc_receiver`: channel to receive messages from the network synchronisation.
    ///   Should be shared with a `ReplicatedEventLog`.
    /// - `to_network`: channel to send out new events. Should be shared with the
    ///   `ReplicatedEventLog`.
    pub fn new(id: ServerId,
               epoch: EpochId,
               name: ServerName,
               net: Network,
               event_log: Arc<ReplicatedEventLog>,
               connection_events: Receiver<ConnectionEvent>,
               rpc_receiver: Receiver<NetworkMessage>,
            ) -> Self
    {
        let (auth_send, auth_recv) = channel(128);
        let (action_send, action_recv) = unbounded_channel();

        if cfg!(feature = "debug") && !net.config().debug_mode
        {
            panic!("Server is built with debug code but network has debug disabled")
        }

        Self {
            my_id: id,
            name,
            version: Self::build_version(),
            net,
            epoch,
            event_log,
            id_generator: ObjectIdGenerator::new(id, epoch),
            rpc_receiver,
            action_receiver: action_recv,
            action_submitter: action_send,
            connection_events,
            connections: ConnectionCollection::new(),
            command_dispatcher: command::CommandDispatcher::new(),
            policy_service: StandardPolicyService::new(),
            auth_client: AuthClient::new(auth_send).expect("Couldn't create auth client"),
            auth_events: auth_recv,
            isupport: Self::build_basic_isupport(),
        }
    }

    fn submit_event(&self, id: impl Into<ObjectId>, detail: impl Into<EventDetails>)
    {
        let id = id.into();
        let detail = detail.into();
        tracing::trace!("Submitting new event {:?} {:?}", id, detail);
        self.event_log.create_event(id, detail);
    }

    /// Retrieve the [`ObjectIdGenerator`] used to generate object identifiers
    pub fn ids(&self) -> &ObjectIdGenerator
    {
        &self.id_generator
    }

    /// Access the IRC network state
    pub fn network(&self) -> &Network
    {
        &self.net
    }

    /// Get the server's name
    pub fn name(&self) -> &ServerName
    {
        &self.name
    }

    /// The server's ID
    pub fn id(&self) -> ServerId
    {
        self.my_id
    }

    /// The server's build version
    pub fn version(&self) -> &str
    {
        &self.version
    }

    /// The server's build flags
    pub fn server_flags(&self) -> state::ServerFlags
    {
        let mut ret = state::ServerFlags::empty();
        if cfg!(feature = "debug")
        {
            ret |= state::ServerFlags::DEBUG;
        }
        ret
    }

    /// This server's entry in the network state
    pub fn me(&self) -> LookupResult<wrapper::Server>
    {
        self.net.server(self.my_id)
    }

    fn command_dispatcher(&self) -> &command::CommandDispatcher
    {
        &self.command_dispatcher
    }

    /// Submit a command action to process in the next loop iteration.
    #[tracing::instrument(skip(self))]
    pub fn add_action(&self, act: CommandAction)
    {
        self.action_submitter.send(act).unwrap();
    }

    /// Access the currently used [`PolicyService`]
    pub fn policy(&self) -> &dyn PolicyService
    {
        &self.policy_service
    }

    /// Find a client connection
    pub fn find_connection(&self, id: ConnectionId) -> Option<&ClientConnection>
    {
        let ret = self.connections.get(id).ok();
        tracing::trace!("Looking up connection id {:?}, {}", id, if ret.is_some() {"found"}else{"not found"});
        ret
    }

    fn lookup_message_source(&self, id: ObjectId) -> Result<Box<dyn messages::MessageSource + '_>, LookupError>
    {
        match id {
            ObjectId::User(u) => Ok(Box::new(self.net.user(u)?)),
            ObjectId::Server(_) => Ok(Box::new(self)), // TODO
            _ => Err(LookupError::WrongType),
        }
    }

    #[tracing::instrument(skip(self))]
    fn apply_event(&mut self, event: Event)
    {
        tracing::trace!("Applying inbound event: {:?}", event);

        let receiver = state_change_receiver::StateChangeReceiver::new();

        if let Err(e) = self.net.apply(&event, &receiver) {
            panic!("Event {:?} failed to apply: {}", event, e);
        }

        while let Ok(change) = receiver.recv.try_recv()
        {
            self.handle_network_update(change);
        }
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

    fn build_version() -> String
    {
        let git_version = crate::build_data::GIT_COMMIT_HASH.map(|s| format!("-{}", s)).unwrap_or_else(String::new);
        let git_dirty = if matches!(crate::build_data::GIT_DIRTY, Some(true)) {
            "-dirty".to_string()
        } else {
            String::new()
        };
        format!("sable-{}{}{}", crate::build_data::PKG_VERSION, git_version, git_dirty)
    }

    /// Run the server
    ///
    /// Arguments:
    /// - `management_channel`: receives management commands from the management service
    /// - `shutdown_channel`: used to signal the server to shut down
    #[tracing::instrument(skip_all)]
    pub async fn run(&mut self, mut management_channel: Receiver<ServerManagementCommand>, mut shutdown_channel: oneshot::Receiver<ShutdownAction>) -> ShutdownAction
    {
        self.submit_event(self.my_id, details::NewServer {
            epoch: self.epoch,
            name: self.name,
            ts: utils::now(),
            flags: self.server_flags(),
            version: self.version().to_string(),
        });
        let mut check_ping_timer = time::interval(Duration::from_secs(5));

        let shutdown_action = loop
        {
            // Between each I/O event, see whether there are any actions we need to process synchronously
            while let Ok(act) = self.action_receiver.try_recv()
            {
                self.apply_action(act);
            }
            select! {
                res = self.connection_events.recv() =>
                {
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
                    match res
                    {
                        Some(AuthEvent::DnsResult(msg)) =>
                        {
                            if let Ok(conn) = self.connections.get(msg.conn) {
                                tracing::info!("DNS lookup finished for {:?}: {}/{:?}", msg.conn,
                                                                                conn.remote_addr(),
                                                                                msg.hostname
                                                                                );
                                if let Some(pc_rc) = &conn.pre_client {
                                    let mut pc = pc_rc.borrow_mut();
                                    if let Some(hostname) = msg.hostname {
                                        conn.send(&message::Notice::new(self, &*pc,
                                                        &format!("*** Found your hostname: {}", hostname)));

                                        pc.hostname = Some(hostname);
                                    } else {
                                        conn.send(&message::Notice::new(self, &*pc,
                                                        "*** Couldn't look up your hostname"));
                                        let no_hostname = Hostname::convert(conn.remote_addr());
                                        match no_hostname {
                                            Ok(n) => pc.hostname = Some(n),
                                            Err(e) => conn.error(&e.to_string())
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
                res = self.rpc_receiver.recv() =>
                {
                    match res {
                        Some(NetworkMessage::NewEvent(event)) =>
                        {
                            self.apply_event(event);
                        },
                        Some(NetworkMessage::ImportNetworkState(new_net)) =>
                        {
                            tracing::debug!("Server got state import");
                            self.net = *new_net;
                        },
                        Some(NetworkMessage::ExportNetworkState(channel)) =>
                        {
                            tracing::debug!("Server got state export request; sending");
                            channel.send(Box::new(self.net.clone())).await.or_log("Error sending network state for export");
                        },
                        None => {
                            panic!("what to do here?");
                        }
                    }
                },
                res = management_channel.recv() =>
                {
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
                _ = check_ping_timer.tick() =>
                {
                    self.check_pings();
                },
                shutdown = &mut shutdown_channel =>
                {
                    match shutdown
                    {
                        Err(e) =>
                        {
                            tracing::error!("Got error ({}) from shutdown channel; exiting", e);
                            break ShutdownAction::Shutdown;
                        }
                        Ok(ShutdownAction::Shutdown) | Ok(ShutdownAction::Restart) =>
                        {
                            // In either of these cases, we're disconnecting from the network and
                            // should announce that. We might be starting again, but it'll be from
                            // a clean slate.
                            break shutdown.unwrap();
                        }
                        Ok(ShutdownAction::Upgrade) =>
                        {
                            // If we're upgrading, then don't signal to the network that we're shutting down.
                            // The actual state save/restore will be called by main() after everything's stopped
                            // processing.
                            return ShutdownAction::Upgrade;
                        }
                    }
                },
            }
        };

        let me = self.net.server(self.my_id).expect("Couldn't say I quit as I have no record of myself");
        self.submit_event(self.my_id, details::ServerQuit { epoch: me.epoch() });

        shutdown_action
    }

    #[tracing::instrument(skip_all, fields(source = ?msg.source))]
    async fn process_connection_event(&mut self, msg: ConnectionEvent)
    {
        match msg.detail {
            ConnectionEventDetail::NewConnection(conn) => {
                tracing::info!("Got new connection");
                let conn = ClientConnection::new(conn);

                conn.send(&message::Notice::new(self, &conn.pre_client,
                            "*** Looking up your hostname"));
                self.auth_client.start_dns_lookup(conn.id(), conn.remote_addr());
                self.connections.add(msg.source, conn);
            },
            ConnectionEventDetail::Message(m) => {
                tracing::info!(msg=?m, "Got message");

                if let Some(message) = ClientMessage::parse(msg.source, &m)
                {
                    let processor = CommandProcessor::new(self);
                    processor.process_message(message).await;
                }
                else
                {
                    tracing::info!("Failed parsing")
                }
            },
            ConnectionEventDetail::Error(e) => {
                tracing::error!(error=?e, "Got connection error");
                if let Ok(conn) = self.connections.get(msg.source) {
                    if let Some(userid) = conn.user_id {
                        self.apply_action(CommandAction::state_change(
                            userid,
                            details::UserQuit {
                                message: format!("I/O error: {}", e)
                            }
                        ));
                    }
                }
                self.connections.remove(msg.source);
            }
        }
    }
}

mod command_action;
mod event_handler;
mod pings;
mod send_helpers;

mod upgrade;
pub use upgrade::ServerState;