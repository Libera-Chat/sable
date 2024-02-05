use super::*;

/// Makes authentication decisions for users attempting to gain oper access
#[delegatable_trait]
pub trait OperAuthenticationService {
    fn authenticate(
        &self,
        oper_config: &crate::network::config::OperConfig,
        user: &str,
        pass: &str,
    ) -> bool;
}

/// Makes authorisation decisions regarding oper actions
#[delegatable_trait]
pub trait OperPolicyService {
    /// Determine whether a given user is permitted to oper up
    fn user_can_oper(&self, user: &wrapper::User) -> PermissionResult;

    /// Utility function to determine whether the given user is opered (regardless of privileges)
    fn require_oper(&self, user: &wrapper::User) -> PermissionResult;

    /// Determine whether the given oper can set a kline
    fn can_set_kline(
        &self,
        oper: &wrapper::User,
        user: &Pattern,
        host: &Pattern,
        duration: i64,
    ) -> PermissionResult;

    /// Determine whether the given oper can disconnect the given target user
    fn can_kill(&self, oper: &wrapper::User, target: &wrapper::User) -> PermissionResult;

    /// Determine whether the given user can see detailed connection information about the target user
    fn can_see_connection_info(&self, source: &wrapper::User, target: &wrapper::User) -> bool;
}
