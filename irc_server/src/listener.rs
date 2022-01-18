use irc_network::*;
use crate::utils::*;
use tokio::{
    net::TcpListener,
    sync::mpsc::{
        Sender,
        Receiver,
        channel
    },
    select
};
use log::error;
use super::connection;
use std::net::SocketAddr;
use super::connection::ConnectionType;

#[derive(Debug)]
enum ListenerControl
{
    Close,
}

pub struct Listener {
    //address: SocketAddr,
    control_channel: Sender<ListenerControl>,
    //connection_type: ConnectionType,
}

pub struct ListenerCollection {
    id_gen: ListenerIdGenerator,
    event_channel: Sender<connection::ConnectionEvent>,
    listeners: Vec<Listener>
}

impl Listener
{
    pub fn new(address: SocketAddr, connection_type: ConnectionType, event_channel: Sender<connection::ConnectionEvent>, listener_id: ListenerId) -> Self
    {
        let (control_send, control_receive) = channel(128);

        tokio::spawn(Self::listen_and_log(event_channel,
                                          control_receive,
                                          address,
                                          connection_type.clone(),
                                          ConnectionIdGenerator::new(listener_id, 1)
                                        ));

        Self {
            //address: address,
            control_channel: control_send,
            //connection_type: connection_type
        }
    }

    async fn listen_and_log(
        event_channel: Sender<connection::ConnectionEvent>,
        control_channel: Receiver<ListenerControl>,
        address: SocketAddr,
        connection_type: ConnectionType,
        id_gen: ConnectionIdGenerator
    )
    {
        match Self::listen_loop(event_channel, control_channel, address, connection_type, id_gen).await
        {
            Ok(_) => return,
            Err(e) => error!("Listener error on {}: {}", address, e)
        }
    }

    async fn listen_loop(
        event_channel: Sender<connection::ConnectionEvent>,
        mut control_channel: Receiver<ListenerControl>,
        address: SocketAddr,
        connection_type: ConnectionType,
        id_gen: ConnectionIdGenerator
    ) -> Result<(), std::io::Error>
    {
        let listener = TcpListener::bind(address).await?;

        loop
        {
            select! {
                res = listener.accept() => {
                    match res {
                        Ok((stream,_)) => {
                            let id = id_gen.next();
                            match connection::Connection::new(id, stream, connection_type.clone(), event_channel.clone()) {
                                Ok(conn) => event_channel.send(connection::ConnectionEvent::new(id, conn)).await.or_log(format!("reporting new connection on {}", address)),
                                Err(e) => error!("Error opening connection for {}: {}", address, e)
                            }
                        },
                        Err(e) => error!("Listener error on {}: {}", address, e)
                    }
                },
                control = control_channel.recv() => {
                    match control {
                        None => break,
                        Some(ListenerControl::Close) => break
                    }
                }
            }
        }

        Ok(())
    }
}

impl Drop for Listener
{
    fn drop(&mut self)
    {
        self.control_channel.try_send(ListenerControl::Close).or_log("closing listener");
    }
}

impl ListenerCollection
{
    pub fn new(channel: Sender<connection::ConnectionEvent>) -> Self
    {
        Self {
            id_gen: ListenerIdGenerator::new(1),
            event_channel: channel,
            listeners: Vec::new()
        }
    }

    pub fn add(&mut self, addr: SocketAddr, connection_type: ConnectionType)
    {
        self.listeners.push(Listener::new(addr, 
                            connection_type,
                            self.event_channel.clone(), 
                            self.id_gen.next()));
    }
}