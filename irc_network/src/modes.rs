//! Defines mode flag enumerations and sets
use ircd_macros::mode_flags;
use serde::{Serialize,Deserialize};
use strum::{
    EnumIter
};

mode_flags!(
    ChannelMode {
        NoExternal (0x01, 'n'),
        TopicLock  (0x02, 't'),
        Secret     (0x04, 's'),
        InviteOnly (0x08, 'i'),
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

macro_rules! define_mode_type {
    (
        $typename:ident
        {
             $( $var:ident => $val:expr ),*
        }
    ) => {
        #[derive(Debug,Clone,Copy,PartialOrd,Ord,PartialEq,Eq,Hash,Serialize,Deserialize)]
        #[derive(EnumIter)]
        pub enum $typename {
            $( $var ),*
        }

        impl $typename
        {
            pub fn mode_letter(&self) -> char
            {
                match self {
                    $( Self:: $var => $val ),*
                }
            }

            pub fn from_char(c: char) -> Option<Self>
            {
                match c {
                    $( $val => Some(Self::$var) ),+,
                    _ => None
                }
            }
        }
    }
}

define_mode_type!(
    ListModeType
    {
        Ban => 'b',
        Quiet => 'q',
        Except => 'e',
        Invex => 'I'
    }
);

define_mode_type!(
    KeyModeType
    {
        Key => 'k'
    }
);
