use super::*;
use crate::ircd::*;
use event::*;
use irc::connection::EventDetail::*;
use irc::policy::*;
use std::{
    collections::HashMap,
};
use async_std::{
    prelude::*,
    channel,
    net::SocketAddr
};

use log::{info,error};
use async_broadcast;
use futures::{select,FutureExt};

mod connection_collection;
use connection_collection::ConnectionCollection;
use command::*;

struct ChannelPair<T>
{
    send: channel::Sender<T>,
    recv: channel::Receiver<T>,
}

pub struct Server
{
    my_id: ServerId,
    name: String,
    net: Network,
    user_idgen: UserIdGenerator,
    channel_idgen: ChannelIdGenerator,
    message_idgen: MessageIdGenerator,
    cmode_idgen: CModeIdGenerator,
    eventlog: event::EventLog,
    event_receiver: async_broadcast::Receiver<Event>,
    from_network: ChannelPair<Event>,
    to_network: channel::Sender<Event>,
    listeners: ListenerCollection,
    connection_events: channel::Receiver<connection::ConnectionEvent>,
    command_dispatcher: command::CommandDispatcher,
    connections: ConnectionCollection,
    policy_service: StandardPolicyService,
}

impl Server
{
    pub fn new(id: ServerId, name: String, to_network: channel::Sender<Event>) -> Self
    {
        let (connevent_send, connevent_recv) = channel::unbounded::<connection::ConnectionEvent>();
        let (net_send, net_recv) = channel::unbounded::<Event>();
        let mut eventlog = event::EventLog::new(EventIdGenerator::new(id, 1));
        let event_receiver = eventlog.attach();

        Self {
            my_id: id,
            name: name,
            net: Network::new(),
            user_idgen: UserIdGenerator::new(id, 1),
            channel_idgen: ChannelIdGenerator::new(id, 1),
            message_idgen: MessageIdGenerator::new(id, 1),
            cmode_idgen: CModeIdGenerator::new(id, 1),
            eventlog: eventlog,
            event_receiver: event_receiver,
            from_network: ChannelPair { send: net_send, recv: net_recv },
            to_network: to_network,
            listeners: ListenerCollection::new(connevent_send),
            connection_events: connevent_recv,
            connections: ConnectionCollection::new(),
            command_dispatcher: command::CommandDispatcher::new(),
            policy_service: StandardPolicyService::new(),
        }
    }

    pub fn add_listener(&mut self, address: SocketAddr)
    {
        self.listeners.add(address);
    }

    fn submit_event(&mut self, event: Event)
    {
        self.eventlog.add(event.clone());
        self.to_network.try_send(event).unwrap();
    }

    pub fn create_event<T: event::DetailType>(&self, target: <T as DetailType>::Target, details: T) -> event::Event
    {
        self.eventlog.create(target.into(), details.into())
    }

    pub fn get_event_sender(&self) -> channel::Sender<Event>
    {
        self.from_network.send.clone()
    }

    pub fn next_user_id(&self) -> UserId
    {
        self.user_idgen.next()
    }

    pub fn next_channel_id(&self) -> ChannelId
    {
        self.channel_idgen.next()
    }

    pub fn next_cmode_id(&self) -> CModeId
    {
        self.cmode_idgen.next()
    }

    pub fn next_message_id(&self) -> MessageId
    {
        self.message_idgen.next()
    }

    pub fn network(&self) -> &Network
    {
        &self.net
    }

    pub fn name(&self) -> &str
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

    pub async fn run(&mut self)
    {
        loop {
            select! {
                res = self.connection_events.next().fuse() => {
                    match res {
                        Some(msg) => {
                            match msg.detail {
                                NewConnection(conn) => {
                                    info!("Got new connection {:?}", msg.source);
                                    self.connections.add(msg.source, ClientConnection::new(conn));
                                },
                                Message(m) => { 
                                    info!("Got message from connection {:?}: {}", msg.source, m);

                                    if let Some(message) = ClientMessage::parse(msg.source, &m)
                                    {
                                        let processor = CommandProcessor::new(&self);
                                        let actions = processor.process_message(message).await;
                                        
                                        for action in actions
                                        {
                                            self.apply_action(action)
                                        }
                                    }
                                },
                                Error(e) => {
                                    error!("Got error from connection {:?}: {:?}", msg.source, e);
                                    if let Ok(conn) = self.connections.get(msg.source) {
                                        if let Some(userid) = conn.user_id {
                                            self.apply_action(CommandAction::StateChange(
                                                self.eventlog.create(userid, EventDetails::UserQuit(details::UserQuit {
                                                    message: format!("I/O error: {}", e)
                                                }))
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
                            // Notify handlers run before it's applied to the network state. If it's a
                            // deletion event of some sort, the handler needs to know what was there before
                            // in order to know who to notify.
                            match self.net.validate(&event) {
                                Ok(_) => {
                                    self.handle_event(&event);
                                    if let Err(e) = self.net.apply(&event) {
                                        panic!("Event validated but failed to apply: {}", e);
                                    }
                                },
                                Err(e) => {
                                    error!("Event failed validation: {}", e);
                                    self.handle_event_failure(&event, &e);
                                }
                            }
                        },
                        None => { 
                            panic!("what to do here?");
                        }
                    }
                },
                res = self.from_network.recv.next().fuse() => {
                    match res {
                        Some(event) => self.eventlog.add(event),
                        None => {
                            panic!("What to do here?");
                        }
                    }
                }
            }
        }
    }
}

mod command_action;
mod event_handler;
mod event_failure;