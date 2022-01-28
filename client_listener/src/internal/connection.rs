use crate::*;
use crate::internal::*;

use std::net::IpAddr;
use tokio::{
    net::{
        TcpStream,
    },
    sync::mpsc::{
        Sender,
        channel,
    },
    task,
};

const SEND_QUEUE_LEN: usize = 100;

pub struct InternalConnection {
    pub id: ConnectionId,
    pub remote_addr: IpAddr,
    pub control_channel: Sender<ConnectionControlDetail>,
    pub connection_type: InternalConnectionType,
}


impl InternalConnection
{
    pub fn new(id: ConnectionId,
                      stream: TcpStream,
                      conntype: InternalConnectionType,
                      events: Sender<InternalConnectionEvent>)
                    -> Result<Self,ConnectionError>
    {
        let (control_send, control_recv) = channel(SEND_QUEUE_LEN);

        let addr = stream.peer_addr()?.ip();
        let connection_type = conntype.clone();

        task::spawn(async move {
            match connection_type {
                InternalConnectionType::Tls(tls_config) => {
                    let tls_acceptor: tokio_rustls::TlsAcceptor = tls_config.into();
                    match tls_acceptor.accept(stream).await
                    {
                        Ok(tls_stream) => {
                            let conntask = ConnectionTask::new(id, tls_stream, control_recv, events.clone());
                            conntask.run().await;
                        }
                        Err(err) => {
                            let _ = events.send(InternalConnectionEvent::ConnectionError(id, err.into())).await;
                        }
                    }
                }
                InternalConnectionType::Clear => {
                    let conntask = ConnectionTask::new(id, stream, control_recv, events.clone());
                    conntask.run().await;
                }
            }
        });

        Ok(Self {
            id: id,
            remote_addr: addr,
            control_channel: control_send,
            connection_type: conntype,
        })
    }

    pub fn data(&self) -> ConnectionData
    {
        ConnectionData {
            id: self.id,
            endpoint: self.remote_addr,
            conn_type: self.connection_type.to_pub()
        }
    }
}
