use crate::ircd::*;
use crate::utils::*;
use async_std::{
    prelude::*,
    net::{TcpListener,SocketAddr},
    channel,
    task,
};
use std::sync::atomic::{AtomicI64,Ordering};
use futures::{select,FutureExt};
use log::error;
use super::connection;

enum ListenerControl
{
    Close,
}

#[derive(Debug)]
pub struct Listener {
    address: SocketAddr,
    control_channel: channel::Sender<ListenerControl>,
}

#[derive(Debug)]
pub struct ListenerCollection {
    last_id: AtomicI64,
    event_channel: channel::Sender<connection::ConnectionEvent>,
    listeners: Vec<Listener>
}

impl Listener
{
    pub fn new(address: SocketAddr, channel: channel::Sender<connection::ConnectionEvent>, listener_id: ServerId) -> Self
    {
        let (control_send, control_receive) = channel::unbounded::<ListenerControl>();

        task::spawn(Self::listen_and_log(channel, control_receive, address, IdGenerator::new(listener_id)));

        Self {
            address: address,
            control_channel: control_send
        }
    }

    async fn listen_and_log(
        event_channel: channel::Sender<connection::ConnectionEvent>,
        control_channel: channel::Receiver<ListenerControl>,
        address: SocketAddr,
        id_gen: IdGenerator
    )
    {
        match Self::listen_loop(event_channel, control_channel, address, id_gen).await
        {
            Ok(_) => return,
            Err(e) => error!("Listener error on {}: {}", address, e)
        }
    }

    async fn listen_loop(
        event_channel: channel::Sender<connection::ConnectionEvent>,
        mut control_channel: channel::Receiver<ListenerControl>,
        address: SocketAddr,
        id_gen: IdGenerator
    ) -> Result<(), std::io::Error>
    {
        let listener = TcpListener::bind(address).await?;
        let mut incoming = listener.incoming();

        loop
        {
            select! {
                res = incoming.next().fuse() => {
                    match res {
                        None => break,
                        Some(s) => {
                            match s {
                                Ok(stream) => {
                                    let id = id_gen.next();
                                    let conn = connection::Connection::new(id,stream,event_channel.clone());
                                    event_channel.send(connection::ConnectionEvent::new(id, conn)).await.or_log(format!("reporting new connection on {}", address));
                                },
                                Err(e) => error!("Listener error on {}: {}", address, e)
                            }
                        }
                    }
                },
                control = control_channel.next().fuse() => {
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
        task::block_on(async {
            self.control_channel.send(ListenerControl::Close).await.or_log("closing listener");
        });
    }
}

impl ListenerCollection
{
    pub fn new(channel: channel::Sender<connection::ConnectionEvent>) -> Self
    {
        Self {
            last_id: AtomicI64::new(1),
            event_channel: channel,
            listeners: Vec::new()
        }
    }

    pub fn add(&mut self, addr: SocketAddr)
    {
        self.listeners.push(Listener::new(addr, 
                            self.event_channel.clone(), 
                            self.last_id.fetch_add(1, Ordering::SeqCst)));
    }
}