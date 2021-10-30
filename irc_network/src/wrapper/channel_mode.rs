use crate::*;
use super::*;

pub struct ChannelMode<'a> {
    network: &'a Network,
    data: &'a state::ChannelMode,
}

impl ChannelMode<'_> {
    pub fn id(&self) -> CModeId {
        self.data.id
    }

    pub fn channel(&self) -> LookupResult<Channel>
    {
        self.network.raw_channels()
                    .filter(|c| c.mode == self.data.id)
                    .next()
                    .ok_or(LookupError::NoChannelForMode(self.data.id))
                    .wrap(self.network)
    }

    pub fn has_mode(&self, m: ChannelModeFlag) -> bool
    {
        self.data.modes.is_set(m)
    }

    pub fn format(&self) -> String
    {
        format!("+{}", self.data.modes.to_chars())
    }
}

impl<'a> super::ObjectWrapper<'a> for ChannelMode<'a> {
    type Underlying = state::ChannelMode;

    fn wrap(net: &'a Network, data: &'a state::ChannelMode) -> Self {
        Self{ network: net, data: data }
    }
}
