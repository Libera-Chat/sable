use crate::internal::*;
use crate::*;

use tokio::{
    net::TcpListener,
    select,
    sync::mpsc::{channel, Receiver, Sender},
};

use std::net::SocketAddr;

pub(crate) struct Listener {
    //address: SocketAddr,
    pub _id: ListenerId,
    pub control_channel: Sender<ListenerControlDetail>,
    //    connection_type: InternalConnectionType,
    //    tls_config: Option<Arc<ServerConfig>>,
}

impl Listener {
    pub fn new(
        listener_id: ListenerId,
        address: SocketAddr,
        connection_type: InternalConnectionType,
        event_channel: Sender<InternalConnectionEventType>,
    ) -> Self {
        let (control_send, control_receive) = channel(128);

        tokio::spawn(Self::listen_and_log(
            event_channel,
            control_receive,
            address,
            connection_type,
            listener_id,
        ));

        Self {
            _id: listener_id,
            control_channel: control_send,
            //            connection_type: connection_type,
            //            tls_config: tls_config,
        }
    }

    async fn listen_and_log(
        event_channel: Sender<InternalConnectionEventType>,
        control_channel: Receiver<ListenerControlDetail>,
        address: SocketAddr,
        connection_type: InternalConnectionType,
        listener_id: ListenerId,
    ) {
        if let Err(e) = match Self::listen_loop(
            event_channel.clone(),
            control_channel,
            address,
            connection_type,
            listener_id,
        )
        .await
        {
            Ok(_) => return,
            Err(e) => {
                event_channel
                    .send(InternalConnectionEventType::Event(
                        InternalConnectionEvent::ListenerError(listener_id, e.into()),
                    ))
                    .await
            }
        } {
            tracing::error!("Error in listener loop: {}", e);
        }
    }

    async fn listen_loop(
        event_channel: Sender<InternalConnectionEventType>,
        mut control_channel: Receiver<ListenerControlDetail>,
        address: SocketAddr,
        connection_type: InternalConnectionType,
        listener_id: ListenerId,
    ) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind(address).await?;
        let id_gen = ConnectionIdGenerator::new(listener_id, 1);

        loop {
            select! {
                res = listener.accept() => {
                    match res {
                        Ok((stream,_)) =>
                        {
                            let id = id_gen.next();
                            let connection_type = connection_type.clone();
                            let event_channel = event_channel.clone();

                            tokio::spawn(async move {
                                if let Err(e) = InternalConnection::create_and_send(id, stream, connection_type, event_channel).await
                                {
                                    tracing::error!("Error creating connection: {}", e);
                                }
                            });
                            continue
                        },
                        Err(e) =>
                        {
                            tracing::error!("Error accepting connection: {}", e);
                        }
                    };
                },
                control = control_channel.recv() => {
                    match control {
                        None => break,
                        Some(ListenerControlDetail::Close) => break,
                        _ => continue,
                    }
                }
            }
        }

        Ok(())
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        if let Err(e) = self.control_channel.try_send(ListenerControlDetail::Close) {
            tracing::error!("Error closing dropped listener: {}", e);
        }
    }
}
