use super::*;

/// Standard implementation of [`DebuggingPolicyService`]
pub struct StandardDebuggingPolicy {}

impl StandardDebuggingPolicy {
    pub fn new() -> Self {
        Self {}
    }
}

impl DebuggingPolicyService for StandardDebuggingPolicy {
    fn can_list_links(&self, _user: &wrapper::User) -> PermissionResult {
        Ok(())
    }
}
