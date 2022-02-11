use crate::*;
use super::*;

/// A wrapper around a [`state::Channel`]
pub struct Channel<'a> {
    network: &'a Network,
    data: &'a state::Channel,
}

impl Channel<'_> {
    /// Return this object's ID
    pub fn id(&self) -> ChannelId {
        self.data.id
    }

    /// The channel's name
    pub fn name(&self) -> &str {
        self.data.name.value()
    }

    /// The [ChannelMode] for this channel
    pub fn mode(&self) -> LookupResult<ChannelMode> {
        self.network.channel_mode(self.data.mode)
    }

    /// Iterate over the channel's members
    pub fn members(&self) -> impl Iterator<Item=Membership> {
        let my_id = self.data.id;
        self.network.raw_memberships().filter(move |x| x.channel == my_id).wrap(self.network)
    }

    /// Test whether the given user is a member of this channel
    pub fn has_member(&self, u: UserId) -> Option<Membership>
    {
        self.members().find(|m| m.user_id() == u)
    }
}

impl<'a> super::ObjectWrapper<'a> for Channel<'a> {
    type Underlying = state::Channel;

    fn wrap(network: &'a Network, data: &'a state::Channel) -> Self
    {
        Self { network, data }
    }
}
