use crate::internal::*;
use crate::*;

use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    select,
    sync::mpsc::{Receiver, Sender},
};

pub(crate) struct ConnectionTask<S> {
    id: ConnectionId,
    conn: S,
    control_channel: Receiver<ConnectionControlDetail>,
    event_channel: Sender<InternalConnectionEventType>,
}

impl<S> ConnectionTask<S>
where
    S: AsyncRead + AsyncWrite,
{
    pub fn new(
        id: ConnectionId,
        stream: S,
        control_channel: Receiver<ConnectionControlDetail>,
        event_channel: Sender<InternalConnectionEventType>,
    ) -> Self {
        Self {
            id,
            conn: stream,
            control_channel,
            event_channel,
        }
    }

    pub async fn run(mut self) {
        let (reader, mut writer) = tokio::io::split(self.conn);
        let reader = BufReader::new(reader);
        let mut lines = reader.lines();
        loop {
            select! {
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
                        if m.len() as u64 > crate::MAX_MSG_SIZE { // in bytes
                            if self.event_channel.send(InternalConnectionEventType::Event(InternalConnectionEvent::ConnectionError(self.id, ConnectionError::InputLineTooLong))).await.is_err() {
                            tracing::error!("Error notifying socket error on connection {:?}", self.id);
                            }
                        } else if self.event_channel.send(InternalConnectionEventType::Event(InternalConnectionEvent::Message(self.id, m))).await.is_err() {
                            tracing::error!("Error notifying socket message on connection {:?}", self.id);
                        }
                    }
                    Err(e) => {
                        if self.event_channel.send(InternalConnectionEventType::Event(InternalConnectionEvent::ConnectionError(self.id, ConnectionError::from(e)))).await.is_err() {
                            tracing::error!("Error notifying socket error on connection {:?}", self.id);
                            return;
                        }
                    }
                }
            }
        }
        if self
            .event_channel
            .send(InternalConnectionEventType::Event(
                InternalConnectionEvent::ConnectionError(self.id, ConnectionError::Closed),
            ))
            .await
            .is_err()
        {
            tracing::error!("Error notifying connection closed on {:?}", self.id);
        }
    }
}
