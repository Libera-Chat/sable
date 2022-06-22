use crate::prelude::*;
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
    pub fn name(&self) -> &ChannelName {
        &self.data.name
    }

    /// The [ChannelMode] for this channel
    pub fn mode(&self) -> ChannelMode {
        ChannelMode::wrap(self.network, &self.data.mode)
    }

    /// Get the list mode object belonging to this channel of the given type
    pub fn list(&self, list_type: ListModeType) -> ListMode
    {
        let list_id = ListModeId::new(self.data.id, list_type);
        self.network.list_mode(list_id)
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

    /// Retrieve the channel's topic, if any
    pub fn topic(&self) -> Option<ChannelTopic>
    {
        self.network.topic_for_channel(self.data.id).ok()
    }
}

impl<'a> super::ObjectWrapper<'a> for Channel<'a> {
    type Underlying = state::Channel;

    fn wrap(network: &'a Network, data: &'a state::Channel) -> Self
    {
        Self { network, data }
    }
}
