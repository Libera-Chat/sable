use crate::ircd::*;

use ircd_macros::{event_details,target_type};

pub struct WrongEventTypeError;

event_details!{
    #[target_type(UserId)]
    struct NewUser {
        pub nickname: String,
        pub username: String,
        pub visible_hostname: String,
        pub realname: String,
    }

    #[target_type(UserId)]
    struct UserQuit {
        pub message: String,
    }

    #[target_type(ChannelId)]
    struct NewChannel {
        pub name: String,
    }

    #[target_type(MembershipId)]
    struct ChannelJoin {
        pub channel: ChannelId,
        pub user: UserId,
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
