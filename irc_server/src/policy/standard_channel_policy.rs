use super::*;
use crate::utils::*;

/// Standard implementation of [`ChannelPolicyService`]
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
            numeric_error!(ChanOpPrivsNeeded, channel)
        }
    }
    else
    {
        numeric_error!(NotOnChannel, channel)
    }
}

impl ChannelPolicyService for StandardChannelPolicy
{
    fn can_join(&self, user: &User, channel: &Channel, key: Option<ChannelKey>) -> PermissionResult
    {
        let chan_key = channel.mode().key();
        if chan_key.is_some() && key != chan_key
        {
            return numeric_error!(BadChannelKey, channel)
        }

        if channel.mode().has_mode(ChannelModeFlag::InviteOnly)
            && user.has_invite_for(channel.id()).is_none()
            && self.ban_resolver.user_matches_list(user, &channel.list(ListModeType::Invex)).is_none()
        {
            return numeric_error!(InviteOnlyChannel, channel);
        }

        if self.ban_resolver.user_matches_list(user, &channel.list(ListModeType::Ban)).is_some()
            && self.ban_resolver.user_matches_list(user, &channel.list(ListModeType::Except)).is_none()
        {
            return numeric_error!(BannedOnChannel, channel)
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
            return numeric_error!(CannotSendToChannel, channel);
        }

        if (self.ban_resolver.user_matches_list(user, &channel.list(ListModeType::Ban)).is_some()
                || self.ban_resolver.user_matches_list(user, &channel.list(ListModeType::Quiet)).is_some())
              && self.ban_resolver.user_matches_list(user, &channel.list(ListModeType::Except)).is_none()
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

        let chan_is_secret = chan.mode().has_mode(ChannelModeFlag::Secret);
        let user_is_invis = member.user()?.mode().has_mode(UserModeFlag::Invisible);
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
        if channel.mode().has_mode(ChannelModeFlag::TopicLock)
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

    fn can_query_list(&self, user: &User, chan: &Channel, mode_type: ListModeType) -> PermissionResult
    {
        match mode_type
        {
            ListModeType::Ban | ListModeType::Quiet => Ok(()),
            ListModeType::Except | ListModeType::Invex => is_channel_operator(user, chan)
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

    fn can_set_key(&self, user: &User, chan: &Channel, _new_key: Option<&ChannelKey>) -> PermissionResult
    {
        is_channel_operator(user, chan)
    }

    fn can_invite(&self, user: &User, chan: &Channel, _target: &User) -> PermissionResult
    {
        is_channel_operator(user, chan)
    }
}