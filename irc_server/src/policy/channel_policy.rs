use super::*;

#[delegatable_trait]
pub trait ChannelPolicyService
{
    fn can_join(&self, user: &User, channel: &Channel) -> PermissionResult;
    fn can_send(&self, user: &User, channel: &Channel, msg: &str) -> PermissionResult;

    fn can_see_user_on_channel(&self, user: &User, member: &Membership) -> PermissionResult;

    fn can_change_mode(&self, user: &User, channel: &Channel, mode: ChannelModeFlag) -> PermissionResult;
    fn can_set_topic(&self, user: &User, channel: &Channel, topic: &str) -> PermissionResult;

    fn can_grant_permission(&self, user: &User, channel: &Channel, target: &User, flag: MembershipFlagFlag) -> PermissionResult;
    fn can_remove_permission(&self, user: &User, channel: &Channel, target: &User, flag: MembershipFlagFlag) -> PermissionResult;

    fn validate_ban_mask(&self, mask: &str, mode_type: ListModeType, channel: &Channel) -> PermissionResult;
    fn can_set_ban(&self, user: &User, chan: &Channel, mode_type: ListModeType, mask: &str) -> PermissionResult;
    fn can_unset_ban(&self, user: &User, chan: &Channel, mode_type: ListModeType, mask: &str) -> PermissionResult;
}
