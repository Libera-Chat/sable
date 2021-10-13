use crate::ircd::*;
use crate::utils::*;
use std::sync::Arc;
use async_std::{
    prelude::*,
    net::TcpStream,
    io::BufReader,
    channel,
    task
};
use futures::{select,FutureExt};
use log::info;
use thiserror::Error;

static SEND_QUEUE_LEN:usize = 100;

#[derive(Debug)]
pub struct Connection {
    pub id: ConnectionId,
    control_channel: channel::Sender<ConnectionControl>,
    conn: Arc<TcpStream>
}

#[derive(Error,Debug)]
pub enum ConnectionError {
    #[error("Connection closed")]
    Closed,
    #[error("I/O Error: {0}")]
    IoError(#[from]std::io::Error),
    #[error("Couldn't send to control channel: {0}")]
    ControlSendError(#[from] channel::SendError<ConnectionControl>),
}

#[derive(Debug)]
pub enum EventDetail {
    NewConnection(Connection),
    Message(String),
    Error(ConnectionError),
}

#[derive(Debug)]
pub struct ConnectionEvent {
    pub source: ConnectionId,
    pub detail: EventDetail
}

struct ConnectionTask {
    id: ConnectionId,
    conn: Arc<TcpStream>,
    control_channel: channel::Receiver<ConnectionControl>,
    event_channel: channel::Sender<ConnectionEvent>
}

pub enum ConnectionControl {
    Close
}

impl Connection
{
    pub fn new(id: ConnectionId, stream: TcpStream, events: channel::Sender<ConnectionEvent>) -> Self
    {
        let (control_send, control_recv) = channel::bounded(SEND_QUEUE_LEN);
        let stream = Arc::new(stream);

        let conntask = ConnectionTask::new(id, Arc::clone(&stream), control_recv, events);
        task::spawn(conntask.run());

        Self {
            id: id,
            control_channel: control_send,
            conn: stream
        }
    }

    pub fn id(&self) -> ConnectionId
    {
        self.id
    }

    pub async fn close(&self) -> Result<(), ConnectionError>
    {
        self.control_channel.send(ConnectionControl::Close).await?;
        Ok(())
    }

    pub async fn send(&self, message: &str) -> Result<(), ConnectionError>
    {
        let mut conn = &*self.conn;
        conn.write_all(message.as_bytes()).await?;
        Ok(())
    }
}

impl Drop for Connection
{
    fn drop(&mut self)
    {
        info!("Dropping connection {:?}", self.id);
    }
}

impl ConnectionEvent
{
    pub fn message(id: ConnectionId, message: String) -> Self
    {
        Self { source: id, detail: EventDetail::Message(message) }
    }

    pub fn error(id: ConnectionId, error: ConnectionError) -> Self
    {
        Self { source: id, detail: EventDetail::Error(error) }
    }

    pub fn new(id: ConnectionId, conn: Connection) -> Self
    {
        Self { source: id, detail: EventDetail::NewConnection(conn) }
    }
}

impl ConnectionTask
{
    fn new(id: ConnectionId, 
        stream: Arc<TcpStream>, 
        control: channel::Receiver<ConnectionControl>,
        events: channel::Sender<ConnectionEvent>) -> Self
    {
        Self {
            id: id,
            conn: stream,
            control_channel: control,
            event_channel: events
        }
    }

    async fn run(mut self)
    {
        let reader = BufReader::new(&*self.conn);
        let mut lines = reader.lines();
        loop
        {
            select!
            {
                control = self.control_channel.next().fuse() => match control {
                    None => { info!("a"); break; },
                    Some(ConnectionControl::Close) => { info!("b"); break; },
                },
                message = lines.next().fuse() => match message {
                    None => {info!("c"); break; },
                    Some(Ok(m)) => {
                        info!("got message");
                        self.event_channel.send(ConnectionEvent::message(self.id, m)).await.or_log("notifying socket message");
                    }
                    Some(Err(e)) => {
                        info!("got error");
                        self.event_channel.send(ConnectionEvent::error(self.id, ConnectionError::from(e))).await.or_log("notifying socket error");
                        return;
                    }
                }
            }
        }
        info!("closing {:?}", self.id);
        self.event_channel.send(ConnectionEvent::error(self.id, ConnectionError::Closed)).await.or_log("notifying connection closed");
    }
}
