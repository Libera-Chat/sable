use super::Network;
use crate::ircd::*;
use crate::ircd::event::*;

impl Network {
    pub(super) fn new_channel(&mut self, target: ChannelId, _event: &Event, details: &details::NewChannel) {
        let channel = state::Channel::new(target, &details.name, details.mode);
        self.channels.insert(channel.id, channel);
    }

    pub(super) fn new_channel_mode(&mut self, target: CModeId, _event: &Event, details: &details::NewChannelMode) {
        let cmode = state::ChannelMode::new(target, details.mode);
        self.channel_modes.insert(cmode.id, cmode);
    }

    pub(super) fn channel_mode_change(&mut self, target: CModeId, _event: &Event, details: &details::ChannelModeChange) {
        if let Some(cmode) = self.channel_modes.get_mut(&target)
        {
            cmode.modes |= details.added;
            cmode.modes &= !details.removed;
        }
    }

    pub(super) fn channel_permission_change(&mut self, target: MembershipId, _event: &Event, details: &details::ChannelPermissionChange) {
        if let Some(membership) = self.memberships.get_mut(&target)
        {
            membership.permissions |= details.added;
            membership.permissions &= !details.removed;
        }
    }

    pub(super) fn user_joined_channel(&mut self, target: MembershipId, _event: &Event, details: &details::ChannelJoin) {
        let membership = state::Membership::new(target, details.user, details.channel, details.permissions);
        self.memberships.insert(membership.id, membership);
    }

    pub(super) fn user_left_channel(&mut self, target: MembershipId, _event: &Event, _details: &details::ChannelPart) {
        self.memberships.remove(&target);
    }
}