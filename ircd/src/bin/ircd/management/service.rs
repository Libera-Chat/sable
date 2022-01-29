use super::*;
use irc_server::server::ServerManagementCommand;

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
    sync::mpsc::{
        Sender,
        Receiver,
        channel,
    },
    sync::oneshot,
    task,
};
use hyper::{
    Body,
    Method,
    Request,
    Response,
    StatusCode,
};

pub struct ManagementServer
{
    command_receiver: Receiver<ManagementCommand>,
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

impl hyper::service::Service<Request<Body>> for ManagementService
{
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>>
    {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future
    {
        let command_sender = self.command_sender.clone();

        Box::pin(async move {
            match (req.method(), req.uri().path())
            {
                (&Method::GET, "/statistics") =>
                {
                    let (send, recv) = oneshot::channel();
                    if command_sender.send(ManagementCommand::ServerCommand(ServerManagementCommand::ServerStatistics(send))).await.is_err()
                    {
                        internal_error()
                    }
                    else if let Ok(stat) = recv.await
                    {
                        Ok(Response::new(Body::from(stat)))
                    }
                    else
                    {
                        internal_error()
                    }
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
    pub fn start(listen_addr: SocketAddr) -> Self
    {
        let (command_sender, command_receiver) = channel(128);

        task::spawn(async move {
            let command_sender = command_sender;

            let service = MakeManagementService { command_sender: command_sender };
            let server = hyper::Server::bind(&listen_addr)
                            .serve(service);

            server.await
        });

        Self {
            command_receiver: command_receiver,
        }
    }

    pub async fn recv(&mut self) -> Option<ManagementCommand>
    {
        self.command_receiver.recv().await
    }
}