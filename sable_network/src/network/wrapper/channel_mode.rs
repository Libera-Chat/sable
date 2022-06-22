use crate::prelude::*;

/// A wrapper around a [`state::ChannelMode`]
pub struct ChannelMode<'a> {
    _network: &'a Network,
    data: &'a state::ChannelMode,
}

impl ChannelMode<'_> {
    /// Test for whether a given simple mode flag is set
    pub fn has_mode(&self, m: ChannelModeFlag) -> bool
    {
        self.data.modes.is_set(m)
    }

    /// Format the current mode flags into a string suitable for client
    /// protocol or human consumption
    pub fn format(&self) -> String
    {
        let mut ret = format!("+{}", self.data.modes.to_chars());
        if self.data.key.is_some()
        {
            ret.push(KeyModeType::Key.mode_letter());
        }
        ret
    }

    /// Get the channel key, if any
    pub fn key(&self) -> Option<ChannelKey>
    {
        self.data.key
    }
}

impl<'a> super::ObjectWrapper<'a> for ChannelMode<'a> {
    type Underlying = state::ChannelMode;

    fn wrap(network: &'a Network, data: &'a state::ChannelMode) -> Self
    {
        Self { _network: network, data }
    }
}
