use super::*;

use futures::{
    stream::{
        FuturesUnordered,
        StreamExt,
    },
};

pub struct AsyncHandlerCollection
{
    futures: FuturesUnordered<AsyncHandler>,
}

impl AsyncHandlerCollection
{
    pub fn new() -> Self
    {
        Self { futures: FuturesUnordered::new() }
    }

    pub fn add(&self, handler: AsyncHandler)
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