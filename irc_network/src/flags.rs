//! Defines mode flag enumerations and sets

use ircd_macros::mode_flags;

mode_flags!(
    ChannelMode {
        NoExternal (0x01, 'n'),
        TopicLock  (0x02, 't'),
        Secret     (0x04, 's'),
    }
);

mode_flags!(
    MembershipFlag {
        Op      (0x01, 'o', '@'),
        Voice   (0x02, 'v', '+'),
    }
);

mode_flags!(
    UserMode {
        Invisible   (0x01, 'i'),
    }
);