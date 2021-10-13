use crate::ircd::*;

use ircd_macros::event_details;

event_details!{
    struct NewUser {
        pub nickname: String,
        pub username: String,
        pub visible_hostname: String,
        pub realname: String,
    }

    struct NewChannel {
        pub name: String,
    }

    struct ChannelJoin {
        pub channel: ChannelId,
        pub user: UserId,
    }

    struct NewMessage {
        pub source: UserId,
        pub target: ObjectId, // Can be user or channel
        pub text: String,
    }
}
