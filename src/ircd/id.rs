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

pub struct IdGenerator {
    server: ServerId,
    last: LocalId
}

impl IdGenerator {
    pub fn new(server: ServerId) -> IdGenerator {
        IdGenerator{server: server, last: 0}
    }

    pub fn next(&mut self) -> Id {
        self.last += 1;
        Id(self.server, self.last)
    }
}