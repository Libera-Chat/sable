use super::*;


#[delegatable_trait]
pub trait UserPolicyService
{
    fn can_set_umode(&self, user: &wrapper::User, mode: UserModeFlag) -> PermissionResult;
    fn can_unset_umode(&self, user: &wrapper::User, mode: UserModeFlag) -> PermissionResult;
}
