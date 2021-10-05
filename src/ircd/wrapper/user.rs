use crate::ircd::*;

use super::*;

pub struct User<'a> {
    network: &'a Network,
    data: &'a state::User,
}

impl User<'_> {
    pub fn nick(&self) -> &str {
        &self.data.nick
    }

    pub fn user(&self) -> &str {
        &self.data.user
    }

    pub fn visible_host(&self) -> &str {
        &self.data.visible_host
    }

    pub fn channels(&self) -> impl Iterator<Item=Membership> {
        let my_id = self.data.id;
        self.network.raw_memberships().filter(move|x| x.user == my_id).wrap(self.network)
    }
}

impl<'a> super::ObjectWrapper<'a> for User<'a> {
    type Underlying = state::User;

    fn wrap(net: &'a Network, data: &'a state::User) -> Self {
        Self{network: net, data: data}
    }
}