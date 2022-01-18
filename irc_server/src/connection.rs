use irc_network::*;
use crate::utils::*;
use crate::errors::*;
use tokio::{
    net::{
        TcpStream,
    },
    io::{
        AsyncRead,
        AsyncWrite,
        BufReader,
        AsyncBufReadExt,
        AsyncWriteExt,
    },
    sync::mpsc::{
        Sender,
        Receiver,
        channel
    },
    task,
    select,
};
use tokio_rustls::TlsAcceptor;
use log::info;

use std::net::IpAddr;
use std::sync::Arc;

static SEND_QUEUE_LEN:usize = 100;

#[derive(Debug)]
pub struct Connection {
    pub id: ConnectionId,
    pub remote_addr: IpAddr,
    control_channel: Sender<ConnectionControl>,
    send_channel: Sender<String>,
    connection_type: ConnectionType,
}

#[derive(Debug)]
pub enum EventDetail {
    NewConnection(Connection),
    Message(String),
    Error(ConnectionError),
    DNSLookupFinished(Option<Hostname>),
}

#[derive(Debug)]
pub struct ConnectionEvent {
    pub source: ConnectionId,
    pub detail: EventDetail
}

struct ConnectionTask<S> {
    id: ConnectionId,
    conn: S,
    control_channel: Receiver<ConnectionControl>,
    send_channel: Receiver<String>,
    event_channel: Sender<ConnectionEvent>
}

pub enum ConnectionControl {
    Close
}

#[derive(Clone)]
pub enum ConnectionType
{
    Clear,
    Tls(Arc<rustls::ServerConfig>)
}

impl std::fmt::Debug for ConnectionType
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        match self {
            Self::Clear => "<clear>".fmt(f),
            Self::Tls(_) => "<tls>".fmt(f)
        }
    }
}

impl Connection
{
    pub fn new(id: ConnectionId, stream: TcpStream, conntype: ConnectionType, events: Sender<ConnectionEvent>) -> Result<Self,ConnectionError>
    {
        let (control_send, control_recv) = channel(SEND_QUEUE_LEN);
        let (send_send, send_recv) = channel(SEND_QUEUE_LEN);

        let addr = stream.peer_addr()?.ip();
        let connection_type = conntype.clone();

        task::spawn(async move {
            match connection_type {
                ConnectionType::Tls(config) => {
                    let acceptor: TlsAcceptor = config.into();
                    match acceptor.accept(stream).await
                    {
                        Ok(tls_stream) => {
                            let conntask = ConnectionTask::new(id, tls_stream, control_recv, send_recv, events.clone());
                            conntask.run().await;
                        }
                        Err(err) => {
                            let _ = events.send(ConnectionEvent {
                                source: id,
                                detail: EventDetail::Error(ConnectionError::IoError(err))
                            }).await;
                        }
                    }
                }
                ConnectionType::Clear => {
                    let conntask = ConnectionTask::new(id, stream, control_recv, send_recv, events.clone());
                    conntask.run().await;
                }
            }
        });

        Ok(Self {
            id: id,
            remote_addr: addr,
            control_channel: control_send,
            send_channel: send_send,
            connection_type: conntype,
        })
    }

    pub fn id(&self) -> ConnectionId
    {
        self.id
    }

    pub fn close(&self) -> Result<(), ConnectionError>
    {
        self.control_channel.try_send(ConnectionControl::Close)?;
        Ok(())
    }

    pub fn send(&self, message: &str) -> Result<(), ConnectionError>
    {
        self.send_channel.try_send(message.to_string())?;
        Ok(())
    }

    pub fn is_tls(&self) -> bool
    {
        match self.connection_type
        {
            ConnectionType::Clear => false,
            ConnectionType::Tls(_) => true,
        }
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

impl<S> ConnectionTask<S>
    where S: AsyncRead + AsyncWrite
{
    fn new(id: ConnectionId, 
        stream: S,
        control: Receiver<ConnectionControl>,
        send: Receiver<String>,
        events: Sender<ConnectionEvent>) -> Self
    {
        Self {
            id: id,
            conn: stream,
            control_channel: control,
            send_channel: send,
            event_channel: events
        }
    }

    async fn run(mut self)
    {
        let (reader, mut writer) = tokio::io::split(self.conn);
        let reader = BufReader::new(reader);
        let mut lines = reader.lines();
        loop
        {
            select!
            {
                control = self.control_channel.recv() => match control {
                    None => { break; },
                    Some(ConnectionControl::Close) => { break; },
                },
                message = self.send_channel.recv() => match message {
                    None => break,
                    Some(msg) => {
                        if writer.write_all(msg.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                },
                message = lines.next_line() => match message {
                    Ok(None) => { break; },
                    Ok(Some(m)) => {
                        self.event_channel.send(ConnectionEvent::message(self.id, m)).await.or_log("notifying socket message");
                    }
                    Err(e) => {
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
