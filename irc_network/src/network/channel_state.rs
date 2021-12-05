use super::Network;
use crate::*;
use crate::event::*;
use crate::update::*;

impl Network {
    pub(super) fn new_channel(&mut self, target: ChannelId, _event: &Event, details: &details::NewChannel, _updates: &dyn NetworkUpdateReceiver)
    {
        let channel = state::Channel::new(target, &details.name, details.mode);
        self.channels.insert(channel.id, channel);
    }

    pub(super) fn new_channel_mode(&mut self, target: CModeId, _event: &Event, details: &details::NewChannelMode, _updates: &dyn NetworkUpdateReceiver)
    {
        let cmode = state::ChannelMode::new(target, details.mode);
        self.channel_modes.insert(cmode.id, cmode);
    }

    pub(super) fn channel_mode_change(&mut self, target: CModeId, _event: &Event, details: &details::ChannelModeChange, updates: &dyn NetworkUpdateReceiver)
    {
        if let Some(cmode) = self.channel_modes.get_mut(&target)
        {
            cmode.modes |= details.added;
            cmode.modes &= !details.removed;
        }
        let mut channel = self.channels.values().filter(|c| c.mode == target);
        if let Some(channel) = channel.next()
        {
            updates.notify(update::ChannelModeChange {
                channel: channel.id,
                mode: target,
                added: details.added,
                removed: details.removed,
                changed_by: details.changed_by,
            });
        }
    }

    pub(super) fn channel_permission_change(&mut self, target: MembershipId, _event: &Event, details: &details::ChannelPermissionChange, updates: &dyn NetworkUpdateReceiver)
    {
        if let Some(membership) = self.memberships.get_mut(&target)
        {
            membership.permissions |= details.added;
            membership.permissions &= !details.removed;

            updates.notify(update::ChannelPermissionChange {
                membership: target,
                added: details.added,
                removed: details.removed,
                changed_by: details.changed_by,
            });
        }
    }

    pub(super) fn user_joined_channel(&mut self, target: MembershipId, _event: &Event, details: &details::ChannelJoin, updates: &dyn NetworkUpdateReceiver)
    {
        let membership = state::Membership::new(target, details.user, details.channel, details.permissions);
        let update = update::ChannelJoin {
            membership: target
        };

        self.memberships.insert(membership.id, membership);
        updates.notify(update);
    }

    pub(super) fn user_left_channel(&mut self, target: MembershipId, _event: &Event, details: &details::ChannelPart, updates: &dyn NetworkUpdateReceiver)
    {
        if let Some(removed_membership) = self.memberships.remove(&target)
        {
            let empty = self.memberships.iter().filter(|(_,v)| v.channel == removed_membership.channel).next().is_none();
            if empty
            {
                self.remove_channel(removed_membership.channel, updates);
            }

            let update = update::ChannelPart {
                membership: removed_membership,
                message: details.message.clone()
            };
            updates.notify(update);
        }
    }

    fn remove_channel(&mut self, id: ChannelId, _updates: &dyn NetworkUpdateReceiver)
    {
/*
        if let Some(removed_channel) = self.channels.remove(&id)
        {
            let update = ::ChannelDeleted(removed_channel);
            updates.notify(update);
        }
*/
        self.channels.remove(&id);
    }
}