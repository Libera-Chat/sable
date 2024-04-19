use std::collections::HashSet;

use super::*;

use UserPermissionError::*;

/// Standard implementation of [`UserPolicyService`]
pub struct StandardUserPolicy {}

impl StandardUserPolicy {
    pub fn new() -> Self {
        Self {}
    }
}

impl UserPolicyService for StandardUserPolicy {
    fn can_set_umode(&self, _user: &wrapper::User, mode: UserModeFlag) -> PermissionResult {
        match mode {
            UserModeFlag::Oper | UserModeFlag::TlsConnection => {
                Err(PermissionError::User(ReadOnlyUmode))
            }
            _ => Ok(()),
        }
    }

    fn can_unset_umode(&self, _user: &wrapper::User, _mode: UserModeFlag) -> PermissionResult {
        Ok(())
    }

    fn can_list_user(&self, to_user: &User, user: &User) -> PermissionResult {
        if !user.mode().has_mode(UserModeFlag::Invisible) {
            return Ok(());
        }

        // If the target user is invisible, check whether they share any channel
        let mut channels1: HashSet<_> = to_user
            .channels()
            .flat_map(|membership| membership.channel().map(|chan| chan.id()))
            .collect();
        let mut channels2: HashSet<_> = user
            .channels()
            .flat_map(|membership| membership.channel().map(|chan| chan.id()))
            .collect();
        if channels1.len() <= channels2.len() {
            std::mem::swap(&mut channels1, &mut channels2);
        }
        for chan in channels2 {
            if channels1.contains(&chan) {
                return Ok(());
            }
        }

        Err(PermissionError::User(Invisible))
    }
}
