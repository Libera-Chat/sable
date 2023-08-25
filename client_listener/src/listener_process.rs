use crate::*;
use internal::*;

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use sable_ipc::{Receiver as IpcReceiver, Sender as IpcSender};
use tokio::select;
use tokio::sync::mpsc::{channel, unbounded_channel, UnboundedReceiver, UnboundedSender};

/// The worker side of the [`ListenerCollection`] system. This should only be constructed
/// by the worker process itself; applications using this system have no cause to interact
/// directly with it.
pub struct ListenerProcess {
    control_receiver: IpcReceiver<ControlMessage>,
    event_sender: Arc<IpcSender<InternalConnectionEvent>>,
    tls_config: Option<Arc<rustls::ServerConfig>>,

    /// Along with event_out_buffer_rx, this is used to send events "in the background".
    ///
    /// Writing to this channel instead of directly to event_sender avoids blocking the
    /// communication task, but does mean losing the ability to respond to errors,
    /// which will be logged instead of returned.
    event_out_buffer_tx: UnboundedSender<InternalConnectionEvent>,
    event_out_buffer_rx: UnboundedReceiver<InternalConnectionEvent>,

    listeners: HashMap<ListenerId, Listener>,
    connections: HashMap<ConnectionId, InternalConnection>,

    shutdown_flag: Arc<AtomicBool>,
}

impl ListenerProcess {
    pub fn new(
        control_receiver: IpcReceiver<ControlMessage>,
        event_sender: IpcSender<InternalConnectionEvent>,
    ) -> Self {
        let (event_out_buffer_tx, event_out_buffer_rx) = unbounded_channel();
        Self {
            control_receiver,
            event_sender: Arc::new(event_sender),
            tls_config: None,
            event_out_buffer_tx,
            event_out_buffer_rx,
            listeners: HashMap::new(),
            connections: HashMap::new(),
            shutdown_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    fn translate_connection_type(
        &self,
        ct: ConnectionType,
    ) -> Result<InternalConnectionType, ListenerError> {
        match ct {
            ConnectionType::Clear => Ok(InternalConnectionType::Clear),
            ConnectionType::Tls => {
                if let Some(conf) = &self.tls_config {
                    Ok(InternalConnectionType::Tls(conf.clone()))
                } else {
                    Err(ListenerError::NoTlsConfig)
                }
            }
        }
    }

    fn build_tls_config(
        &self,
        settings: TlsSettings,
    ) -> Result<Arc<rustls::ServerConfig>, rustls::Error> {
        let key = rustls::PrivateKey(settings.key);
        let certs: Vec<rustls::Certificate> = settings
            .cert_chain
            .into_iter()
            .map(rustls::Certificate)
            .collect();

        let client_cert_verifier =
            internal::client_verifier::AcceptAnyClientCertVerifier::new(&certs[0]);

        Ok(Arc::new(
            rustls::ServerConfig::builder()
                .with_safe_defaults()
                .with_client_cert_verifier(Arc::new(client_cert_verifier))
                .with_single_cert(certs, key)?,
        ))
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (event_send, mut event_recv) = channel(128);

        // Golden rule of this loop: don't await anything that could possibly block.
        // If this task blocks on, e.g., sending to a connection's channel when it's full,
        // and that connection task blocks on trying to send to us while the event channel
        // is full, the whole listener process will deadlock.
        loop {
            if self.shutdown_flag.load(Ordering::Relaxed) {
                tracing::info!("Listener shutting down due to send error");
                break;
            }

            select! {
                event = self.event_out_buffer_rx.recv() =>
                {
                    let event_sender = Arc::clone(&self.event_sender);
                    let shutdown_flag = Arc::clone(&self.shutdown_flag);
                    if let Err(e) = event_sender.send(&event.unwrap()).await {
                        shutdown_flag.store(true, Ordering::Relaxed);
                        panic!("Error sending connection event: {}", e);
                    }
                }

                control = self.control_receiver.recv() =>
                {
                    match control
                    {
                        Ok(ControlMessage::Connection(id, msg)) =>
                        {
                            if let Some(conn) = self.connections.get(&id)
                            {
                                if let Err(e) = conn.control_channel.try_send(msg)
                                {
                                    self.event_out_buffer_tx.send(InternalConnectionEvent::ConnectionError(id, e.into())).unwrap();
                                }
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
                                            let listener = Listener::new(id, address, ct, event_send.clone());

                                            self.listeners.insert(id, listener);
                                        }
                                        Err(e) =>
                                        {
                                            self.event_out_buffer_tx.send(InternalConnectionEvent::ListenerError(id,e)).unwrap();
                                        }
                                    }
                                }
                                ListenerControlDetail::Close =>
                                {
                                    if let Some(listener) = self.listeners.get(&id)
                                    {
                                        if let Err(e) = listener.control_channel.try_send(msg)
                                        {
                                            self.event_out_buffer_tx.send(InternalConnectionEvent::ListenerError(id, e.into())).unwrap();
                                        }
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
                                self.event_out_buffer_tx.send(InternalConnectionEvent::BadTlsConfig).unwrap();
                            }
                        }
                        Ok(ControlMessage::Shutdown) =>
                        {
                            break;
                        }
                        Ok(ControlMessage::SaveForUpgrade) =>
                        {
                            // This shouldn't ever get here; just ignore it if it does
                            continue;
                        }
                        Err(_) =>
                        {
                            self.event_out_buffer_tx.send(InternalConnectionEvent::CommunicationError).unwrap();
                            break;
                        }
                    }
                }
                event = event_recv.recv() =>
                {
                    match event
                    {
                        Some(InternalConnectionEventType::New(conn)) =>
                        {
                            let data = conn.data();
                            tracing::warn!("Sending new connection {:?}", data);
                            self.event_out_buffer_tx.send(InternalConnectionEvent::NewConnection(data)).unwrap();
                            self.connections.insert(conn.id, conn);
                        }
                        Some(InternalConnectionEventType::Event(evt)) =>
                        {
                            tracing::trace!("Sending connection event {:?}", evt);
                            self.event_out_buffer_tx.send(evt).unwrap();
                        }
                        None => break
                    }
                }
            }
        }
        Ok(())
    }
}
