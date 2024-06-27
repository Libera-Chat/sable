//! Defines the various object and event ID types

use super::modes::ListModeType;
use super::validated::*;
use sable_macros::object_ids;
use thiserror::Error;

pub type LocalId = i64;

#[derive(Debug, Error)]
#[error("Mismatched object ID type for event")]
pub struct WrongIdTypeError;

object_ids!(ObjectId (ObjectIdGenerator) {
    Server: (LocalId,);
    Epoch: (LocalId,);
    Event: sequential;
    User: sequential;
    HistoricUser: (UserId, u32);
    UserConnection: sequential;
    Channel: sequential;
    ChannelTopic: sequential;
    ListMode: (ChannelId,ListModeType);
    ListModeEntry: sequential;
    Message: sequential;

    NetworkBan: sequential;

    Nickname: (Nickname,);
    ChannelName: (ChannelName,);

    Membership: (UserId, ChannelId);
    Invite: (UserId, ChannelId);

    Config: (LocalId,);
    AuditLogEntry: sequential;

    Account: sequential;
    NickRegistration: sequential;
    ChannelRegistration: sequential;

    ChannelAccess: (AccountId, ChannelRegistrationId);
    ChannelRole: sequential;

    SaslSession: sequential;
});

impl HistoricUserId {
    pub fn user(&self) -> &UserId {
        &self.0
    }

    pub fn serial(&self) -> u32 {
        self.1
    }
}

impl NicknameId {
    pub fn nick(&self) -> &Nickname {
        &self.0
    }
}

impl ListModeId {
    pub fn channel(&self) -> ChannelId {
        self.0
    }
    pub fn list_type(&self) -> ListModeType {
        self.1
    }
}

impl InviteId {
    pub fn user(&self) -> UserId {
        self.0
    }
    pub fn channel(&self) -> ChannelId {
        self.1
    }
}

impl MembershipId {
    pub fn user(&self) -> UserId {
        self.0
    }
    pub fn channel(&self) -> ChannelId {
        self.1
    }
}

impl ChannelAccessId {
    pub fn account(&self) -> AccountId {
        self.0
    }
    pub fn channel(&self) -> ChannelRegistrationId {
        self.1
    }
}
