use super::*;
use sable_macros::define_messages;
use sable_network::network::wrapper::{Channel, ChannelMode, ListModeEntry, Server, User};

define_messages! {
    001(Welcome)    => { (network_name: &str, nick: &Nickname)  => ":Welcome to the {network_name} Internet Relay Chat network, {nick}" },
    002(YourHostIs) => { (server_name: &ServerName, version: &str)     => ":Your host is {server_name}, running version {version}" },
    005(ISupport)   => { (data: &str)                           => "{data} :are supported by this server" },

    221(UserModeIs)                => { (modestring: &str)         => ":{modestring}" },
    311(WhoisUser)              => { (nick: &User.nick(), user=nick.user(), host=nick.visible_host(), realname=nick.realname())
                                                                => "{nick} {user} {host} * :{realname}" },
    312(WhoisServer)            => { (nick: &User.nick(), server: &Server.name(), info=server.id())
                                                                => "{nick} {server} :{info:?}"},
    315(EndOfWho)               => { (arg: &str)                => "{arg} :End of /WHO list" },
    318(EndOfWhois)             => { (user: &User.nick())       => "{user} :End of /WHOIS" },
    319(WhoisChannels)          => { (user: &User.nick(), chanlist: &str)
                                                                => "{user} :{chanlist}" },

    324(ChannelModeIs)          => { (chan: &Channel.name(), modes: &ChannelMode.format())
                                                                => "{chan} {modes}" },

    330(WhoisAccount)           => { (nick: &User.nick(), account: &Nickname)
                                                                => "{nick} {account} :is logged in as" },

    331(NoTopic)                => { (chan: &Channel.name())    => "{chan} :No topic is set"},
    332(TopicIs)                => { (chan: &Channel.name(), text: &str)
                                                                => "{chan} :{text}" },
    333(TopicSetBy)             => { (chan: &Channel.name(), info: &str, timestamp: i64)
                                                                => "{chan} {info} {timestamp}" },

    352(WhoReply)               => { (chname: &str, user: &User.user(), host=user.visible_host(), server: &Server.name(),
                                      nick=user.nick(), status: &str, hopcount: usize, realname=&user.realname())
                                                => "{chname} {user} {host} {server} {nick} {status} :{hopcount} {realname}" },
    353(NamesReply)             => { (is_pub: char, chan: &Channel.name(), content: &str)
                                                                => "{is_pub} {chan} :{content}" },
    366(EndOfNames)             => { (chan: &Channel.name())    => "{chan} :End of names list" },

    381(YoureOper)              => { ()                         => "You are now an IRC operator" },


    401(NoSuchTarget)           => { (unknown: &str)            => "{unknown} :No such nick/channel" },
    403(NoSuchChannel)          => { (chname: &ChannelName)     => "{chname} :No such channel" },
    404(CannotSendToChannel)    => { (chan: &ChannelName)       => "{chan} :Cannot send to channel" },
    410(InvalidCapCmd)          => { (subcommand: &str)         => "{subcommand} :Invalid CAP command" },
    421(UnknownCommand)         => { (command: &str)            => "{command} :Unknown command" },
    432(ErroneousNickname)      => { (nick: &str)               => "{nick} :Erroneous nickname" },
    433(NicknameInUse)          => { (nick: &Nickname)          => "{nick} :Nickname is already in use." },
    441(UserNotOnChannel)       => { (user: &User.nick(), chan: &Channel.name())
                                                                => "{user} {chan} :They're not on that channel" },
    442(NotOnChannel)           => { (chan: &ChannelName)       => "{chan} :You're not on that channel" },
    451(NotRegistered)          => { ()                         => ":You have not registered" },
    461(NotEnoughParameters)    => { (command: &str)            => "{command} :Not enough parameters" },
    462(AlreadyRegistered)      => { ()                         => ":You are already connected and cannot handshake again" },
    472(UnknownMode)            => { (c: char)                  => "{c} :Unknown mode character" },
    479(InvalidChannelName)     => { (name: &str)               => "{name} :Illegal channel name" },
    482(ChanOpPrivsNeeded)      => { (chan: &ChannelName)       => "{chan} :You're not a channel operator" },

    502(CantChangeOtherUserMode) => { ()                => ":Can't change mode for other users" },

    367(BanList)        => { (chan: &Channel.name(), entry: &ListModeEntry.pattern(), setter=entry.setter(), ts=entry.timestamp())
        => "{chan} {entry} {setter} {ts}"},
    368(EndOfBanList)   => { (chan: &Channel.name())    => "{chan} :End of channel ban list" },

    728(QuietList)        => { (chan: &Channel.name(), entry: &ListModeEntry.pattern(), setter=entry.setter(), ts=entry.timestamp())
        => "{chan} {entry} {setter} {ts}"},
    729(EndOfQuietList)   => { (chan: &Channel.name())    => "{chan} :End of channel quiet list" },

    346(InviteList)        => { (chan: &Channel.name(), entry: &ListModeEntry.pattern(), setter=entry.setter(), ts=entry.timestamp())
        => "{chan} {entry} {setter} {ts}"},
    347(EndOfInviteList)   => { (chan: &Channel.name())    => "{chan} :End of channel invite list" },

    348(ExceptList)        => { (chan: &Channel.name(), entry: &ListModeEntry.pattern(), setter=entry.setter(), ts=entry.timestamp())
        => "{chan} {entry} {setter} {ts}"},
    349(EndOfExceptList)   => { (chan: &Channel.name())    => "{chan} :End of channel exception list" },

    465(YoureBanned)        => { (msg: &str)    => "You are banned from this server: {msg}" },

    473(InviteOnlyChannel)  => { (chan: &ChannelName)      => "{chan} :Cannot join channel (+i) - you must be invited" },
    474(BannedOnChannel)    => { (chan: &ChannelName)      => "{chan} :Cannot join channel (+b) - you are banned" },
    475(BadChannelKey)      => { (chan: &ChannelName)      => "{chan} :Cannot join channel (+k) - bad key" },

    481(NotOper)            => { ()     => ":You're not an IRC operator" },
    491(NoOperConf)         => { ()     => ":No oper configuration found" },

    440(ServicesNotAvailable) => { () => ":Services are not available"},

    903(SaslSuccess)        => { () => ":SASL authentication successful" },
    904(SaslFail)           => { () => ":SASL authentication failed" },
    906(SaslAborted)        => { () => ":SASL authentication aborted" }
}
