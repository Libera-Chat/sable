use crate::id::*;
use crate::validated::*;
use crate::flags::*;
use serde::{
    Serialize,
    Deserialize
};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NickBinding {
    pub nick: Nickname,
    pub user: UserId,
    pub timestamp: u64,
    pub created: EventId,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct User {
    pub id: UserId,
    pub server: ServerId,

    pub user: Username,
    pub visible_host: Hostname,
    pub realname: String,

    pub mode_id: UModeId,
}

#[derive(Debug,Clone,Serialize,Deserialize)]

pub struct UserMode {
    pub id: UModeId,
    pub modes: UserModeSet,
}

impl NickBinding
{
    pub fn new(nick: Nickname, user: UserId, timestamp: u64, created: EventId) -> Self
    {
        Self { nick: nick, user: user, timestamp: timestamp, created: created }
    }
}

impl User {
    pub fn new(id: UserId, server: ServerId,
               user: &Username, visible_host: &Hostname,
               realname: &str, mode_id: UModeId) -> Self
    {
        Self {
            id: id,
            server: server,
            user: user.clone(),
            visible_host: visible_host.clone(),
            realname: realname.to_string(),
            mode_id: mode_id,
        }
    }
}

impl UserMode {
    pub fn new(id: UModeId, mode: UserModeSet) -> Self
    {
        Self {
            id: id,
            modes: mode
        }
    }
}