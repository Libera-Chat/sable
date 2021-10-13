use crate::ircd::*;

#[derive(Debug)]
pub struct Channel {
    pub id: ChannelId,
    pub name: String,
}

#[derive(Debug)]
pub struct Membership {
    pub id: MembershipId,
    pub channel: ChannelId,
    pub user: UserId,
}

impl Channel {
    pub fn new(id: ChannelId, name: &str) -> Channel {
        Channel{ id: id, name: name.to_string() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Membership {
    pub fn new(id: MembershipId, user: UserId, channel: ChannelId) -> Membership {
        Membership{ id: id, user: user, channel: channel }
    }

    pub fn user(&self) -> UserId {
        self.user
    }

    pub fn channel(&self) -> ChannelId {
        self.channel
    }
}