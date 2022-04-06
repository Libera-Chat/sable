use super::*;
use irc_server::server::ServerManagementCommand;
use irc_server::server::ServerManagementCommandType;
use rpc_protocols::ShutdownAction;

use std::{
    net::SocketAddr,
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll
    }
};
use tokio::{
    sync::{
        mpsc::{
            Sender,
            Receiver,
            channel,
        },
        broadcast,
        oneshot
    },
    task,
};
use hyper::{
    Body,
    Method,
    Request,
    Response,
    StatusCode,
};
use tracing::Instrument;

pub struct ManagementServer
{
    command_receiver: Receiver<ManagementCommand>,
    server_task: task::JoinHandle<Result<(), hyper::Error>>,
}

struct ManagementService
{
    command_sender: Sender<ManagementCommand>
}

struct MakeManagementService
{
    command_sender: Sender<ManagementCommand>
}

fn internal_error() -> hyper::Result<Response<Body>>
{
    let mut response = Response::default();
    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    Ok(response)
}

impl ManagementService
{
    async fn server_management_command(command_sender: Sender<ManagementCommand>, cmd: ServerManagementCommandType) -> Result<Response<Body>, hyper::Error>
    {
        let (send, recv) = oneshot::channel();
        let cmd = ServerManagementCommand { cmd, response: send };
        if command_sender.send(ManagementCommand::ServerCommand(cmd)).await.is_err()
        {
            internal_error()
        }
        else if let Ok(response) = recv.await
        {
            Ok(Response::new(Body::from(response)))
        }
        else
        {
            internal_error()
        }
    }

    async fn shutdown_command(command_sender: Sender<ManagementCommand>, cmd: ShutdownAction) -> Result<Response<Body>, hyper::Error>
    {
        if command_sender.send(ManagementCommand::Shutdown(cmd)).await.is_ok()
        {
            Ok(Response::new(Body::empty()))
        }
        else
        {
            internal_error()
        }
    }
}

impl hyper::service::Service<Request<Body>> for ManagementService
{
    type Response = Response<Body>;
    type Error = hyper::Error;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>>
    {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future
    {
        let command_sender = self.command_sender.clone();

        tracing::info!(method=?req.method(), path=?req.uri().path(), "Got management request");

        Box::pin(async move {
            match (req.method(), req.uri().path())
            {
                (&Method::GET, "/statistics") =>
                {
                    Self::server_management_command(command_sender, ServerManagementCommandType::ServerStatistics).await
                }
                (&Method::GET, "/dump-network") =>
                {
                    Self::server_management_command(command_sender, ServerManagementCommandType::DumpNetwork).await
                }
                (&Method::GET, "/dump-events") =>
                {
                    Self::server_management_command(command_sender, ServerManagementCommandType::DumpEvents).await
                }
                (&Method::POST, "/shutdown") =>
                {
                    Self::shutdown_command(command_sender, ShutdownAction::Shutdown).await
                }
                (&Method::POST, "/restart") =>
                {
                    Self::shutdown_command(command_sender, ShutdownAction::Restart).await
                }
                (&Method::POST, "/upgrade") =>
                {
                    Self::shutdown_command(command_sender, ShutdownAction::Upgrade).await
                }
                _ =>
                {
                    let mut response = Response::default();
                    *response.status_mut() = StatusCode::NOT_FOUND;
                    Ok(response)

                }
            }
        })
    }
}

impl<T> hyper::service::Service<T> for MakeManagementService {
    type Response = ManagementService;
    type Error = hyper::Error;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let sender = self.command_sender.clone();
        let fut = async move { Ok(ManagementService { command_sender: sender }) };
        Box::pin(fut)
    }
}

impl ManagementServer
{
    pub fn start(listen_addr: SocketAddr, mut shutdown: broadcast::Receiver<ShutdownAction>) -> Self
    {
        let (command_sender, command_receiver) = channel(128);

        let server_task = task::spawn(async move {
            let command_sender = command_sender;

            let service = MakeManagementService { command_sender };
            let server = hyper::Server::bind(&listen_addr)
                            .serve(service)
                            .with_graceful_shutdown(async { shutdown.recv().await.ok(); });

            server.await
        }.instrument(tracing::info_span!("management server")));

        Self {
            command_receiver,
            server_task,
        }
    }

    pub async fn recv(&mut self) -> Option<ManagementCommand>
    {
        self.command_receiver.recv().await
    }

    pub async fn wait(self) -> Result<(), Box<dyn std::error::Error>>
    {
        Ok(self.server_task.await??)
    }
}