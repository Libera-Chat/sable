use crate::ircd::*;
use super::*;

pub struct UserMode<'a> {
    network: &'a Network,
    data: &'a state::UserMode,
}

impl UserMode<'_> {
    pub fn id(&self) -> UModeId {
        self.data.id
    }

    pub fn user(&self) -> LookupResult<User>
    {
        self.network.raw_users()
                    .filter(|u| u.mode_id == self.data.id)
                    .next()
                    .ok_or(LookupError::NoUserForMode(self.data.id))
                    .wrap(self.network)
    }

    pub fn has_mode(&self, m: UserModeFlag) -> bool
    {
        self.data.modes.is_set(m)
    }

    pub fn format(&self) -> String
    {
        format!("+{}", self.data.modes.to_chars())
    }
}

impl<'a> super::ObjectWrapper<'a> for UserMode<'a> {
    type Underlying = state::UserMode;

    fn wrap(net: &'a Network, data: &'a state::UserMode) -> Self {
        Self{ network: net, data: data }
    }
}
