use crate::*;
use crate::id::*;
use crate::validated::*;
use serde::{
    Serialize,
    Deserialize
};
use irc_strings::matches::Pattern;

/// A channel
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Channel {
    pub id: ChannelId,
    pub name: ChannelName,
    pub mode: ChannelModeId,
}

/// A channel membership
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Membership {
    pub id: MembershipId,
    pub channel: ChannelId,
    pub user: UserId,
    pub permissions: MembershipFlagSet,
}

/// A channel mode
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ChannelMode {
    pub id: ChannelModeId,
    pub modes: ChannelModeSet,
    pub key: Option<ChannelKey>,
}

/// A channel list-type mode (e.g. bqeI)
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ListMode {
    pub id: ListModeId,
    pub list_type: ListModeType,
}

/// An entry in a list mode
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ListModeEntry {
    pub id: ListModeEntryId,
    pub list: ListModeId,
    pub timestamp: i64,
    pub setter: String,
    pub pattern: Pattern,
}

/// A channel topic
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ChannelTopic {
    pub id: ChannelTopicId,
    pub channel: ChannelId,
    pub text: String,
    pub setter_info: String,
    pub timestamp: i64,
}

/// An invitation to a channel. The user and channel are encapsulated in the
/// [InviteId] type.
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ChannelInvite {
    pub id: InviteId,
    pub source: UserId,
    pub timestamp: i64
}

impl Channel {
    pub fn new(id: ChannelId, name: ChannelName, mode: ChannelModeId) -> Self
    {
        Channel { id, name, mode }
    }
}

impl ChannelMode {
    pub fn new(id: ChannelModeId, modes: ChannelModeSet) -> Self
    {
        ChannelMode { id, modes, key: None }
    }
}

impl ChannelTopic {
    pub fn new(id: ChannelTopicId, channel: ChannelId, text: String, setter_info: String, timestamp: i64) -> ChannelTopic
    {
        ChannelTopic { id, channel, text, setter_info, timestamp }
    }
}

impl Membership {
    pub fn new(id: MembershipId, user: UserId, channel: ChannelId, permissions: MembershipFlagSet) -> Membership
    {
        Membership{ id, user, channel, permissions }
    }
}

impl ListMode {
    pub fn new(id: ListModeId, list_type: ListModeType) -> Self
    {
        Self { id, list_type }
    }
}

impl ListModeEntry {
    pub fn new(id: ListModeEntryId, list: ListModeId, timestamp: i64, setter: String, pattern: Pattern) -> Self
    {
        Self { id, list, timestamp, setter, pattern }
    }
}

impl ChannelInvite {
    pub fn new(id: InviteId, source: UserId, timestamp: i64) -> Self
    {
        Self { id, source, timestamp }
    }
}