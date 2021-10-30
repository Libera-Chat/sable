use super::{Network,LookupError,LookupResult};
use crate::*;

use crate::wrapper::*;

use LookupError::*;

impl Network {
    pub fn user(&self, id: UserId) -> LookupResult<wrapper::User> {
        let r: LookupResult<&state::User> = self.users.get(&id).ok_or(NoSuchUser(id));
        r.wrap(&self)
    }

    pub fn users(&self) -> impl std::iter::Iterator<Item=wrapper::User> + '_ {
        self.raw_users().wrap(&self)
    }

    pub fn raw_users(&self) -> impl std::iter::Iterator<Item=&state::User> {
        self.users.values()
    }

    pub fn user_by_nick(&self, nick: &Nickname) -> LookupResult<wrapper::User>
    {
        self.users.values().filter(|x| &x.nick == nick.value()).next().ok_or(NoSuchNick(nick.to_string())).wrap(self)
    }

    pub fn user_mode(&self, id: UModeId) -> LookupResult<wrapper::UserMode>
    {
        self.user_modes.get(&id).ok_or(NoSuchUserMode(id)).wrap(self)
    }

    pub fn channel(&self, id: ChannelId) -> LookupResult<wrapper::Channel> {
        self.channels.get(&id).ok_or(NoSuchChannel(id)).wrap(self)
    }

    pub fn channels(&self) -> impl std::iter::Iterator<Item=wrapper::Channel> + '_ {
        self.raw_channels().wrap(self)
    }

    pub fn raw_channels(&self) -> impl std::iter::Iterator<Item=&state::Channel> {
        self.channels.values()
    }

    pub fn channel_by_name(&self, name: &ChannelName) -> LookupResult<wrapper::Channel>
    {
        self.channels.values().filter(|x| &x.name == name.value()).next().ok_or(NoSuchChannelName(name.to_string())).wrap(self)
    }

    pub fn channel_mode(&self, id: CModeId) -> LookupResult<wrapper::ChannelMode>
    {
        self.channel_modes.get(&id).ok_or(NoSuchChannelMode(id)).wrap(self)
    }

    pub fn membership(&self, id: MembershipId) -> LookupResult<wrapper::Membership> {
        self.memberships.get(&id).ok_or(NoSuchMembership(id)).wrap(self)
    }

    pub fn memberships(&self) -> impl std::iter::Iterator<Item=wrapper::Membership> + '_ {
        self.raw_memberships().wrap(self)
    }

    pub fn raw_memberships(&self) -> impl std::iter::Iterator<Item=&state::Membership> {
        self.memberships.values()
    }
}