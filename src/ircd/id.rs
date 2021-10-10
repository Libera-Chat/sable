use std::sync::atomic::{AtomicI64,Ordering};

pub type ServerId = i64;
pub type LocalId = i64;

#[derive(Debug,Clone,Copy,Hash,PartialEq,Eq)]
pub struct Id(ServerId, LocalId);

impl Id {
    pub fn new(server: ServerId, i: LocalId) -> Self {
        Self(server, i)
    }

    pub fn server(&self) -> &ServerId { &self.0 }
    pub fn local(&self) -> &LocalId { &self.1 }
}

#[derive(Debug)]
pub struct IdGenerator {
    server: ServerId,
    last: AtomicI64
}

impl IdGenerator {
    pub fn new(server: ServerId) -> IdGenerator {
        IdGenerator{server: server, last: AtomicI64::new(1)}
    }

    pub fn next(&self) -> Id {
        Id(self.server, self.last.fetch_add(1, Ordering::SeqCst))
    }
}