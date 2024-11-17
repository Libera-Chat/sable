use super::*;
use crate::prelude::*;

/// Describes a message's source
pub enum MessageSource<'a> {
    /// A user currently in the network state
    User(User<'a>),
    /// A server currently in the network state
    Server(Server<'a>),
    /// A user which is no longer in the network state (for example, because they just quit)
    HistoricUser(state::HistoricUser),
}

impl MessageSource<'_> {
    pub fn user(&self) -> Option<&User> {
        match self {
            Self::User(u) => Some(u),
            _ => None,
        }
    }
}

impl<'a> From<User<'a>> for MessageSource<'a> {
    fn from(value: User<'a>) -> Self {
        Self::User(value)
    }
}

impl<'a> From<Server<'a>> for MessageSource<'a> {
    fn from(value: Server<'a>) -> Self {
        Self::Server(value)
    }
}

/// Describes a message's target
pub enum MessageTarget<'a> {
    /// Message sent to a user
    User(User<'a>),
    /// Message sent to a channel
    Channel(Channel<'a>),
}

impl MessageTarget<'_> {
    pub fn user(&self) -> Option<&User> {
        match self {
            Self::User(u) => Some(&u),
            _ => None,
        }
    }

    pub fn channel(&self) -> Option<&Channel> {
        match self {
            Self::Channel(c) => Some(&c),
            _ => None,
        }
    }
}

impl ToString for MessageTarget<'_> {
    fn to_string(&self) -> String {
        match self {
            Self::User(u) => u.nuh(),
            Self::Channel(c) => c.name().to_string(),
        }
    }
}

/// A wrapper around a [`state::Message`]
pub struct Message<'a> {
    network: &'a Network,
    data: &'a state::Message,
}

pub trait WrappedMessage {
    /// Return this object's ID
    fn id(&self) -> MessageId;

    /// The user who sent this message
    fn source(&self) -> LookupResult<impl WrappedUser>;

    /// The target to which the message was sent
    fn target(&self) -> LookupResult<MessageTarget>;

    /// Whether this is a privmsg or a notice
    fn message_type(&self) -> state::MessageType;

    /// The message content
    fn text(&self) -> &str;

    /// The message's timestamp
    fn ts(&self) -> i64;
}

impl WrappedMessage for Message<'_> {
    fn id(&self) -> MessageId {
        self.data.id
    }

    #[allow(refining_impl_trait)]
    fn source(&self) -> LookupResult<User> {
        self.network.user(self.data.source)
    }

    fn target(&self) -> LookupResult<MessageTarget> {
        match self.data.target {
            ObjectId::User(id) => Ok(MessageTarget::User(self.network.user(id)?)),
            ObjectId::Channel(id) => Ok(MessageTarget::Channel(self.network.channel(id)?)),
            _ => Err(LookupError::WrongType),
        }
    }

    fn message_type(&self) -> state::MessageType {
        self.data.message_type
    }

    fn text(&self) -> &str {
        &self.data.text
    }

    fn ts(&self) -> i64 {
        self.data.ts
    }
}

impl<'a> super::ObjectWrapper<'a> for Message<'a> {
    type Underlying = state::Message;

    fn wrap(network: &'a Network, data: &'a state::Message) -> Self {
        Self { network, data }
    }

    fn raw(&self) -> &'a Self::Underlying {
        self.data
    }
}
