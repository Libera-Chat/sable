use crate::*;

use super::*;

/// A wrapper around a [`state::Membership`]
pub struct Membership<'a> {
    network: &'a Network,
    data: &'a state::Membership,
}

impl Membership<'_> {
    /// Return this object's ID
    pub fn id(&self) -> MembershipId {
        self.data.id
    }

    /// The ID of the associated user
    pub fn user_id(&self) -> UserId {
        self.data.user
    }

    /// The associated user object
    pub fn user(&self) -> LookupResult<User> {
        self.network.user(self.data.user)
    }

    /// The ID of the associated channel
    pub fn channel_id(&self) -> ChannelId {
        self.data.channel
    }

    /// The associated channel object
    pub fn channel(&self) -> LookupResult<Channel> {
        self.network.channel(self.data.channel)
    }

    /// Permission flags currently assigned to this user in this channel
    pub fn permissions(&self) -> MembershipFlagSet {
        self.data.permissions
    }
}

impl<'a> super::ObjectWrapper<'a> for Membership<'a> {
    type Underlying = state::Membership;
    fn wrap(network: &'a Network, data: &'a state::Membership) -> Self
    {
        Self { network, data }
    }


}