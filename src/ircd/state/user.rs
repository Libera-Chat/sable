use crate::ircd::id::UserId;

#[derive(Debug)]
pub struct User {
    pub id: UserId,
    pub nick: String,
    pub user: String,
    pub visible_host: String,
    pub realname: String,
}

impl User {
    pub fn new(id: UserId, nick: &str, user: &str, visible_host: &str, realname: &str) -> User {
        User {
            id: id,
            nick: nick.to_string(),
            user: user.to_string(),
            visible_host: visible_host.to_string(),
            realname: realname.to_string(),
        }
    }
}