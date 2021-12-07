use crate::*;

/// A wrapper around a [`state::NickBinding`]
pub struct NickBinding<'a>
{
    network: &'a Network,
    data: &'a state::NickBinding,
}

impl NickBinding<'_>
{
    /// Return this object's ID
    pub fn nick(&self) -> Nickname
    {
        self.data.nick
    }

    pub fn user(&self) -> LookupResult<wrapper::User>
    {
        self.network.user(self.data.user)
    }

    pub fn timestamp(&self) -> i64
    {
        self.data.timestamp
    }

    pub fn created(&self) -> EventId
    {
        self.data.created
    }
}

impl<'a> super::ObjectWrapper<'a> for NickBinding<'a> {
    type Underlying = state::NickBinding;

    fn wrap(net: &'a Network, data: &'a state::NickBinding) -> Self {
        Self{network: net, data: data}
    }
}