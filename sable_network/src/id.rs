//! Defines the various object and event ID types

use super::modes::ListModeType;
use super::validated::*;
use sable_macros::object_ids;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Serialize, Deserialize)]
pub struct Uuid7(Uuid);

impl Deref for Uuid7 {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Uuid7 {
    pub fn new_now() -> Self {
        Self(Uuid::now_v7())
    }
}

#[derive(Debug, Error)]
#[error("Mismatched object ID type for event")]
pub struct WrongIdTypeError;

pub type EpochId = u64;

object_ids!(ObjectId (ObjectIdGenerator) {
    Server: (u16,);
    Event: snowflake;
    User: snowflake;
    HistoricUser: (UserId, u32);
    UserConnection: snowflake;
    Channel: snowflake;
    ChannelTopic: snowflake;
    ListMode: (ChannelId,ListModeType);
    ListModeEntry: snowflake;
    Message: (Uuid7,);

    NetworkBan: snowflake;

    Nickname: (Nickname,);
    ChannelName: (ChannelName,);

    Membership: (UserId, ChannelId);
    Invite: (UserId, ChannelId);

    Config: (u64,);
    AuditLogEntry: snowflake;

    Account: snowflake;
    NickRegistration: snowflake;
    ChannelRegistration: snowflake;

    ChannelAccess: (AccountId, ChannelRegistrationId);
    ChannelRole: snowflake;

    SaslSession: snowflake;
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

impl UserId {
    /// Construct an ID for an alias user, based on a numeric configured ID.
    /// The resulting snowflake will have timestamp and server portions set to 0,
    /// with the provided `id` in the serial bits.
    ///
    /// Note that `id` must be less than 4096 - i.e. it must fit in 12 bits.
    pub fn alias(id: u16) -> Self {
        if id > 4095 {
            panic!("Attempted to construct an alias user ID >4095")
        }
        Self(Snowflake(id as u64))
    }
}
