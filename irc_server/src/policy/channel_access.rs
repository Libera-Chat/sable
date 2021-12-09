use super::*;
use crate::utils::*;

pub trait ChannelPolicyService
{
    fn can_send(&self, user: &User, channel: &Channel, msg: &str) -> PermissionResult;

    fn can_see_user_on_channel(&self, user: &User, member: &Membership) -> PermissionResult;

    fn can_change_mode(&self, user: &User, channel: &Channel, mode: ChannelModeFlag) -> PermissionResult;
    fn can_set_topic(&self, user: &User, channel: &Channel, topic: &str) -> PermissionResult;

    fn can_grant_permission(&self, user: &User, channel: &Channel, target: &User, flag: MembershipFlagFlag) -> PermissionResult;
    fn can_remove_permission(&self, user: &User, channel: &Channel, target: &User, flag: MembershipFlagFlag) -> PermissionResult;
}

fn is_channel_operator(user: &User, channel: &Channel) -> PermissionResult
{
    if let Some(membership) = user.is_in_channel(channel.id())
    {
        if membership.permissions().is_set(MembershipFlagFlag::Op) {
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

    fn can_see_user_on_channel(&self, user: &User, member: &Membership) -> PermissionResult
    {
        let chan = member.channel()?;
        let user_is_on_chan = user.is_in_channel(chan.id()).is_some();
        if user_is_on_chan
        {
            return Ok(());
        }

        let chan_is_secret = chan.mode()?.has_mode(ChannelModeFlag::Secret);
        let user_is_invis = member.user()?.mode()?.has_mode(UserModeFlag::Invisible);
        if chan_is_secret || user_is_invis
        {
            return numeric_error!(NotOnChannel, &chan);
        }
        Ok(())
    }

    fn can_change_mode(&self, user: &User, channel: &Channel, _mode: ChannelModeFlag) -> PermissionResult
    {
        is_channel_operator(user, channel)
    }

    fn can_set_topic(&self, user: &User, channel: &Channel, _topic: &str) -> PermissionResult
    {
        if channel.mode()?.has_mode(ChannelModeFlag::TopicLock)
        {
            is_channel_operator(user, channel)
        }
        else
        {
            Ok(())
        }
    }

    fn can_grant_permission(&self, user: &User, channel: &Channel, _target: &User, _flag: MembershipFlagFlag) -> PermissionResult
    {
        is_channel_operator(user, channel)
    }

    fn can_remove_permission(&self, user: &User, channel: &Channel, _target: &User, _flag: MembershipFlagFlag) -> PermissionResult
    {
        is_channel_operator(user, channel)
    }
}