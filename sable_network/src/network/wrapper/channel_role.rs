use crate::prelude::*;

pub struct ChannelRole<'a> {
    network: &'a Network,
    data: &'a state::ChannelRole,
}

impl ChannelRole<'_> {
    pub fn id(&self) -> ChannelRoleId {
        self.data.id
    }

    pub fn channel(&self) -> Option<wrapper::ChannelRegistration> {
        self.data
            .channel
            .and_then(|id| self.network.channel_registration(id).ok())
    }

    pub fn name(&self) -> &state::ChannelRoleName {
        &self.data.name
    }

    pub fn flags(&self) -> state::ChannelAccessSet {
        self.data.flags
    }

    pub fn dominates(&self, other: &Self) -> bool {
        self.data.flags.dominates(&other.data.flags)
    }

    pub fn is_builtin(&self) -> bool {
        // Builtin roles have builtin names
        !matches!(self.data.name, state::ChannelRoleName::Custom(_))
    }
}

impl<'a> super::ObjectWrapper<'a> for ChannelRole<'a> {
    type Underlying = state::ChannelRole;

    fn wrap(network: &'a Network, data: &'a Self::Underlying) -> Self {
        Self { network, data }
    }

    fn raw(&self) -> &'a Self::Underlying {
        self.data
    }
}
