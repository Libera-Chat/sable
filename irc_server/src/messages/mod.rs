use irc_network::*;
use crate::server::command_processor::*;
use crate::client;

pub trait MessageSource
{
    fn format(&self) -> String;
}

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

pub trait MessageType : std::fmt::Display + std::fmt::Debug
{ }

#[derive(Debug)]
pub struct TargetedNumeric(String);

impl std::fmt::Display for TargetedNumeric { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { self.0.fmt(f) } }
impl MessageType for TargetedNumeric { }

pub trait Numeric : std::fmt::Debug
{
    fn format_for(&self, source: &dyn MessageSource, target: &dyn MessageTarget) -> TargetedNumeric;
}

pub mod message;
pub mod numeric;
