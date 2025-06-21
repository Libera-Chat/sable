use super::*;
use crate::prelude::*;

/// A wrapper around a [`state::ChannelTopic`]
pub struct ChannelTopic<'a> {
    network: &'a Network,
    data: &'a state::ChannelTopic,
}

impl ChannelTopic<'_> {
    /// Return this object's ID
    pub fn id(&self) -> ChannelTopicId {
        self.data.id
    }

    /// Return the `Channel` to which this mode object is attached
    pub fn channel(&self) -> LookupResult<Channel<'_>> {
        self.network.channel(self.data.channel)
    }

    /// Return the topic text
    pub fn text(&self) -> &str {
        &self.data.text
    }

    /// Return the setter information
    pub fn setter(&self) -> &str {
        &self.data.setter_info
    }

    /// Timestamp when this topic was set
    pub fn timestamp(&self) -> i64 {
        self.data.timestamp
    }
}

impl<'a> super::ObjectWrapper<'a> for ChannelTopic<'a> {
    type Underlying = state::ChannelTopic;

    fn wrap(network: &'a Network, data: &'a state::ChannelTopic) -> Self {
        Self { network, data }
    }

    fn raw(&self) -> &'a Self::Underlying {
        self.data
    }
}
