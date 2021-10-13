use crate::ircd::*;

use super::*;

pub struct Membership<'a> {
    network: &'a Network,
    data: &'a state::Membership,
}

impl Membership<'_> {
    pub fn id(&self) -> MembershipId {
        self.data.id
    }
    
    pub fn user_id(&self) -> UserId {
        self.data.user
    }

    pub fn user(&self) -> Option<User> {
        self.network.user(self.data.user)
    }

    pub fn channel_id(&self) -> ChannelId {
        self.data.channel
    }

    pub fn channel(&self) -> Option<Channel> {
        self.network.channel(self.data.channel)
    }
}

impl<'a> super::ObjectWrapper<'a> for Membership<'a> {
    type Underlying = state::Membership;
    fn wrap(net: &'a Network, data: &'a state::Membership) -> Membership<'a> {
        Membership{ network: net, data: data }
    }


}