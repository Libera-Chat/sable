use super::*;
use crate::TaggableMessage;
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
        let user = self.net.user(detail.user.user.id)?;
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

            connection.send(&message::Mode::new(&user, &user, &user.mode().format()));

            connection.send(&message::Notice::new(&self.name.to_string(), &user,
                    "The network is currently running in debug mode. Do not send any sensitive information such as passwords."));
        }
        Ok(())
    }

    fn handle_nick_change(&mut self, detail: &update::UserNickChange) -> HandleResult
    {
        // This fires after the nick change is applied to the network state, so we
        // have to construct the n!u@h string explicitly
        let source = self.net.user(detail.user.id)?;
        let source_string = format!("{}!{}@{}", detail.old_nick, source.user(), source.visible_host());
        let message = message::Nick::new(&source_string, &detail.new_nick);
        let mut notified = HashSet::new();

        if let Ok(conn) = self.connections.get_user(source.id())
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
        if let Ok(conn) = self.connections.get_user(details.user.user.id)
        {
            conn.send(&message::Mode::new(&details.changed_by,
                                          &details.user,
                                          &utils::format_umode_changes(&details.added, &details.removed)));
        }
        Ok(())
    }

    fn handle_user_quit(&mut self, detail: &update::UserQuit) -> HandleResult
    {
        if let Some(conn) = self.connections.remove_user(detail.user.user.id)
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
                conn.send(&message::Quit::new(&detail.user, &detail.message));
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
        let (mut changes, params) = utils::format_cmode_changes(detail);
        for p in params
        {
            changes.push(' ');
            changes.push_str(&p);
        }

        let msg = message::Mode::new(&detail.changed_by, &detail.channel, &changes);

        self.send_to_channel_members(&wrapper::Channel::wrap(&self.net, &detail.channel), msg);

        Ok(())
    }

    fn handle_list_mode_added(&self, detail: &update::ListModeAdded) -> HandleResult
    {
        let chan = self.net.channel(detail.channel.id)?;
        let mode_char = detail.list_type.mode_letter();

        let changes = format!("+{} {}", mode_char, detail.pattern);
        let msg = message::Mode::new(&detail.set_by, &chan, &changes);

        self.send_to_channel_members_where(&chan, msg,
                |m| self.policy().should_see_list_change(m, detail.list_type)
        );

        Ok(())
    }

    fn handle_list_mode_removed(&self, detail: &update::ListModeRemoved) -> HandleResult
    {
        let chan = self.net.channel(detail.channel.id)?;
        let mode_char = detail.list_type.mode_letter();

        let changes = format!("-{} {}", mode_char, detail.pattern);
        let msg = message::Mode::new(&detail.removed_by, &chan, &changes);

        self.send_to_channel_members_where(&chan, msg,
            |m| self.policy().should_see_list_change(m, detail.list_type)
        );

        Ok(())
    }

    fn handle_channel_topic(&self, detail: &update::ChannelTopicChange) -> HandleResult
    {
        let chan = self.net.channel(detail.channel.id)?;

        let msg = message::Topic::new(&detail.setter, &chan, &detail.new_text);

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
        let (mut changes, args) = utils::format_channel_perm_changes(&detail.user.nickname, &detail.added, &detail.removed);

        changes += " ";
        changes += &args.join(" ");

        let msg = message::Mode::new(&detail.changed_by, &detail.channel, &changes);
        let chan = self.net.channel(detail.channel.id)?;

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
        let membership = self.net.membership(detail.membership.id)?;
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
        let message = message::Part::new(&source, &detail.channel.name, &detail.message);

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
        let msg = message::Invite::new(&detail.source, &detail.user, &detail.channel.name);
        self.send_to_user_id_if_local(detail.user.user.id, msg);
        Ok(())
    }

    fn handle_channel_rename(&self, detail: &update::ChannelRename) -> HandleResult
    {
        let channel = self.net.channel(detail.channel.id)?;

        for member in channel.members()
        {
            let user = member.user()?;
            if self.connections.get_user(user.id()).is_ok()
            {
                let part_message = message::Part::new(&user, &detail.old_name, "Channel name changing");
                let join_message = message::Join::new(&user, &channel);
                self.send_to_user_if_local(&user, part_message);
                self.send_to_user_if_local(&user, join_message);
                self.notify_joining_user(&member).ok();
            }
        }

        Ok(())
    }

    fn handle_new_message(&self, detail: &update::NewMessage) -> HandleResult
    {
        let message_send = message::Message::new(&detail.source,
                                                 &detail.target,
                                                 detail.message.message_type,
                                                 &detail.message.text)
                                        .with_tag(capability::server_time::server_time_tag(detail.message.ts));

        match &detail.target {
            update::HistoricMessageTarget::Channel(channel) => {
                self.send_to_channel_members_where(&wrapper::Channel::wrap(&self.net, &channel), message_send,
                                                    |m| if let update::HistoricMessageSource::User(u) = &detail.source { u.user.id != m.user_id() } else { true });
            },
            update::HistoricMessageTarget::User(user) => {
                self.send_to_user_if_local(&wrapper::User::wrap(&self.net, &user.user), message_send);
            },
            update::HistoricMessageTarget::Unknown => ()
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn handle_new_server(&self, detail: &update::NewServer) -> HandleResult
    {
        tracing::trace!("Got new server");

        let server = self.net.server(detail.server.id)?;

        self.event_log.enable_server(*server.name(), server.id());

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn handle_server_quit(&self, detail: &update::ServerQuit) -> HandleResult
    {
        tracing::trace!("Got server quit");

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
        let entry = self.net.audit_entry(detail.entry.id)?;
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