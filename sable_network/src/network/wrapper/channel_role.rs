use crate::prelude::*;

pub struct ChannelRole<'a> {
    network: &'a Network,
    data: &'a state::ChannelRole,
}

impl ChannelRole<'_>
{
    pub fn id(&self) -> ChannelRoleId
    {
        self.data.id
    }

    pub fn channel(&self) -> Option<wrapper::ChannelRegistration>
    {
        self.data.channel.map(|id| self.network.channel_registration(id).ok()).flatten()
    }

    pub fn name(&self) -> &state::ChannelRoleName
    {
        &self.data.name
    }

    pub fn flags(&self) -> state::ChannelAccessSet
    {
        self.data.flags
    }

    pub fn dominates(&self, other: &Self) -> bool
    {
        self.data.flags.dominates(&other.data.flags)
    }
}

impl<'a> super::ObjectWrapper<'a> for ChannelRole<'a> {
    type Underlying = state::ChannelRole;

    fn wrap(network: &'a Network, data: &'a Self::Underlying) -> Self
    {
        Self{ network, data }
    }

    fn raw(&self) -> &'a Self::Underlying { self.data }
}
