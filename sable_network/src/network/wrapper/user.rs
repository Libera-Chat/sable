use super::*;
use crate::prelude::*;

/// A wrapper around a [`state::User`]
#[derive(Debug)]
pub struct User<'a> {
    network: &'a Network,
    data: &'a state::User,
}

/// Common functionality for current and historic user objects
pub trait WrappedUser {
    /// Return this object's ID
    fn id(&self) -> UserId;

    /// Return the historic user ID referring to this user's current state
    fn historic_id(&self) -> HistoricUserId;

    /// Infallibly returns a nickname for this user.
    /// If a nickname binding exists, then the associated nick is returned; otherwise,
    /// a fallback nick based on the hash of the user ID is used - this is the same
    /// computed nickname used in case of binding collisions.
    fn nick(&self) -> Nickname;

    /// The user's username
    fn user(&self) -> &Username;

    /// The user's visible hostname, for client protocol purposes
    fn visible_host(&self) -> &Hostname;

    /// The user's realname
    fn realname(&self) -> &Realname;

    /// Returns the user's reason for being away, or the empty string if they are not
    fn away_reason(&self) -> Option<&AwayReason>;

    /// The user's nick!user@host mask, as used in the IRC client protocol
    fn nuh(&self) -> String;

    /// Return the user's account name, if any
    fn account_name(&self) -> Option<Nickname>;
}

impl WrappedUser for User<'_> {
    fn id(&self) -> UserId {
        self.data.id
    }

    fn historic_id(&self) -> HistoricUserId {
        HistoricUserId::new(self.data.id, self.data.serial)
    }

    fn nick(&self) -> Nickname {
        self.network.infallible_nick_for_user(self.data.id)
    }

    fn user(&self) -> &Username {
        &self.data.user
    }

    fn visible_host(&self) -> &Hostname {
        &self.data.visible_host
    }

    fn realname(&self) -> &Realname {
        &self.data.realname
    }

    fn away_reason(&self) -> Option<&AwayReason> {
        self.data.away_reason.as_ref()
    }

    fn nuh(&self) -> String {
        format!(
            "{}!{}@{}",
            self.nick().value(),
            self.data.user.value(),
            self.data.visible_host.value()
        )
    }

    fn account_name(&self) -> Option<Nickname> {
        self.account().ok().flatten().map(|a| a.name())
    }
}

impl<'a> User<'a> {
    /// Return the nickname binding currently active for this user
    pub fn nick_binding(&self) -> LookupResult<NickBinding> {
        self.network.nick_binding_for_user(self.data.id)
    }

    /// The user's current modes
    pub fn mode(&self) -> UserMode {
        UserMode::wrap(self.network, &self.data.mode)
    }

    /// Iterate over the user's connections
    pub fn connections(&self) -> impl Iterator<Item = UserConnection> + '_ {
        let my_id = self.data.id;
        self.network
            .raw_user_connections()
            .filter(move |c| c.user == my_id)
            .wrap(self.network)
    }

    /// Iterate over the user's channel memberships
    pub fn channels(&self) -> impl Iterator<Item = Membership> {
        let my_id = self.data.id;
        self.network
            .raw_memberships()
            .filter(move |x| x.user == my_id)
            .wrap(self.network)
    }

    /// Test whether the user is in a given channel
    pub fn is_in_channel(&self, c: ChannelId) -> Option<Membership> {
        self.channels().find(|m| m.channel_id() == c)
    }

    /// Test whether an invite exists for this user to a given channel
    pub fn has_invite_for(&self, c: ChannelId) -> Option<ChannelInvite> {
        self.network
            .channel_invite(InviteId::new(self.data.id, c))
            .ok()
    }

    /// Test whether this user is a network operator
    pub fn is_oper(&self) -> bool {
        self.data.oper_privileges.is_some()
    }

    /// Access the user's operator privilege information
    pub fn oper_privileges(&self) -> Option<&state::UserPrivileges> {
        self.data.oper_privileges.as_ref()
    }

    /// Return the user's session key, if any
    pub fn session_key(&self) -> Option<&state::UserSessionKey> {
        self.data.session_key.as_ref()
    }

    /// Return the user's account, if any
    pub fn account(&self) -> LookupResult<Option<super::Account<'a>>> {
        self.data
            .account
            .map(|id| self.network.account(id))
            .transpose()
    }

    /// Determine whether this user refers to a compatibility alias
    pub fn is_alias_user(&self) -> Option<&config::AliasUser> {
        self.network.user_is_alias(self.data.id)
    }
}

impl<'a> super::ObjectWrapper<'a> for User<'a> {
    type Underlying = state::User;

    fn wrap(network: &'a Network, data: &'a state::User) -> Self {
        Self { network, data }
    }

    fn raw(&self) -> &'a Self::Underlying {
        self.data
    }
}
