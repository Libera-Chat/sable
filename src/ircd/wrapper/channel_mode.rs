use crate::ircd::*;

pub struct ChannelMode<'a> {
    _network: &'a Network,
    data: &'a state::ChannelMode,
}

impl ChannelMode<'_> {
    pub fn id(&self) -> CModeId {
        self.data.id
    }

    pub fn has_mode(&self, m: ChannelModeFlags) -> bool
    {
        self.data.modes & m == m
    }

    pub fn format(&self) -> String
    {
        let mut s = "+".to_string();
        if self.has_mode(ChannelModeFlags::NO_EXTERNAL) {
            s += "n";
        }
        s
    }
}

impl<'a> super::ObjectWrapper<'a> for ChannelMode<'a> {
    type Underlying = state::ChannelMode;

    fn wrap(net: &'a Network, data: &'a state::ChannelMode) -> Self {
        Self{ _network: net, data: data }
    }
}
