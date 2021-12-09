use crate::*;
use crate::id::*;
use crate::validated::*;
use serde::{
    Serialize,
    Deserialize
};

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
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ChannelTopic {
    pub id: ChannelTopicId,
    pub channel: ChannelId,
    pub text: String,
    pub setter_info: String,
    pub timestamp: i64,
}

impl Channel {
    pub fn new(id: ChannelId, name: &ChannelName, mode: ChannelModeId) -> Self
    {
        Channel{ id: id, name: name.clone(), mode: mode }
    }
}

impl ChannelMode {
    pub fn new(id: ChannelModeId, modes: ChannelModeSet) -> Self
    {
        ChannelMode{ id: id, modes: modes }
    }
}

impl ChannelTopic {
    pub fn new(id: ChannelTopicId, channel: ChannelId, text: String, setter_info: String, timestamp: i64) -> ChannelTopic
    {
        ChannelTopic { id: id, channel: channel, text: text, setter_info: setter_info, timestamp: timestamp }
    }
}

impl Membership {
    pub fn new(id: MembershipId, user: UserId, channel: ChannelId, perms: MembershipFlagSet) -> Membership 
    {
        Membership{ id: id, user: user, channel: channel, permissions: perms }
    }
}