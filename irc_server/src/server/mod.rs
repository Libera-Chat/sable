use super::*;
use irc_network::*;
use event::*;
use crate::connection::EventDetail::*;
use crate::policy::*;
use std::{
    collections::HashMap,
    sync::mpsc,
};
use async_std::{
    prelude::*,
    channel,
    net::SocketAddr
};

use log::{info,error};
use futures::{select,FutureExt};

mod connection_collection;
use connection_collection::ConnectionCollection;
use command::*;

pub enum EventLogUpdate
{
    NewEvent(ObjectId, EventDetails),
    EpochUpdate(EpochId)
}

pub struct Server
{
    my_id: ServerId,
    name: ServerName,
    net: Network,
    id_generator: ObjectIdGenerator,
    event_receiver: channel::Receiver<Event>,
    event_submitter: channel::Sender<EventLogUpdate>,
    action_receiver: mpsc::Receiver<CommandAction>,
    action_submitter: mpsc::Sender<CommandAction>,
    listeners: ListenerCollection,
    connection_events: channel::Receiver<connection::ConnectionEvent>,
    command_dispatcher: command::CommandDispatcher,
    connections: ConnectionCollection,
    policy_service: StandardPolicyService,
    dns_client: dns::DnsClient,
}

impl Server
{
    pub fn new(id: ServerId,
               name: ServerName,
               from_network: channel::Receiver<Event>,
               to_network: channel::Sender<EventLogUpdate>) -> Self
    {
        let (connevent_send, connevent_recv) = channel::unbounded::<connection::ConnectionEvent>();
        let (action_send, action_recv) = mpsc::channel();

        Self {
            my_id: id,
            name: name,
            net: Network::new(),
            id_generator: ObjectIdGenerator::new(id, EpochId::new(0)),
            event_receiver: from_network,
            event_submitter: to_network,
            action_receiver: action_recv,
            action_submitter: action_send,
            listeners: ListenerCollection::new(connevent_send.clone()),
            connection_events: connevent_recv,
            connections: ConnectionCollection::new(),
            command_dispatcher: command::CommandDispatcher::new(),
            policy_service: StandardPolicyService::new(),
            dns_client: DnsClient::new(connevent_send),
        }
    }

    pub fn add_listener(&mut self, address: SocketAddr)
    {
        self.listeners.add(address);
    }

    fn submit_event(&self, id: impl Into<ObjectId>, detail: impl Into<EventDetails>)
    {
        self.event_submitter.try_send(EventLogUpdate::NewEvent(id.into(), detail.into())).unwrap();
    }

    pub fn next_user_id(&self) -> UserId
    {
        self.id_generator.next_user()
    }

    pub fn next_channel_id(&self) -> ChannelId
    {
        self.id_generator.next_channel()
    }

    pub fn next_cmode_id(&self) -> CModeId
    {
        self.id_generator.next_cmode()
    }

    pub fn next_message_id(&self) -> MessageId
    {
        self.id_generator.next_message()
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
        log::debug!("Applying inbound event: {:?}", event);

        // Separate pre_handle and post-handle: some event handlers (e.g. for events that destroy
        // objects) need to run before the event is applied (e.g. to access the state that's about to
        // be removed), while most are easier to write if they run afterwards and can immediately see
        // the changes already applied.
        match self.net.validate(event.target, &event.details) {
            Ok(_) => {
                self.pre_handle_event(&event);
                if let Err(e) = self.net.apply(&event) {
                    panic!("Event validated but failed to apply: {}", e);
                }
                self.post_handle_event(&event);
            },
            Err(e) => {
                error!("Event failed validation: {}", e);
                self.handle_event_failure(&event, &e);
            }
        }
    }

    pub async fn run(&mut self, mut shutdown_channel: channel::Receiver<()>)
    {
        // Pump inbound network messages first to catch up to current state, then introduce ourselves
        // Keep track of events sent previously by our server ID, so we can update the epoch and
        // not collide.
        let mut my_epoch = EpochId::new(0);

        while let Ok(event) = self.event_receiver.try_recv()
        {
            info!("Processing bootstrap event {:?}", event.id);
            if event.id.server() == self.my_id && event.id.epoch() > my_epoch
            {
                info!("Updating epoch: {:?}", event.id);
                my_epoch = event.id.epoch();
            }
            self.apply_event(event);
        }

        // Now we know what's happened before, we can set ourselves a new epoch to avoid ID collisions,
        // and introduce ourselves appropriately
        let new_epoch = my_epoch.next();

        self.id_generator.set_epoch(new_epoch);
        self.event_submitter.try_send(EventLogUpdate::EpochUpdate(new_epoch)).expect("failed to submit epoch update");
        self.submit_event(self.my_id, details::NewServer{name: self.name.clone(), ts: utils::now() });

        loop {
            // Between each I/O event, see whether there are any actions we need to process synchronously
            while let Ok(act) = self.action_receiver.try_recv()
            {
                self.apply_action(act);
            }
            select! {
                res = self.connection_events.next().fuse() => {
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
                                                let no_hostname = Hostname::new(conn.remote_addr().to_string());
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
                res = self.event_receiver.next().fuse() => {
                    match res {
                        Some(event) => {
                            self.apply_event(event);
                        },
                        None => { 
                            panic!("what to do here?");
                        }
                    }
                },
                _ = shutdown_channel.next().fuse() => {
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
mod event_failure;