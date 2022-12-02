use crate::prelude::*;

pub struct ChannelRegistration<'a> {
    _network: &'a Network,
    data: &'a state::ChannelRegistration,
}

impl ChannelRegistration<'_>
{
    pub fn id(&self) -> ChannelRegistrationId
    {
        self.data.id
    }
}

impl<'a> super::ObjectWrapper<'a> for ChannelRegistration<'a> {
    type Underlying = state::ChannelRegistration;

    fn wrap(net: &'a Network, data: &'a Self::Underlying) -> Self
    {
        Self{ _network: net, data }
    }

    fn raw(&self) -> &'a Self::Underlying { self.data }
}

