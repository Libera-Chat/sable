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
    pub mode: CModeId,
}

/// A channel membership
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Membership {
    pub id: MembershipId,
    pub channel: ChannelId,
    pub user: UserId,
    pub permissions: ChannelPermissionSet,
}

/// A channel mode
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ChannelMode {
    pub id: CModeId,
    pub modes: ChannelModeSet,
}

impl Channel {
    pub fn new(id: ChannelId, name: &ChannelName, mode: CModeId) -> Self
    {
        Channel{ id: id, name: name.clone(), mode: mode }
    }
}

impl ChannelMode {
    pub fn new(id: CModeId, modes: ChannelModeSet) -> Self
    {
        ChannelMode{ id: id, modes: modes }
    }
}

impl Membership {
    pub fn new(id: MembershipId, user: UserId, channel: ChannelId, perms: ChannelPermissionSet) -> Membership 
    {
        Membership{ id: id, user: user, channel: channel, permissions: perms }
    }
}