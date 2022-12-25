use crate::prelude::*;
use super::*;

pub struct ChannelRegistration<'a> {
    network: &'a Network,
    data: &'a state::ChannelRegistration,
}

impl ChannelRegistration<'_>
{
    pub fn id(&self) -> ChannelRegistrationId
    {
        self.data.id
    }

    pub fn name(&self) -> &ChannelName
    {
        &self.data.channelname
    }

    pub fn access_entries(&self) -> impl Iterator<Item=ChannelAccess>
    {
        let my_id = self.data.id;
        self.network.channel_accesses().filter(move |a| a.id().channel() == my_id)
    }

    /// Access the list of roles defined for this channel
    pub fn roles(&self) -> impl Iterator<Item=ChannelRole>
    {
        let my_id = self.data.id;
        self.network.channel_roles().filter(move |r| r.raw().channel == Some(my_id))
    }

    /// Look up a role by name
    pub fn role_named(&self, name: &state::ChannelRoleName) -> Option<ChannelRole>
    {
        self.roles().find(|r| r.name() == name)
    }
}

impl<'a> super::ObjectWrapper<'a> for ChannelRegistration<'a> {
    type Underlying = state::ChannelRegistration;

    fn wrap(network: &'a Network, data: &'a Self::Underlying) -> Self
    {
        Self{ network, data }
    }

    fn raw(&self) -> &'a Self::Underlying { self.data }
}

