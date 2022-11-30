use crate::prelude::*;

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

    pub mode: UserMode,
    pub oper_privileges: Option<UserPrivileges>,

    pub account: Option<AccountId>,

    pub session_key: Option<UserSessionKey>,
}

/// A persistent session key. If present on a [`User`], then that user's session
/// is persistent, and knowledge of the key can be used by subsequent connections
/// to attach to the existing session
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct UserSessionKey
{
    pub timestamp: i64,
    pub enabled_by: EventId,
    pub key_hash: String,
}

/// A user mode. Changing modes does not need to update the user object, only
/// the mode.
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct UserMode {
    pub modes: UserModeSet,
}

/// A user's operator privileges
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct UserPrivileges {
    pub oper_name: String,
}

impl NickBinding
{
    pub fn new(nick: Nickname, user: UserId, timestamp: i64, created: EventId) -> Self
    {
        Self { nick, user, timestamp, created }
    }
}

impl User {
    pub fn new(id: UserId, server: ServerId,
               user: Username, visible_host: Hostname,
               realname: String, mode: UserMode,
               account: Option<AccountId>) -> Self
    {
        Self {
            id,
            server,
            user,
            visible_host,
            realname,
            mode,
            oper_privileges: None,
            account,
            session_key: None,
        }
    }
}

impl UserMode {
    pub fn new(modes: UserModeSet) -> Self
    {
        Self {
            modes
        }
    }
}
