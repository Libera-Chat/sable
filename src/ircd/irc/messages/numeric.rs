use ircd_macros::define_messages;
use crate::wrapper::*;
use crate::ircd::validated::*;

define_messages! {
    001(Welcome)    => { (network_name: &str, nick: &Nickname) => ":Welcome to the {network_name} Internet Relay Chat network, {nick}" },

    324(ChannelModeIs)          => { (chan: &Channel.name(), modes: &ChannelMode.format())  => "{chan} {modes}" },
    353(NamesReply)             => { (is_pub: char, chan: &Channel.name(), content: &str)   => "{is_pub} {chan} :{content}" },
    366(EndOfNames)             => { (chan: &Channel.name())                                => "{chan} :End of names list" },

    401(NoSuchTarget)           => { (unknown: &str)            => "{unknown} :No such nick/channel" },
    403(NoSuchChannel)          => { (chname: &ChannelName)     => "{chname} :No such channel" },
    404(CannotSendToChannel)    => { (chan: &Channel.name())    => "{chan} :Cannot send to channel" },
    421(UnknownCommand)         => { (command: &str)            => "{command} :Unknown command" },
    432(ErroneousNickname)      => { (nick: &str)               => "{nick} :Erroneous nickname" },
    433(NicknameInUse)          => { (nick: &Nickname)          => "{nick} :Nickname is already in use." },
    441(UserNotOnChannel)       => { (user: &User.nick(), chan: &Channel.name())
                                                                => "{user} {chan} :They're not on that channel" },
    442(NotOnChannel)           => { (chan: &Channel.name())    => "{chan} :You're not on that channel" },
    451(NotRegistered)          => { ()                         => ":You have not registered" },
    461(NotEnoughParameters)    => { (command: &str)            => "{command} :Not enough parameters" },
    462(AlreadyRegistered)      => { ()                         => ":You are already connected and cannot handshake again" },
    472(UnknownMode)            => { (c: char)                  => "{c} :Unknown mode character" },
    479(InvalidChannelName)     => { (name: &str)               => "{name} :Illegal channel name" },
    482(ChanOpPrivsNeeded)      => { (chan: &Channel.name())    => "{chan} :You're not a channel operator" },

    502(CantChangeOtherUserMode) => { ()                => ":Can't change mode for other users" },
}