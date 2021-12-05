use crate::*;

use ircd_macros::{event_details,target_type};

pub struct WrongEventTypeError;

event_details!(EventDetails => {
    #[target_type(NicknameId)]
    struct BindNickname {
        pub user: UserId,
    }

    #[target_type(UserId)]
    struct NewUser {
        pub nickname: Nickname,
        pub username: Username,
        pub visible_hostname: Hostname,
        pub realname: String,
        pub mode_id: UModeId,
        pub server: ServerId,
    }

    #[target_type(UserId)]
    struct UserQuit {
        pub message: String,
    }

    #[target_type(UModeId)]
    struct NewUserMode {
        pub mode: UserModeSet,
    }

    #[target_type(UModeId)]
    struct UserModeChange {
        pub changed_by: ObjectId,
        pub added: UserModeSet,
        pub removed: UserModeSet,
    }

    #[target_type(ChannelId)]
    struct NewChannel {
        pub name: ChannelName,
        pub mode: CModeId,
    }

    #[target_type(CModeId)]
    struct NewChannelMode {
        pub mode: ChannelModeSet,
    }

    #[target_type(CModeId)]
    struct ChannelModeChange {
        pub changed_by: ObjectId,
        pub added: ChannelModeSet,
        pub removed: ChannelModeSet
    }

    #[target_type(MembershipId)]
    struct ChannelJoin {
        pub channel: ChannelId,
        pub user: UserId,
        pub permissions: ChannelPermissionSet,
    }

    #[target_type(MembershipId)]
    struct ChannelPermissionChange {
        pub changed_by: ObjectId,
        pub added: ChannelPermissionSet,
        pub removed: ChannelPermissionSet,
    }

    #[target_type(MembershipId)]
    struct ChannelPart {
        pub message: String,
    }

    #[target_type(MessageId)]
    struct NewMessage {
        pub source: UserId,
        pub target: ObjectId, // Can be user or channel
        pub text: String,
    }

    #[target_type(ServerId)]
    struct NewServer {
        pub epoch: EpochId,
        pub name: ServerName,
        pub ts: i64,
    }

    #[target_type(ServerId)]
    struct ServerPing {
        pub ts: i64,
    }

    #[target_type(ServerId)]
    struct ServerQuit {
        pub introduced_by: EventId,
    }
});
