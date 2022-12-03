use super::*;

use ChannelPermissionError::*;
use UserPermissionError::*;

/// Standard implementation of [`ChannelPolicyService`]
pub struct StandardChannelPolicy
{
    ban_resolver: StandardBanResolver
}

impl StandardChannelPolicy {
    pub fn new() -> Self
    {
        Self {
            ban_resolver: StandardBanResolver::new()
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
            Err(PermissionError::Channel(*channel.name(), UserNotOp))
        }
    }
    else
    {
        Err(PermissionError::Channel(*channel.name(), UserNotOnChannel))
    }
}

fn has_access(user: &User, channel: &Channel, flag: ChannelAccessFlag) -> PermissionResult
{
    let Ok(Some(account)) = user.account() else {
        return Err(PermissionError::User(NotLoggedIn));
    };
    let Some(channel_reg) = channel.is_registered() else {
        return Err(PermissionError::Channel(*channel.name(), NotRegistered))
    };
    let Some(access) = account.has_access_in(channel_reg.id()) else {
        return Err(PermissionError::Channel(*channel.name(), NoAccess))
    };
    if access.flags().is_set(flag) {
        Ok(())
    } else {
        Err(PermissionError::Channel(*channel.name(), NoAccess))
    }
}

impl ChannelPolicyService for StandardChannelPolicy
{
    fn can_join(&self, user: &User, channel: &Channel, key: Option<ChannelKey>) -> PermissionResult
    {
        let chan_key = channel.mode().key();
        if chan_key.is_some() && key != chan_key
        {
            return Err(PermissionError::Channel(*channel.name(), BadChannelKey));
        }

        if channel.mode().has_mode(ChannelModeFlag::InviteOnly)
            && user.has_invite_for(channel.id()).is_none()
            && self.ban_resolver.user_matches_list(user, &channel.list(ListModeType::Invex)).is_none()
        {
            return Err(PermissionError::Channel(*channel.name(), InviteOnlyChannel));
        }

        if self.ban_resolver.user_matches_list(user, &channel.list(ListModeType::Ban)).is_some()
            && self.ban_resolver.user_matches_list(user, &channel.list(ListModeType::Except)).is_none()
        {
            return Err(PermissionError::Channel(*channel.name(), UserIsBanned));
        }

        Ok(())
    }

    fn can_send(&self, user: &User, channel: &Channel, _msg: &str) -> PermissionResult
    {
        if let Some(membership) = user.is_in_channel(channel.id())
        {
            // Being in the channel and opped or voiced overrides everything
            if membership.permissions().is_set(MembershipFlagFlag::Op)
                || membership.permissions().is_set(MembershipFlagFlag::Voice)
            {
                return Ok(());
            }
        }
        else if channel.mode().has_mode(ChannelModeFlag::NoExternal)
        {
            // If it's +n and they're not in it, no point testing anything else
            return Err(PermissionError::Channel(*channel.name(), CannotSendToChannel));
        }

        if (self.ban_resolver.user_matches_list(user, &channel.list(ListModeType::Ban)).is_some()
                || self.ban_resolver.user_matches_list(user, &channel.list(ListModeType::Quiet)).is_some())
              && self.ban_resolver.user_matches_list(user, &channel.list(ListModeType::Except)).is_none()
        {
            return Err(PermissionError::Channel(*channel.name(), CannotSendToChannel));
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

        let chan_is_secret = chan.mode().has_mode(ChannelModeFlag::Secret);
        let user_is_invis = member.user()?.mode().has_mode(UserModeFlag::Invisible);
        if chan_is_secret || user_is_invis
        {
            return Err(PermissionError::Channel(*chan.name(), UserNotOnChannel));
        }
        Ok(())
    }

    fn can_change_mode(&self, user: &User, channel: &Channel, _mode: ChannelModeFlag) -> PermissionResult
    {
        has_access(user, channel, ChannelAccessFlag::Op).or(is_channel_operator(user, channel))
    }

    fn can_set_topic(&self, user: &User, channel: &Channel, _topic: &str) -> PermissionResult
    {
        if channel.mode().has_mode(ChannelModeFlag::TopicLock)
        {
            has_access(user, channel, ChannelAccessFlag::Op).or(is_channel_operator(user, channel))
        }
        else
        {
            Ok(())
        }
    }

    fn can_grant_permission(&self, user: &User, channel: &Channel, _target: &User, _flag: MembershipFlagFlag) -> PermissionResult
    {
        has_access(user, channel, ChannelAccessFlag::Op).or(is_channel_operator(user, channel))
    }

    fn can_remove_permission(&self, user: &User, channel: &Channel, _target: &User, _flag: MembershipFlagFlag) -> PermissionResult
    {
        has_access(user, channel, ChannelAccessFlag::Op).or(is_channel_operator(user, channel))
    }

    fn validate_ban_mask(&self, _mask: &str, _mode_type: ListModeType, _channel: &Channel) -> PermissionResult
    {
        Ok(())
    }

    fn can_set_ban(&self, user: &User, channel: &Channel, _mode_type: ListModeType, _mask: &str) -> PermissionResult
    {
        has_access(user, channel, ChannelAccessFlag::Op).or(is_channel_operator(user, channel))
    }

    fn can_unset_ban(&self, user: &User, channel: &Channel, _mode_type: ListModeType, _mask: &str) -> PermissionResult
    {
        has_access(user, channel, ChannelAccessFlag::Op).or(is_channel_operator(user, channel))
    }

    fn can_query_list(&self, user: &User, channel: &Channel, mode_type: ListModeType) -> PermissionResult
    {
        match mode_type
        {
            ListModeType::Ban | ListModeType::Quiet => Ok(()),
            ListModeType::Except | ListModeType::Invex =>
                has_access(user, channel, ChannelAccessFlag::Op).or(is_channel_operator(user, channel))
        }
    }

    fn should_see_list_change(&self, member: &Membership, mode_type: ListModeType) -> bool
    {
        match mode_type
        {
            ListModeType::Ban | ListModeType::Quiet => true,
            ListModeType::Except | ListModeType::Invex => member.permissions().is_set(MembershipFlagFlag::Op)
        }
    }

    fn can_set_key(&self, user: &User, channel: &Channel, _new_key: Option<&ChannelKey>) -> PermissionResult
    {
        has_access(user, channel, ChannelAccessFlag::Op).or(is_channel_operator(user, channel))
    }

    fn can_invite(&self, user: &User, channel: &Channel, _target: &User) -> PermissionResult
    {
        has_access(user, channel, ChannelAccessFlag::Op).or(is_channel_operator(user, channel))
    }
}