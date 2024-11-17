use crate::capability::ClientCapability;
use crate::capability::WithSupportedTags;
use crate::errors::HandleResult;
use crate::messages::MessageSink;
use crate::prelude::numeric;
use crate::server::ClientServer;
use sable_network::prelude::*;
use sable_network::rpc::NetworkHistoryUpdate;
use sable_network::utils::*;

use super::message;

/// Extension trait to translate a network history entry into client protocol messages
pub(crate) trait SendHistoryItem<Item> {
    fn send_item(
        &self,
        item: &Item,
        conn: impl MessageSink,
        from_entry: &impl HistoryItem,
    ) -> HandleResult;
}

impl SendHistoryItem<NetworkHistoryUpdate> for ClientServer {
    fn send_item(
        &self,
        item: &NetworkHistoryUpdate,
        conn: impl MessageSink,
        _from_entry: &impl HistoryItem,
    ) -> HandleResult {
        match &item.change {
            NetworkStateChange::UserAwayChange(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::UserNickChange(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::UserModeChange(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::UserQuit(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ChannelModeChange(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ChannelTopicChange(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ListModeAdded(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ListModeRemoved(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::MembershipFlagChange(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ChannelJoin(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ChannelKick(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ChannelPart(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ChannelInvite(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ChannelRename(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::NewMessage(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::NewUser(_)
            | NetworkStateChange::NewUserConnection(_)
            | NetworkStateChange::UserConnectionDisconnected(_)
            | NetworkStateChange::NewServer(_)
            | NetworkStateChange::ServerQuit(_)
            | NetworkStateChange::NewAuditLogEntry(_)
            | NetworkStateChange::UserLoginChange(_)
            | NetworkStateChange::HistoryServerUpdate(_)
            | NetworkStateChange::ServicesUpdate(_)
            | NetworkStateChange::EventComplete(_) => Ok(()),
        }
    }
}

impl SendHistoryItem<HistoryLogEntry> for ClientServer {
    fn send_item(
        &self,
        item: &HistoryLogEntry,
        conn: impl MessageSink,
        _from_entry: &impl HistoryItem,
    ) -> HandleResult {
        match &item.details {
            NetworkStateChange::UserAwayChange(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::UserNickChange(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::UserModeChange(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::UserQuit(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ChannelModeChange(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ChannelTopicChange(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ListModeAdded(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ListModeRemoved(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::MembershipFlagChange(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ChannelJoin(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ChannelKick(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ChannelPart(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ChannelInvite(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::ChannelRename(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::NewMessage(detail) => self.send_item(detail, conn, item),
            NetworkStateChange::NewUser(_)
            | NetworkStateChange::NewUserConnection(_)
            | NetworkStateChange::UserConnectionDisconnected(_)
            | NetworkStateChange::NewServer(_)
            | NetworkStateChange::ServerQuit(_)
            | NetworkStateChange::NewAuditLogEntry(_)
            | NetworkStateChange::UserLoginChange(_)
            | NetworkStateChange::HistoryServerUpdate(_)
            | NetworkStateChange::ServicesUpdate(_)
            | NetworkStateChange::EventComplete(_) => Ok(()),
        }
    }
}

impl SendHistoryItem<update::UserAwayChange> for ClientServer {
    fn send_item(
        &self,
        item: &update::UserAwayChange,
        conn: impl MessageSink,
        from_entry: &impl HistoryItem,
    ) -> HandleResult {
        let net = self.network();
        let source = net.historic_user(item.user)?;

        if Some(*item.user.user()) == conn.user_id() {
            // Echo back to the user
            let message = match item.new_reason {
                None => numeric::Unaway::new(),
                Some(_) => numeric::NowAway::new(),
            };
            conn.send(message.format_for(self, source));
        } else {
            // Tell other users sharing a channel if they enabled away-notify
            let message = match item.new_reason {
                None => message::Unaway::new(source),
                Some(reason) => message::Away::new(source, reason.value()),
            };
            let message = message.with_tags_from(from_entry, &net);
            conn.send(message.with_required_capabilities(ClientCapability::AwayNotify));
        }

        Ok(())
    }
}

impl SendHistoryItem<update::UserNickChange> for ClientServer {
    fn send_item(
        &self,
        item: &update::UserNickChange,
        conn: impl MessageSink,
        from_entry: &impl HistoryItem,
    ) -> HandleResult {
        let net = self.network();
        let source = net.historic_user(item.user)?;
        let message = message::Nick::new(source, &item.new_nick).with_tags_from(from_entry, &net);

        conn.send(message);

        Ok(())
    }
}

impl SendHistoryItem<update::UserModeChange> for ClientServer {
    fn send_item(
        &self,
        item: &update::UserModeChange,
        conn: impl MessageSink,
        from_entry: &impl HistoryItem,
    ) -> HandleResult {
        let net = self.network();
        let source = net.historic_user(item.user)?;
        let message = message::Mode::new(
            source,
            source,
            &format_umode_changes(&item.added, &item.removed),
        )
        .with_tags_from(from_entry, &net);

        conn.send(message);

        Ok(())
    }
}

impl SendHistoryItem<update::UserQuit> for ClientServer {
    fn send_item(
        &self,
        item: &update::UserQuit,
        conn: impl MessageSink,
        from_entry: &impl HistoryItem,
    ) -> HandleResult {
        let net = self.network();
        let source = net.historic_user(item.user)?;
        let message = message::Quit::new(source, &item.message).with_tags_from(from_entry, &net);

        conn.send(message);

        Ok(())
    }
}

impl SendHistoryItem<update::ChannelModeChange> for ClientServer {
    fn send_item(
        &self,
        item: &update::ChannelModeChange,
        conn: impl MessageSink,
        from_entry: &impl HistoryItem,
    ) -> HandleResult {
        let net = self.network();
        let source = net.message_source(&item.changed_by)?;
        let channel = net.channel(item.channel)?;
        let (mut changes, params) = format_cmode_changes(item);

        for p in params {
            changes.push(' ');
            changes.push_str(&p);
        }

        let message =
            message::Mode::new(&source, &channel, &changes).with_tags_from(from_entry, &net);

        conn.send(message);

        Ok(())
    }
}

impl SendHistoryItem<update::ChannelTopicChange> for ClientServer {
    fn send_item(
        &self,
        item: &update::ChannelTopicChange,
        conn: impl MessageSink,
        from_entry: &impl HistoryItem,
    ) -> HandleResult {
        let net = self.network();
        let source = net.message_source(&item.setter)?;
        let channel = net.channel(item.channel)?;

        let message = message::Topic::new(&source, &channel.name(), &item.new_text)
            .with_tags_from(from_entry, &net);

        conn.send(message);

        Ok(())
    }
}

impl SendHistoryItem<update::ListModeAdded> for ClientServer {
    fn send_item(
        &self,
        item: &update::ListModeAdded,
        conn: impl MessageSink,
        from_entry: &impl HistoryItem,
    ) -> HandleResult {
        let net = self.network();
        let source = net.message_source(&item.set_by)?;
        let channel = net.channel(item.channel)?;

        let text = format!("+{} {}", item.list_type.mode_char(), item.pattern);
        let message = message::Mode::new(&source, &channel, &text).with_tags_from(from_entry, &net);
        conn.send(message);
        Ok(())
    }
}

impl SendHistoryItem<update::ListModeRemoved> for ClientServer {
    fn send_item(
        &self,
        item: &update::ListModeRemoved,
        conn: impl MessageSink,
        from_entry: &impl HistoryItem,
    ) -> HandleResult {
        let net = self.network();
        let source = net.message_source(&item.removed_by)?;
        let channel = net.channel(item.channel)?;

        let text = format!("-{} {}", item.list_type.mode_char(), item.pattern);
        let message = message::Mode::new(&source, &channel, &text).with_tags_from(from_entry, &net);
        conn.send(message);
        Ok(())
    }
}

impl SendHistoryItem<update::MembershipFlagChange> for ClientServer {
    fn send_item(
        &self,
        item: &update::MembershipFlagChange,
        conn: impl MessageSink,
        from_entry: &impl HistoryItem,
    ) -> HandleResult {
        let net = self.network();
        let source = net.message_source(&item.changed_by)?;
        let user = net.historic_user(item.user)?;
        let channel = net.channel(item.membership.channel())?;

        let (mut changes, args) =
            format_channel_perm_changes(&user.nickname, &item.added, &item.removed);

        changes += " ";
        changes += &args.join(" ");

        let message =
            message::Mode::new(&source, &channel, &changes).with_tags_from(from_entry, &net);

        conn.send(message);

        Ok(())
    }
}

impl SendHistoryItem<update::ChannelJoin> for ClientServer {
    fn send_item(
        &self,
        item: &update::ChannelJoin,
        conn: impl MessageSink,
        from_entry: &impl HistoryItem,
    ) -> HandleResult {
        let net = self.network();
        let user = net.historic_user(item.user)?;
        let membership = net.membership(item.membership)?;
        let channel = membership.channel()?;

        let message = message::Join::new(user, &channel.name()).with_tags_from(from_entry, &net);

        conn.send(message);

        if !membership.permissions().is_empty() {
            let (mut changes, args) = format_channel_perm_changes(
                &user.nickname,
                &membership.permissions(),
                &MembershipFlagSet::new(),
            );

            changes += " ";
            changes += &args.join(" ");

            let msg = message::Mode::new(user, &channel, &changes);
            conn.send(msg);
        }

        if let Some(away_reason) = user.away_reason() {
            let message =
                message::Away::new(user, away_reason.value()).with_tags_from(from_entry, &net);

            conn.send(message.with_required_capabilities(ClientCapability::AwayNotify));
        }

        Ok(())
    }
}

impl SendHistoryItem<update::ChannelKick> for ClientServer {
    fn send_item(
        &self,
        item: &update::ChannelKick,
        conn: impl MessageSink,
        from_entry: &impl HistoryItem,
    ) -> HandleResult {
        let net = self.network();
        let source = net.message_source(&item.source)?;
        let user = net.historic_user(item.user)?;
        let channel = net.channel(item.membership.channel)?;

        let message = message::Kick::new(&source, user, &channel.name(), &item.message)
            .with_tags_from(from_entry, &net);

        conn.send(message);

        Ok(())
    }
}

impl SendHistoryItem<update::ChannelPart> for ClientServer {
    fn send_item(
        &self,
        item: &update::ChannelPart,
        conn: impl MessageSink,
        from_entry: &impl HistoryItem,
    ) -> HandleResult {
        let net = self.network();
        let user = net.historic_user(item.user)?;
        let channel = net.channel(item.membership.channel)?;

        // If editing this behaviour, make sure that the faked version in the channel rename
        // handler stays in sync
        let message = message::Part::new(user, &channel.name(), &item.message)
            .with_tags_from(from_entry, &net);

        conn.send(message);

        Ok(())
    }
}

impl SendHistoryItem<update::ChannelInvite> for ClientServer {
    fn send_item(
        &self,
        item: &update::ChannelInvite,
        conn: impl MessageSink,
        from_entry: &impl HistoryItem,
    ) -> HandleResult {
        let net = self.network();
        let source = net.message_source(&item.source)?;
        let user = net.historic_user(item.user)?;
        let channel = net.channel(item.invite.channel())?;

        let message =
            message::Invite::new(&source, user, &channel.name()).with_tags_from(from_entry, &net);

        conn.send(message);

        Ok(())
    }
}

impl SendHistoryItem<update::ChannelRename> for ClientServer {
    fn send_item(
        &self,
        _item: &update::ChannelRename,
        _conn: impl MessageSink,
        _from_entry: &impl HistoryItem,
    ) -> HandleResult {
        // Not part of history, so it is handled entirely in send_realtime.rs.
        // See https://github.com/ircv3/ircv3-specifications/issues/532
        Ok(())
    }
}

impl SendHistoryItem<update::NewMessage> for ClientServer {
    fn send_item(
        &self,
        item: &update::NewMessage,
        conn: impl MessageSink,
        from_entry: &impl HistoryItem,
    ) -> HandleResult {
        let net = self.network();
        let source = net.message_source(&item.source)?;
        let target = net.message_target(&item.target)?;
        let message = net.message(item.message)?;

        let message =
            message::Message::new(&source, &target, message.message_type(), message.text())
                .with_tags_from(from_entry, &net);

        // Users should only see their own message echoed if they've asked for it,
        // unless it's sent to themself
        match &item.source {
            state::HistoricMessageSourceId::User(user) => {
                if conn.user_id() == Some(*user.user())
                    && item.target.user().map(|id| id.user()) != Some(user.user())
                {
                    conn.send(message.with_required_capabilities(ClientCapability::EchoMessage));
                } else {
                    conn.send(message);
                }
            }
            _ => conn.send(message),
        }

        Ok(())
    }
}
