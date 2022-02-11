use crate::*;
use crate::internal::*;

use tokio::{
    net::TcpListener,
    sync::mpsc::{
        Sender,
        Receiver,
        channel
    },
    select
};

use std::net::SocketAddr;

pub(crate) struct Listener {
    //address: SocketAddr,
    pub _id: ListenerId,
    pub control_channel: Sender<ListenerControlDetail>,
//    connection_type: InternalConnectionType,
//    tls_config: Option<Arc<ServerConfig>>,
}

impl Listener
{
    pub fn new(listener_id: ListenerId,
               address: SocketAddr,
               connection_type: InternalConnectionType,
               event_channel: Sender<InternalConnectionEventType>,
            ) -> Self
    {
        let (control_send, control_receive) = channel(128);

        tokio::spawn(Self::listen_and_log(event_channel,
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
    )
    {
        if let Err(e) =
            match Self::listen_loop(event_channel.clone(), control_channel, address, connection_type, listener_id).await
            {
                Ok(_) => return,
                Err(e) => event_channel.send(InternalConnectionEventType::Event(InternalConnectionEvent::ListenerError(listener_id, e.into()))).await,
            }
        {
            tracing::error!("Error in listener loop: {}", e);
        }
    }

    async fn listen_loop(
        event_channel: Sender<InternalConnectionEventType>,
        mut control_channel: Receiver<ListenerControlDetail>,
        address: SocketAddr,
        connection_type: InternalConnectionType,
        listener_id: ListenerId,
    ) -> Result<(), std::io::Error>
    {
        let listener = TcpListener::bind(address).await?;
        let id_gen = ConnectionIdGenerator::new(listener_id, 1);

        loop
        {
            select! {
                res = listener.accept() => {
                    let event = match res {
                        Ok((stream,_)) =>
                        {
                            let id = id_gen.next();
                            match InternalConnection::new(id, stream, connection_type.clone(), event_channel.clone())
                            {
                                Ok(conn) => {
                                    if event_channel.send(InternalConnectionEventType::New(conn)).await.is_err()
                                    {
                                        InternalConnectionEvent::CommunicationError
                                    }
                                    else
                                    {
                                        continue;
                                    }
                                },
                                Err(e) => InternalConnectionEvent::ConnectionError(id, e)
                            }
                        },
                        Err(e) => InternalConnectionEvent::ListenerError(listener_id, e.into())
                    };
                    if let Err(e) = event_channel.send(InternalConnectionEventType::Event(event)).await
                    {
                        tracing::error!("Error sending connection event: {}", e);
                    }
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

impl Drop for Listener
{
    fn drop(&mut self)
    {
        if let Err(e) = self.control_channel.try_send(ListenerControlDetail::Close)
        {
            tracing::error!("Error closing dropped listener: {}", e);
        }
    }
}
