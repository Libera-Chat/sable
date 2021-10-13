use super::Network;
use crate::ircd::*;

use crate::ircd::wrapper::*;

impl Network {
    pub fn user(&self, id: UserId) -> Option<wrapper::User> {
        self.users.get(&id).wrap(&self)
    }

    pub fn users(&self) -> impl std::iter::Iterator<Item=wrapper::User> + '_ {
        self.raw_users().wrap(&self)
    }

    pub fn raw_users(&self) -> impl std::iter::Iterator<Item=&state::User> {
        self.users.values()
    }

    pub fn user_by_nick(&self, nick: &str) -> Option<wrapper::User>
    {
        self.users.values().filter(|x| x.nick == nick).next().wrap(self)
    }

    pub fn channel(&self, id: ChannelId) -> Option<wrapper::Channel> {
        self.channels.get(&id).wrap(self)
    }

    pub fn channels(&self) -> impl std::iter::Iterator<Item=wrapper::Channel> + '_ {
        self.raw_channels().wrap(self)
    }

    pub fn raw_channels(&self) -> impl std::iter::Iterator<Item=&state::Channel> {
        self.channels.values()
    }

    pub fn channel_by_name(&self, name: &str) -> Option<wrapper::Channel>
    {
        self.channels.values().filter(|x| x.name() == name).next().wrap(self)
    }

    pub fn membership(&self, id: MembershipId) -> Option<wrapper::Membership> {
        self.memberships.get(&id).wrap(self)
    }

    pub fn memberships(&self) -> impl std::iter::Iterator<Item=wrapper::Membership> + '_ {
        self.raw_memberships().wrap(self)
    }

    pub fn raw_memberships(&self) -> impl std::iter::Iterator<Item=&state::Membership> {
        self.memberships.values()
    }

}