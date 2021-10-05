use super::Network;
use crate::ircd::*;

use crate::ircd::wrapper::*;

impl Network {
    pub fn user(&self, id: Id) -> Option<wrapper::User> {
        self.users.get(&id).wrap(&self)
    }

    pub fn users(&self) -> impl std::iter::Iterator<Item=wrapper::User> + '_ {
        self.raw_users().wrap(&self)
    }

    pub fn raw_users(&self) -> impl std::iter::Iterator<Item=&state::User> {
        self.users.values()
    }

    pub fn channel(&self, id: Id) -> Option<wrapper::Channel> {
        self.channels.get(&id).wrap(self)
    }

    pub fn channels(&self) -> impl std::iter::Iterator<Item=wrapper::Channel> + '_ {
        self.raw_channels().wrap(self)
    }

    pub fn raw_channels(&self) -> impl std::iter::Iterator<Item=&state::Channel> {
        self.channels.values()
    }

    pub fn membership(&self, id: Id) -> Option<wrapper::Membership> {
        self.memberships.get(&id).wrap(self)
    }

    pub fn memberships(&self) -> impl std::iter::Iterator<Item=wrapper::Membership> + '_ {
        self.raw_memberships().wrap(self)
    }

    pub fn raw_memberships(&self) -> impl std::iter::Iterator<Item=&state::Membership> {
        self.memberships.values()
    }

}