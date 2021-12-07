use crate::*;
use super::*;

/// A wrapper around a [`state::ChannelMode`]
pub struct ChannelMode<'a> {
    network: &'a Network,
    data: &'a state::ChannelMode,
}

impl ChannelMode<'_> {
    /// Return this object's ID
    pub fn id(&self) -> CModeId {
        self.data.id
    }

    /// Return the `Channel` to which this mode object is attached
    pub fn channel(&self) -> LookupResult<Channel>
    {
        self.network.raw_channels()
                    .filter(|c| c.mode == self.data.id)
                    .next()
                    .ok_or(LookupError::NoChannelForMode(self.data.id))
                    .wrap(self.network)
    }

    /// Test for whether a given simple mode flag is set
    pub fn has_mode(&self, m: ChannelModeFlag) -> bool
    {
        self.data.modes.is_set(m)
    }

    /// Format the current mode flags into a string suitable for client
    /// protocol or human consumption
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
