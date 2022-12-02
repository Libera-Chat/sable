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
    pub account: AccountId,
    pub channel: ChannelRegistrationId,
}
