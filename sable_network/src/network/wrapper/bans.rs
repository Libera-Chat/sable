use crate::{
    prelude::*,
    network::ban::*
};

/// A wrapper around a [`state::NetworkBan`]
pub struct NetworkBan<'a> {
    _network: &'a Network,
    data: &'a state::NetworkBan,
}

impl NetworkBan<'_> {
    /// Return this object's ID
    pub fn id(&self) -> NetworkBanId
    {
        self.data.id
    }

    /// The ban match criteria
    pub fn matcher(&self) -> &NetworkBanMatch
    {
        &self.data.matcher
    }

    /// The action to be applied by the ban
    pub fn action(&self) -> &NetworkBanAction
    {
        &self.data.action
    }

    /// Details of who set this ban
    pub fn setter(&self) -> &str
    {
        &self.data.setter_info
    }

    /// When the ban was set
    pub fn timestamp(&self) -> i64
    {
        self.data.timestamp
    }

    /// When the ban expires
    pub fn expires(&self) -> i64
    {
        self.data.expires
    }

    /// The user-visible reason
    pub fn reason(&self) -> &str
    {
        &self.data.reason
    }

    /// The oper-visible reason
    pub fn oper_reason(&self) -> Option<&str>
    {
        self.data.oper_reason.as_deref()
    }
}

impl<'a> super::ObjectWrapper<'a> for NetworkBan<'a> {
    type Underlying = state::NetworkBan;

    fn wrap(net: &'a Network, data: &'a state::NetworkBan) -> Self
    {
        Self{ _network: net, data }
    }

    fn raw(&self) -> &'a Self::Underlying { self.data }
}
