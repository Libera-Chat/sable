use super::{Network,LookupError,LookupResult};
use crate::event::*;
use crate::*;

use crate::wrapper::*;

use LookupError::*;

impl Network {
    pub fn clock(&self) -> &EventClock {
        &self.clock
    }

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

    pub fn nick_binding(&self, nick: &Nickname) -> LookupResult<wrapper::NickBinding>
    {
        self.nick_bindings.get(nick).ok_or(NoSuchNick(nick.to_string())).wrap(self)
    }

    pub fn raw_nick_bindings(&self) -> impl std::iter::Iterator<Item=&state::NickBinding>
    {
        self.nick_bindings.values()
    }

    pub fn nick_bindings(&self) -> impl std::iter::Iterator<Item=wrapper::NickBinding>
    {
        self.nick_bindings.values().wrap(&self)
    }

    pub fn nick_binding_for_user(&self, user: UserId) -> LookupResult<wrapper::NickBinding>
    {
        self.raw_nick_bindings().filter(|b| b.user == user).next().ok_or(NoNickForUser(user)).wrap(self)
    }

    pub fn user_by_nick(&self, nick: &Nickname) -> LookupResult<wrapper::User>
    {
        self.user(self.nick_bindings.get(nick).ok_or(NoSuchNick(nick.to_string()))?.user)
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

    pub fn server(&self, id: ServerId) -> LookupResult<wrapper::Server> {
        self.servers.get(&id).ok_or(NoSuchServer(id)).wrap(self)
    }

    pub fn servers(&self) -> impl std::iter::Iterator<Item=wrapper::Server>
    {
        self.servers.values().wrap(self)
    }

    pub fn message(&self, id: MessageId) -> LookupResult<wrapper::Message>
    {
        self.messages.get(&id).ok_or(NoSuchMessage(id)).wrap(self)
    }
}