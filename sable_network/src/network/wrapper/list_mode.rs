use crate::prelude::*;

/// A (virtual) wrapper around a ban-type list for a channel
pub struct ListMode<'a> {
    network: &'a Network,
    id: ListModeId,
}

impl<'a> ListMode<'a> {
    /// Return this object's ID
    pub fn id(&self) -> ListModeId
    {
        self.id
    }

    /// The list type
    pub fn list_type(&self) -> ListModeType
    {
        self.id.list_type()
    }

    /// The corresponding channel object
    pub fn channel(&self) -> LookupResult<wrapper::Channel>
    {
        self.network.channel(self.id.channel())
    }

    /// The entries in the list
    pub fn entries(&self) -> impl Iterator<Item=wrapper::ListModeEntry>
    {
        self.network.entries_for_list(self.id)
    }

    pub(crate)fn new(network: &'a Network, id: ListModeId) -> Self
    {
        Self { network, id }
    }
}
