use crate::id::*;
use crate::validated::*;
use crate::flags::*;

#[derive(Debug)]
pub struct User {
    pub id: UserId,
    pub server: ServerId,

    pub nick: Nickname,
    pub user: Username,
    pub visible_host: Hostname,
    pub realname: String,

    pub mode_id: UModeId,
}

#[derive(Debug)]
pub struct UserMode {
    pub id: UModeId,
    pub modes: UserModeSet,
}

impl User {
    pub fn new(id: UserId, server: ServerId,
               nick: &Nickname, user: &Username, visible_host: &Hostname,
               realname: &str, mode_id: UModeId) -> Self
    {
        Self {
            id: id,
            server: server,
            nick: nick.clone(),
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