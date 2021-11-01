use ircd_macros::object_ids;
use thiserror::Error;

pub type LocalId = i64;

#[derive(Debug,Error)]
#[error("Mismatched object ID type for event")]
pub struct WrongIdTypeError;

object_ids!(ObjectId, {
    Server: (LocalId,);
    Epoch: (LocalId,);
    Event: sequential;
    User: sequential;
    UMode: sequential;
    Channel: sequential;
    CMode: sequential;
    Message: sequential;

    Membership: (UserId, ChannelId);
});

object_ids!(LocalObjectId, {
    Listener: (LocalId,) sequential;
    Connection: (ListenerId,LocalId) sequential;
});

impl EventId {
    pub fn server(&self) -> ServerId { self.0 }
    pub fn epoch(&self) -> EpochId { self.1 }
    pub fn local(&self) -> LocalId { self.2 }
}

impl EpochId {
    pub fn next(&self) -> Self
    {
        Self( self.0 + 1 )
    }
}