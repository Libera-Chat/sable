use crate::ircd::*;

use super::*;

pub struct User<'a> {
    network: &'a Network,
    data: &'a state::User,
}

impl User<'_> {
    pub fn id(&self) -> UserId {
        self.data.id
    }
    
    pub fn nick(&self) -> &Nickname {
        &self.data.nick
    }

    pub fn user(&self) -> &Username {
        &self.data.user
    }

    pub fn visible_host(&self) -> &Hostname {
        &self.data.visible_host
    }

    pub fn mode(&self) -> LookupResult<UserMode> {
        self.network.user_mode(self.data.mode_id)
    }

    pub fn channels(&self) -> impl Iterator<Item=Membership> {
        let my_id = self.data.id;
        self.network.raw_memberships().filter(move|x| x.user == my_id).wrap(self.network)
    }

    pub fn is_in_channel(&self, c: ChannelId) -> Option<Membership>
    {
        self.channels().filter(|m| m.channel_id() == c).next()
    }
}

impl<'a> super::ObjectWrapper<'a> for User<'a> {
    type Underlying = state::User;

    fn wrap(net: &'a Network, data: &'a state::User) -> Self {
        Self{network: net, data: data}
    }
}