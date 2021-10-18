use ircd_macros::define_messages;
use super::MessageTarget;

define_messages! {
    001(Welcome) => { (network_name: &str, nick: &str) => ":Welcome to the {network_name} Internet Relay Chat network, {nick}" },

    401(NoSuchTarget) => { (unknown: &impl MessageTarget.format()) => "{unknown} :No such nick/channel" },
    421(UnknownCommand) => { (command: &str) => "{command} :Unknown command" },
    451(NotRegistered) => { () => ":You have not registered" },
    461(NotEnoughParameters) => { (command: &str) => "{command} :Not enough parameters" },
    462(AlreadyRegistered) => { () => ":You are already connected and cannot handshake again" },
}