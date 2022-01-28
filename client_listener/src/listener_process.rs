use crate::*;
use internal::*;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::{
    sync::mpsc::{
        channel
    },
    select,
};
use tokio_unix_ipc::{
    Sender as IpcSender,
    Receiver as IpcReceiver,
};

pub struct ListenerProcess
{
    control_receiver: IpcReceiver<ControlMessage>,
    event_sender: IpcSender<InternalConnectionEvent>,
    tls_config: Option<Arc<rustls::ServerConfig>>,

    listeners: HashMap<ListenerId, Listener>,
    connections: HashMap<ConnectionId, InternalConnection>
}

impl ListenerProcess
{
    pub fn new(control_receiver: IpcReceiver<ControlMessage>, event_sender: IpcSender<InternalConnectionEvent>) -> Self
    {
        Self {
            control_receiver: control_receiver,
            event_sender: event_sender,
            tls_config: None,
            listeners: HashMap::new(),
            connections: HashMap::new(),
        }
    }

    fn translate_connection_type(&self, ct: ConnectionType) -> Result<InternalConnectionType,ListenerError>
    {
        match ct
        {
            ConnectionType::Clear => Ok(InternalConnectionType::Clear),
            ConnectionType::Tls =>
            {
                if let Some(conf) = &self.tls_config
                {
                    Ok(InternalConnectionType::Tls(conf.clone()))
                }
                else
                {
                    Err(ListenerError::NoTlsConfig)
                }
            }
        }
    }

    fn build_tls_config(&self, settings: TlsSettings) -> Result<Arc<rustls::ServerConfig>, rustls::Error>
    {
        let key = rustls::PrivateKey(settings.key);
        let certs = settings.cert_chain.into_iter().map(|c| rustls::Certificate(c)).collect();

        Ok(Arc::new(rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)?))
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>>
    {
        let (event_send, mut event_recv) = channel(128);
        let (connection_send, mut connection_recv) = channel(128);

        loop
        {
            select!
            {
                control = self.control_receiver.recv() =>
                {
                    match control
                    {
                        Ok(ControlMessage::Connection(id, msg)) =>
                        {
                            if let Some(conn) = self.connections.get(&id)
                            {
                                conn.control_channel.send(msg).await?;
                            }
                        }
                        Ok(ControlMessage::Listener(id, msg)) =>
                        {
                            match msg
                            {
                                ListenerControlDetail::Add(address, conn_type) =>
                                {
                                    match self.translate_connection_type(conn_type)
                                    {
                                        Ok(ct) =>
                                        {
                                            let listener = Listener::new(id, address, ct, event_send.clone(), connection_send.clone());

                                            self.listeners.insert(id, listener);
                                        }
                                        Err(e) =>
                                        {
                                            self.event_sender.send(InternalConnectionEvent::ListenerError(id,e)).await?;
                                        }
                                    }
                                }
                                ListenerControlDetail::Close =>
                                {
                                    if let Some(listener) = self.listeners.get(&id)
                                    {
                                        listener.control_channel.send(msg).await?;
                                    }
                                }
                            }
                        }
                        Ok(ControlMessage::LoadTlsSettings(settings)) =>
                        {
                            if let Ok(config) = self.build_tls_config(settings)
                            {
                                self.tls_config = Some(config);
                            }
                            else
                            {
                                self.event_sender.send(InternalConnectionEvent::BadTlsConfig).await?;
                            }
                        }
                        Ok(ControlMessage::Shutdown) =>
                        {
                            break;
                        }
                        Err(_) =>
                        {
                            self.event_sender.send(InternalConnectionEvent::CommunicationError).await?;
                            break;
                        }
                    }
                }
                event = event_recv.recv() =>
                {
                    if let Some(event) = event
                    {
                        self.event_sender.send(event).await?;
                    }
                    else
                    {
                        break;
                    }
                }
                conn = connection_recv.recv() =>
                {
                    if let Some(conn) = conn
                    {
                        self.event_sender.send(InternalConnectionEvent::NewConnection(conn.data())).await?;
                        self.connections.insert(conn.id, conn);
                    }
                }
            }
        }
        Ok(())
    }
}