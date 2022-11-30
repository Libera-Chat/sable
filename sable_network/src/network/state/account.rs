use crate::prelude::*;

use serde::{
    Serialize,
    Deserialize
};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Account
{
    pub id: AccountId,
    pub name: Nickname,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NickRegistration
{
    pub id: NickRegistrationId,
    pub nick: Nickname,
    pub account: AccountId,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ChannelRegistration
{
    pub id: ChannelRegistrationId,
    pub channelname: ChannelName,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ChannelAccess
{
    pub id: ChannelAccessId,
    pub account: AccountId,
    pub channel: ChannelRegistrationId,
}
