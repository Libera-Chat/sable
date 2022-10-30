use super::*;

use std::{
    future::Future,
    task::{ Poll, ready },
    pin::Pin,
    sync::Arc,
};
use futures::{
    stream::{
        FuturesUnordered,
        StreamExt,
    },
    future::BoxFuture,
};

pub type AsyncHandler<'a> = BoxFuture<'a, CommandResult>;

/// Wrapper type to take a boxed Future representing an async command handler,
/// drive it to completion, and handle any errors returned by the handler, notifying
/// the client connection as appropriate.
pub struct AsyncHandlerWrapper<'a>
{
    handler: AsyncHandler<'a>,
    command: Arc<ClientCommand<'a>>,
}

impl<'a> AsyncHandlerWrapper<'a>
{
    pub fn new(handler: AsyncHandler<'a>, command: Arc<ClientCommand<'a>>) -> Self
    {
        Self { handler, command }
    }
}

impl<'a> Future for AsyncHandlerWrapper<'a>
{
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output>
    {
        let result = ready!(Pin::new(&mut self.handler).poll(cx));
        if let Err(e) = result
        {
            match e
            {
                CommandError::UnderlyingError(err) => {
                    panic!("Error occurred handling command {} from {:?}: {}", self.command.command, self.command.connection.id(), err);
                },
                CommandError::UnknownError(desc) => {
                    panic!("Error occurred handling command {} from {:?}: {}", self.command.command, self.command.connection.id(), desc);
                },
                CommandError::Numeric(num) => {
                    let targeted = num.as_ref().format_for(self.command.server, &self.command.source());
                    self.command.connection.send(&targeted);
                },
                CommandError::CustomError => {
                }
            }
        }

        Poll::Ready(())
    }
}

pub struct AsyncHandlerCollection<'collection>
{
    futures: FuturesUnordered<AsyncHandlerWrapper<'collection>>,
}

impl<'collection> AsyncHandlerCollection<'collection>
{
    pub fn new() -> Self
    {
        Self { futures: FuturesUnordered::new() }
    }

    pub fn add<'handler>(&self, handler: AsyncHandlerWrapper<'handler>)
        where 'handler: 'collection
    {
        self.futures.push(handler);
    }

    pub async fn poll(&mut self)
    {
        // Poll as many as possible of the futures we're storing
        while let Some(_) = self.futures.next().await { }
    }

    pub fn is_empty(&self) -> bool
    {
        self.futures.is_empty()
    }
}