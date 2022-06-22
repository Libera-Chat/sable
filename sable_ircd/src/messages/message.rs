use sable_network::prelude::*;
use wrapper::Channel;

use sable_macros::define_messages;
use super::*;

define_messages! {
    Cap     => { (source, target, subcmd: &str, text: &str) => ":{source} CAP {target} {subcmd} :{text}" },
    Nick    => { (source, newnick: &Nickname)               => ":{source} NICK {newnick}" },
    Join    => { (source, chan: &Channel.name())            => ":{source} JOIN {chan}" },
    Part    => { (source, chan: &ChannelName, msg: &str)    => ":{source} PART {chan} :{msg}" },
    Invite  => { (source, target, chan: &ChannelName)    => ":{source} INVITE {target} :{chan}" },
    Quit    => { (source, message: &str)                    => ":{source} QUIT :{message}" },
    Topic   => { (source, chan: &Channel.name(), text: &str)=> ":{source} TOPIC {chan} :{text}" },

    Mode    => { (source, target, changes: &str)            => ":{source} MODE {target} {changes}" },

    Notice  => { (source, target, message: &str)            => ":{source} NOTICE {target} :{message}" },
    Privmsg => { (source, target, message: &str)            => ":{source} PRIVMSG {target} :{message}" },
    Message => { (source, target, message_type: state::MessageType, message: &str)
                                                            => ":{source} {message_type} {target} :{message}" },

    Ping    => { (source, target, cookie: &str)             => ":{source} PING {target} :{cookie}" },
    Pong    => { (source, cookie: &str)                     => ":{source} PONG {source} :{cookie}" },

    Error   => { (text: &str)   => "ERROR :{text}" }
}