use crate::ircd::*;
use irc::Server;
use ircd_macros::{
    define_messages,
    define_numerics
};


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

pub trait Message : std::fmt::Display
{ }

use wrapper::*;

define_messages! {
    Join => { (chan: &Channel.name()) => ":{source} JOIN {chan}" },
    Quit => { (message: &str) => ":{source} QUIT :{message}" },
    Privmsg => { (target, message: &str) => ":{source} PRIVMSG {target} :{message}" },
}

define_numerics! {
    001 => { (network_name: &str, nick: &str) => ":Welcome to the {network_name} Internet Relay Chat network, {nick}" }
}