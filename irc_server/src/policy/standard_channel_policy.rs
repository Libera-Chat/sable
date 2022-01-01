use super::*;
use crate::utils::*;

pub struct StandardChannelPolicy
{
    ban_resolver: Box<dyn BanResolver>
}

impl StandardChannelPolicy {
    pub fn new() -> Self
    {
        Self { 
            ban_resolver: Box::new(StandardBanResolver::new())
        }
    }
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

impl ChannelPolicyService for StandardChannelPolicy
{
    fn can_join(&self, user: &User, channel: &Channel) -> PermissionResult
    {
        if self.ban_resolver.user_matches_list(user, &channel.mode()?.list(ListModeType::Ban)?).is_some()
        {
            numeric_error!(BannedOnChannel, channel)
        }
        else
        {
            Ok(())
        }
    }

    fn can_send(&self, user: &User, channel: &Channel, _msg: &str) -> PermissionResult
    {
        if channel.mode()?.has_mode(ChannelModeFlag::NoExternal) 
            && user.is_in_channel(channel.id()).is_none()
        {
            numeric_error!(CannotSendToChannel, channel)
        }
        else if self.ban_resolver.user_matches_list(user, &channel.mode()?.list(ListModeType::Ban)?).is_some()
        {
            numeric_error!(CannotSendToChannel, channel)
        }
        else
        {
            Ok(())
        }
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

    fn validate_ban_mask(&self, _mask: &str, _mode_type: ListModeType, _channel: &Channel) -> PermissionResult
    {
        Ok(())
    }

    fn can_set_ban(&self, user: &User, chan: &Channel, _mode_type: ListModeType, _mask: &str) -> PermissionResult
    {
        is_channel_operator(user, chan)
    }

    fn can_unset_ban(&self, user: &User, chan: &Channel, _mode_type: ListModeType, _mask: &str) -> PermissionResult
    {
        is_channel_operator(user, chan)
    }
}