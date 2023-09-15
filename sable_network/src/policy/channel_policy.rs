use super::*;

/// A `ChannelPolicyService` makes access decisions regarding channel permissions
#[delegatable_trait]
pub trait ChannelPolicyService {
    /// Determine whether the given user can join the given channel
    fn can_join(&self, user: &User, channel: &Channel, key: Option<ChannelKey>)
        -> PermissionResult;

    /// Determine whether the given user can kick the other given user to the given channel
    fn can_kick(
        &self,
        user: &User,
        channel: &Channel,
        target: &User,
        msg: &str,
    ) -> PermissionResult;

    /// Determine whether the given user can change the name of the given channel
    fn can_rename(
        &self,
        user: &User,
        channel: &Channel,
        new_name: &ChannelName,
        msg: Option<&str>,
    ) -> PermissionResult;

    /// Determine whether the given user can send to the given channel
    fn can_send(&self, user: &User, channel: &Channel, msg: &str) -> PermissionResult;

    /// Determine whether one user can see that another is in a channel - e.g. in /whois, /names, etc.
    fn can_see_user_on_channel(&self, user: &User, member: &Membership) -> PermissionResult;

    /// Determine whether the given user can change a specified mode on the given channel
    fn can_change_mode(
        &self,
        user: &User,
        channel: &Channel,
        mode: ChannelModeFlag,
    ) -> PermissionResult;
    /// Determine whether the given user can set the given topic on the given channel
    fn can_set_topic(&self, user: &User, channel: &Channel, topic: &str) -> PermissionResult;

    /// Determine whether the given user can grant a channel privilege flag to the given target user
    fn can_grant_permission(
        &self,
        user: &User,
        channel: &Channel,
        target: &User,
        flag: MembershipFlagFlag,
    ) -> PermissionResult;
    /// Determine whether the given user can remove a channel privilege flag from the given target user
    fn can_remove_permission(
        &self,
        user: &User,
        channel: &Channel,
        target: &User,
        flag: MembershipFlagFlag,
    ) -> PermissionResult;

    /// Determine whether the given string is a valid ban mask for the given channel
    fn validate_ban_mask(
        &self,
        mask: &str,
        mode_type: ListModeType,
        channel: &Channel,
    ) -> PermissionResult;
    /// Determine whether the given user can set a list mode with the given mask on the given channel
    fn can_set_ban(
        &self,
        user: &User,
        chan: &Channel,
        mode_type: ListModeType,
        mask: &str,
    ) -> PermissionResult;
    /// Determine whether the given user can remove a list mode entry from the given channel
    fn can_unset_ban(
        &self,
        user: &User,
        chan: &Channel,
        mode_type: ListModeType,
        mask: &str,
    ) -> PermissionResult;

    /// Determine whether the given user can query a particular type of list mode on a channel
    fn can_query_list(
        &self,
        user: &User,
        chan: &Channel,
        mode_type: ListModeType,
    ) -> PermissionResult;
    /// Determine whether the given user should see changes to the given type of list mode
    fn should_see_list_change(&self, membership: &Membership, mode_type: ListModeType) -> bool;

    /// Determine whether the given user can set a channel key
    fn can_set_key(
        &self,
        user: &User,
        chan: &Channel,
        new_key: Option<&ChannelKey>,
    ) -> PermissionResult;
    /// Determine whether the given user can invite the given target to a channel
    fn can_invite(&self, user: &User, chan: &Channel, target: &User) -> PermissionResult;
}
