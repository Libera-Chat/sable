use crate::id::*;
use crate::validated::*;
use crate::modes::*;
use serde::{
    Serialize,
    Deserialize
};

/// A nickname binding.
/// 
/// The binding denotes current ownership of a given nickname at a point in
/// time. Although nicknames are the primary identifier for a user in the
/// client protocol, the server protocol permits a user to exist detached from
/// its nickname; in the case of nick collisions this is resolved by binding to
/// a unique numeric nickname derived from the user ID.
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NickBinding {
    pub nick: Nickname,
    pub user: UserId,
    pub timestamp: i64,
    pub created: EventId,
}

/// A user.
/// 
/// Note that the user's nickname is not included here; that is stored in a
/// separate [`NickBinding`] record.
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct User {
    pub id: UserId,
    pub server: ServerId,

    pub user: Username,
    pub visible_host: Hostname,
    pub realname: String,

    pub mode_id: UserModeId,
}

/// A user mode. Changing modes does not need to update the user object, only
/// the mode.
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct UserMode {
    pub id: UserModeId,
    pub modes: UserModeSet,
}


impl NickBinding
{
    pub fn new(nick: Nickname, user: UserId, timestamp: i64, created: EventId) -> Self
    {
        Self { nick: nick, user: user, timestamp: timestamp, created: created }
    }
}

impl User {
    pub fn new(id: UserId, server: ServerId,
               user: &Username, visible_host: &Hostname,
               realname: &str, mode_id: UserModeId) -> Self
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
    pub fn new(id: UserModeId, mode: UserModeSet) -> Self
    {
        Self {
            id: id,
            modes: mode
        }
    }
}
