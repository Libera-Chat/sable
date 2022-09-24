use crate::prelude::*;

/// A wrapper around a [`state::KLine`]
pub struct KLine<'a> {
    _network: &'a Network,
    data: &'a state::KLine,
}


impl KLine<'_> {
    /// Return this object's ID
    pub fn id(&self) -> NetworkBanId
    {
        self.data.id
    }

    /// The host part of the ban
    pub fn host(&self) -> &Pattern
    {
        &self.data.host
    }

    /// The ident part of the ban
    pub fn user(&self) -> &Pattern
    {
        &self.data.user
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

impl<'a> super::ObjectWrapper<'a> for KLine<'a> {
    type Underlying = state::KLine;

    fn wrap(net: &'a Network, data: &'a state::KLine) -> Self
    {
        Self{ _network: net, data }
    }

    fn raw(&self) -> &'a Self::Underlying { self.data }
}
