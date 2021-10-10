use crate::ircd::Id;

#[derive(Debug)]
pub struct User {
    pub id: Id,
    pub nick: String,
    pub user: String,
    pub visible_host: String,
}

impl User {
    pub fn new(id: Id, nick: &str, user: &str, visible_host: &str) -> User {
        User {
            id: id,
            nick: nick.to_string(),
            user: user.to_string(),
            visible_host: visible_host.to_string()
        }
    }
}