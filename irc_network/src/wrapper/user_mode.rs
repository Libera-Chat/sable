use crate::*;
use super::*;

/// A wrapper around a [`state::UserMode`]
pub struct UserMode<'a> {
    network: &'a Network,
    data: &'a state::UserMode,
}

impl UserMode<'_> {
    /// Return this object's ID
    pub fn id(&self) -> UserModeId {
        self.data.id
    }

    /// The associated user object
    pub fn user(&self) -> LookupResult<User>
    {
        self.network.raw_users()
                    .find(|u| u.mode_id == self.data.id)
                    .ok_or(LookupError::NoUserForMode(self.data.id))
                    .wrap(self.network)
    }

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
        Self { network, data }
    }
}
