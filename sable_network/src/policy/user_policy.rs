use super::*;

/// Makes policy decisions regarding users
#[delegatable_trait]
pub trait UserPolicyService {
    /// Determine whether a given user can set a given user mode on themselves
    fn can_set_umode(&self, user: &wrapper::User, mode: UserModeFlag) -> PermissionResult;
    /// Determine whether a given user can unset a given user mode on themselves
    fn can_unset_umode(&self, user: &wrapper::User, mode: UserModeFlag) -> PermissionResult;
    /// Determine whether `to_user` can discover `user` without knowing their nick
    /// (eg. with `WHO *`)
    fn can_list_user(&self, to_user: &User, user: &User) -> PermissionResult;
}
