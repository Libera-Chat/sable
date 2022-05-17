use serde::{
    Serialize,
    Deserialize
};
use strum::EnumIter;

mod repository;
pub use repository::*;

pub mod message_tag;
pub use message_tag::TaggableMessage;

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
        MessageTags: 0x01 => "message-tags",
        ServerTime:  0x02 => "server-time"
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

    pub fn set(&mut self, cap: ClientCapability)
    {
        self.0 |= cap as u64;
    }

    pub fn unset(&mut self, cap: ClientCapability)
    {
        self.0 &= !(cap as u64);
    }
}