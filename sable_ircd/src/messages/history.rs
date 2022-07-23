use sable_network::prelude::*;
use sable_network::utils::*;
use crate::messages::MessageSink;
use crate::capability::CapableMessage;
use crate::capability::ClientCapability;
use crate::capability::WithSupportedTags;
use crate::errors::HandleResult;

use super::message;

/// Extension trait to translate a network history entry into client protocol messages
pub(crate) trait SendHistoryItem
{
    fn send_to(&self, conn: &impl MessageSink, from_entry: &HistoryLogEntry) -> HandleResult;
}

impl SendHistoryItem for HistoryLogEntry
{
    fn send_to(&self, conn: &impl MessageSink, _from_entry: &HistoryLogEntry) -> HandleResult
    {
        match &self.details
        {
            NetworkStateChange::NewUser(detail) => detail.send_to(conn, self),
            NetworkStateChange::UserNickChange(detail) => detail.send_to(conn, self),
            NetworkStateChange::UserModeChange(detail) => detail.send_to(conn, self),
            NetworkStateChange::UserQuit(detail) => detail.send_to(conn, self),
            NetworkStateChange::BulkUserQuit(detail) => detail.send_to(conn, self),
            NetworkStateChange::ChannelModeChange(detail) => detail.send_to(conn, self),
            NetworkStateChange::ChannelTopicChange(detail) => detail.send_to(conn, self),
            NetworkStateChange::ListModeAdded(detail) => detail.send_to(conn, self),
            NetworkStateChange::ListModeRemoved(detail) => detail.send_to(conn, self),
            NetworkStateChange::MembershipFlagChange(detail) => detail.send_to(conn, self),
            NetworkStateChange::ChannelJoin(detail) => detail.send_to(conn, self),
            NetworkStateChange::ChannelPart(detail) => detail.send_to(conn, self),
            NetworkStateChange::ChannelInvite(detail) => detail.send_to(conn, self),
            NetworkStateChange::ChannelRename(detail) => detail.send_to(conn, self),
            NetworkStateChange::NewMessage(detail) => detail.send_to(conn, self),
            NetworkStateChange::NewServer(detail) => detail.send_to(conn, self),
            NetworkStateChange::ServerQuit(detail) => detail.send_to(conn, self),
            NetworkStateChange::NewAuditLogEntry(detail) => detail.send_to(conn, self),
        }
    }
}

impl SendHistoryItem for update::NewUser
{
    fn send_to(&self, _conn: &impl MessageSink, _from_entry: &HistoryLogEntry) -> HandleResult
    {
        Ok(())
    }
}

impl SendHistoryItem for update::UserNickChange
{
    fn send_to(&self, conn: &impl MessageSink, from_entry: &HistoryLogEntry) -> HandleResult
    {
        let source_str = format!("{}!{}@{}", self.old_nick, self.user.user, self.user.visible_host);
        let message = message::Nick::new(&source_str, &self.new_nick)
                                    .with_tags_from(from_entry);

        conn.send(&message);

        Ok(())
    }
}

impl SendHistoryItem for update::UserModeChange
{
    fn send_to(&self, conn: &impl MessageSink, from_entry: &HistoryLogEntry) -> HandleResult
    {
        let message = message::Mode::new(&self.user, &self.user, &format_umode_changes(&self.added, &self.removed))
                                    .with_tags_from(from_entry);


        conn.send(&message);

        Ok(())
    }
}

impl SendHistoryItem for update::UserQuit
{
    fn send_to(&self, conn: &impl MessageSink, from_entry: &HistoryLogEntry) -> HandleResult
    {
        let message = message::Quit::new(&self.user, &self.message)
                                    .with_tags_from(from_entry);

        conn.send(&message);

        Ok(())
    }
}

impl SendHistoryItem for update::BulkUserQuit
{
    fn send_to(&self, _conn: &impl MessageSink, _from_entry: &HistoryLogEntry) -> HandleResult
    {
        Ok(())
    }
}

impl SendHistoryItem for update::ChannelModeChange
{
    fn send_to(&self, conn: &impl MessageSink, from_entry: &HistoryLogEntry) -> HandleResult
    {
        let (mut changes, params) = format_cmode_changes(self);
        for p in params
        {
            changes.push(' ');
            changes.push_str(&p);
        }

        let message = message::Mode::new(&self.changed_by, &self.channel, &changes)
                                    .with_tags_from(from_entry);

        conn.send(&message);

        Ok(())
    }
}

impl SendHistoryItem for update::ChannelTopicChange
{
    fn send_to(&self, conn: &impl MessageSink, from_entry: &HistoryLogEntry) -> HandleResult
    {
        let message = message::Topic::new(&self.setter, &self.channel.name, &self.new_text)
                                    .with_tags_from(from_entry);

        conn.send(&message);

        Ok(())
    }
}

impl SendHistoryItem for update::ListModeAdded
{
    fn send_to(&self, conn: &impl MessageSink, from_entry: &HistoryLogEntry) -> HandleResult
    {
        let text = format!("+{} {}", self.list_type.mode_letter(), self.pattern);
        let message = message::Mode::new(&self.set_by, &self.channel, &text)
                                    .with_tags_from(from_entry);
        conn.send(&message);
        Ok(())
    }
}

impl SendHistoryItem for update::ListModeRemoved
{
    fn send_to(&self, conn: &impl MessageSink, from_entry: &HistoryLogEntry) -> HandleResult
    {
        let text = format!("-{} {}", self.list_type.mode_letter(), self.pattern);
        let message = message::Mode::new(&self.removed_by, &self.channel, &text)
                                    .with_tags_from(from_entry);
        conn.send(&message);
        Ok(())
    }
}

impl SendHistoryItem for update::MembershipFlagChange
{
    fn send_to(&self, conn: &impl MessageSink, from_entry: &HistoryLogEntry) -> HandleResult
    {
        let (mut changes, args) = format_channel_perm_changes(&self.user.nickname, &self.added, &self.removed);

        changes += " ";
        changes += &args.join(" ");

        let message = message::Mode::new(&self.changed_by, &self.channel, &changes)
                                    .with_tags_from(from_entry);

        conn.send(&message);

        Ok(())
    }
}

impl SendHistoryItem for update::ChannelJoin
{
    fn send_to(&self, conn: &impl MessageSink, from_entry: &HistoryLogEntry) -> HandleResult
    {
        let message = message::Join::new(&self.user, &self.channel.name)
                                    .with_tags_from(from_entry);

        conn.send(&message);

        Ok(())
    }
}

impl SendHistoryItem for update::ChannelPart
{
    fn send_to(&self, conn: &impl MessageSink, from_entry: &HistoryLogEntry) -> HandleResult
    {
        let message = message::Part::new(&self.user, &self.channel.name, &self.message)
                                    .with_tags_from(from_entry);

        conn.send(&message);

        Ok(())
    }
}

impl SendHistoryItem for update::ChannelInvite
{
    fn send_to(&self, conn: &impl MessageSink, from_entry: &HistoryLogEntry) -> HandleResult
    {
        let message = message::Invite::new(&self.source, &self.user, &self.channel.name)
                                    .with_tags_from(from_entry);

        conn.send(&message);

        Ok(())

    }
}

impl SendHistoryItem for update::ChannelRename
{
    fn send_to(&self, _conn: &impl MessageSink, _from_entry: &HistoryLogEntry) -> HandleResult
    {
        todo!();
    }
}

impl SendHistoryItem for update::NewMessage
{
    fn send_to(&self, conn: &impl MessageSink, from_entry: &HistoryLogEntry) -> HandleResult
    {
        let message = message::Message::new(&self.source, &self.target, self.message.message_type, &self.message.text)
                                    .with_tags_from(from_entry);

        // Users should only see their own messages echoed if they've asked for it
        match &self.source
        {
            update::HistoricMessageSource::User(user) =>
            {
                if conn.user_id() == Some(user.user.id)
                {
                    conn.send(&message.with_required_capability(ClientCapability::EchoMessage));
                }
                else
                {
                    conn.send(&message);
                }
            }
            _ => conn.send(&message)
        }

        Ok(())
    }
}

impl SendHistoryItem for update::NewServer
{
    fn send_to(&self, _conn: &impl MessageSink, _from_entry: &HistoryLogEntry) -> HandleResult
    {
        Ok(())
    }
}

impl SendHistoryItem for update::ServerQuit
{
    fn send_to(&self, _conn: &impl MessageSink, _from_entry: &HistoryLogEntry) -> HandleResult
    {
        Ok(())
    }
}

impl SendHistoryItem for update::NewAuditLogEntry
{
    fn send_to(&self, _conn: &impl MessageSink, _from_entry: &HistoryLogEntry) -> HandleResult
    {
        todo!();
    }
}
