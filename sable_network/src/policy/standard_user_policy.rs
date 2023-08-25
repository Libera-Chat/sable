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
}
