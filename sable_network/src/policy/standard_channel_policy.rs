use super::*;
use state::{ChannelAccessFlag, ChannelRoleName};
use ChannelPermissionError::*;

/// Standard implementation of [`ChannelPolicyService`]
pub struct StandardChannelPolicy {
    ban_resolver: StandardBanResolver,
}

impl StandardChannelPolicy {
    pub fn new() -> Self {
        Self {
            ban_resolver: StandardBanResolver::new(),
        }
    }
}

fn has_assigned_access(user: &User, channel: &Channel) -> Option<state::ChannelAccessSet> {
    let account = user.account().ok()??;
    let channel_reg = channel.is_registered()?;

    account
        .has_access_in(channel_reg.id())
        .and_then(|a| a.role().ok().map(|r| r.flags()))
}

fn has_ephemeral_access(user: &User, channel: &Channel) -> Option<state::ChannelAccessSet> {
    // If they're not in the channel, they can't have +o or +v
    let member = user.is_in_channel(channel.id())?;

    if member.permissions().is_set(MembershipFlagFlag::Op) {
        channel
            .has_role_named(&ChannelRoleName::BuiltinOp)
            .map(|r| r.flags())
    } else if member.permissions().is_set(MembershipFlagFlag::Voice) {
        channel
            .has_role_named(&ChannelRoleName::BuiltinVoice)
            .map(|r| r.flags())
    } else {
        None
    }
}

fn has_default_access(channel: &Channel) -> Option<state::ChannelAccessSet> {
    channel
        .has_role_named(&ChannelRoleName::BuiltinAll)
        .map(|r| r.flags())
}

fn has_access(user: &User, channel: &Channel, flag: ChannelAccessFlag) -> PermissionResult {
    let assigned = has_assigned_access(user, channel);
    let ephemeral = has_ephemeral_access(user, channel);
    let default = has_default_access(channel);

    if assigned.map(|f| f.is_set(flag)) == Some(true) {
        Ok(())
    } else if ephemeral.map(|f| f.is_set(flag)) == Some(true) {
        Ok(())
    } else if default.map(|f| f.is_set(flag)) == Some(true) {
        Ok(())
    } else {
        Err(PermissionError::Channel(*channel.name(), UserNotOp))
    }
}

impl ChannelPolicyService for StandardChannelPolicy {
    fn can_join(
        &self,
        user: &User,
        channel: &Channel,
        key: Option<ChannelKey>,
    ) -> PermissionResult {
        if has_access(user, channel, ChannelAccessFlag::InviteSelf).is_ok() {
            return Ok(());
        }

        let chan_key = channel.mode().key();
        if chan_key.is_some() && key != chan_key {
            return Err(PermissionError::Channel(*channel.name(), BadChannelKey));
        }

        if channel.mode().has_mode(ChannelModeFlag::InviteOnly)
            && user.has_invite_for(channel.id()).is_none()
            && self
                .ban_resolver
                .user_matches_list(user, &channel.list(ListModeType::Invex))
                .is_none()
        {
            return Err(PermissionError::Channel(*channel.name(), InviteOnlyChannel));
        }

        if self
            .ban_resolver
            .user_matches_list(user, &channel.list(ListModeType::Ban))
            .is_some()
            && self
                .ban_resolver
                .user_matches_list(user, &channel.list(ListModeType::Except))
                .is_none()
        {
            return Err(PermissionError::Channel(*channel.name(), UserIsBanned));
        }

        Ok(())
    }

    fn can_send(&self, user: &User, channel: &Channel, _msg: &str) -> PermissionResult {
        if channel.mode().has_mode(ChannelModeFlag::NoExternal)
            && user.is_in_channel(channel.id()).is_none()
        {
            // If it's +n and they're not in it, no point testing anything else
            return Err(PermissionError::Channel(
                *channel.name(),
                CannotSendToChannel,
            ));
        }

        // AlwaysSend check replaces conventional op/voice, as this flag is normally assigned
        // to those roles
        if has_access(user, channel, ChannelAccessFlag::AlwaysSend).is_ok() {
            return Ok(());
        }

        if (self
            .ban_resolver
            .user_matches_list(user, &channel.list(ListModeType::Ban))
            .is_some()
            || self
                .ban_resolver
                .user_matches_list(user, &channel.list(ListModeType::Quiet))
                .is_some())
            && self
                .ban_resolver
                .user_matches_list(user, &channel.list(ListModeType::Except))
                .is_none()
        {
            return Err(PermissionError::Channel(
                *channel.name(),
                CannotSendToChannel,
            ));
        }

        Ok(())
    }

    fn can_see_user_on_channel(&self, user: &User, member: &Membership) -> PermissionResult {
        let chan = member.channel()?;
        let user_is_on_chan = user.is_in_channel(chan.id()).is_some();
        if user_is_on_chan {
            return Ok(());
        }

        let chan_is_secret = chan.mode().has_mode(ChannelModeFlag::Secret);
        let user_is_invis = member.user()?.mode().has_mode(UserModeFlag::Invisible);
        if chan_is_secret || user_is_invis {
            return Err(PermissionError::Channel(*chan.name(), UserNotOnChannel));
        }
        Ok(())
    }

    fn can_change_mode(
        &self,
        user: &User,
        channel: &Channel,
        _mode: ChannelModeFlag,
    ) -> PermissionResult {
        has_access(user, channel, ChannelAccessFlag::SetSimpleMode)
    }

    fn can_set_topic(&self, user: &User, channel: &Channel, _topic: &str) -> PermissionResult {
        if channel.mode().has_mode(ChannelModeFlag::TopicLock) {
            has_access(user, channel, ChannelAccessFlag::Topic)
        } else {
            Ok(())
        }
    }

    fn can_grant_permission(
        &self,
        user: &User,
        channel: &Channel,
        target: &User,
        flag: MembershipFlagFlag,
    ) -> PermissionResult {
        let required_permission = match (flag, user.id() == target.id()) {
            (MembershipFlagFlag::Op, true) => ChannelAccessFlag::OpSelf,
            (MembershipFlagFlag::Op, false) => ChannelAccessFlag::OpGrant,
            (MembershipFlagFlag::Voice, true) => ChannelAccessFlag::VoiceSelf,
            (MembershipFlagFlag::Voice, false) => ChannelAccessFlag::VoiceGrant,
        };
        has_access(user, channel, required_permission)
    }

    fn can_remove_permission(
        &self,
        user: &User,
        channel: &Channel,
        target: &User,
        flag: MembershipFlagFlag,
    ) -> PermissionResult {
        let required_permission = match (flag, user.id() == target.id()) {
            (MembershipFlagFlag::Op, true) => ChannelAccessFlag::OpSelf,
            (MembershipFlagFlag::Op, false) => ChannelAccessFlag::OpGrant,
            (MembershipFlagFlag::Voice, true) => ChannelAccessFlag::VoiceSelf,
            (MembershipFlagFlag::Voice, false) => ChannelAccessFlag::VoiceGrant,
        };
        has_access(user, channel, required_permission)
    }

    fn validate_ban_mask(
        &self,
        _mask: &str,
        _mode_type: ListModeType,
        _channel: &Channel,
    ) -> PermissionResult {
        Ok(())
    }

    fn can_set_ban(
        &self,
        user: &User,
        channel: &Channel,
        mode_type: ListModeType,
        _mask: &str,
    ) -> PermissionResult {
        let required_permission = match mode_type {
            ListModeType::Ban => ChannelAccessFlag::BanAdd,
            ListModeType::Quiet => ChannelAccessFlag::QuietAdd,
            ListModeType::Except => ChannelAccessFlag::ExemptAdd,
            ListModeType::Invex => ChannelAccessFlag::InvexAdd,
        };
        has_access(user, channel, required_permission)
    }

    fn can_unset_ban(
        &self,
        user: &User,
        channel: &Channel,
        mode_type: ListModeType,
        _mask: &str,
    ) -> PermissionResult {
        let required_permission = match mode_type {
            ListModeType::Ban => ChannelAccessFlag::BanRemoveAny,
            ListModeType::Quiet => ChannelAccessFlag::QuietRemoveAny,
            ListModeType::Except => ChannelAccessFlag::ExemptRemoveAny,
            ListModeType::Invex => ChannelAccessFlag::InvexRemoveAny,
        };
        has_access(user, channel, required_permission)
    }

    fn can_query_list(
        &self,
        user: &User,
        channel: &Channel,
        mode_type: ListModeType,
    ) -> PermissionResult {
        let required_permission = match mode_type {
            ListModeType::Ban => ChannelAccessFlag::BanView,
            ListModeType::Quiet => ChannelAccessFlag::QuietView,
            ListModeType::Except => ChannelAccessFlag::ExemptView,
            ListModeType::Invex => ChannelAccessFlag::InvexView,
        };
        has_access(user, channel, required_permission)
    }

    fn should_see_list_change(&self, member: &Membership, mode_type: ListModeType) -> bool {
        let required_permission = match mode_type {
            ListModeType::Ban => ChannelAccessFlag::BanView,
            ListModeType::Quiet => ChannelAccessFlag::QuietView,
            ListModeType::Except => ChannelAccessFlag::ExemptView,
            ListModeType::Invex => ChannelAccessFlag::InvexView,
        };

        let Ok(user) = member.user() else {
            return false;
        };
        let Ok(channel) = member.channel() else {
            return false;
        };

        has_access(&user, &channel, required_permission).is_ok()
            || member.permissions().is_set(MembershipFlagFlag::Op)
    }

    fn can_set_key(
        &self,
        user: &User,
        channel: &Channel,
        _new_key: Option<&ChannelKey>,
    ) -> PermissionResult {
        has_access(user, channel, ChannelAccessFlag::SetKey)
    }

    fn can_invite(&self, user: &User, channel: &Channel, _target: &User) -> PermissionResult {
        has_access(user, channel, ChannelAccessFlag::InviteOther)
    }
}
