use super::*;

pub struct StandardUserPolicy
{
}

impl StandardUserPolicy
{
    pub fn new() -> Self
    {
        Self {
        }
    }
}

impl UserPolicyService for StandardUserPolicy
{
    fn can_set_umode(&self, _user: &wrapper::User, mode: UserModeFlag) -> PermissionResult
    {
        if mode == UserModeFlag::Oper
        {
            return Err(PermissionError::CustomError);
        }

        Ok(())
    }

    fn can_unset_umode(&self, _user: &wrapper::User, _mode: UserModeFlag) -> PermissionResult
    {
        Ok(())
    }
}