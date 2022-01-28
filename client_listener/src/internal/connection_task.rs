use crate::*;
use crate::internal::*;

use tokio::{
    io::{
        AsyncRead,
        AsyncWrite,
        BufReader,
        AsyncBufReadExt,
        AsyncWriteExt,
    },
    sync::mpsc::{
        Sender,
        Receiver
    },
    select,
};


pub struct ConnectionTask<S> {
    id: ConnectionId,
    conn: S,
    control_channel: Receiver<ConnectionControlDetail>,
    event_channel: Sender<InternalConnectionEvent>
}



impl<S> ConnectionTask<S>
    where S: AsyncRead + AsyncWrite
{
    pub fn new(id: ConnectionId,
        stream: S,
        control: Receiver<ConnectionControlDetail>,
        events: Sender<InternalConnectionEvent>) -> Self
    {
        Self {
            id: id,
            conn: stream,
            control_channel: control,
            event_channel: events
        }
    }

    pub async fn run(mut self)
    {
        let (reader, mut writer) = tokio::io::split(self.conn);
        let reader = BufReader::new(reader);
        let mut lines = reader.lines();
        loop
        {
            select!
            {
                control = self.control_channel.recv() => match control
                {
                    None => { break; },
                    Some(ConnectionControlDetail::Close) => { break; },
                    Some(ConnectionControlDetail::Send(msg)) => {
                        if writer.write_all(msg.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                },
                message = lines.next_line() => match message {
                    Ok(None) => { break; },
                    Ok(Some(m)) => {
                        if self.event_channel.send(InternalConnectionEvent::Message(self.id, m)).await.is_err() {
                            log::error!("Error notifying socket message on connection {:?}", self.id);
                        }
                    }
                    Err(e) => {
                        if self.event_channel.send(InternalConnectionEvent::ConnectionError(self.id, ConnectionError::from(e))).await.is_err() {
                            log::error!("Error notifying socket error on connection {:?}", self.id);
                            return;
                        }
                    }
                }
            }
        }
        log::info!("closing {:?}", self.id);
        if self.event_channel.send(InternalConnectionEvent::ConnectionError(self.id, ConnectionError::Closed)).await.is_err() {
            log::error!("Error notifying connection closed on {:?}", self.id);
        }
    }
}
