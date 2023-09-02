use super::*;
use crate::prelude::*;

/// A wrapper around a [`state::User`]
pub struct User<'a> {
    network: &'a Network,
    data: &'a state::User,
}

impl<'a> User<'a> {
    /// Return this object's ID
    pub fn id(&self) -> UserId {
        self.data.id
    }

    /// Infallibly returns a nickname for this user.
    /// If a nickname binding exists, then the associated nick is returned; otherwise,
    /// a fallback nick based on the hash of the user ID is used - this is the same
    /// computed nickname used in case of binding collisions.
    pub fn nick(&self) -> Nickname {
        self.network.infallible_nick_for_user(self.data.id)
    }

    /// Return the nickname binding currently active for this user
    pub fn nick_binding(&self) -> LookupResult<NickBinding> {
        self.network.nick_binding_for_user(self.data.id)
    }

    /// The user's username
    pub fn user(&self) -> &Username {
        &self.data.user
    }

    /// The user's visible hostname, for client protocol purposes
    pub fn visible_host(&self) -> &Hostname {
        &self.data.visible_host
    }

    /// The user's realname
    pub fn realname(&self) -> &str {
        &self.data.realname
    }

    /// The user's current modes
    pub fn mode(&self) -> UserMode {
        UserMode::wrap(self.network, &self.data.mode)
    }

    /// The server through which the user is connected
    pub fn server(&self) -> LookupResult<super::Server> {
        self.network.server(self.data.server)
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

    /// Returns the user's reason for being away, or the empty string if they are not
    pub fn away_reason(&self) -> &str {
        self.data.away_reason.as_ref()
    }

    /// The user's nick!user@host mask, as used in the IRC client protocol
    pub fn nuh(&self) -> String {
        format!(
            "{}!{}@{}",
            self.nick().value(),
            self.data.user.value(),
            self.data.visible_host.value()
        )
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
