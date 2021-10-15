use super::Network;
use crate::ircd::*;
use crate::ircd::event::*;

impl Network {
    pub fn new_user(&mut self, target: UserId, _event: &Event, user: &details::NewUser)
    {
        let user = state::User::new(target, &user.nickname, &user.username, &user.visible_hostname, &user.realname);
        self.users.insert(user.id, user);
    }

    pub fn user_quit(&mut self, target: UserId, _event: &Event, _quit: &details::UserQuit)
    {
        self.memberships.retain(|_, v| v.user != target);
        self.users.remove(&target);
    }
}