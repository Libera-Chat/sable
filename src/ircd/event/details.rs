use crate::ircd::Id;

#[derive(Clone,Debug)]
pub struct NewUser {
    pub nickname: String,
    pub username: String,
    pub visible_hostname: String,
}

#[derive(Clone,Debug)]
pub struct NewChannel {
    pub name: String,
}

#[derive(Clone,Debug)]
pub struct ChannelJoin {
    pub channel: Id,
    pub user: Id,
}