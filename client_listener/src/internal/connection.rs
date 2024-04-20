use crate::internal::*;
use crate::*;

use sha1::{Digest, Sha1};
use std::{convert::TryInto, net::IpAddr};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::mpsc::{channel, Sender},
};

const SEND_QUEUE_LEN: usize = 100;

pub(crate) struct InternalConnection {
    pub id: ConnectionId,
    pub remote_addr: IpAddr,
    pub control_channel: Sender<ConnectionControlDetail>,
    pub tls_info: Option<TlsInfo>,
}

impl InternalConnection {
    pub async fn create_and_send(
        id: ConnectionId,
        stream: TcpStream,
        conntype: InternalConnectionType,
        events: Sender<InternalConnectionEventType>,
    ) -> Result<(), ConnectionError> {
        let (control_send, control_recv) = channel(SEND_QUEUE_LEN);

        let addr = stream.peer_addr()?.ip();
        let connection_type = conntype.clone();
        let mut tls_info = None;

        match connection_type {
            InternalConnectionType::Tls(tls_config) => {
                let tls_acceptor: tokio_rustls::TlsAcceptor = tls_config.into();
                match tls_acceptor.accept(stream).await {
                    Ok(mut tls_stream) => {
                        // We need to flush to make sure the tls handshake has finished, before the client
                        // info will be available
                        tls_stream.flush().await?;

                        let fingerprint = tls_stream
                            .get_ref()
                            .1
                            .peer_certificates()
                            .and_then(|c| c.first())
                            .map(|cert| {
                                let mut hasher = Sha1::new();
                                hasher.update(&cert.0);
                                hex::encode(hasher.finalize()).as_str().try_into().unwrap()
                            });

                        tls_info = Some(TlsInfo { fingerprint });

                        let conntask =
                            ConnectionTask::new(id, tls_stream, control_recv, events.clone());
                        tokio::spawn(conntask.run());
                    }
                    Err(err) => {
                        let _ = events
                            .send(InternalConnectionEventType::Event(
                                InternalConnectionEvent::ConnectionError(id, err.into()),
                            ))
                            .await;
                    }
                }
            }
            InternalConnectionType::Clear => {
                let conntask = ConnectionTask::new(id, stream, control_recv, events.clone());
                tokio::spawn(conntask.run());
            }
        }

        let conn = Self {
            id,
            remote_addr: addr,
            control_channel: control_send,
            tls_info,
        };

        if events
            .send(InternalConnectionEventType::New(conn))
            .await
            .is_err()
        {
            tracing::error!("Error sending new connection");
        };

        Ok(())
    }

    pub fn data(&self) -> ConnectionData {
        ConnectionData {
            id: self.id,
            remote_addr: self.remote_addr,
            tls_info: self.tls_info.clone(),
        }
    }
}
