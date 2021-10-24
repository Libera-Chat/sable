use ircd_macros::modeflags;

modeflags!(
    ChannelMode {
        NoExternal (0x01, 'n'),
        TopicLock  (0x02, 't'),
    }
);
