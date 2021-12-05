use crate::*;
use super::*;

pub struct Message<'a> {
    network: &'a Network,
    data: &'a state::Message,
}

pub enum MessageTarget<'a>
{
    User(User<'a>),
    Channel(Channel<'a>),
}

impl Message<'_> {
    pub fn id(&self) -> MessageId
    {
        self.data.id
    }

    pub fn source(&self) -> LookupResult<User>
    {
        self.network.user(self.data.source)
    }

    pub fn target(&self) -> LookupResult<MessageTarget>
    {
        match self.data.target {
            ObjectId::User(id) => Ok(MessageTarget::User(self.network.user(id)?)),
            ObjectId::Channel(id) => Ok(MessageTarget::Channel(self.network.channel(id)?)),
            _ => Err(LookupError::WrongType)
        }
    }

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