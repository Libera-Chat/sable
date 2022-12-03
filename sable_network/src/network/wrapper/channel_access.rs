use crate::prelude::*;

pub struct ChannelAccess<'a> {
    network: &'a Network,
    data: &'a state::ChannelAccess,
}

impl ChannelAccess<'_>
{
    pub fn id(&self) -> ChannelAccessId
    {
        self.data.id
    }

    pub fn user(&self) -> LookupResult<wrapper::Account>
    {
        self.network.account(self.data.id.account())
    }

    pub fn channel(&self) -> LookupResult<wrapper::ChannelRegistration>
    {
        self.network.channel_registration(self.data.id.channel())
    }

    pub fn flags(&self) -> ChannelAccessSet
    {
        self.data.flags
    }
}

impl<'a> super::ObjectWrapper<'a> for ChannelAccess<'a> {
    type Underlying = state::ChannelAccess;

    fn wrap(network: &'a Network, data: &'a Self::Underlying) -> Self
    {
        Self{ network, data }
    }

    fn raw(&self) -> &'a Self::Underlying { self.data }
}
