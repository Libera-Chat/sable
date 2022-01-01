//! Defines mode flag enumerations and sets
use ircd_macros::mode_flags;
use serde::{Serialize,Deserialize};

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

#[derive(Debug,Clone,Copy,PartialOrd,Ord,PartialEq,Eq,Hash,Serialize,Deserialize)]
pub enum ListModeType {
    Ban,
}

impl ListModeType
{
    pub fn mode_letter(&self) -> char
    {
        match self {
            Self::Ban => 'b'
        }
    }

    pub fn from_char(c: char) -> Option<Self>
    {
        match c {
            'b' => Some(Self::Ban),
            _ => None
        }
    }
}