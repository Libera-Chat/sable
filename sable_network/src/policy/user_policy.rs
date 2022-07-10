use super::*;

/// Makes policy decisions regarding users
#[delegatable_trait]
pub trait UserPolicyService
{
    /// Determine whether a given user can set a given user mode on themselves
    fn can_set_umode(&self, user: &wrapper::User, mode: UserModeFlag) -> PermissionResult;
    /// Determine whether a given user can unset a given user mode on themselves
    fn can_unset_umode(&self, user: &wrapper::User, mode: UserModeFlag) -> PermissionResult;
}
