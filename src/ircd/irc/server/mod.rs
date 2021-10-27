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
use futures::{select,FutureExt};

mod connection_collection;
use connection_collection::ConnectionCollection;
use command::*;

pub struct Server
{
    my_id: ServerId,
    name: String,
    net: Network,
    user_idgen: UserIdGenerator,
    channel_idgen: ChannelIdGenerator,
    message_idgen: MessageIdGenerator,
    cmode_idgen: CModeIdGenerator,
    event_receiver: channel::Receiver<Event>,
    event_submitter: channel::Sender<(ObjectId,EventDetails)>,
    listeners: ListenerCollection,
    connection_events: channel::Receiver<connection::ConnectionEvent>,
    command_dispatcher: command::CommandDispatcher,
    connections: ConnectionCollection,
    policy_service: StandardPolicyService,
}

impl Server
{
    pub fn new(id: ServerId,
               name: String,
               from_network: channel::Receiver<Event>,
               to_network: channel::Sender<(ObjectId, EventDetails)>) -> Self
    {
        let (connevent_send, connevent_recv) = channel::unbounded::<connection::ConnectionEvent>();

        Self {
            my_id: id,
            name: name,
            net: Network::new(),
            user_idgen: UserIdGenerator::new(id, 1),
            channel_idgen: ChannelIdGenerator::new(id, 1),
            message_idgen: MessageIdGenerator::new(id, 1),
            cmode_idgen: CModeIdGenerator::new(id, 1),
            event_receiver: from_network,
            event_submitter: to_network,
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

    fn submit_event(&self, id: impl Into<ObjectId>, detail: impl Into<EventDetails>)
    {
        self.event_submitter.try_send((id.into(), detail.into())).unwrap();
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
                        },
                        None => { 
                            panic!("what to do here?");
                        }
                    }
                },
            }
        }
    }
}

mod command_action;
mod event_handler;
mod event_failure;