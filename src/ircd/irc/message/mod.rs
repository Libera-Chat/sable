use crate::ircd::*;
use irc::Server;
use ircd_macros::define_messages;


pub trait MessageSource
{
    fn format(&self) -> String;
}

pub trait MessageTarget
{
    fn format(&self) -> String;
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

impl MessageTarget for String
{
    fn format(&self) -> String { self.clone() }
}

// Used when command parsing/processing fails
impl MessageTarget for irc::CommandSource<'_>
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

use wrapper::*;

define_messages! {
    Join => { (chan: &Channel.name()) => ":{source} JOIN {chan}" },
    Quit => { (message: &str) => ":{source} QUIT :{message}" },
    Privmsg => { (target, message: &str) => ":{source} PRIVMSG {target} :{message}" },
}

define_messages! {
    001(Welcome) => { (network_name: &str, nick: &str) => ":Welcome to the {network_name} Internet Relay Chat network, {nick}" },

    401(NoSuchTarget) => { (unknown: &impl MessageTarget.format()) => "{unknown} :No such nick/channel" },
    421(UnknownCommand) => { (command: &str) => "{command} :Unknown command" },
    451(NotRegistered) => { () => ":You have not registered" },
    461(NotEnoughParameters) => { (command: &str) => "{command} :Not enough parameters" },
    462(AlreadyRegistered) => { () => ":You are already connected and cannot handshake again" },
}