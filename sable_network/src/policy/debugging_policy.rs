#![allow(clippy::crate_in_macro_def)]

use super::*;

/// Makes authorisation decisions regarding inspecting private but non-sensitive
/// network information
#[delegatable_trait]
pub trait DebuggingPolicyService {
    /// Determine whether a given user is permitted to oper up
    fn can_list_links(&self, user: &wrapper::User) -> PermissionResult;
}
