//! Networking code for the sync protocol

use super::message::Message;
use super::*;
use crate::validated::{ServerName, Validated};

use futures::future;
use std::{
    convert::TryInto,
    net::SocketAddr,
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    sync::Mutex,
};
use tokio::{
    io,
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpSocket, TcpStream},
    select,
    sync::{
        mpsc::{channel, UnboundedSender},
        oneshot,
    },
    task::{JoinError, JoinHandle},
};
use tokio_rustls::{TlsAcceptor, TlsConnector, TlsStream};

use rustls::{server::AllowAnyAuthenticatedClient, ClientConfig, ServerConfig};
use sha1::{Digest, Sha1};
use x509_parser::prelude::*;

use rand::prelude::*;
use thiserror::Error;
use tracing::instrument;

/// An interface to the gossip network used to synchronise state.
pub struct GossipNetwork {
    fanout: usize,
    tls_client_config: Arc<ClientConfig>,
    shutdown_send: Mutex<Option<oneshot::Sender<()>>>,
    task_state: Arc<NetworkTaskState>,
    me: PeerConfig,
}

/// State that's shared between the listener task and client code
///
/// Note that all additions to this struct must keep it `Send` and `Sync`.
struct NetworkTaskState {
    peers: Vec<Peer>,
    listen_addr: SocketAddr,
    tls_server_config: Arc<ServerConfig>,
    message_sender: UnboundedSender<Request>,
}

struct Peer {
    conf: PeerConfig,
    enabled: AtomicBool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GossipNetworkState {
    peer_states: Vec<(ServerName, bool)>,
}

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("Send error: {0}")]
    Send(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Error joining task: {0}")]
    Join(#[from] JoinError),
    #[error("Listen task already spawned")]
    AlreadyListening,
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Authorisation failure: {0}")]
    AuthzError(String),
    #[error("Operation timed out")]
    Timeout,
}
pub type NetworkResult = Result<(), NetworkError>;

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for NetworkError {
    fn from(e: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::Send(e.to_string())
    }
}

impl GossipNetwork {
    pub fn new(
        net_config: SyncConfig,
        node_config: NodeConfig,
        message_sender: UnboundedSender<Request>,
    ) -> Self {
        let ca_cert = net_config.load_ca_cert().expect("Error loading CA");

        let (client_cert, client_key) = node_config
            .load_cert_and_keys()
            .expect("Error loading client cert");

        let mut root_store = rustls::RootCertStore::empty();
        root_store
            .add(&ca_cert)
            .expect("Error adding certificate to store");

        let client_config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store.clone())
            .with_single_cert(client_cert.clone(), client_key.clone())
            .expect("Bad TLS client config");

        let server_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(AllowAnyAuthenticatedClient::new(root_store.clone()))
            .with_single_cert(client_cert, client_key)
            .expect("Bad TLS server config");

        let mut peers = net_config.peers;
        let my_index = peers
            .iter()
            .position(|p| p.address == node_config.listen_addr)
            .expect("Couldn't find myself in the network config");
        let me = peers.remove(my_index);

        Self {
            fanout: net_config.fanout,
            tls_client_config: Arc::new(client_config),
            shutdown_send: Mutex::new(None),
            me,
            task_state: Arc::new(NetworkTaskState {
                listen_addr: node_config.listen_addr,
                peers: peers
                    .into_iter()
                    .map(|c| Peer {
                        conf: c,
                        enabled: AtomicBool::new(false),
                    })
                    .collect(),
                tls_server_config: Arc::new(server_config),
                message_sender,
            }),
        }
    }

    pub fn restore(
        state: GossipNetworkState,
        net_config: SyncConfig,
        node_config: NodeConfig,
        message_sender: UnboundedSender<Request>,
    ) -> Self {
        let ret = Self::new(net_config, node_config, message_sender);

        for (peer, enabled) in state.peer_states {
            if enabled {
                ret.enable_peer(&peer);
            }
        }

        ret
    }

    pub fn save_state(&self) -> GossipNetworkState {
        GossipNetworkState {
            peer_states: self
                .task_state
                .peers
                .iter()
                .map(|peer| (peer.conf.name.clone(), peer.enabled.load(Ordering::SeqCst)))
                .collect(),
        }
    }

    pub fn shutdown(&self) {
        if let Ok(mut shutdown_send) = self.shutdown_send.lock() {
            if let Some(sender) = shutdown_send.take() {
                sender.send(()).ok();
            }
        }
    }

    pub fn me(&self) -> &PeerConfig {
        &self.me
    }

    pub fn enable_peer(&self, name: &ServerName) {
        tracing::debug!("enabling peer {}", name);
        for p in self.task_state.peers.iter() {
            if &p.conf.name == name {
                p.enabled.store(true, Ordering::SeqCst);
            }
        }
    }

    pub fn disable_peer(&self, name: &ServerName) {
        tracing::debug!("disabling peer {}", name);

        for p in self.task_state.peers.iter() {
            if &p.conf.name == name {
                p.enabled.store(false, Ordering::SeqCst);
            }
        }
    }

    #[instrument(skip_all)]
    pub fn choose_peer(&self) -> Option<&PeerConfig> {
        let ret = self
            .task_state
            .peers
            .iter()
            .filter(|p| p.enabled.load(Ordering::SeqCst))
            .choose(&mut rand::thread_rng())
            .map(|p| &p.conf);

        if ret.is_none() {
            tracing::info!("No active peer available to choose");
        }
        ret
    }

    #[instrument(skip_all)]
    pub fn choose_any_peer(&self) -> Option<&PeerConfig> {
        let ret = self
            .task_state
            .peers
            .iter()
            .choose(&mut rand::thread_rng())
            .map(|p| &p.conf);

        if ret.is_none() {
            tracing::info!("No peer available to choose");
        }
        ret
    }

    /// Choose a peer at random that isn't in the provided list
    pub fn choose_peer_except(&self, except: &Vec<ServerName>) -> Option<&PeerConfig> {
        let ret = self
            .task_state
            .peers
            .iter()
            .filter(|p| p.enabled.load(Ordering::SeqCst) && !except.contains(&p.conf.name))
            .choose(&mut rand::thread_rng())
            .map(|p| &p.conf);

        if ret.is_none() {
            tracing::info!("No active peer available to choose");
        }
        ret
    }

    /// Find a peer config with the given server name
    pub fn find_peer(&self, name: &ServerName) -> Option<&PeerConfig> {
        let ret = self
            .task_state
            .peers
            .iter()
            .filter(|p| p.enabled.load(Ordering::Relaxed))
            .find(|p| &p.conf.name == name)
            .map(|p| &p.conf);

        if ret.is_none() {
            tracing::info!("No peer named {} available", name);
        }
        ret
    }

    pub async fn propagate(&self, msg: &Message) {
        let mut tasks = Vec::new();

        let chosen_peers = self
            .task_state
            .peers
            .iter()
            .filter(|p| p.enabled.load(Ordering::SeqCst))
            .choose_multiple(&mut rand::thread_rng(), self.fanout);

        if chosen_peers.is_empty() {
            tracing::info!("No peers available to propagate message");
        }

        for peer in chosen_peers {
            tasks.push(self.send_to(&peer.conf, msg.clone()));
        }

        future::join_all(tasks).await;
    }

    pub async fn send_to(
        &self,
        peer: &PeerConfig,
        msg: Message,
    ) -> Result<JoinHandle<NetworkResult>, NetworkError> {
        tracing::trace!("Sending to {:?}: {:?}", peer.address, msg);
        let result = self
            .do_send_to(peer, msg, self.task_state.message_sender.clone())
            .await;

        if let Err(e) = &result {
            tracing::error!("Error sending network event: {}", e);
        }

        result
    }

    pub async fn send_and_process(
        &self,
        peer: &PeerConfig,
        msg: Message,
        response_sender: UnboundedSender<Request>,
    ) -> Result<JoinHandle<NetworkResult>, NetworkError> {
        tracing::trace!("Sending to {:?}: {:?}", peer.address, msg);
        self.do_send_to(peer, msg, response_sender).await
    }

    fn get_socket_for_addr(addr: &SocketAddr) -> std::io::Result<TcpSocket> {
        match addr {
            SocketAddr::V4(_) => TcpSocket::new_v4(),
            SocketAddr::V6(_) => TcpSocket::new_v6(),
        }
    }

    #[instrument(skip(self, response_sender))]
    async fn do_send_to(
        &self,
        peer: &PeerConfig,
        msg: Message,
        response_sender: UnboundedSender<Request>,
    ) -> Result<JoinHandle<NetworkResult>, NetworkError> {
        let mut local_addr = self.task_state.listen_addr;
        local_addr.set_port(0);
        let socket = Self::get_socket_for_addr(&local_addr)?;
        socket.bind(local_addr)?;
        let connector = TlsConnector::from(Arc::clone(&self.tls_client_config));
        let conn = socket.connect(peer.address).await?;
        let server_name = (&peer.name.value() as &str)
            .try_into()
            .expect("Invalid server name");
        let stream = connector.connect(server_name, conn).await?;

        let task_state = Arc::clone(&self.task_state);
        Ok(tokio::spawn(async move {
            let result = task_state
                .send_and_handle_response(stream.into(), msg, response_sender)
                .await;

            if let Err(e) = &result {
                tracing::error!("Error in outbound network sync connection: {}", e);
            }

            result
        }))
    }

    pub async fn spawn_listen_task(&self) -> Result<JoinHandle<()>, NetworkError> {
        let listener = TcpListener::bind(self.task_state.listen_addr).await?;
        let task_state = Arc::clone(&self.task_state);

        let (shutdown_send, shutdown_recv) = oneshot::channel();
        {
            let mut guard = self
                .shutdown_send
                .lock()
                .map_err(|e| NetworkError::InternalError(e.to_string()))?;
            if guard.is_some() {
                Err(NetworkError::AlreadyListening)?;
            }
            let _ = guard.insert(shutdown_send);
        }

        Ok(tokio::spawn(async move {
            if let Err(e) = task_state.listen_loop(listener, shutdown_recv).await {
                tracing::error!("Error in network sync listener: {}", e);
            }
        }))
    }
}

impl NetworkTaskState {
    #[instrument(skip(self))]
    async fn listen_loop(
        self: Arc<NetworkTaskState>,
        listener: TcpListener,
        mut shutdown: oneshot::Receiver<()>,
    ) -> Result<(), io::Error> {
        let tls_acceptor = TlsAcceptor::from(Arc::clone(&self.tls_server_config));

        loop {
            select! {
                res = listener.accept() =>
                {
                    if let Ok((conn, _)) = res
                    {
                        let tls_acceptor = tls_acceptor.clone();
                        let sender = self.message_sender.clone();
                        let self_copy = Arc::clone(&self);
                        tokio::spawn(async move {
                            if let Err(e) = self_copy.handle_connection(tls_acceptor, conn, sender).await
                            {
                                tracing::error!("Error in network sync connection handler: {}", e);
                            }
                        });
                    }
                },
                _ = &mut shutdown =>
                {
                    break
                }
            }
        }

        Ok(())
    }

    #[instrument(skip(tls_acceptor, self))]
    async fn handle_connection(
        self: Arc<NetworkTaskState>,
        tls_acceptor: TlsAcceptor,
        conn: TcpStream,
        message_sender: UnboundedSender<Request>,
    ) -> Result<(), NetworkError> {
        let stream = tls_acceptor.accept(conn).await?;

        if let Err(e) = self
            .read_and_handle_message(stream.into(), message_sender)
            .await
        {
            tracing::error!("Error handling message: {}", e);
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn send_and_handle_response(
        self: Arc<NetworkTaskState>,
        mut stream: TlsStream<TcpStream>,
        message: Message,
        response_sender: UnboundedSender<Request>,
    ) -> Result<(), NetworkError> {
        let buf = serde_json::to_vec(&message)?;
        stream.write_u32(buf.len().try_into().unwrap()).await?;
        stream.write_all(&buf).await?;

        self.read_and_handle_message(stream, response_sender)
            .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn read_and_handle_message(
        self: Arc<NetworkTaskState>,
        mut stream: TlsStream<TcpStream>,
        message_sender: UnboundedSender<Request>,
    ) -> Result<(), NetworkError> {
        // Get the peer name we're talking to from the tls certificate
        let (tcp_stream, state) = stream.get_ref();
        let peer_certs = state
            .peer_certificates()
            .ok_or_else(|| NetworkError::InternalError("No peer certificates?".to_string()))?;

        let (_, cert) = X509Certificate::from_der(&peer_certs[0].0)
            .map_err(|e| NetworkError::InternalError(format!("Invalid peer certificate? {}", e)))?;

        let peer_name = cert
            .subject()
            .iter_common_name()
            .next()
            .and_then(|cn| cn.as_str().ok())
            .ok_or_else(|| NetworkError::InternalError("Couldn't parse peer CN".to_string()))?
            .to_string();
        let peer_name = ServerName::convert(&peer_name).map_err(|_| {
            NetworkError::InternalError(format!(
                "Invalid server name in certificate: {}",
                &peer_name
            ))
        })?;

        let peer = self
            .peers
            .iter()
            .find(|p| p.conf.name == peer_name)
            .ok_or_else(|| {
                NetworkError::AuthzError("Couldn't find peer configuration".to_string())
            })?;
        let peer_conf = &peer.conf;

        let remote_addr = tcp_stream.peer_addr()?;
        if remote_addr.ip() != peer_conf.address.ip() {
            return Err(NetworkError::AuthzError(format!(
                "IP address doesn't match peer configuration ({}/{})",
                remote_addr.ip(),
                peer_conf.address.ip()
            )));
        }

        let expected_fingerprint = &peer_conf.fingerprint;
        let mut cert_hasher = Sha1::new();
        cert_hasher.update(&peer_certs[0].0);
        let remote_fingerprint = hex::encode(cert_hasher.finalize());

        if &remote_fingerprint != expected_fingerprint {
            return Err(NetworkError::AuthzError(format!(
                "Certificate doesn't match ({}/{})",
                remote_fingerprint, expected_fingerprint
            )));
        }

        loop {
            let length = stream.read_u32().await?;

            let mut buf = vec![0; length.try_into().unwrap()];
            stream.read_exact(&mut buf).await?;

            let msg: Message = serde_json::from_slice(&buf)?;

            if matches!(msg.content, MessageDetail::Done) {
                return Ok(());
            }

            tracing::trace!("Processing inbound message: {:?}", msg);

            let (req_send, mut req_recv) = channel(8);
            let req = Request {
                received_from: peer_name.clone(),
                response: req_send,
                message: msg,
            };

            message_sender.send(req)?;

            while let Some(response) = req_recv.recv().await {
                tracing::trace!("Sending network response: {:?}", response);
                let buf = serde_json::to_vec(&response).expect("Failed to serialise response");
                stream.write_u32(buf.len().try_into().unwrap()).await?;
                stream.write_all(&buf).await?;

                if matches!(response.content, MessageDetail::Done) {
                    tracing::trace!("Got done, ending connection");
                    return Ok(());
                }
            }
        }
    }
}
