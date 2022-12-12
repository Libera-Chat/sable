use crate::prelude::*;

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

    pub fn access_entries(&self) -> impl Iterator<Item=wrapper::ChannelAccess>
    {
        let my_id = self.data.id;
        self.network.channel_accesses().filter(move |a| a.id().channel() == my_id)
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

