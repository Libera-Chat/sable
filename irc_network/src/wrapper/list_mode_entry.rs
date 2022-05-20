use crate::*;
use irc_strings::matches::Pattern;

/// A wrapper around a [`state::ChannelMode`]
pub struct ListModeEntry<'a> {
    network: &'a Network,
    data: &'a state::ListModeEntry,
}

impl ListModeEntry<'_> {
    /// Return this object's ID
    pub fn id(&self) -> ListModeEntryId
    {
        self.data.id
    }

    /// The mode list to which this belongs
    pub fn channel(&self) -> LookupResult<wrapper::Channel>
    {
        self.network.channel(self.data.list.channel())
    }

    /// The hostmask being banned (or whatever else)
    pub fn pattern(&self) -> &Pattern
    {
        &self.data.pattern
    }

    /// Details of who set this entry
    pub fn setter(&self) -> &str
    {
        &self.data.setter
    }

    /// When the entry was set
    pub fn timestamp(&self) -> i64
    {
        self.data.timestamp
    }
}

impl<'a> super::ObjectWrapper<'a> for ListModeEntry<'a> {
    type Underlying = state::ListModeEntry;

    fn wrap(network: &'a Network, data: &'a state::ListModeEntry) -> Self
    {
        Self { network, data }
    }
}
