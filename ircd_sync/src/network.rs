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
};
use std::{
    net::SocketAddr,
    sync::Arc,
    convert::TryInto,
};
use futures::future;

use rustls::{
    ClientConfig,
    ServerConfig,
    server::AllowAnyAuthenticatedClient,
};

use serde_json;
use rand::prelude::*;
use thiserror::Error;

pub struct Network
{
    listen_addr: SocketAddr,
    peers: Vec<PeerConfig>,
    fanout: usize,
    tls_client_config: Arc<ClientConfig>,
    tls_server_config: Arc<ServerConfig>,
    message_sender: Sender<Request>,
    shutdown_send: oneshot::Sender<()>,
    shutdown_recv: Option<oneshot::Receiver<()>>
}

#[derive(Debug,Error)]
pub enum NetworkError
{
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
    #[error("Send error: {0}")]
    SendError(String),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Error joining task: {0}")]
    JoinError(#[from] JoinError),
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for NetworkError
{
    fn from(e: tokio::sync::mpsc::error::SendError<T>) -> Self
    {
        Self::SendError(e.to_string())
    }
}

impl Network
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

        let (shutdown_send, shutdown_recv) = oneshot::channel();

        Self {
            listen_addr: node_config.listen_addr,
            peers: peers,
            fanout: net_config.fanout,
            tls_client_config: Arc::new(client_config),
            tls_server_config: Arc::new(server_config),
            message_sender: message_sender,
            shutdown_send: shutdown_send,
            shutdown_recv: Some(shutdown_recv),
        }
    }

    pub fn shutdown(self)
    {
        self.shutdown_send.send(()).ok();
    }

    pub fn choose_peer(&self) -> Option<&PeerConfig>
    {
        self.peers.iter().choose(&mut rand::thread_rng())
    }

    pub async fn propagate(&self, msg: &Message)
    {
        let mut tasks = Vec::new();

        for peer in self.peers.iter().choose_multiple(&mut rand::thread_rng(), self.fanout)
        {
            tasks.push(self.send_to(peer, msg.clone()));
        }

        future::join_all(tasks).await;
    }

    pub async fn send_to(&self, peer: &PeerConfig, msg: Message)
    {
        tracing::trace!("Sending to {:?}: {:?}", peer.address, msg);
        if let Err(e) = self.do_send_to(peer, msg, self.message_sender.clone()).await
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

    async fn do_send_to(&self, peer: &PeerConfig, msg: Message, response_sender: Sender<Request>)
                -> Result<JoinHandle<()>, NetworkError>
    {
        let connector = TlsConnector::from(Arc::clone(&self.tls_client_config));
        let conn = TcpStream::connect(peer.address).await?;
        let server_name = (&peer.name as &str).try_into().expect("Invalid server name");
        let stream = connector.connect(server_name, conn).await?;

        Ok(tokio::spawn(async move {
            if let Err(e) = Self::send_and_handle_response(stream, msg, response_sender).await
            {
                tracing::error!("Error in outbound network sync connection: {}", e);
            }
        }))
    }

    pub async fn spawn_listen_task(&mut self) -> Result<JoinHandle<()>, io::Error>
    {
        let listener = TcpListener::bind(self.listen_addr).await?;
        let tls_acceptor = TlsAcceptor::from(Arc::clone(&self.tls_server_config));
        let sender = self.message_sender.clone();
        let shutdown = self.shutdown_recv.take().ok_or(io::ErrorKind::Other)?;

        Ok(tokio::spawn(async move {
            if let Err(e) = Self::listen_loop(listener, tls_acceptor, sender, shutdown).await
            {
                tracing::error!("Error in network sync listener: {}", e);
            }
        }))
    }

    async fn listen_loop(listener: TcpListener, tls_acceptor: TlsAcceptor, message_sender: Sender<Request>, mut shutdown: oneshot::Receiver<()>) -> Result<(), io::Error>
    {
        loop
        {
            select!
            {
                res = listener.accept() =>
                {
                    if let Ok((conn, _)) = res
                    {
                        let tls_acceptor = tls_acceptor.clone();
                        let sender = message_sender.clone();
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

    async fn handle_connection(tls_acceptor: TlsAcceptor, conn: TcpStream, message_sender: Sender<Request>) -> Result<(), NetworkError>
    {
        let stream = tls_acceptor.accept(conn).await?;

        if let Err(e) = Self::read_and_handle_message(stream, message_sender).await
        {
            tracing::error!("Error handling message: {}", e);
        }

        Ok(())
    }

    async fn send_and_handle_response<S>(mut stream: S, message: Message, response_sender: Sender<Request>) -> Result<(), NetworkError>
        where S: AsyncRead + AsyncWrite + Unpin
    {
        let buf = serde_json::to_vec(&message)?;
        stream.write_u32(buf.len().try_into().unwrap()).await?;
        stream.write_all(&buf).await?;

        Self::read_and_handle_message(stream, response_sender).await?;

        Ok(())
    }

    async fn read_and_handle_message<S>(mut stream: S, message_sender: Sender<Request>) -> Result<(), NetworkError>
        where S: AsyncRead + AsyncWrite + Unpin
    {
        loop {
            let length = stream.read_u32().await?;

            let mut buf = vec![0; length.try_into().unwrap()];
            stream.read_exact(&mut buf).await?;

            let msg: Message = serde_json::from_slice(&buf)?;

            if matches!(msg, Message::Done)
            {
                return Ok(());
            }

            tracing::trace!("Processing inbound message: {:?}", msg);

            let (req_send, mut req_recv) = channel(8);
            let req = Request {
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

                if matches!(response, Message::Done)
                {
                    tracing::trace!("Got done, ending connection");
                    return Ok(());
                }
            }
        }
    }
}