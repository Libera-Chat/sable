use crate::config::*;
use sable_network::{
    config::TlsData,
    rpc::{ServerManagementCommand, ServerManagementCommandType, ShutdownAction},
};

use hyper::{Body, Method, Request, Response, StatusCode};
use sha1::{Digest, Sha1};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use thiserror::Error;
use tokio::{
    net::{TcpListener, TcpStream},
    select,
    sync::{
        mpsc::{channel, Receiver, Sender},
        oneshot,
    },
    task,
};
use tokio_rustls::TlsAcceptor;
use tracing::Instrument;

pub enum ManagementCommand {
    ServerCommand(ServerManagementCommand),
    Shutdown(ShutdownAction),
}

pub struct ManagementServer {
    command_receiver: Receiver<ManagementCommand>,
    server_task: task::JoinHandle<Result<(), hyper::Error>>,
    service_data: Arc<ManagementServiceData>,
}

struct ManagementService {
    data: Arc<ManagementServiceData>,
    authorised_fingerprint: AuthorisedFingerprint,
}

struct ManagementServiceData {
    command_sender: Sender<ManagementCommand>,
    authorised_fingerprints: Vec<AuthorisedFingerprint>,
}

#[derive(Debug, Error)]
enum ManagementServiceError {
    #[error("Other error: {0}")]
    Other(&'static str),
    #[error("Certificate fingerprint not recognised")]
    InvalidFingerprint,
}

fn internal_error() -> hyper::Result<Response<Body>> {
    let mut response = Response::default();
    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    Ok(response)
}

impl ManagementService {
    async fn server_management_command(
        command_sender: Sender<ManagementCommand>,
        cmd: ServerManagementCommandType,
    ) -> Result<Response<Body>, hyper::Error> {
        let (send, recv) = oneshot::channel();
        let cmd = ServerManagementCommand {
            cmd,
            response: send,
        };
        if command_sender
            .send(ManagementCommand::ServerCommand(cmd))
            .await
            .is_err()
        {
            internal_error()
        } else if let Ok(response) = recv.await {
            Ok(Response::new(Body::from(response)))
        } else {
            internal_error()
        }
    }

    async fn shutdown_command(
        command_sender: Sender<ManagementCommand>,
        cmd: ShutdownAction,
    ) -> Result<Response<Body>, hyper::Error> {
        if command_sender
            .send(ManagementCommand::Shutdown(cmd))
            .await
            .is_ok()
        {
            Ok(Response::new(Body::empty()))
        } else {
            internal_error()
        }
    }
}

impl hyper::service::Service<Request<Body>> for ManagementService {
    type Response = Response<Body>;
    type Error = hyper::Error;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let command_sender = self.data.command_sender.clone();

        tracing::debug!(method=?req.method(), path=?req.uri().path(), user=?self.authorised_fingerprint.name, "Got management request");

        Box::pin(async move {
            match (req.method(), req.uri().path()) {
                (&Method::GET, "/statistics") => {
                    Self::server_management_command(
                        command_sender,
                        ServerManagementCommandType::ServerStatistics,
                    )
                    .await
                }
                (&Method::GET, "/dump-network") => {
                    Self::server_management_command(
                        command_sender,
                        ServerManagementCommandType::DumpNetwork,
                    )
                    .await
                }
                (&Method::GET, "/dump-events") => {
                    Self::server_management_command(
                        command_sender,
                        ServerManagementCommandType::DumpEvents,
                    )
                    .await
                }
                (&Method::POST, "/shutdown") => {
                    Self::shutdown_command(command_sender, ShutdownAction::Shutdown).await
                }
                (&Method::POST, "/restart") => {
                    Self::shutdown_command(command_sender, ShutdownAction::Restart).await
                }
                (&Method::POST, "/upgrade") => {
                    Self::shutdown_command(command_sender, ShutdownAction::Upgrade).await
                }
                _ => {
                    let mut response = Response::default();
                    *response.status_mut() = StatusCode::NOT_FOUND;
                    Ok(response)
                }
            }
        })
    }
}

impl ManagementServer {
    fn server_config(data: TlsData, client_ca: Vec<u8>) -> Arc<rustls::ServerConfig> {
        let mut root_store = rustls::RootCertStore::empty();
        root_store
            .add(&rustls::Certificate(client_ca))
            .expect("Error adding certificate to store");

        Arc::new(
            rustls::ServerConfig::builder()
                .with_safe_defaults()
                .with_client_cert_verifier(rustls::server::AllowAnyAuthenticatedClient::new(
                    root_store.clone(),
                ))
                .with_single_cert(
                    data.cert_chain
                        .into_iter()
                        .map(rustls::Certificate)
                        .collect(),
                    rustls::PrivateKey(data.key),
                )
                .expect("Bad TLS server config"),
        )
    }

    async fn handle_connection(
        conn: TcpStream,
        acceptor: Arc<TlsAcceptor>,
        data: Arc<ManagementServiceData>,
    ) -> Result<(), anyhow::Error> {
        let stream = acceptor.accept(conn).await?;
        let (_, tls_state) = stream.get_ref();
        let client_cert = tls_state
            .peer_certificates()
            .ok_or(ManagementServiceError::Other(
                "Couldn't access peer certificate",
            ))?;
        let mut hasher = Sha1::new();
        hasher.update(&client_cert[0].0);
        let fingerprint = hex::encode(hasher.finalize());

        let authorised_fingerprint = data
            .authorised_fingerprints
            .iter()
            .find(|f| f.fingerprint == fingerprint)
            .ok_or(ManagementServiceError::InvalidFingerprint)?
            .clone();

        let service = ManagementService {
            authorised_fingerprint,
            data,
        };
        let http = hyper::server::conn::Http::new();
        http.serve_connection(stream, service).await?;

        Ok(())
    }

    pub fn start(
        config: ManagementConfig,
        tls_data: TlsData,
        mut shutdown: oneshot::Receiver<()>,
    ) -> Self {
        let (command_sender, command_receiver) = channel(128);

        let client_ca = config
            .load_client_ca()
            .expect("Failed to load management client CA");
        let service_data = Arc::new(ManagementServiceData {
            command_sender,
            authorised_fingerprints: config.authorised_fingerprints,
        });

        let data = Arc::clone(&service_data);
        let listen_address = config.address;
        let server_task = task::spawn(async move {
            let tls_config = Self::server_config(tls_data, client_ca);
            let acceptor = Arc::new(TlsAcceptor::from(Arc::clone(&tls_config)));
            let listener = TcpListener::bind(&listen_address).await.expect("Failed to bind to management address");

            loop
            {
                select!
                {
                    res = listener.accept() =>
                    {
                        if let Ok((conn, _)) = res
                        {
                            if let Err(e) = Self::handle_connection(conn, Arc::clone(&acceptor), Arc::clone(&data)).await
                            {
                                tracing::warn!("Error handling management connection: {}", e);
                            }
                        }
                    }
                    _ = &mut shutdown =>
                    {
                        break;
                    }
                }
            }
            Ok(())
        }.instrument(tracing::info_span!("management server")));

        Self {
            command_receiver,
            server_task,
            service_data,
        }
    }

    pub async fn recv(&mut self) -> Option<ManagementCommand> {
        self.command_receiver.recv().await
    }

    pub async fn wait(self) -> Result<(), anyhow::Error> {
        Ok(self.server_task.await??)
    }
}
