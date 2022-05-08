use super::*;
use irc_network::update;
use irc_network::NetworkStateChange;
use irc_network::wrapper::ObjectWrapper;
use crate::errors::*;
use std::collections::HashSet;

impl Server
{
    #[tracing::instrument(skip(self))]
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
            ChannelInvite(details) => self.handle_invite(details),
            ChannelRename(details) => self.handle_channel_rename(details),
            MembershipFlagChange(details) => self.handle_chan_perm_change(details),
            NewMessage(details) => self.handle_new_message(details),
            NewServer(details) => self.handle_new_server(details),
            ServerQuit(details) => self.handle_server_quit(details),
            NewAuditLogEntry(details) => self.report_audit_entry(details),
        };
        if let Err(e) = res
        {
            tracing::error!("Error handling network state update {:?}: {}", change, e);
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
            connection.send(&numeric::Numeric002::new_for(&self.name.to_string(), &user.nick(), &self.name, &self.version));
            for line in self.isupport.data().iter()
            {
                connection.send(&numeric::ISupport::new_for(&self.name.to_string(), &user.nick(), line));
            }

            connection.send(&message::Mode::new(&user, &user, &user.mode()?.format()));

            connection.send(&message::Notice::new(&self.name.to_string(), &user,
                    "The network is currently running in debug mode. Do not send any sensitive information such as passwords."));
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

    fn handle_user_quit(&mut self, detail: &update::UserQuit) -> HandleResult
    {
        if let Some(conn) = self.connections.remove_user(detail.user.id)
        {
            conn.send(&message::Error::new(&format!("Closing link: {}", detail.message)));
        }

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

    fn handle_bulk_quit(&mut self, detail: &update::BulkUserQuit) -> HandleResult
    {
        for item in &detail.items
        {
            self.handle_user_quit(item)?;
        }
        Ok(())
    }

    fn handle_channel_mode_change(&self, detail: &update::ChannelModeChange) -> HandleResult
    {
        let mode = self.net.channel_mode(detail.mode)?;
        let chan = mode.channel()?;
        let source = self.lookup_message_source(detail.changed_by)?;

        let (mut changes, params) = utils::format_cmode_changes(detail);
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
                |m| self.policy().should_see_list_change(m, detail.list_type)
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
            |m| self.policy().should_see_list_change(m, detail.list_type)
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

        self.notify_joining_user(&membership)?;

        Ok(())
    }

    fn handle_part(&self, detail: &update::ChannelPart) -> HandleResult
    {
        let source = self.net.user(detail.membership.user)?;
        let message = message::Part::new(&source, &detail.channel_name, &detail.message);

        // This gets called after the part is applied to the network state,
        // so the user themselves needs to be notified separately.
        //
        // Also, if this was the last user to leave, then the channel no longer
        // exists, so we need to do this before trying to look it up
        if let Ok(conn) = self.connections.get_user(source.id())
        {
            conn.send(&message);
        }

        let channel = self.net.channel(detail.membership.channel)?;

        for m in channel.members()
        {
            if let Ok(conn) = self.connections.get_user(m.user()?.id())
            {
                conn.send(&message);
            }
        }
        Ok(())
    }

    fn handle_invite(&self, detail: &update::ChannelInvite) -> HandleResult
    {
        let target = self.net.user(detail.id.user())?;
        let chan = self.net.channel(detail.id.channel())?;
        let source = self.net.user(detail.source)?;

        let msg = message::Invite::new(&source, &target, &chan);
        self.send_to_user_if_local(&target, msg);
        Ok(())
    }

    fn handle_channel_rename(&self, detail: &update::ChannelRename) -> HandleResult
    {
        let channel = self.net.channel(detail.id)?;

        for member in channel.members()
        {
            let user = member.user()?;
            if self.connections.get_user(user.id()).is_ok()
            {
                let part_message = message::Part::new(&user, channel.name(), "Channel name changing");
                self.send_to_user_if_local(&user, part_message);
                self.notify_joining_user(&member).ok();
            }
        }

        Ok(())
    }

    fn handle_new_message(&self, detail: &update::NewMessage) -> HandleResult
    {
        let message = self.net.message(detail.message)?;
        let source = message.source()?;

        let message_send = message::Message::new(&source, &message.target()?, message.message_type(), message.text());

        match message.target()? {
            wrapper::MessageTarget::Channel(channel) => {
                self.send_to_channel_members_where(&channel, message_send, |m| m.user_id() != source.id());
            },
            wrapper::MessageTarget::User(user) => {
                self.send_to_user_if_local(&user, message_send);
            },
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn handle_new_server(&self, detail: &update::NewServer) -> HandleResult
    {
        tracing::info!("Got new server");

        let server = self.net.server(detail.id)?;

        self.event_log.enable_server(*server.name(), server.id());

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn handle_server_quit(&self, detail: &update::ServerQuit) -> HandleResult
    {
        tracing::info!("Got server quit");

        if detail.server.id == self.my_id && detail.server.epoch == self.epoch
        {
            // The network thinks we're no longer alive. Shut down to avoid desyncs
            panic!("Network thinks we're dead. Making it so");
        }

        self.event_log.disable_server(detail.server.name, detail.server.id, detail.server.epoch);

        Ok(())
    }

    fn report_audit_entry(&self, detail: &update::NewAuditLogEntry) -> HandleResult
    {
        let entry = self.net.audit_entry(detail.id)?;
        let text = serde_json::to_string(&entry).unwrap_or_else(|e| format!("ERROR: failed to serialize audit event: {}", e));

        for conn in self.connections.iter()
        {
            if let Some(user_id) = conn.user_id
            {
                if let Ok(user) = self.net.user(user_id)
                {
                    if user.is_oper()
                    {
                        let msg = message::Notice::new(&self, &user, &format!("Network event: {}", &text));

                        conn.send(&msg);
                    }
                }
            }
        }

        Ok(())
    }
}