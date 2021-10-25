use crate::ircd::*;

use ircd_macros::{event_details,target_type};

pub struct WrongEventTypeError;

event_details!{
    #[target_type(UserId)]
    struct NewUser {
        pub nickname: Nickname,
        pub username: Username,
        pub visible_hostname: Hostname,
        pub realname: String,
    }

    #[target_type(UserId)]
    struct UserQuit {
        pub message: String,
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
}
