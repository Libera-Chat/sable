use super::*;

use futures::stream::{FuturesUnordered, StreamExt};

pub struct AsyncHandlerCollection<'a> {
    futures: FuturesUnordered<AsyncHandler<'a>>,
}

impl Default for AsyncHandlerCollection<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> AsyncHandlerCollection<'a> {
    pub fn new() -> Self {
        Self {
            futures: FuturesUnordered::new(),
        }
    }

    pub fn add(&self, handler: AsyncHandler<'a>) {
        self.futures.push(handler);
    }

    pub async fn poll(&mut self) {
        // Poll as many as possible of the futures we're storing
        while (self.futures.next().await).is_some() {}
    }

    pub fn is_empty(&self) -> bool {
        self.futures.is_empty()
    }
}
