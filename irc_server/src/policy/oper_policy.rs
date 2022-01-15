use super::*;

#[delegatable_trait]
pub trait OperAuthenticationService
{
    fn authenticate(&self, oper_config: &irc_network::config::OperConfig, user: &str, pass: &str) -> bool;
}

#[delegatable_trait]
pub trait OperPolicyService
{
    fn user_can_oper(&self, user: &wrapper::User) -> PermissionResult;

    fn require_oper(&self, user: &wrapper::User) -> PermissionResult;

    fn can_set_kline(&self, oper: &wrapper::User, user: &Pattern, host: &Pattern, duration: i64) -> PermissionResult;
    fn can_kill(&self, oper: &wrapper::User, target: &wrapper::User) -> PermissionResult;
}
