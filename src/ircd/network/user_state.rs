use super::*;


impl Network {
    pub(super) fn new_user(&mut self, target: UserId, _event: &Event, user: &details::NewUser)
    {
        let user = state::User::new(target, &user.nickname, &user.username, &user.visible_hostname, &user.realname);
        self.users.insert(user.id, user);
    }

    pub(super) fn validate_new_user(&self, _target: UserId, user: &details::NewUser) -> ValidationResult
    {
        if self.users.iter().filter(|u| &u.1.nick == user.nickname.value()).count() > 0 {
            Err(ValidationError::NickInUse(user.nickname.clone()))
        } else {
            Ok(())
        }
    }

    pub(super) fn user_quit(&mut self, target: UserId, _event: &Event, _quit: &details::UserQuit)
    {
        self.memberships.retain(|_, v| v.user != target);
        self.users.remove(&target);
    }
}