use irc_network::*;
use crate::server::command_processor::*;
use crate::client;

/// Trait describing an object that can be the source of a client protocol message
pub trait MessageSource
{
    fn format(&self) -> String;
}

/// Trait describing an object that can be the target of a client protocol message
pub trait MessageTarget
{
    fn format(&self) -> String;
}

impl MessageSource for &crate::Server
{
    fn format(&self) -> String { self.name().to_string() }
}

impl MessageSource for crate::Server
{
    fn format(&self) -> String { self.name().to_string() }
}

impl MessageSource for ServerName
{
    fn format(&self) -> String { self.to_string() }
}

impl MessageSource for String
{
    fn format(&self) -> String { self.clone() }
}

impl MessageSource for wrapper::User<'_>
{
    fn format(&self) -> String { format!("{}!{}@{}", self.nick(), self.user(), self.visible_host()) }
}

impl MessageTarget for wrapper::User<'_>
{
    fn format(&self) -> String { self.nick().to_string() }
}

impl MessageTarget for wrapper::Channel<'_>
{
    fn format(&self) -> String { self.name().to_string() }
}

impl MessageTarget for client::PreClient
{
    fn format(&self) -> String { "*".to_string() }
}

impl MessageTarget for Option<std::cell::RefCell<client::PreClient>>
{
    fn format(&self) -> String { "*".to_string() }
}

impl MessageTarget for Nickname
{
    fn format(&self) -> String { self.value().to_string() }
}

impl MessageTarget for wrapper::MessageTarget<'_>
{
    fn format(&self) -> String
    {
        match self
        {
            Self::Channel(c) => c.format(),
            Self::User(u) => MessageTarget::format(u)
        }
    }
}

// Used when command parsing/processing fails
impl MessageTarget for CommandSource<'_>
{
    fn format(&self) -> String
    {
        match self
        {
            Self::User(u) => <wrapper::User as MessageTarget>::format(&u),
            Self::PreClient(pc) => <client::PreClient as MessageTarget>::format(&*pc.borrow())
        }
    }
}

/// Trait describing a client protocol message type
pub trait MessageType : std::fmt::Display + std::fmt::Debug
{ }

/// A `Numeric` that has been formatted for a specific source and target
#[derive(Debug)]
pub struct TargetedNumeric(String);

impl std::fmt::Display for TargetedNumeric
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        self.0.fmt(f)
    }
}

impl MessageType for TargetedNumeric { }

/// Trait describing a numeric message
pub trait Numeric : std::fmt::Debug
{
    fn format_for(&self, source: &dyn MessageSource, target: &dyn MessageTarget) -> TargetedNumeric;
}

pub mod message;
pub mod numeric;
