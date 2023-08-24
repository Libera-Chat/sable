use crate::prelude::*;

use serde::{Deserialize, Serialize};

/// A channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: ChannelId,
    pub name: ChannelName,
    pub mode: ChannelMode,
}

/// A channel membership
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Membership {
    pub id: MembershipId,
    pub channel: ChannelId,
    pub user: UserId,
    pub permissions: MembershipFlagSet,
}

/// A channel mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMode {
    pub modes: ChannelModeSet,
    pub key: Option<ChannelKey>,
}

/// An entry in a list mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListModeEntry {
    pub id: ListModeEntryId,
    pub list: ListModeId,
    pub timestamp: i64,
    pub setter: String,
    pub pattern: Pattern,
}

/// A channel topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelTopic {
    pub id: ChannelTopicId,
    pub channel: ChannelId,
    pub text: String,
    pub setter_info: String,
    pub timestamp: i64,
}

/// An invitation to a channel. The user and channel are encapsulated in the
/// [InviteId] type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInvite {
    pub id: InviteId,
    pub source: UserId,
    pub timestamp: i64,
}

impl Channel {
    pub fn new(id: ChannelId, name: ChannelName, mode: ChannelMode) -> Self {
        Channel { id, name, mode }
    }
}

impl ChannelMode {
    pub fn new(modes: ChannelModeSet) -> Self {
        ChannelMode { modes, key: None }
    }
}

impl ChannelTopic {
    pub fn new(
        id: ChannelTopicId,
        channel: ChannelId,
        text: String,
        setter_info: String,
        timestamp: i64,
    ) -> ChannelTopic {
        ChannelTopic {
            id,
            channel,
            text,
            setter_info,
            timestamp,
        }
    }
}

impl Membership {
    pub fn new(
        id: MembershipId,
        user: UserId,
        channel: ChannelId,
        permissions: MembershipFlagSet,
    ) -> Membership {
        Membership {
            id,
            user,
            channel,
            permissions,
        }
    }
}

impl ListModeEntry {
    pub fn new(
        id: ListModeEntryId,
        list: ListModeId,
        timestamp: i64,
        setter: String,
        pattern: Pattern,
    ) -> Self {
        Self {
            id,
            list,
            timestamp,
            setter,
            pattern,
        }
    }
}

impl ChannelInvite {
    pub fn new(id: InviteId, source: UserId, timestamp: i64) -> Self {
        Self {
            id,
            source,
            timestamp,
        }
    }
}
