use crate::ircd::*;
use super::*;
use std::{
    sync::Arc,
    collections::HashMap,
};
use async_std::{
    prelude::*,
    channel,
    net::SocketAddr
};
use crate::ircd::irc::connection::EventDetail::*;
use log::{info,error};


pub struct Server
{
    my_id: ServerId,
    name: String,
    net: Network,
    id_gen: Arc<IdGenerator>,
    eventlog: event::EventLog,
    event_offset: event::EventOffset,
    listeners: ListenerCollection,
    connection_events: channel::Receiver<connection::ConnectionEvent>,
    client_connections: HashMap<Id, ClientConnection>,
}

impl Server
{
    pub fn new(id: ServerId, name: String) -> Self
    {
        let (connevent_send, connevent_recv) = channel::unbounded::<connection::ConnectionEvent>();
        let idgen = Arc::new(IdGenerator::new(id));
        let eventlog = event::EventLog::new(Arc::clone(&idgen));
        let event_offset = eventlog.get_offset();

        Self {
            my_id: id,
            name: name,
            net: Network::new(),
            id_gen: Arc::clone(&idgen),
            eventlog: eventlog,
            event_offset: event_offset,
            listeners: ListenerCollection::new(connevent_send),
            connection_events: connevent_recv,
            client_connections: HashMap::new(),
        }
    }

    pub fn add_listener(&mut self, address: SocketAddr)
    {
        self.listeners.add(address);
    }

    pub fn create_event(&self, target: Id, details: event::EventDetails) -> event::Event
    {
        self.eventlog.create(target, details)
    }

    pub fn create_id(&self) -> Id
    {
        self.id_gen.next()
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

    pub fn find_connection(&self, id: Id) -> Option<&ClientConnection>
    {
        self.client_connections.get(&id)
    }

    pub async fn run(&mut self)
    {
        while let Some(msg) = self.connection_events.next().await {
            match msg.detail {
                NewConnection(conn) => {
                    info!("Got new connection {:?}", msg.source);
                    self.client_connections.insert(msg.source, ClientConnection::new(conn));
                },
                Message(m) => { 
                    info!("Got message from connection {:?}: {}", msg.source, m);

                    if let Some(message) = ClientMessage::parse(msg.source, &m)
                    {
                        let mut processor = CommandProcessor::new(&self);
                        processor.process_message(message).await;
                        
                        for action in processor.actions()
                        {
                            self.apply_action(action)
                        }

                        while let Some(event) = self.eventlog.next_for(&mut self.event_offset)
                        {
                            self.net.apply(event);
                        }
                    }
                },
                Error(e) => {
                    error!("Got error from connection {:?}: {:?}", msg.source, e);
                    self.client_connections.remove(&msg.source);
                }
            }
        }
    }

    pub fn apply_action(&mut self, action: CommandAction)
    {
        match action {
            CommandAction::RegisterClient(id) => {
                if let Some(conn) = self.client_connections.get_mut(&id)
                {
                    let pre_client = match &mut conn.pre_client {
                        None => { return; },
                        Some(pc) => pc
                    };
                    let new_user_id = self.id_gen.next();
                    let register_event = self.eventlog.create(
                                                    new_user_id, 
                                                    event::EventDetails::NewUser(event::details::NewUser {
                                                        nickname: pre_client.nick.replace(None).unwrap(),
                                                        username: pre_client.user.replace(None).unwrap(),
                                                        visible_hostname: "example.com".to_string()
                                                    })
                                                );
                    self.eventlog.add(register_event);
                    conn.pre_client = None;
                    conn.user_id = Some(new_user_id);
                }
            },
            CommandAction::StateChange(event) => {
                self.eventlog.add(event);
            }
        }
    }
}
