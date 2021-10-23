use crate::ircd::wrapper::*;
use ircd_macros::define_messages;

define_messages! {
    Join => { (source, chan: &Channel.name()) => ":{source} JOIN {chan}" },
    Part => { (source, chan: &Channel.name(), msg: &str) => ":{source} PART {chan} :{msg}" },
    Quit => { (source, message: &str) => ":{source} QUIT :{message}" },

    Privmsg => { (source, target, message: &str) => ":{source} PRIVMSG {target} :{message}" },

    Ping => { (source, target, cookie: &str) => ":{source} PING {target} :{cookie}" },
    Pong => { (source, cookie: &str) => ":{source} PONG {source} :{cookie}" },
}