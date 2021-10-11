use crate::ircd::Id;

use ircd_macros::event_details;

event_details!{
    struct NewUser {
        pub nickname: String,
        pub username: String,
        pub visible_hostname: String,
    };

    struct NewChannel {
        pub name: String,
    };

    struct ChannelJoin {
        pub channel: Id,
        pub user: Id,
    };
}
