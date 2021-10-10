use crate::ircd::Id;

#[derive(Debug)]
pub struct Channel {
    pub id: Id,
    pub name: String,
}

#[derive(Debug)]
pub struct Membership {
    pub id: Id,
    pub channel: Id,
    pub user: Id,
}

impl Channel {
    pub fn new(id: Id, name: &str) -> Channel {
        Channel{ id: id, name: name.to_string() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Membership {
    pub fn new(id: Id, user: Id, channel: Id) -> Membership {
        Membership{ id: id, user: user, channel: channel }
    }

    pub fn user(&self) -> Id {
        self.user
    }

    pub fn channel(&self) -> Id {
        self.channel
    }
}