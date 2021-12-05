use ircd_macros::object_ids;
use thiserror::Error;
use crate::validated::*;

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

    Nickname: (Nickname,);

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

impl NicknameId {
    pub fn nick(&self) -> &Nickname { &self.0 }
}