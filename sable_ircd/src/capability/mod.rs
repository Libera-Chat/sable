use serde::{
    Serialize,
    Deserialize
};
use strum::EnumIter;

mod repository;
pub use repository::*;

mod capability_message;
pub use capability_message::*;

pub mod message_tag;
pub use message_tag::TaggableMessage;

mod with_tags;
pub(crate) use with_tags::WithSupportedTags;

pub mod server_time;


macro_rules! define_capabilities {
    (
        $typename:ident
        {
            $( $cap:ident : $val:literal => $name:literal ),*
        }
    ) => {
        #[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
        #[derive(EnumIter)]
        #[repr(u64)]
        pub enum $typename
        {
            $( $cap = $val ),*
        }

        impl $typename
        {
            pub fn name(&self) -> &'static str
            {
                match self
                {
                    $( Self::$cap => $name ),*
                }
            }

            pub fn flag(&self) -> u64
            {
                *self as u64
            }
        }
    };
}

define_capabilities! (
    ClientCapability
    {
        MessageTags:            0x01 => "message-tags",
        ServerTime:             0x02 => "server-time",
        EchoMessage:            0x04 => "echo-message",
        ChatHistory:            0x08 => "draft/chathistory",
        PersistentSession:      0x10 => "sable/persistent-session"
    }
);

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct ClientCapabilitySet(u64);

impl ClientCapabilitySet
{
    pub fn new() -> Self
    {
        Self(0)
    }

    pub fn has(&self, cap: ClientCapability) -> bool
    {
        0 != self.0 & cap as u64
    }

    pub fn has_all(&self, caps: ClientCapabilitySet) -> bool
    {
        (self.0 & caps.0) == caps.0
    }

    pub fn set(&mut self, cap: ClientCapability)
    {
        self.0 |= cap as u64;
    }

    pub fn unset(&mut self, cap: ClientCapability)
    {
        self.0 &= !(cap as u64);
    }
}

impl From<ClientCapability> for ClientCapabilitySet
{
    fn from(cap: ClientCapability) -> Self
    {
        Self(cap as u64)
    }
}

impl Default for ClientCapabilitySet
{
    fn default() -> Self
    {
        Self::new()
    }
}