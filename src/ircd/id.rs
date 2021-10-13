use ircd_macros::object_ids;

pub type ServerId = i64;
pub type LocalId = i64;

object_ids! {
    Event: (ServerId, LocalId) sequential;
    User: (ServerId, LocalId) sequential;
    Channel: (ServerId, LocalId) sequential;
    Membership: (UserId, ChannelId);
    Listener: (LocalId,) sequential;
    Connection: (ListenerId, LocalId,) sequential;
    Message: (ServerId, LocalId) sequential;
}

impl EventId {
    pub fn server(&self) -> ServerId { self.0 }
    pub fn local(&self) -> LocalId { self.1 }
}