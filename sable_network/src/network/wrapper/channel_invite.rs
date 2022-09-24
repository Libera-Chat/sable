use crate::prelude::*;
use super::*;

/// A wrapper around a [`state::ChannelInvite`]
pub struct ChannelInvite<'a> {
    network: &'a Network,
    data: &'a state::ChannelInvite,
}

impl ChannelInvite<'_> {
    /// Return this object's ID
    pub fn id(&self) -> InviteId {
        self.data.id
    }

    /// Return the [User] who was invited
    pub fn user(&self) -> LookupResult<User>
    {
        self.network.user(self.data.id.user())
    }

    /// Return the [Channel] to which this invite applies
    pub fn channel(&self) -> LookupResult<Channel>
    {
        self.network.channel(self.data.id.channel())
    }

    /// Return the user ID who sent the invite
    pub fn source(&self) -> LookupResult<User>
    {
        self.network.user(self.data.source)
    }

    /// Timestamp when this invite was sent
    pub fn timestamp(&self) -> i64
    {
        self.data.timestamp
    }
}

impl<'a> super::ObjectWrapper<'a> for ChannelInvite<'a> {
    type Underlying = state::ChannelInvite;

    fn wrap(network: &'a Network, data: &'a state::ChannelInvite) -> Self
    {
        Self { network, data }
    }

    fn raw(&self) -> &'a Self::Underlying { self.data }
}
