//! Networking code for the sync protocol

use super::*;
use super::message::Message;

use tokio::{
    net::{
        TcpListener,
        TcpStream,
    },
    io,
    io::{
        AsyncRead,
        AsyncWrite,
        AsyncReadExt,
        AsyncWriteExt,
    },
    sync::{
        mpsc::{
            Sender,
            channel
        },
        oneshot,
    },
    task::{
        JoinHandle,
        JoinError
    },
    select
};
use tokio_rustls::{
    TlsAcceptor,
    TlsConnector,
    TlsStream,
};
use std::{
    net::SocketAddr,
    sync::Arc,
    sync::Mutex,
    convert::TryInto,
    sync::atomic::{
        AtomicBool,
        Ordering
    },
};
use futures::future;

use rustls::{
    ClientConfig,
    ServerConfig,
    server::AllowAnyAuthenticatedClient,
};
use x509_parser::prelude::*;

use rand::prelude::*;
use thiserror::Error;
use tracing::instrument;

/// An interface to the gossip network used to synchronise state.
pub struct GossipNetwork
{
    listen_addr: SocketAddr,
    fanout: usize,
    tls_client_config: Arc<ClientConfig>,
    shutdown_send: Mutex<Option<oneshot::Sender<()>>>,
    task_state: Arc<NetworkTaskState>,
}

/// State that's shared between the listener task and client code
///
/// Note that all additions to this struct must keep it `Send` and `Sync`.
struct NetworkTaskState
{
    peers: Vec<Peer>,
    tls_server_config: Arc<ServerConfig>,
    message_sender: Sender<Request>,
}

struct Peer
{
    conf: PeerConfig,
    enabled: AtomicBool,
}

#[derive(Debug,serde::Serialize,serde::Deserialize)]
pub struct GossipNetworkState
{
    peer_states: Vec<(String, bool)>
}

#[derive(Debug,Error)]
pub enum NetworkError
{
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
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for NetworkError
{
    fn from(e: tokio::sync::mpsc::error::SendError<T>) -> Self
    {
        Self::Send(e.to_string())
    }
}

impl GossipNetwork
{
    pub fn new(net_config: NetworkConfig, node_config: NodeConfig, message_sender: Sender<Request>) -> Self
    {
        let ca_cert = net_config.load_ca_cert().expect("Error loading CA");

        let (client_cert, client_key) = node_config.load_cert_and_keys().expect("Error loading client cert");

        let mut root_store = rustls::RootCertStore::empty();
        root_store.add(&ca_cert).expect("Error adding certificate to store");

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
        peers.retain(|p| p.address != node_config.listen_addr);

        Self {
            listen_addr: node_config.listen_addr,
            fanout: net_config.fanout,
            tls_client_config: Arc::new(client_config),
            shutdown_send: Mutex::new(None),
            task_state: Arc::new(NetworkTaskState {
                peers: peers.into_iter().map(|c| Peer { conf: c, enabled: AtomicBool::new(false) }).collect(),
                tls_server_config: Arc::new(server_config),
                message_sender,
            })
        }
    }

    pub fn restore(state: GossipNetworkState, net_config: NetworkConfig, node_config: NodeConfig, message_sender: Sender<Request>) -> Self
    {
        let ret = Self::new(net_config, node_config, message_sender);

        for (peer,enabled) in state.peer_states
        {
            if enabled
            {
                ret.enable_peer(&peer);
            }
        }

        ret
    }

    pub fn save_state(&self) -> GossipNetworkState
    {
        GossipNetworkState {
            peer_states: self.task_state.peers.iter().map(|peer| (peer.conf.name.clone(), peer.enabled.load(Ordering::SeqCst))).collect()
        }
    }

    pub fn shutdown(&self)
    {
        if let Ok(mut shutdown_send) = self.shutdown_send.lock()
        {
            if let Some(sender) = shutdown_send.take()
            {
                sender.send(()).ok();
            }
        }
    }

    pub fn enable_peer(&self, name: &str)
    {
        tracing::info!(name, "enabling peer");
        for p in self.task_state.peers.iter()
        {
            if p.conf.name == name
            {
                p.enabled.store(true, Ordering::SeqCst);
            }
        }
    }

    pub fn disable_peer(&self, name: &str)
    {
        tracing::info!(name, "disabling peer");

        for p in self.task_state.peers.iter()
        {
            if p.conf.name == name
            {
                p.enabled.store(false, Ordering::SeqCst);
            }
        }
    }

    #[instrument(skip_all)]
    pub fn choose_peer(&self) -> Option<&PeerConfig>
    {
        let ret = self.task_state.peers.iter()
                             .filter(|p| p.enabled.load(Ordering::SeqCst))
                             .choose(&mut rand::thread_rng())
                             .map(|p| &p.conf);

        if ret.is_none()
        {
            tracing::info!("No active peer available to choose");
        }
        ret
    }

    #[instrument(skip_all)]
    pub fn choose_any_peer(&self) -> Option<&PeerConfig>
    {
        let ret = self.task_state.peers.iter()
                             .choose(&mut rand::thread_rng())
                             .map(|p| &p.conf);

        if ret.is_none()
        {
            tracing::info!("No peer available to choose");
        }
        ret
    }

    pub async fn propagate(&self, msg: &Message)
    {
        let mut tasks = Vec::new();

        let chosen_peers = self.task_state.peers.iter()
                                          .filter(|p| p.enabled.load(Ordering::SeqCst))
                                          .choose_multiple(&mut rand::thread_rng(), self.fanout);

        if chosen_peers.is_empty()
        {
            tracing::info!("No peers available to propagate message");
        }

        for peer in chosen_peers
        {
            tasks.push(self.send_to(&peer.conf, msg.clone()));
        }

        future::join_all(tasks).await;
    }

    pub async fn send_to(&self, peer: &PeerConfig, msg: Message)
    {
        tracing::trace!("Sending to {:?}: {:?}", peer.address, msg);
        if let Err(e) = self.do_send_to(peer, msg, self.task_state.message_sender.clone()).await
        {
            tracing::error!("Error sending network event: {}", e);
        }
    }

    pub async fn send_and_process(&self, peer: &PeerConfig, msg: Message, response_sender: Sender<Request>)
                 -> Result<JoinHandle<()>, NetworkError>
    {
        tracing::trace!("Sending to {:?}: {:?}", peer.address, msg);
        Ok(self.do_send_to(peer, msg, response_sender).await?)
    }

    #[instrument(skip(self,response_sender))]
    async fn do_send_to(&self, peer: &PeerConfig, msg: Message, response_sender: Sender<Request>)
                -> Result<JoinHandle<()>, NetworkError>
    {
        let connector = TlsConnector::from(Arc::clone(&self.tls_client_config));
        let conn = TcpStream::connect(peer.address).await?;
        let server_name = (&peer.name as &str).try_into().expect("Invalid server name");
        let stream = connector.connect(server_name, conn).await?;

        Ok(tokio::spawn(async move {
            if let Err(e) = Self::send_and_handle_response(stream.into(), msg, response_sender).await
            {
                tracing::error!("Error in outbound network sync connection: {}", e);
            }
        }))
    }

    pub async fn spawn_listen_task(&self) -> Result<JoinHandle<()>, NetworkError>
    {
        let listener = TcpListener::bind(self.listen_addr).await?;
        let task_state = Arc::clone(&self.task_state);

        let (shutdown_send, shutdown_recv) = oneshot::channel();
        {
            let mut guard = self.shutdown_send.lock().map_err(|e| NetworkError::InternalError(e.to_string()))?;
            if guard.is_some()
            {
                Err(NetworkError::AlreadyListening)?;
            }
            let _ = guard.insert(shutdown_send);
        }

        Ok(tokio::spawn(async move {
            if let Err(e) = Self::listen_loop(listener, shutdown_recv, task_state).await
            {
                tracing::error!("Error in network sync listener: {}", e);
            }
        }))
    }

    #[instrument(skip(task_state))]
    async fn listen_loop(listener: TcpListener, mut shutdown: oneshot::Receiver<()>, task_state: Arc<NetworkTaskState>) -> Result<(), io::Error>
    {
        let tls_acceptor = TlsAcceptor::from(Arc::clone(&task_state.tls_server_config));

        loop
        {
            select!
            {
                res = listener.accept() =>
                {
                    if let Ok((conn, _)) = res
                    {
                        let tls_acceptor = tls_acceptor.clone();
                        let sender = task_state.message_sender.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(tls_acceptor, conn, sender).await
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

    #[instrument(skip(tls_acceptor))]
    async fn handle_connection(tls_acceptor: TlsAcceptor, conn: TcpStream, message_sender: Sender<Request>) -> Result<(), NetworkError>
    {
        let stream = tls_acceptor.accept(conn).await?;

        if let Err(e) = Self::read_and_handle_message(stream.into(), message_sender).await
        {
            tracing::error!("Error handling message: {}", e);
        }

        Ok(())
    }

    #[instrument]
    async fn send_and_handle_response<IO>(mut stream: TlsStream<IO>, message: Message, response_sender: Sender<Request>) -> Result<(), NetworkError>
        where IO: AsyncRead + AsyncWrite + Unpin + std::fmt::Debug
    {
        let buf = serde_json::to_vec(&message)?;
        stream.write_u32(buf.len().try_into().unwrap()).await?;
        stream.write_all(&buf).await?;

        Self::read_and_handle_message(stream, response_sender).await?;

        Ok(())
    }

    #[instrument]
    async fn read_and_handle_message<IO>(mut stream: TlsStream<IO>, message_sender: Sender<Request>) -> Result<(), NetworkError>
        where IO: AsyncRead + AsyncWrite + Unpin + std::fmt::Debug
    {
        // Get the peer name we're talking to from the tls certificate
        let (_, state) = stream.get_ref();
        let peer_certs = state.peer_certificates()
                                    .ok_or(NetworkError::InternalError("No peer certificates?".to_string()))?;
        let (_,cert) = X509Certificate::from_der(&peer_certs[0].0)
                                    .map_err(|e| NetworkError::InternalError(format!("Invalid peer certificate? {}", e)))?;
        let peer_name = cert.subject()
                            .iter_common_name()
                            .next()
                            .and_then(|cn| cn.as_str().ok())
                            .ok_or(NetworkError::InternalError("Couldn't parse peer CN".to_string()))?
                            .to_string();

        loop
        {
            let length = stream.read_u32().await?;

            let mut buf = vec![0; length.try_into().unwrap()];
            stream.read_exact(&mut buf).await?;

            let msg: Message = serde_json::from_slice(&buf)?;

            if matches!(msg.content, MessageDetail::Done)
            {
                return Ok(());
            }

            tracing::trace!("Processing inbound message: {:?}", msg);

            let (req_send, mut req_recv) = channel(8);
            let req = Request {
                received_from: peer_name.clone(),
                response: req_send,
                message: msg
            };

            message_sender.send(req).await?;

            while let Some(response) = req_recv.recv().await
            {
                tracing::trace!("Sending network response: {:?}", response);
                let buf = serde_json::to_vec(&response).expect("Failed to serialise response");
                stream.write_u32(buf.len().try_into().unwrap()).await?;
                stream.write_all(&buf).await?;

                if matches!(response.content, MessageDetail::Done)
                {
                    tracing::trace!("Got done, ending connection");
                    return Ok(());
                }
            }
        }
    }
}