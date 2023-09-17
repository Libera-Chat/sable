use crate::errors::HandleResult;
use crate::messages::MessageSink;
use crate::ClientServer;
use sable_network::network::update::HistoricUser;
use sable_network::prelude::wrapper::ObjectWrapper;
use sable_network::prelude::*;

use super::send_history::SendHistoryItem;
use super::*;

/// Extension trait for network updates that behave differently in realtime than in history playback
pub(crate) trait SendRealtimeItem: SendHistoryItem {
    // Default implementation delegates to the historic version
    fn send_now(
        &self,
        conn: &impl MessageSink,
        from_entry: &HistoryLogEntry,
        _server: &ClientServer,
    ) -> HandleResult;
}

impl SendRealtimeItem for HistoryLogEntry {
    fn send_now(
        &self,
        conn: &impl MessageSink,
        _from_entry: &HistoryLogEntry,
        server: &ClientServer,
    ) -> HandleResult {
        match &self.details {
            NetworkStateChange::ChannelJoin(detail) => detail.send_now(conn, self, server),
            NetworkStateChange::ChannelRename(detail) => detail.send_now(conn, self, server),
            _ => self.send_to(conn, self),
        }
    }
}

impl SendRealtimeItem for update::ChannelJoin {
    fn send_now(
        &self,
        conn: &impl MessageSink,
        from_entry: &HistoryLogEntry,
        server: &ClientServer,
    ) -> HandleResult {
        // When a user joins, we need to send topic, names, etc. We can't easily do that when replaying
        // history, because we don't have direct access to the historic member list.

        self.send_to(conn, from_entry)?;

        // If we're notifying someone other than the joining user, we're done now
        if conn.user_id() != Some(self.user.user.id) {
            return Ok(());
        }

        // If we get here, the user we're notifying is the joining user
        let network = server.network();
        let channel = network.channel(self.channel.id)?;
        let user = network.user(self.user.user.id)?;

        if let Some(topic) = channel.topic() {
            conn.send(numeric::TopicIs::new(&channel, topic.text()).format_for(server, &self.user));
            conn.send(
                numeric::TopicSetBy::new(&channel, topic.setter(), topic.timestamp())
                    .format_for(server, &self.user),
            );
        }

        crate::utils::send_channel_names(server, conn, &user, &channel)?;

        Ok(())
    }
}

impl SendRealtimeItem for update::ChannelRename {
    fn send_now(
        &self,
        conn: &impl MessageSink,
        from_entry: &HistoryLogEntry,
        server: &ClientServer,
    ) -> HandleResult {
        if conn.capabilities().has(ClientCapability::ChannelRename) {
            conn.send(
                message::Rename::new(&self.source, &self.old_name, &self.new_name, &self.message)
                    .with_tags_from(from_entry)
                    .with_required_capabilities(ClientCapability::ChannelRename),
            );

            Ok(())
        } else {
            // For clients which don't support draft/channel-rename, emulate by making
            // them PART + JOIN:

            let Some(user_id) = conn.user_id() else {
                return Ok(());
            };

            let network = server.network();
            let channel = network.channel(self.channel.id)?;
            let Some(membership) = channel.has_member(user_id) else {
                tracing::warn!("Cannot send ChannelRename to non-member {:?}", user_id);
                return Ok(());
            };
            let user = network.user(user_id)?;

            conn.send(
                message::Part::new(
                    &user,
                    &self.old_name,
                    &format!("Channel renamed to {}: {}", &self.new_name, &self.message),
                )
                .except_capability(ClientCapability::ChannelRename),
            );

            update::ChannelJoin {
                channel: self.channel.clone(),
                membership: membership.raw().clone(),
                user: HistoricUser {
                    user: user.raw().clone(),
                    account: user.account().ok().flatten().map(|acc| acc.name()),
                    nickname: user.nick(),
                },
            }
            .send_now(conn, from_entry, server)
        }
    }
}
