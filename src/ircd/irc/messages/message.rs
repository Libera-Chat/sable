use crate::ircd::wrapper::*;
use ircd_macros::define_messages;

define_messages! {
    Join => { (chan: &Channel.name()) => ":{source} JOIN {chan}" },
    Part => { (chan: &Channel.name(), msg: &str) => ":{source} PART {chan} :{msg}" },
    Quit => { (message: &str) => ":{source} QUIT :{message}" },
    Privmsg => { (target, message: &str) => ":{source} PRIVMSG {target} :{message}" },
}