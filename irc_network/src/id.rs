//! Defines the various object and event ID types

use ircd_macros::object_ids;
use thiserror::Error;
use crate::validated::*;
use crate::modes::ListModeType;

pub type LocalId = i64;

#[derive(Debug,Error)]
#[error("Mismatched object ID type for event")]
pub struct WrongIdTypeError;

object_ids!(ObjectId, {
    Server: (LocalId,);
    Epoch: (LocalId,);
    Event: sequential;
    User: sequential;
    UserMode: sequential;
    Channel: sequential;
    ChannelMode: sequential;
    ChannelTopic: sequential;
    ListMode: (ChannelModeId,ListModeType);
    ListModeEntry: sequential;
    Message: sequential;

    NetworkBan: sequential;

    Nickname: (Nickname,);
    ChannelName: (ChannelName,);

    Membership: (UserId, ChannelId);
    Invite: (UserId, ChannelId);

    Config: (LocalId,);
    AuditLogEntry: sequential;
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

impl ListModeId {
    pub fn mode(&self) -> ChannelModeId { self.0 }
    pub fn list_type(&self) -> ListModeType { self.1 }
}

impl InviteId {
    pub fn user(&self) -> UserId { self.0 }
    pub fn channel(&self) -> ChannelId { self.1 }
}