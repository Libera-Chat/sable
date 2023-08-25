use super::*;
use crate::network::config::OperConfig;

use UserPermissionError::*;

use pwhash::unix;

/// Standard implementation of [`OperPolicyService`] and [`OperAuthenticationService`]
pub struct StandardOperPolicy {}

impl StandardOperPolicy {
    pub fn new() -> Self {
        Self {}
    }
}

impl OperPolicyService for StandardOperPolicy {
    fn user_can_oper(&self, _user: &wrapper::User) -> PermissionResult {
        Ok(())
    }

    fn require_oper(&self, user: &wrapper::User) -> PermissionResult {
        if user.is_oper() {
            Ok(())
        } else {
            Err(PermissionError::User(NotOper))
        }
    }

    fn can_set_kline(
        &self,
        oper: &wrapper::User,
        _user: &Pattern,
        _host: &Pattern,
        _duration: i64,
    ) -> PermissionResult {
        self.require_oper(oper)
    }

    fn can_kill(&self, oper: &wrapper::User, _target: &wrapper::User) -> PermissionResult {
        self.require_oper(oper)
    }
}

impl OperAuthenticationService for StandardOperPolicy {
    fn authenticate(&self, oper_config: &OperConfig, user: &str, pass: &str) -> bool {
        user == oper_config.name && unix::verify(pass, &oper_config.hash)
    }
}
