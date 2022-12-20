use super::{Network,LookupError,LookupResult};
use crate::network::event::*;
use crate::prelude::*;
use crate::network::state_utils;

use crate::network::wrapper::*;

use LookupError::*;

impl Network {
    /// The current event clock for this network state.
    pub fn clock(&self) -> &EventClock {
        &self.clock
    }

    /// Look up a user by ID.
    pub fn user(&self, id: UserId) -> LookupResult<wrapper::User> {
        let r: LookupResult<&state::User> = self.users.get(&id).ok_or(NoSuchUser(id));
        r.wrap(self)
    }

    /// Return an iterator over all users.
    pub fn users(&self) -> impl std::iter::Iterator<Item=wrapper::User> + '_ {
        self.raw_users().wrap(self)
    }

    /// Return an iterator over the raw `state::User` objects.
    pub fn raw_users(&self) -> impl std::iter::Iterator<Item=&state::User> {
        self.users.values()
    }

    /// Return a nickname binding for the given nick.
    pub fn nick_binding(&self, nick: &Nickname) -> LookupResult<wrapper::NickBinding>
    {
        self.nick_bindings.get(nick).ok_or_else(|| NoSuchNick(nick.to_string())).wrap(self)
    }

    /// Iterate over raw [`state::NickBinding`] objects
    pub fn raw_nick_bindings(&self) -> impl std::iter::Iterator<Item=&state::NickBinding>
    {
        self.nick_bindings.values()
    }

    /// Iterate over nickname bindings
    pub fn nick_bindings(&self) -> impl std::iter::Iterator<Item=wrapper::NickBinding>
    {
        self.nick_bindings.values().wrap(self)
    }

    /// Return the current nick binding information for a given user ID
    pub fn nick_binding_for_user(&self, user: UserId) -> LookupResult<wrapper::NickBinding>
    {
        self.raw_nick_bindings().find(|b| b.user == user).ok_or(NoNickForUser(user)).wrap(self)
    }

    /// Return a nickname for the given user. If a nick binding for that user exists, it is used,
    /// otherwise a hashed nickname (as used in collision resolution) is returned
    pub fn infallible_nick_for_user(&self, user: UserId) -> Nickname
    {
        if let Ok(binding) = self.nick_binding_for_user(user)
        {
            binding.nick()
        }
        else
        {
            state_utils::hashed_nick_for(user)
        }
    }

    /// Look up the user currently using the given nickname
    pub fn user_by_nick(&self, nick: &Nickname) -> LookupResult<wrapper::User>
    {
        self.user(self.nick_bindings.get(nick).ok_or_else(|| NoSuchNick(nick.to_string()))?.user)
    }

    /// Look up a channel by ID
    pub fn channel(&self, id: ChannelId) -> LookupResult<wrapper::Channel> {
        self.channels.get(&id).ok_or(NoSuchChannel(id)).wrap(self)
    }

    /// Iterate over channels
    pub fn channels(&self) -> impl std::iter::Iterator<Item=wrapper::Channel> + '_ {
        self.raw_channels().wrap(self)
    }

    /// Iterate over raw [`state::Channel`] objects
    pub fn raw_channels(&self) -> impl std::iter::Iterator<Item=&state::Channel> {
        self.channels.values()
    }

    /// Look up a raw channel by name.
    pub(crate) fn raw_channel_by_name(&self, name: &ChannelName) -> LookupResult<&state::Channel>
    {
        self.channels.values().find(|x| &x.name == name).ok_or_else(|| NoSuchChannelName(*name))
    }

    /// Look up a channel by name.
    pub fn channel_by_name(&self, name: &ChannelName) -> LookupResult<wrapper::Channel>
    {
        self.channels.values().find(|x| &x.name == name).ok_or_else(|| NoSuchChannelName(*name)).wrap(self)
    }

    /// Look up a ban-type list by ID
    pub fn list_mode(&self, id: ListModeId) -> wrapper::ListMode
    {
        wrapper::ListMode::new(self, id)
    }

    /// Find the channel mode entry corresponding to a given banlist
    pub fn channel_for_list(&self, id: ListModeId) -> LookupResult<wrapper::Channel>
    {
        self.channels.get(&id.channel()).ok_or(NoChannelForList(id)).wrap(self)
    }

    /// The list entries belonging to a given list ID
    pub fn entries_for_list(&self, id: ListModeId) -> impl std::iter::Iterator<Item=wrapper::ListModeEntry>
    {
        self.list_mode_entries.values().filter(move |x| x.list == id).wrap(self)
    }

    /// Look up a channel topic by ID.
    pub fn channel_topic(&self, id: ChannelTopicId) -> LookupResult<wrapper::ChannelTopic>
    {
        self.channel_topics.get(&id).ok_or(NoSuchChannelTopic(id)).wrap(self)
    }

    /// Find the topic associated with a given channel, if any.
    pub fn topic_for_channel(&self, id: ChannelId) -> LookupResult<wrapper::ChannelTopic>
    {
        self.channel_topics.values().find(|t| t.channel == id).ok_or(NoTopicForChannel(id)).wrap(self)
    }

    /// Look up a membership by ID.
    pub fn membership(&self, id: MembershipId) -> LookupResult<wrapper::Membership> {
        self.memberships.get(&id).ok_or(NoSuchMembership(id)).wrap(self)
    }

    /// Iterate over all memberships.
    pub fn memberships(&self) -> impl std::iter::Iterator<Item=wrapper::Membership> + '_ {
        self.raw_memberships().wrap(self)
    }

    /// Iterate over raw membership states
    pub fn raw_memberships(&self) -> impl std::iter::Iterator<Item=&state::Membership> {
        self.memberships.values()
    }

    /// Look up an invite by ID
    pub fn channel_invite(&self, id: InviteId) -> LookupResult<wrapper::ChannelInvite>
    {
        self.channel_invites.get(&id).ok_or(NoSuchInvite(id)).wrap(self)
    }

    /// Look up a server by ID
    pub fn server(&self, id: ServerId) -> LookupResult<wrapper::Server> {
        self.servers.get(&id).ok_or(NoSuchServer(id)).wrap(self)
    }

    /// Iterate over servers
    pub fn servers(&self) -> impl std::iter::Iterator<Item=wrapper::Server>
    {
        self.servers.values().wrap(self)
    }

    /// Look up a message by ID
    pub fn message(&self, id: MessageId) -> LookupResult<wrapper::Message>
    {
        self.messages.get(&id).ok_or(NoSuchMessage(id)).wrap(self)
    }

    /// Iterate over K:Lines
    pub fn klines(&self) -> impl std::iter::Iterator<Item=wrapper::KLine>
    {
        self.klines.values().wrap(self)
    }

    /// Retrieve the server name of the current active services
    pub fn current_services(&self) -> Option<ServerName>
    {
        self.current_services.and_then(|id| self.servers.get(&id))
                             .map(|s| s.name.clone())
    }

    /// Retrieve the current network configuration
    pub fn config(&self) -> &config::NetworkConfig
    {
        &self.config
    }

    /// Retrieve an audit log entry
    pub fn audit_entry(&self, id: AuditLogEntryId) -> LookupResult<&state::AuditLogEntry>
    {
        self.audit_log.get(&id).ok_or(NoSuchAuditLogEntry(id))
    }

    /// Retrieve an account
    pub fn account(&self, id: AccountId) -> LookupResult<wrapper::Account>
    {
        self.accounts.get(&id).ok_or(NoSuchAccount(id)).wrap(self)
    }

    /// Retrieve an account by name
    pub fn account_by_name(&self, name: Nickname) -> LookupResult<wrapper::Account>
    {
        self.accounts.values().find(|a| a.name == name)
                              .ok_or(LookupError::NoSuchAccountNamed(name))
                              .wrap(self)
    }

    /// Iterate over accounts
    pub fn accounts(&self) -> impl Iterator<Item=wrapper::Account>
    {
        self.accounts.values().wrap(self)
    }

    /// Retrieve a nickname registration
    pub fn nick_registration(&self, id: NickRegistrationId) -> LookupResult<wrapper::NickRegistration>
    {
        self.nick_registrations.get(&id).ok_or(NoSuchNickRegistration(id)).wrap(self)
    }

    /// Iterate over nick registrations
    pub fn nick_registrations(&self) -> impl Iterator<Item=wrapper::NickRegistration>
    {
        self.nick_registrations.values().wrap(self)
    }

    /// Retrieve a channel registration
    pub fn channel_registration(&self, id: ChannelRegistrationId) -> LookupResult<wrapper::ChannelRegistration>
    {
        self.channel_registrations.get(&id).ok_or(NoSuchChannelRegistration(id)).wrap(self)
    }

    /// Retrieve a channel registration by name
    pub fn channel_registration_by_name(&self, name: ChannelName) -> LookupResult<wrapper::ChannelRegistration>
    {
        self.channel_registrations.values()
                                  .find(|c| c.channelname == name)
                                  .ok_or(ChannelNotRegistered(name))
                                  .wrap(self)
    }

    /// Iterate over channel registrations
    pub fn channel_registrations(&self) -> impl Iterator<Item=wrapper::ChannelRegistration>
    {
        self.channel_registrations.values().wrap(self)
    }

    /// Retrieve a channel access entry
    pub fn channel_access(&self, id: ChannelAccessId) -> LookupResult<wrapper::ChannelAccess>
    {
        self.channel_accesses.get(&id).ok_or(NoSuchChannelAccess(id)).wrap(self)
    }

    /// Iterate over channel access entries
    pub fn channel_accesses(&self) -> impl Iterator<Item=wrapper::ChannelAccess>
    {
        self.channel_accesses.values().wrap(self)
    }

    /// Retrieve a channel role
    pub fn channel_role(&self, id: ChannelRoleId) -> LookupResult<wrapper::ChannelRole>
    {
        self.channel_roles.get(&id).ok_or(NoSuchChannelRole(id)).wrap(self)
    }

    /// Iterate over all channel roles
    pub fn channel_roles(&self) -> impl Iterator<Item=wrapper::ChannelRole>
    {
        self.channel_roles.values().wrap(self)
    }
}