use crate::prelude::*;

/// A wrapper around a [`state::UserMode`]
pub struct UserMode<'a> {
    _network: &'a Network,
    data: &'a state::UserMode,
}

impl UserMode<'_> {
    /// Test whether the associated user has a given mode flag set
    pub fn has_mode(&self, m: UserModeFlag) -> bool
    {
        self.data.modes.is_set(m)
    }

    /// Format the user's modes into a string for client protocol or human
    /// consumption
    pub fn format(&self) -> String
    {
        format!("+{}", self.data.modes.to_chars())
    }
}

impl<'a> super::ObjectWrapper<'a> for UserMode<'a> {
    type Underlying = state::UserMode;

    fn wrap(network: &'a Network, data: &'a state::UserMode) -> Self
    {
        Self { _network: network, data }
    }

    fn raw(&self) -> &'a Self::Underlying { self.data }
}
