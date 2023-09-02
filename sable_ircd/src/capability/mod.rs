use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use strum::EnumIter;

mod repository;
pub use repository::*;

mod with_tags;
pub(crate) use with_tags::WithSupportedTags;

mod capability_condition;
pub use capability_condition::*;

pub mod server_time;

macro_rules! define_capabilities {
    (
        $typename:ident
        {
            $( $cap:ident : $val:literal => ($name:literal, $def:literal) ),*
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
            /// Exhaustive list of all known capabilities
            const ALL: &'static [ClientCapability] = &[ $(Self::$cap),* ];

            /// On-the-wire name of the capability
            pub fn name(self) -> &'static str
            {
                match self
                {
                    $( Self::$cap => $name ),*
                }
            }

            /// Whether the capability is available without some handler explicitly
            /// enabling it
            pub fn is_default(&self) -> bool
            {
                match self
                {
                    $( Self::$cap => $def ),*
                }
            }

            /// Bit used as a mask in [`ClientCapabilitySet`]/[`AtomicCapabilitySet`]
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
        ServerTime:             0x02 => ("server-time", true),
        EchoMessage:            0x04 => ("echo-message", true),
        Sasl:                   0x08 => ("sasl", false),
        Batch:                  0x10 => ("batch", true),
        LabeledResponse:        0x20 => ("labeled-response", true),
        UserhostInNames:        0x40 => ("userhost-in-names", true),
        AwayNotify:             0x80 => ("away-notify", true),

        ChatHistory:            0x100 => ("draft/chathistory", true),
        PersistentSession:      0x200 => ("sable.libera.chat/persistent-session", true),
        AccountRegistration:    0x400 => ("sable.libera.chat/account-registration", true)
    }
);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientCapabilitySet(u64);

impl ClientCapabilitySet {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn has(&self, cap: ClientCapability) -> bool {
        0 != self.0 & cap as u64
    }

    pub fn has_all(&self, caps: ClientCapabilitySet) -> bool {
        (self.0 & caps.0) == caps.0
    }

    pub fn has_any(&self, caps: ClientCapabilitySet) -> bool {
        (self.0 & caps.0) != 0
    }

    pub fn set(&mut self, cap: ClientCapability) {
        self.0 |= cap as u64;
    }

    pub fn set_all(&mut self, caps: ClientCapabilitySet) {
        self.0 |= caps.0;
    }

    pub fn unset(&mut self, cap: ClientCapability) {
        self.0 &= !(cap as u64);
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = ClientCapability> + 'a {
        ClientCapability::ALL
            .iter()
            .cloned()
            .filter(|cap| self.has(*cap))
    }
}

pub struct AtomicCapabilitySet(AtomicU64);

impl AtomicCapabilitySet {
    pub fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    pub fn has(&self, cap: ClientCapability) -> bool {
        0 != self.0.load(Ordering::Relaxed) & cap as u64
    }

    pub fn has_all(&self, caps: ClientCapabilitySet) -> bool {
        (self.0.load(Ordering::Relaxed) & caps.0) == caps.0
    }

    pub fn has_any(&self, caps: ClientCapabilitySet) -> bool {
        (self.0.load(Ordering::Relaxed) & caps.0) != 0
    }

    pub fn set(&self, cap: ClientCapability) {
        self.0.fetch_or(cap as u64, Ordering::Relaxed);
    }

    pub fn set_all(&self, caps: ClientCapabilitySet) {
        self.0.fetch_or(caps.0, Ordering::Relaxed);
    }

    pub fn unset(&mut self, cap: ClientCapability) {
        self.0.fetch_and(!(cap as u64), Ordering::Relaxed);
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = ClientCapability> + 'a {
        ClientCapability::ALL
            .iter()
            .cloned()
            .filter(|cap| self.has(*cap))
    }
}

impl From<ClientCapability> for ClientCapabilitySet {
    fn from(cap: ClientCapability) -> Self {
        Self(cap as u64)
    }
}

impl From<ClientCapabilitySet> for AtomicCapabilitySet {
    fn from(caps: ClientCapabilitySet) -> Self {
        Self(AtomicU64::new(caps.0))
    }
}

impl From<AtomicCapabilitySet> for ClientCapabilitySet {
    fn from(caps: AtomicCapabilitySet) -> Self {
        Self(caps.0.load(Ordering::Relaxed))
    }
}

impl From<&AtomicCapabilitySet> for ClientCapabilitySet {
    fn from(caps: &AtomicCapabilitySet) -> Self {
        Self(caps.0.load(Ordering::Relaxed))
    }
}

impl Default for ClientCapabilitySet {
    fn default() -> Self {
        Self::new()
    }
}
