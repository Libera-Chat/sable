use super::Network;
use crate::*;
use crate::event::*;
use crate::update::*;
use strum::IntoEnumIterator;

impl Network {
    fn channel_for_mode(&self, mode_id: ChannelModeId) -> Option<&state::Channel>
    {
        self.channels.values().filter(|c| c.mode == mode_id).next()
    }

    pub(super) fn new_channel(&mut self, target: ChannelId, _event: &Event, details: &details::NewChannel, _updates: &dyn NetworkUpdateReceiver)
    {
        if let Ok(_existing) = self.channel_by_name(&details.name)
        {
            // TODO: handle conflict
            panic!("Conflicting channel names");
        }
        let channel = state::Channel::new(target, &details.name, details.mode);
        self.channels.insert(channel.id, channel);
    }

    pub(super) fn new_channel_mode(&mut self, target: ChannelModeId, _event: &Event, details: &details::NewChannelMode, _updates: &dyn NetworkUpdateReceiver)
    {
        let cmode = state::ChannelMode::new(target, details.mode);
        self.channel_modes.insert(cmode.id, cmode);

        for list_type in ListModeType::iter()
        {
            let new_list = state::ListMode::new(ListModeId::new(target, list_type), list_type);
            self.channel_list_modes.insert(new_list.id, new_list);
        }
    }

    pub(super) fn channel_mode_change(&mut self, target: ChannelModeId, _event: &Event, details: &details::ChannelModeChange, updates: &dyn NetworkUpdateReceiver)
    {
        if let Some(cmode) = self.channel_modes.get_mut(&target)
        {
            cmode.modes |= details.added;
            cmode.modes &= !details.removed;
            match details.key_change
            {
                OptionChange::NoChange => (),
                OptionChange::Unset => cmode.key = None,
                OptionChange::Set(key) => cmode.key = Some(key)
            };
        }
        if let Some(channel) = self.channel_for_mode(target)
        {
            updates.notify(update::ChannelModeChange {
                channel: channel.id,
                mode: target,
                added: details.added,
                removed: details.removed,
                key_change: details.key_change,
                changed_by: details.changed_by,
            });
        }
    }

    fn translate_setter_info(&self, setter: ObjectId) -> String
    {
        match setter
        {
            ObjectId::User(user_id) =>
            {
                if let Ok(user) = self.user(user_id) {
                    format!("{}!{}@{}", user.nick(), user.user(), user.visible_host())
                } else {
                    String::from("<unknown>")
                }
            }
            ObjectId::Server(server_id) =>
            {
                if let Some(server) = self.servers.get(&server_id) {
                    server.name.to_string()
                } else {
                    String::from("<unknown>")
                }
            }
            _ => String::from("<unknown>")
        }
    }

    pub(super) fn new_channel_topic(&mut self, target: ChannelTopicId, event: &Event, details: &details::NewChannelTopic, updates: &dyn NetworkUpdateReceiver)
    {
        if let Some(existing) = self.channel_topics.values().filter(|t| t.channel == details.channel).next()
        {
            // This is a conflict - we can't have two topics for one channel. Keep the newer, drop the older.
            // As usual, use ID comparison as a tiebreaker if the timestamps are equal
            if existing.timestamp > event.timestamp || (existing.timestamp == event.timestamp && existing.id < target)
            {
                // The existing one is newer and wins. Do nothing.
                return;
            }
            // The new one wins. Drop the old before we process the new
            let existing_id = existing.id;
            self.channel_topics.remove(&existing_id);
        }

        // If there was an existing topic for this channel, there isn't any more. Carry on.

        let update = update::ChannelTopicChange{
            topic: target,
            new_text: details.text.clone(),
            setter: details.setter,
            timestamp: event.timestamp,
        };

        let setter_info = self.translate_setter_info(details.setter);

        let new_topic = state::ChannelTopic::new(
            target,
            details.channel,
            details.text.clone(),
            setter_info,
            event.timestamp
        );
        self.channel_topics.insert(target, new_topic);
        updates.notify(update);
    }

    pub(super) fn new_list_mode_entry(&mut self, target: ListModeEntryId, event: &Event, details: &details::NewListModeEntry, updates: &dyn NetworkUpdateReceiver)
    {
        let setter_info = self.translate_setter_info(details.setter.into());

        let entry = state::ListModeEntry::new(
            target,
            details.list,
            event.timestamp,
            setter_info,
            details.pattern.clone()
        );
        self.list_mode_entries.insert(entry.id, entry);

        if let Some(channel) = self.channel_for_mode(details.list.mode())
        {
            let update = update::ListModeAdded {
                channel: channel.id,
                list: details.list,
                list_type: details.list.list_type(),
                pattern: details.pattern.clone(),
                set_by: details.setter.into(),
            };
            updates.notify(update);
        }
    }

    pub(super) fn del_list_mode_entry(&mut self, target: ListModeEntryId, _event: &Event, details: &details::DelListModeEntry, updates: &dyn NetworkUpdateReceiver)
    {
        if let Some(removed) = self.list_mode_entries.remove(&target)
        {
            if let Some(list) = self.channel_list_modes.get(&removed.list)
            {
                if let Some(channel) = self.channel_for_mode(list.id.mode())
                {
                    let update = update::ListModeRemoved {
                        channel: channel.id,
                        list: removed.list,
                        list_type: list.list_type,
                        pattern: removed.pattern,
                        removed_by: details.removed_by.into(),
                    };
                    updates.notify(update);
                }
            }
        }
    }

    pub(super) fn channel_permission_change(&mut self, target: MembershipId, _event: &Event, details: &details::MembershipFlagChange, updates: &dyn NetworkUpdateReceiver)
    {
        if let Some(membership) = self.memberships.get_mut(&target)
        {
            membership.permissions |= details.added;
            membership.permissions &= !details.removed;

            updates.notify(update::MembershipFlagChange {
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
        self.memberships.insert(membership.id, membership);

        // If there was an invite for them, it's no longer needed
        self.channel_invites.remove(&InviteId::new(details.user, details.channel));

        let update = update::ChannelJoin {
            membership: target
        };
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

    pub(super) fn new_channel_invite(&mut self, target: InviteId, event: &Event, detail: &details::ChannelInvite, updates: &dyn NetworkUpdateReceiver)
    {
        let invite = state::ChannelInvite::new(target, detail.source, event.timestamp);
        self.channel_invites.insert(invite.id, invite);
        updates.notify(update::ChannelInvite { id: target, source: detail.source });
    }

    fn remove_channel(&mut self, id: ChannelId, _updates: &dyn NetworkUpdateReceiver)
    {
        if let Some(chan) = self.channels.remove(&id)
        {
            self.channel_modes.remove(&chan.mode);
            if let Some(topic) = self.channel_topics.values().filter(|t| t.channel == chan.id).next()
            {
                let topic_id = topic.id;
                self.channel_topics.remove(&topic_id);
            }
            if let Some(mode) = self.channel_modes.remove(&chan.mode)
            {
                for list_type in ListModeType::iter()
                {
                    let list_id = ListModeId::new(mode.id, list_type);
                    self.channel_list_modes.remove(&list_id);
                }
            }
        }
        self.channel_invites.retain(|i,_| i.channel() != id);
    }
}