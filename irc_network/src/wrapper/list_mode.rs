use crate::*;

/// A wrapper around a [`state::ChannelMode`]
pub struct ListMode<'a> {
    network: &'a Network,
    data: &'a state::ListMode,
}

impl ListMode<'_> {
    /// Return this object's ID
    pub fn id(&self) -> ListModeId
    {
        self.data.id
    }

    /// The list type
    pub fn list_type(&self) -> ListModeType
    {
        self.data.list_type
    }

    /// The corresponding channel mode structure
    pub fn mode(&self) -> LookupResult<wrapper::ChannelMode>
    {
        self.network.mode_for_list(self.data.id)
    }

    /// The entries in the list
    pub fn entries(&self) -> impl Iterator<Item=wrapper::ListModeEntry>
    {
        self.network.entries_for_list(self.data.id)
    }
}

impl<'a> super::ObjectWrapper<'a> for ListMode<'a> {
    type Underlying = state::ListMode;

    fn wrap(network: &'a Network, data: &'a state::ListMode) -> Self
    {
        Self { network, data }
    }
}
