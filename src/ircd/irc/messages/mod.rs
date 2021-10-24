use crate::ircd::*;
use irc::Server;
use irc::command::*;

pub trait MessageSource
{
    fn format(&self) -> String;
}

pub trait MessageTarget
{
    fn format(&self) -> String;
}

impl MessageSource for &Server
{
    fn format(&self) -> String { self.name().to_string() }
}

impl MessageSource for Server
{
    fn format(&self) -> String { self.name().to_string() }
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

impl MessageTarget for irc::client::PreClient
{
    fn format(&self) -> String { "*".to_string() }
}

impl MessageTarget for Nickname
{
    fn format(&self) -> String { self.value().clone() }
}

// Used when command parsing/processing fails
impl MessageTarget for CommandSource<'_>
{
    fn format(&self) -> String
    {
        match self
        {
            Self::User(u) => <wrapper::User as MessageTarget>::format(&u),
            Self::PreClient(pc) => <irc::PreClient as MessageTarget>::format(&*pc.borrow())
        }
    }
}

pub trait Message : std::fmt::Display + std::fmt::Debug
{ }

#[derive(Debug)]
pub struct TargetedNumeric(String);

impl std::fmt::Display for TargetedNumeric { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { self.0.fmt(f) } }
impl Message for TargetedNumeric { }

pub trait Numeric : std::fmt::Debug
{
    fn format_for(&self, source: &dyn MessageSource, target: &dyn MessageTarget) -> TargetedNumeric;
}

pub mod message;
pub mod numeric;
