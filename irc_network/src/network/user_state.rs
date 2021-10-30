use super::*;


impl Network {
    pub(super) fn new_user(&mut self, target: UserId, _event: &Event, user: &details::NewUser)
    {
        let user = state::User::new(target,
                                    &user.nickname,
                                    &user.username,
                                    &user.visible_hostname,
                                    &user.realname,
                                    user.mode_id,
                                );
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

    pub(super) fn user_nick_change(&mut self, target: UserId, _event: &Event, detail: &details::UserNickChange)
    {
        if let Some(user) = self.users.get_mut(&target)
        {
            user.nick = detail.new_nick.clone();
        }
    }

    pub(super) fn validate_nick_change(&self, _target: UserId, change: &details::UserNickChange) -> ValidationResult
    {
        if self.users.iter().filter(|u| &u.1.nick == change.new_nick.value()).count() > 0 {
            Err(ValidationError::NickInUse(change.new_nick.clone()))
        } else {
            Ok(())
        }
    }

    pub(super) fn new_user_mode(&mut self, target: UModeId, _event: &Event, mode: &details::NewUserMode)
    {
        let mode = state::UserMode::new(target, mode.mode);
        self.user_modes.insert(target, mode);
    }

    pub(super) fn user_mode_change(&mut self, target: UModeId, _event: &Event, mode: &details::UserModeChange)
    {
        if let Some(umode) = self.user_modes.get_mut(&target)
        {
            umode.modes |= mode.added;
            umode.modes &= !mode.removed;
        }
    }

    pub(super) fn user_quit(&mut self, target: UserId, _event: &Event, _quit: &details::UserQuit)
    {
        self.memberships.retain(|_, v| v.user != target);
        self.users.remove(&target);
    }
}