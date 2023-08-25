use super::*;

/// A `ChannelPolicyService` makes access decisions regarding channel permissions
#[delegatable_trait]
pub trait RegistrationPolicyService {
    /// Determine whether the given user can view the access list for a channel
    fn can_view_access(
        &self,
        source: &wrapper::User,
        channel: &wrapper::ChannelRegistration,
    ) -> PermissionResult;

    /// Determine whether the given user can view role permissions for a channel
    fn can_view_roles(
        &self,
        source: &wrapper::User,
        channel: &wrapper::ChannelRegistration,
    ) -> PermissionResult;

    /// Determine whether the given user can change access on a channel for a target user
    fn can_change_access_for(
        &self,
        source: &wrapper::Account,
        channel: &wrapper::ChannelRegistration,
        target: &wrapper::Account,
    ) -> PermissionResult;

    /// Determine whether the given user can grant the given role
    fn can_grant_role(
        &self,
        source: &wrapper::Account,
        channel: &wrapper::ChannelRegistration,
        role: &wrapper::ChannelRole,
    ) -> PermissionResult;

    /// Determine whether the given user can edit the given role
    fn can_edit_role(
        &self,
        source: &wrapper::Account,
        channel: &wrapper::ChannelRegistration,
        role: &wrapper::ChannelRole,
    ) -> PermissionResult;

    /// Determine whether the given user can create/edit a role with the given flags
    fn can_create_role(
        &self,
        source: &wrapper::Account,
        channel: &wrapper::ChannelRegistration,
        with_flags: &state::ChannelAccessSet,
    ) -> PermissionResult;
}
