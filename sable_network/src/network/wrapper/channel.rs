use crate::prelude::*;
use super::*;

/// A wrapper around a [`state::Channel`]
pub struct Channel<'a> {
    network: &'a Network,
    data: &'a state::Channel,
}

impl<'a> Channel<'a> {
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

    /// Retrieve the corresponding registration, if any
    pub fn is_registered(&self) -> Option<ChannelRegistration<'a>>
    {
        self.network.channel_registration_by_name(self.data.name).ok()
    }

    /// Search for a role with the given name applicable to this channel
    pub fn has_role_named(&self, name: &state::ChannelRoleName) -> Option<ChannelRole>
    {
        match self.is_registered()
        {
            Some(registration) => {
                // We'd like to just do `registration.role_named(name)` here, but can't
                // because the borrow checker isn't smart enough to realise that the only
                // thing borrowed from registration is the same network that's borrowed
                // in self.
                self.network.channel_roles().find(|role| role.channel().map(|c| c.id()) == Some(registration.id()) && role.name() == name)
            },
            None => self.network.find_default_role(name)
        }
    }
}

impl<'a> super::ObjectWrapper<'a> for Channel<'a> {
    type Underlying = state::Channel;

    fn wrap(network: &'a Network, data: &'a state::Channel) -> Self
    {
        Self { network, data }
    }

    fn raw(&self) -> &'a Self::Underlying { self.data }
}