use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ListenerId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ConnectionId(ListenerId, u32);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ListenerIdGenerator {
    last: AtomicU32,
}

impl ListenerIdGenerator {
    pub fn new(start: u32) -> Self {
        Self { last: start.into() }
    }

    pub fn next(&self) -> ListenerId {
        ListenerId(self.last.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionIdGenerator {
    listener: ListenerId,
    last: AtomicU32,
}

impl ConnectionIdGenerator {
    pub fn new(listener: ListenerId, start: u32) -> Self {
        Self {
            listener,
            last: start.into(),
        }
    }

    pub fn next(&self) -> ConnectionId {
        ConnectionId(self.listener, self.last.fetch_add(1, Ordering::Relaxed))
    }
}
