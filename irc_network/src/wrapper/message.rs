use crate::*;
use super::*;

/// A wrapper around a [`state::Message`]
pub struct Message<'a> {
    network: &'a Network,
    data: &'a state::Message,
}

/// Describe a message's target
pub enum MessageTarget<'a>
{
    /// Message sent to a user
    User(User<'a>),
    /// Message sent to a channel
    Channel(Channel<'a>),
}

impl Message<'_> {
    /// Return this object's ID
    pub fn id(&self) -> MessageId
    {
        self.data.id
    }

    /// The user who sent this message
    pub fn source(&self) -> LookupResult<User>
    {
        self.network.user(self.data.source)
    }

    /// The target to which the message was sent
    pub fn target(&self) -> LookupResult<MessageTarget>
    {
        match self.data.target {
            ObjectId::User(id) => Ok(MessageTarget::User(self.network.user(id)?)),
            ObjectId::Channel(id) => Ok(MessageTarget::Channel(self.network.channel(id)?)),
            _ => Err(LookupError::WrongType)
        }
    }

    /// The message content
    pub fn text(&self) -> &str
    {
        &self.data.text
    }
}

impl<'a> super::ObjectWrapper<'a> for Message<'a> {
    type Underlying = state::Message;

    fn wrap(net: &'a Network, data: &'a state::Message) -> Self {
        Self{network: net, data: data}
    }
}