use super::*;

use sable_network::prelude::state::ChannelAccessSet;

impl<DB> ServicesServer<DB>
{
    pub(super) fn build_default_roles(&self, for_channel: ChannelRegistrationId) -> Vec<state::ChannelRole>
    {
        let mut ret = Vec::new();

        for (name, flags) in &self.config.default_roles
        {
            let mut flag_set = ChannelAccessSet::new();

            for flag in flags
            {
                flag_set |= *flag;
            }

            ret.push(state::ChannelRole {
                id: self.node.ids().next_channel_role(),
                channel: for_channel,
                name: name.clone(),
                flags: flag_set
            });
        }

        ret
    }
}