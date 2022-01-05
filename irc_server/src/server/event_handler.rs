use super::*;
use irc_network::update;
use irc_network::NetworkStateChange;
use irc_network::wrapper::ObjectWrapper;
use crate::errors::*;
use std::collections::HashSet;

impl Server
{
    pub(super) fn handle_network_update(&mut self, change: NetworkStateChange)
    {
        use NetworkStateChange::*;

        let res = match &change {
            NewUser(details) => self.handle_new_user(details),
            UserNickChange(details) => self.handle_nick_change(details),
            UserModeChange(details) => self.handle_umode_change(details),
            UserQuit(details) => self.handle_user_quit(details),
            BulkUserQuit(details) => self.handle_bulk_quit(details),
            ChannelModeChange(details) => self.handle_channel_mode_change(details),
            ListModeAdded(details) => self.handle_list_mode_added(details),
            ListModeRemoved(details) => self.handle_list_mode_removed(details),
            ChannelTopicChange(details) => self.handle_channel_topic(details),
            ChannelJoin(details) => self.handle_join(details),
            ChannelPart(details) => self.handle_part(details),
            MembershipFlagChange(details) => self.handle_chan_perm_change(details),
            NewMessage(details) => self.handle_new_message(details),
            ServerQuit(details) => self.handle_server_quit(details),
        };
        if let Err(e) = res
        {
            log::error!("Error handling network state update {:?}: {}", change, e);
        }
    }

    fn handle_new_user(&mut self, detail: &update::NewUser) -> HandleResult
    {
        let user = self.net.user(detail.user)?;
        if let Ok(connection) = self.connections.get_user_mut(user.id())
        {
            connection.pre_client = None;
            connection.user_id = Some(user.id());

            connection.send(&numeric::Numeric001::new_for(&self.name.to_string(), &user.nick(), "test", &user.nick()));
            for line in self.isupport.data().iter()
            {
                connection.send(&numeric::ISupport::new_for(&self.name.to_string(), &user.nick(), line));
            }
        }
        Ok(())
    }

    fn handle_nick_change(&mut self, detail: &update::UserNickChange) -> HandleResult
    {
        // This fires after the nick change is applied to the network state, so we 
        // have to construct the n!u@h string explicitly
        let source = self.net.user(detail.user)?;
        let source_string = format!("{}!{}@{}", detail.old_nick, source.user(), source.visible_host());
        let message = message::Nick::new(&source_string, &detail.new_nick);
        let mut notified = HashSet::new();

        if let Ok(conn) = self.connections.get_user(detail.user)
        {
            notified.insert(conn.id());
            conn.send(&message);
        }

        for membership in source.channels()
        {
            let chan = membership.channel()?;
            for m2 in chan.members()
            {
                if let Ok(conn) = self.connections.get_user(m2.user_id())
                {
                    if ! notified.contains(&conn.id())
                    {
                        notified.insert(conn.id());
                        conn.send(&message);
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_umode_change(&mut self, details: &update::UserModeChange) -> HandleResult
    {
        let user = self.net.user(details.user_id)?;
        let source = self.lookup_message_source(details.changed_by)?;

        if let Ok(conn) = self.connections.get_user(user.id())
        {
            conn.send(&message::Mode::new(source.as_ref(),
                                          &user,
                                          &utils::format_umode_changes(&details.added, &details.removed)));
        }
        Ok(())
    }

    fn handle_user_quit(&self, detail: &update::UserQuit) -> HandleResult
    {
        let mut to_notify = HashSet::new();

        for m1 in &detail.memberships
        {
            let m1: wrapper::Membership = ObjectWrapper::wrap(&self.net, m1);
            for m2 in m1.channel()?.members()
            {
                to_notify.insert(m2.user_id());
            }
        }

        for u in to_notify
        {
            if let Ok(conn) = self.connections.get_user(u)
            {
                let nuh = format!("{}!{}@{}", detail.nickname, detail.user.user, detail.user.visible_host);
                conn.send(&message::Quit::new(&nuh, &detail.message));
            }
        }
        Ok(())
    }

    fn handle_bulk_quit(&self, detail: &update::BulkUserQuit) -> HandleResult
    {
        for item in &detail.items
        {
            self.handle_user_quit(&item)?;
        }
        Ok(())
    }

    fn handle_channel_mode_change(&self, detail: &update::ChannelModeChange) -> HandleResult
    {
        let mode = self.net.channel_mode(detail.mode)?;
        let chan = mode.channel()?;
        let source = self.lookup_message_source(detail.changed_by)?;

        let (mut changes, params) = utils::format_cmode_changes(&detail);
        for p in params
        {
            changes.push(' ');
            changes.push_str(&p);
        }
        let msg = message::Mode::new(source.as_ref(), &chan, &changes);

        self.send_to_channel_members(&chan, msg);

        Ok(())
    }

    fn handle_list_mode_added(&self, detail: &update::ListModeAdded) -> HandleResult
    {
        let chan = self.net.channel(detail.channel)?;
        let mode_char = detail.list_type.mode_letter();
        let source = self.lookup_message_source(detail.set_by)?;

        let changes = format!("+{} {}", mode_char, detail.pattern);
        let msg = message::Mode::new(source.as_ref(), &chan, &changes);

        self.send_to_channel_members_where(&chan, msg,
                |m| self.policy().should_see_list_change(&m, detail.list_type)
        );

        Ok(())
    }

    fn handle_list_mode_removed(&self, detail: &update::ListModeRemoved) -> HandleResult
    {
        let chan = self.net.channel(detail.channel)?;
        let mode_char = detail.list_type.mode_letter();
        let source = self.lookup_message_source(detail.removed_by)?;

        let changes = format!("-{} {}", mode_char, detail.pattern);
        let msg = message::Mode::new(source.as_ref(), &chan, &changes);

        self.send_to_channel_members_where(&chan, msg,
            |m| self.policy().should_see_list_change(&m, detail.list_type)
        );

        Ok(())
    }

    fn handle_channel_topic(&self, detail: &update::ChannelTopicChange) -> HandleResult
    {
        let topic = self.net.channel_topic(detail.topic)?;
        let chan = topic.channel()?;

        let source = self.lookup_message_source(detail.setter)?;
        let msg = message::Topic::new(source.as_ref(), &chan, &detail.new_text);

        for m in chan.members()
        {
            let member = m.user()?;
            if let Ok(conn) = self.connections.get_user(member.id()) {
                conn.send(&msg);
            }
        }
        Ok(())
    }

    fn handle_chan_perm_change(&self, detail: &update::MembershipFlagChange) -> HandleResult
    {
        let membership = self.net.membership(detail.membership)?;
        let chan = membership.channel()?;
        let target = membership.user()?;
        let source = self.lookup_message_source(detail.changed_by)?;

        let (mut changes, args) = utils::format_channel_perm_changes(&target, &detail.added, &detail.removed);

        changes += " ";
        changes += &args.join(" ");

        let msg = message::Mode::new(source.as_ref(), &chan, &changes);

        for m in chan.members()
        {
            let member = m.user()?;
            if let Ok(conn) = self.connections.get_user(member.id()) {
                conn.send(&msg);
            }
        }
        Ok(())
    }

    fn handle_join(&self, detail: &update::ChannelJoin) -> HandleResult
    {
        let membership = self.net.membership(detail.membership)?;
        let user = membership.user()?;
        let channel = membership.channel()?;

        for m in channel.members()
        {
            let member = m.user()?;
            if let Ok(conn) = self.connections.get_user(member.id()) {
                conn.send(&message::Join::new(&user, &channel));
            }
        }

        if let Ok(conn) = self.connections.get_user(user.id()) {
            if ! membership.permissions().is_empty()
            {
                let (mut changes, args) = utils::format_channel_perm_changes(&user, &membership.permissions(), &MembershipFlagSet::new());

                changes += " ";
                changes += &args.join(" ");

                let msg = message::Mode::new(self, &channel, &changes);
                conn.send(&msg);
            }

            if let Ok(topic) = self.net.topic_for_channel(channel.id())
            {
                conn.send(&numeric::TopicIs::new(&channel, topic.text())
                          .format_for(self, &user));
                conn.send(&numeric::TopicSetBy::new(&channel, topic.setter(), topic.timestamp())
                          .format_for(self, &user));
            }

            crate::utils::send_channel_names(self, conn, &channel)?;
        }

        Ok(())
    }

    fn handle_part(&self, detail: &update::ChannelPart) -> HandleResult
    {
        let source = self.net.user(detail.membership.user)?;
        let channel = self.net.channel(detail.membership.channel)?;
        let message = message::Part::new(&source, &channel, &detail.message);

        // This gets called after the part is applied to the network state,
        // so the user themselves needs to be notified separately
        if let Ok(conn) = self.connections.get_user(source.id())
        {
            conn.send(&message);
        }

        for m in channel.members()
        {
            if let Ok(conn) = self.connections.get_user(m.user()?.id())
            {
                conn.send(&message);
            }
        }
        Ok(())
    }

    fn handle_new_message(&self, detail: &update::NewMessage) -> HandleResult
    {
        let message = self.net.message(detail.message)?;
        let source = message.source()?;

        match message.target()? {
            wrapper::MessageTarget::Channel(channel) => {
                for m in channel.members() {
                    let member = m.user()?;
                    if member.id() == source.id() {
                        continue;
                    }
                    if let Ok(conn) = self.connections.get_user(member.id()) {
                        conn.send(&message::Privmsg::new(&source, &channel, message.text()));
                    }
                }
                Ok(())
            },
            wrapper::MessageTarget::User(user) => {
                if let Ok(conn) = self.connections.get_user(user.id()) {
                    conn.send(&message::Privmsg::new(&source, &user, message.text()));

                }
                Ok(())
            },
        }
    }

    fn handle_server_quit(&self, detail: &update::ServerQuit) -> HandleResult
    {
        if detail.server.id == self.my_id
        {
            // The network thinks we're no longer alive. Shut down to avoid desyncs
            panic!("Network thinks we're dead. Making it so");
        }
        Ok(())
    }
}