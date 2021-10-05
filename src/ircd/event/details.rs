use crate::ircd::Id;

#[derive(Clone)]
pub struct NewUser {
    pub nickname: String,
    pub username: String,
    pub visible_hostname: String,
}

#[derive(Clone)]
pub struct NewChannel {
    pub name: String,
}

#[derive(Clone)]
pub struct ChannelJoin {
    pub channel: Id,
    pub user: Id,
}