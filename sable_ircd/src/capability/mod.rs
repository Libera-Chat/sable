use serde::{
    Serialize,
    Deserialize
};
use strum::EnumIter;
use std::sync::atomic::{AtomicU64,Ordering};

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

        ChatHistory:            0x101 => "draft/chathistory",
        PersistentSession:      0x102 => "sable/persistent-session",
        AccountRegistration:    0x104 => "sable/account-registration"
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

pub struct AtomicCapabilitySet(AtomicU64);

impl AtomicCapabilitySet
{
    pub fn new() -> Self
    {
        Self(AtomicU64::new(0))
    }

    pub fn has(&self, cap: ClientCapability) -> bool
    {
        0 != self.0.load(Ordering::Relaxed) & cap as u64
    }

    pub fn has_all(&self, caps: ClientCapabilitySet) -> bool
    {
        (self.0.load(Ordering::Relaxed) & caps.0) == caps.0
    }

    pub fn set(&self, cap: ClientCapability)
    {
        self.0.fetch_or(cap as u64, Ordering::Relaxed);
    }

    pub fn unset(&mut self, cap: ClientCapability)
    {
        self.0.fetch_and(!(cap as u64), Ordering::Relaxed);
    }

    pub fn reset(&self, caps: ClientCapabilitySet)
    {
        self.0.store(caps.0, Ordering::Relaxed);
    }
}

impl From<ClientCapability> for ClientCapabilitySet
{
    fn from(cap: ClientCapability) -> Self
    {
        Self(cap as u64)
    }
}

impl From<ClientCapabilitySet> for AtomicCapabilitySet
{
    fn from(caps: ClientCapabilitySet) -> Self
    {
        Self(AtomicU64::new(caps.0))
    }
}

impl From<AtomicCapabilitySet> for ClientCapabilitySet
{
    fn from(caps: AtomicCapabilitySet) -> Self
    {
        Self(caps.0.load(Ordering::Relaxed))
    }
}

impl From<&AtomicCapabilitySet> for ClientCapabilitySet
{
    fn from(caps: &AtomicCapabilitySet) -> Self
    {
        Self(caps.0.load(Ordering::Relaxed))
    }
}

impl Default for ClientCapabilitySet
{
    fn default() -> Self
    {
        Self::new()
    }
}