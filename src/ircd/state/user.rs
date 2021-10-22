use crate::ircd::id::UserId;
use crate::ircd::validated::*;

#[derive(Debug)]
pub struct User {
    pub id: UserId,
    pub nick: Nickname,
    pub user: Username,
    pub visible_host: Hostname,
    pub realname: String,
}

impl User {
    pub fn new(id: UserId, nick: &Nickname, user: &Username, visible_host: &Hostname, realname: &str) -> Self
    {
        Self {
            id: id,
            nick: nick.clone(),
            user: user.clone(),
            visible_host: visible_host.clone(),
            realname: realname.to_string(),
        }
    }
}