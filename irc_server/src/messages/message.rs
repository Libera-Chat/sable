use irc_network::wrapper::*;
use irc_network::validated::*;
use ircd_macros::define_messages;
use super::*;

define_messages! {
    Nick    => { (source, newnick: &Nickname)               => ":{source} NICK {newnick}" },
    Join    => { (source, chan: &Channel.name())            => ":{source} JOIN {chan}" },
    Part    => { (source, chan: &Channel.name(), msg: &str) => ":{source} PART {chan} :{msg}" },
    Quit    => { (source, message: &str)                    => ":{source} QUIT :{message}" },

    Mode    => { (source, target, changes: &str)            => ":{source} MODE {target} {changes}" },

    Notice  => { (source, target, message: &str)            => ":{source} NOTICE {target} :{message}" },
    Privmsg => { (source, target, message: &str)            => ":{source} PRIVMSG {target} :{message}" },

    Ping    => { (source, target, cookie: &str)             => ":{source} PING {target} :{cookie}" },
    Pong    => { (source, cookie: &str)                     => ":{source} PONG {source} :{cookie}" },
}