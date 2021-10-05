use super::Network;
use crate::ircd::*;
use crate::ircd::event::*;

impl Network {
    pub fn new_user(&mut self, event: &Event, user: &details::NewUser) {
        let user = state::User::new(event.target, &user.nickname, &user.username, &user.visible_hostname);
        self.users.insert(user.id, user);
    }
}