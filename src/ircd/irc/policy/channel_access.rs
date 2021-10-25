use super::*;
use irc::utils::*;

pub trait ChannelPolicyService
{
    fn can_send(&self, user: &User, channel: &Channel, msg: &str) -> PermissionResult;
    fn can_change_mode(&self, user: &User, channel: &Channel, mode: ChannelModeFlag) -> PermissionResult;
    fn can_grant_permission(&self, user: &User, channel: &Channel, target: &User, flag: ChannelPermissionFlag) -> PermissionResult;
    fn can_remove_permission(&self, user: &User, channel: &Channel, target: &User, flag: ChannelPermissionFlag) -> PermissionResult;
}

fn is_channel_operator(user: &User, channel: &Channel) -> PermissionResult
{
    if let Some(membership) = user.is_in_channel(channel.id())
    {
        if membership.permissions().is_set(ChannelPermissionFlag::Op) {
            Ok(())
        } else {
            numeric_error!(ChanOpPrivsNeeded, &channel)
        }
    }
    else
    {
        numeric_error!(NotOnChannel, &channel)
    }
}

impl ChannelPolicyService for StandardPolicyService
{
    fn can_send(&self, user: &User, channel: &Channel, _msg: &str) -> PermissionResult
    {
        if channel.mode()?.has_mode(ChannelModeFlag::NoExternal) 
            && user.is_in_channel(channel.id()).is_none()
        {
            return numeric_error!(CannotSendToChannel, channel);
        }
        Ok(())
    }

    fn can_change_mode(&self, user: &User, channel: &Channel, _mode: ChannelModeFlag) -> PermissionResult
    {
        is_channel_operator(user, channel)
    }

    fn can_grant_permission(&self, user: &User, channel: &Channel, _target: &User, _flag: ChannelPermissionFlag) -> PermissionResult
    {
        is_channel_operator(user, channel)
    }

    fn can_remove_permission(&self, user: &User, channel: &Channel, _target: &User, _flag: ChannelPermissionFlag) -> PermissionResult
    {
        is_channel_operator(user, channel)
    }
}