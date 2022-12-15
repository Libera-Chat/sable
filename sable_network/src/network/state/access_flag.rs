use std::ops::{BitOr, BitOrAssign, Not, BitAnd};

use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[derive(EnumString)]
#[strum(serialize_all="snake_case")]
#[serde(rename_all="snake_case")]
#[repr(u64)]
pub enum ChannelAccessFlag
{
    Founder     = 0x8000_0000_0000,

    AccessView  = 0x0100_0000_0000,
    AccessEdit  = 0x0200_0000_0000,
    RoleView    = 0x0400_0000_0000,
    RoleEdit    = 0x0800_0000_0000,

    OpSelf      = 0x0010_0000_0000,
    OpGrant     = 0x0020_0000_0000,
    OpAuto      = 0x0040_0000_0000,

    VoiceSelf   = 0x0001_0000_0000,
    VoiceGrant  = 0x0002_0000_0000,
    VoiceAuto   = 0x0004_0000_0000,

    AlwaysSend      = 0x0000_0001,
    InviteSelf      = 0x0000_0002,
    InviteOther     = 0x0000_0004,

    ReceiveOp       = 0x0000_0010,
    ReceiveVoice    = 0x0000_0020,
    ReceiveOpmod    = 0x0000_0040,

    Topic           = 0x0000_0100,
    Kick            = 0x0000_0200,
    SetSimpleMode   = 0x0000_0400,
    SetKey          = 0x0000_0800,

    BanView         = 0x0001_0000,
    BanAdd          = 0x0002_0000,
    BanRemoveOwn    = 0x0004_0000,
    BanRemoveAny    = 0x0008_0000,

    QuietView       = 0x0010_0000,
    QuietAdd        = 0x0020_0000,
    QuietRemoveOwn  = 0x0040_0000,
    QuietRemoveAny  = 0x0080_0000,

    ExemptView      = 0x0100_0000,
    ExemptAdd       = 0x0200_0000,
    ExemptRemoveOwn = 0x0400_0000,
    ExemptRemoveAny = 0x0800_0000,

    InvexView       = 0x1000_0000,
    InvexAdd        = 0x2000_0000,
    InvexRemoveOwn  = 0x4000_0000,
    InvexRemoveAny  = 0x8000_0000,
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
pub struct ChannelAccessSet(u64);

#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
pub struct ChannelAccessMask(u64);

impl ChannelAccessSet
{
    pub fn new() -> Self { Self(0) }

    pub fn is_set(&self, flag: ChannelAccessFlag) -> bool
    {
        (self.0 & flag as u64) != 0
    }

    pub fn is_empty(&self) -> bool
    {
        self.0 == 0
    }

    pub fn dominates(&self, other: &ChannelAccessSet) -> bool
    {
        use ChannelAccessFlag::*;

        let mut my_flags = self.0;

        // Founder can grant/edit anything
        if my_flags & (Founder as u64) != 0
        {
            return true;
        }

        // Otherwise, the first flag in each pair implies the ability to grant the second one(s)
        let implied_flags = [
            (AccessEdit, AccessView),
            (RoleEdit, RoleView),
            (OpGrant, OpSelf),
            (OpGrant, OpAuto),
            (OpSelf, OpAuto),
            (VoiceGrant, VoiceSelf),
            (VoiceGrant, VoiceAuto),
            (VoiceSelf, VoiceAuto),
            (InviteOther, InviteSelf),
            (BanRemoveAny, BanRemoveOwn),
            (QuietRemoveAny, QuietRemoveOwn),
            (ExemptRemoveAny, ExemptRemoveOwn),
            (InvexRemoveAny, InvexRemoveOwn),
        ];

        for (flag, implied) in implied_flags
        {
            if my_flags & (flag as u64) != 0
            {
                my_flags |= implied as u64;
            }
        }

        (other.0 & !my_flags) == 0
    }
}

impl From<ChannelAccessFlag> for ChannelAccessSet
{
    fn from(value: ChannelAccessFlag) -> Self
    {
        Self(value as u64)
    }
}

impl BitOr for ChannelAccessFlag
{
    type Output = ChannelAccessSet;

    fn bitor(self, rhs: Self) -> Self::Output
    {
        ChannelAccessSet(self as u64 | rhs as u64)
    }
}

impl BitOr<ChannelAccessFlag> for ChannelAccessSet
{
    type Output = Self;

    fn bitor(self, rhs: ChannelAccessFlag) -> Self::Output
    {
        Self(self.0 | rhs as u64)
    }
}

impl BitOr for ChannelAccessSet
{
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output
    {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign<ChannelAccessFlag> for ChannelAccessSet
{
    fn bitor_assign(&mut self, rhs: ChannelAccessFlag)
    {
        self.0 |= rhs as u64
    }
}

impl BitOrAssign for ChannelAccessSet
{
    fn bitor_assign(&mut self, rhs: Self)
    {
        self.0 |= rhs.0
    }
}

impl Not for ChannelAccessFlag
{
    type Output = ChannelAccessMask;

    fn not(self) -> Self::Output
    {
        ChannelAccessMask(!(self as u64))
    }
}

impl Not for ChannelAccessSet
{
    type Output = ChannelAccessMask;

    fn not(self) -> Self::Output
    {
        ChannelAccessMask(!self.0)
    }
}

impl BitAnd<ChannelAccessMask> for ChannelAccessSet
{
    type Output = Self;

    fn bitand(self, rhs: ChannelAccessMask) -> Self::Output
    {
        Self(self.0 & rhs.0)
    }
}