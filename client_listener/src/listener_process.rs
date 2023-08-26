use crate::*;
use internal::*;

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use sable_ipc::{Receiver as IpcReceiver, Sender as IpcSender};
use tokio::select;
use tokio::sync::mpsc::{channel, unbounded_channel, UnboundedReceiver};

/// The worker side of the [`ListenerCollection`] system. This should only be constructed
/// by the worker process itself; applications using this system have no cause to interact
/// directly with it.
pub struct ListenerProcess {
    control_receiver: IpcReceiver<ControlMessage>,
    event_sender: Arc<IpcSender<InternalConnectionEvent>>,
    tls_config: Option<Arc<rustls::ServerConfig>>,

    listeners: HashMap<ListenerId, Listener>,
    connections: HashMap<ConnectionId, InternalConnection>,

    shutdown_flag: Arc<AtomicBool>,
}

impl ListenerProcess {
    pub fn new(
        control_receiver: IpcReceiver<ControlMessage>,
        event_sender: IpcSender<InternalConnectionEvent>,
    ) -> Self {
        Self {
            control_receiver,
            event_sender: Arc::new(event_sender),
            tls_config: None,
            listeners: HashMap::new(),
            connections: HashMap::new(),
            shutdown_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    fn translate_connection_type(
        tls_config: &Option<Arc<rustls::ServerConfig>>,
        ct: ConnectionType,
    ) -> Result<InternalConnectionType, ListenerError> {
        match ct {
            ConnectionType::Clear => Ok(InternalConnectionType::Clear),
            ConnectionType::Tls => {
                if let Some(conf) = &tls_config {
                    Ok(InternalConnectionType::Tls(conf.clone()))
                } else {
                    Err(ListenerError::NoTlsConfig)
                }
            }
        }
    }

    fn build_tls_config(settings: TlsSettings) -> Result<Arc<rustls::ServerConfig>, rustls::Error> {
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

    /// Sends events to the parent process, spawning the task into the background.
    ///
    /// Spawning a separate task avoids blocking the communication task, but does mean
    /// losing the ability to respond to errors, which will be logged instead of returned.
    async fn run_event_sender(
        mut ipc_event_recv: UnboundedReceiver<InternalConnectionEvent>,
        event_sender: Arc<IpcSender<InternalConnectionEvent>>,
        shutdown_flag: Arc<AtomicBool>,
    ) {
        while let Some(event) = ipc_event_recv.recv().await {
            if let Err(e) = event_sender.send(&event).await {
                shutdown_flag.store(true, Ordering::Relaxed);
                panic!("Error sending connection event: {}", e);
            }
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (connection_event_send, mut connection_event_recv) = channel(128);
        let (ipc_event_send, ipc_event_recv) = unbounded_channel();

        let ipc_event_sender = tokio::spawn(ListenerProcess::run_event_sender(
            ipc_event_recv,
            self.event_sender.clone(),
            self.shutdown_flag.clone(),
        ));

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
                                    ipc_event_send.send(InternalConnectionEvent::ConnectionError(id, e.into())).unwrap();
                                }
                            }
                        }
                        Ok(ControlMessage::Listener(id, msg)) =>
                        {
                            match msg
                            {
                                ListenerControlDetail::Add(address, conn_type) =>
                                {
                                    match Self::translate_connection_type(&self.tls_config, conn_type)
                                    {
                                        Ok(ct) =>
                                        {
                                            let listener = Listener::new(id, address, ct, connection_event_send.clone());

                                            self.listeners.insert(id, listener);
                                        }
                                        Err(e) =>
                                        {
                                            ipc_event_send.send(InternalConnectionEvent::ListenerError(id,e)).unwrap();
                                        }
                                    }
                                }
                                ListenerControlDetail::Close =>
                                {
                                    if let Some(listener) = self.listeners.get(&id)
                                    {
                                        if let Err(e) = listener.control_channel.try_send(msg)
                                        {
                                            ipc_event_send.send(InternalConnectionEvent::ListenerError(id, e.into())).unwrap();
                                        }
                                    }
                                }
                            }
                        }
                        Ok(ControlMessage::LoadTlsSettings(settings)) =>
                        {
                            if let Ok(config) = Self::build_tls_config(settings)
                            {
                                self.tls_config = Some(config);
                            }
                            else
                            {
                                ipc_event_send.send(InternalConnectionEvent::BadTlsConfig).unwrap();
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
                            ipc_event_send.send(InternalConnectionEvent::CommunicationError).unwrap();
                            break;
                        }
                    }
                }
                event = connection_event_recv.recv() =>
                {
                    match event
                    {
                        Some(InternalConnectionEventType::New(conn)) =>
                        {
                            let data = conn.data();
                            tracing::trace!("Sending new connection {:?}", data);
                            ipc_event_send.send(InternalConnectionEvent::NewConnection(data)).unwrap();
                            self.connections.insert(conn.id, conn);
                        }
                        Some(InternalConnectionEventType::Event(evt)) =>
                        {
                            tracing::trace!("Sending connection event {:?}", evt);
                            ipc_event_send.send(evt).unwrap();
                        }
                        None => break
                    }
                }
            }
        }
        ipc_event_sender.abort();
        Ok(())
    }
}
