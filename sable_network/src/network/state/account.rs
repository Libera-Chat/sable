use crate::prelude::*;

use serde::{
    Serialize,
    Deserialize
};

#[derive(PartialEq,Debug,Clone,Serialize,Deserialize)]
pub struct Account
{
    pub id: AccountId,
    pub name: Nickname,

    pub authorised_fingerprints: Vec<String>,
}

#[derive(PartialEq,Debug,Clone,Serialize,Deserialize)]
pub struct NickRegistration
{
    pub id: NickRegistrationId,
    pub nick: Nickname,
    pub account: AccountId,
}

#[derive(PartialEq,Debug,Clone,Serialize,Deserialize)]
pub struct ChannelRegistration
{
    pub id: ChannelRegistrationId,
    pub channelname: ChannelName,
}

#[derive(PartialEq,Debug,Clone,Serialize,Deserialize)]
pub struct ChannelAccess
{
    pub id: ChannelAccessId,
    pub role: ChannelRoleId,
}

#[derive(PartialEq,Eq,Hash,Debug,Clone)]
#[derive(serde_with::SerializeDisplay,serde_with::DeserializeFromStr)]
pub enum ChannelRoleName
{
    BuiltinFounder,
    BuiltinOp,
    BuiltinVoice,
    BuiltinAll,
    Custom(CustomRoleName),
}

#[derive(PartialEq,Debug,Clone,Serialize,Deserialize)]
pub struct ChannelRole
{
    pub id: ChannelRoleId,
    pub channel: Option<ChannelRegistrationId>,
    pub name: ChannelRoleName,
    pub flags: super::ChannelAccessSet,
}

impl std::str::FromStr for ChannelRoleName
{
    type Err = <CustomRoleName as Validated>::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        Ok(match s
        {
            "builtin:founder" => Self::BuiltinFounder,
            "builtin:op" => Self::BuiltinOp,
            "builtin:voice" => Self::BuiltinVoice,
            "builtin:all" => Self::BuiltinAll,
            _ => Self::Custom(s.parse()?)
        })
    }
}

impl std::borrow::Borrow<str> for ChannelRoleName
{
    fn borrow(&self) -> &str
    {
        match self
        {
            Self::BuiltinFounder => "builtin:founder",
            Self::BuiltinOp => "builtin:op",
            Self::BuiltinVoice => "builtin:voice",
            Self::BuiltinAll => "builtin:all",
            Self::Custom(s) => s.borrow(),
        }
    }
}

impl std::fmt::Display for ChannelRoleName
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        use std::borrow::Borrow;
        f.write_str(self.borrow())
    }
}