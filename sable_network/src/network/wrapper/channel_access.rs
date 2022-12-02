use crate::prelude::*;

pub struct ChannelAccess<'a> {
    _network: &'a Network,
    data: &'a state::ChannelAccess,
}

impl ChannelAccess<'_>
{
    pub fn id(&self) -> ChannelAccessId
    {
        self.data.id
    }
}


impl<'a> super::ObjectWrapper<'a> for ChannelAccess<'a> {
    type Underlying = state::ChannelAccess;

    fn wrap(net: &'a Network, data: &'a Self::Underlying) -> Self
    {
        Self{ _network: net, data }
    }

    fn raw(&self) -> &'a Self::Underlying { self.data }
}
