use super::*;
use irc_network::*;
use event::*;
use crate::connection::EventDetail::*;
use crate::policy::*;
use utils::OrLog;
use rpc_protocols::*;

use tokio::{
    sync::mpsc::{
        Sender,
        Receiver,
        channel
    },
    time,
    select,
};
use strum::IntoEnumIterator;

use std::{
    collections::HashMap,
    time::Duration,
    net::SocketAddr,
};

use log::{info,error};

pub mod command_processor;
use command_processor::*;

mod connection_collection;
use connection_collection::ConnectionCollection;
use command::*;

mod state_change_receiver;

mod isupport;
pub use isupport::*;

pub struct Server
{
    my_id: ServerId,
    name: ServerName,
    net: Network,
    epoch: EpochId,
    id_generator: ObjectIdGenerator,
    rpc_receiver: Receiver<NetworkMessage>,
    event_submitter: Sender<EventLogUpdate>,
    action_receiver: std::sync::mpsc::Receiver<CommandAction>,
    action_submitter: std::sync::mpsc::Sender<CommandAction>,
    listeners: ListenerCollection,
    connection_events: Receiver<connection::ConnectionEvent>,
    command_dispatcher: command::CommandDispatcher,
    connections: ConnectionCollection,
    policy_service: StandardPolicyService,
    dns_client: dns::DnsClient,
    isupport: ISupportBuilder,
}

impl Server
{
    pub fn new(id: ServerId,
               name: ServerName,
               rpc_receiver: Receiver<NetworkMessage>,
               to_network: Sender<EventLogUpdate>) -> Self
    {
        let (connevent_send, connevent_recv) = channel(128);
        let (action_send, action_recv) = std::sync::mpsc::channel();

        let epoch = EpochId::new(utils::now());

        Self {
            my_id: id,
            name: name,
            net: Network::new(),
            epoch: epoch,
            id_generator: ObjectIdGenerator::new(id, epoch),
            rpc_receiver: rpc_receiver,
            event_submitter: to_network,
            action_receiver: action_recv,
            action_submitter: action_send,
            listeners: ListenerCollection::new(connevent_send.clone()),
            connection_events: connevent_recv,
            connections: ConnectionCollection::new(),
            command_dispatcher: command::CommandDispatcher::new(),
            policy_service: StandardPolicyService::new(),
            dns_client: DnsClient::new(connevent_send),
            isupport: Self::build_basic_isupport(),
        }
    }

    pub fn add_listener(&mut self, address: SocketAddr, connection_type: ConnectionType)
    {
        self.listeners.add(address, connection_type);
    }

    fn submit_event(&self, id: impl Into<ObjectId>, detail: impl Into<EventDetails>)
    {
        let id = id.into();
        let detail = detail.into();
        log::trace!("Submitting new event {:?} {:?}", id, detail);
        self.event_submitter.try_send(EventLogUpdate::NewEvent(id, detail)).unwrap();
    }

    pub fn ids(&self) -> &ObjectIdGenerator
    {
        &self.id_generator
    }

    pub fn network(&self) -> &Network
    {
        &self.net
    }

    pub fn name(&self) -> &ServerName
    {
        &self.name
    }

    pub fn id(&self) -> ServerId
    {
        self.my_id
    }

    pub fn me(&self) -> LookupResult<wrapper::Server>
    {
        self.net.server(self.my_id)
    }

    pub fn command_dispatcher(&self) -> &command::CommandDispatcher
    {
        &self.command_dispatcher
    }

    pub fn add_action(&self, act: CommandAction)
    {
        self.action_submitter.send(act).unwrap();
    }

    pub fn policy(&self) -> &dyn PolicyService
    {
        &self.policy_service
    }

    pub fn find_connection(&self, id: ConnectionId) -> Option<&ClientConnection>
    {
        self.connections.get(id).ok()
    }

    fn lookup_message_source(&self, id: ObjectId) -> Result<Box<dyn messages::MessageSource + '_>, LookupError>
    {
        match id {
            ObjectId::User(u) => Ok(Box::new(self.net.user(u)?)),
            ObjectId::Server(_) => Ok(Box::new(self)), // TODO
            _ => Err(LookupError::WrongType),
        }
    }

    fn apply_event(&mut self, event: Event)
    {
        log::trace!("Applying inbound event: {:?}", event);

        let receiver = state_change_receiver::StateChangeReceiver::new();

        if let Err(e) = self.net.apply(&event, &receiver) {
            panic!("Event {:?} failed to apply: {}", event, e);
        }

        while let Ok(change) = receiver.recv.try_recv()
        {
            self.handle_network_update(change);
        }
    }

    fn build_basic_isupport() -> ISupportBuilder
    {
        let mut ret = ISupportBuilder::new();
        ret.add(ISupportEntry::simple("EXCEPTS"));
        ret.add(ISupportEntry::simple("INVEX"));
        ret.add(ISupportEntry::simple("FNC"));

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

    pub async fn run(&mut self, mut shutdown_channel: Receiver<()>)
    {
        self.event_submitter.try_send(EventLogUpdate::EpochUpdate(self.epoch)).expect("failed to submit epoch update");
        self.submit_event(self.my_id, details::NewServer{ epoch: self.epoch, name: self.name.clone(), ts: utils::now() });
        let mut check_ping_timer = time::interval(Duration::from_secs(5));

        loop {
            // Between each I/O event, see whether there are any actions we need to process synchronously
            while let Ok(act) = self.action_receiver.try_recv()
            {
                self.apply_action(act);
            }
            select! {
                res = self.connection_events.recv() => {
                    match res {
                        Some(msg) => {
                            match msg.detail {
                                NewConnection(conn) => {
                                    info!("Got new connection {:?}", msg.source);
                                    let conn = ClientConnection::new(conn);

                                    conn.send(&message::Notice::new(self, &conn.pre_client,
                                                ":*** Looking up your hostname"));
                                    self.dns_client.start_lookup(conn.id(), conn.remote_addr());
                                    self.connections.add(msg.source, conn);
                                },
                                DNSLookupFinished(hostname) => {
                                    if let Ok(conn) = self.connections.get(msg.source) {
                                        info!("DNS lookup finished for {:?}: {}/{:?}", msg.source,
                                                                                     conn.remote_addr(),
                                                                                     hostname
                                                                                     );
                                        if let Some(pc_rc) = &conn.pre_client {
                                            let mut pc = pc_rc.borrow_mut();
                                            if let Some(hostname) = hostname {
                                                conn.send(&message::Notice::new(self, &*pc,
                                                                &format!(":*** Found your hostname: {}", hostname)));

                                                pc.hostname = Some(hostname);
                                            } else {
                                                conn.send(&message::Notice::new(self, &*pc,
                                                                ":*** Couldn't look up your hostname"));
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
                                Message(m) => {
                                    info!("Got message from connection {:?}: {}", msg.source, m);

                                    if let Some(message) = ClientMessage::parse(msg.source, &m)
                                    {
                                        let processor = CommandProcessor::new(&self);
                                        processor.process_message(message).await;
                                    }
                                },
                                Error(e) => {
                                    error!("Got error from connection {:?}: {:?}", msg.source, e);
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
                        },
                        None => {
                            panic!("what to do here?");
                        }
                    }
                },
                res = self.rpc_receiver.recv() => {
                    match res {
                        Some(NetworkMessage::NewEvent(event)) =>
                        {
                            self.apply_event(event);
                        },
                        Some(NetworkMessage::ImportNetworkState(new_net)) =>
                        {
                            log::debug!("Server got state import");
                            self.net = new_net;
                        },
                        Some(NetworkMessage::ExportNetworkState(channel)) =>
                        {
                            log::debug!("Server got state export request; sending");
                            channel.send(self.net.clone()).await.or_log("Error sending network state for export");
                        },
                        None => {
                            panic!("what to do here?");
                        }
                    }
                },
                _ = check_ping_timer.tick() => {
                    self.check_pings();
                },
                _ = shutdown_channel.recv() => {
                    break;
                },
            }
        }

        let me = self.net.server(self.my_id).expect("Couldn't say I quit as I have no record of myself");
        self.submit_event(self.my_id, details::ServerQuit{ introduced_by: me.introduced_by() });
    }
}

mod command_action;
mod event_handler;
mod pings;
mod send_helpers;